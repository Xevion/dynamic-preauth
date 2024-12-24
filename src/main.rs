use std::env;
use std::sync::LazyLock;

use futures_util::{FutureExt, StreamExt};
use models::{IncomingMessage, OutgoingMessage};
use salvo::cors::Cors;
use salvo::http::{HeaderValue, Method, StatusCode, StatusError};
use salvo::logging::Logger;
use salvo::prelude::{
    handler, CatchPanic, Listener, Request, Response, Router, Server, Service, StaticDir,
    TcpListener, WebSocketUpgrade,
};
use salvo::websocket::{Message, WebSocket};
use salvo::writing::Json;
use salvo::Depot;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing_subscriber::EnvFilter;

use crate::models::State;

static STORE: LazyLock<Mutex<State>> = LazyLock::new(State::new);

mod models;
mod utility;

#[handler]
async fn session_middleware(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    match req.cookie("Session") {
        Some(cookie) => {
            // Check if the session exists
            match cookie.value().parse::<usize>() {
                Ok(session_id) => {
                    let mut store = STORE.lock().await;
                    if !store.sessions.contains_key(&session_id) {
                        let new_session_id = store.new_session(res).await;
                        depot.insert("session_id", new_session_id);
                        tracing::debug!(
                            existing_session_id = session_id,
                            new_session_id = new_session_id,
                            "Session provided in cookie, but does not exist"
                        );
                    }
                }
                Err(parse_error) => {
                    tracing::debug!(
                        invalid_session_id = cookie.value(),
                        error = ?parse_error,
                        "Session provided in cookie, but is not a valid number"
                    );
                    let mut store = STORE.lock().await;
                    let id = store.new_session(res).await;

                    depot.insert("session_id", id);
                }
            }
        }
        None => {
            tracing::debug!("Session was not provided in cookie");
            let mut store = STORE.lock().await;
            let id = store.new_session(res).await;

            depot.insert("session_id", id);
        }
    }
}

#[handler]
async fn connect(req: &mut Request, res: &mut Response, depot: &Depot) -> Result<(), StatusError> {
    let session_id = get_session_id(req, depot).unwrap();
    WebSocketUpgrade::new()
        .upgrade(req, res, move |ws| async move {
            handle_socket(session_id, ws).await;
        })
        .await
}

async fn handle_socket(session_id: usize, websocket: WebSocket) {
    // Split the socket into a sender and receive of messages.
    let (socket_tx, mut socket_rx) = websocket.split();

    // Use an unbounded channel to handle buffering and flushing of messages to the websocket...
    let (tx_channel, tx_channel_rx) = mpsc::unbounded_channel();
    let transmit = UnboundedReceiverStream::new(tx_channel_rx);
    let fut_handle_tx_buffer = transmit.forward(socket_tx).map(|result| {
        if let Err(e) = result {
            tracing::error!(error = ?e, "websocket send error");
        }
    });
    tokio::task::spawn(fut_handle_tx_buffer);

    let mut store = STORE.lock().await;
    let session = store
        .sessions
        .get_mut(&session_id)
        .expect("Unable to get session");
    session.tx = Some(tx_channel);

    session.send_message(OutgoingMessage::State {
        id: session_id,
        session: session.clone(),
    });
    drop(store);

    // Handle incoming messages
    let fut = async move {
        tracing::info!(
            "WebSocket connection established for session_id: {}",
            session_id
        );

        while let Some(result) = socket_rx.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(error) => {
                    tracing::error!(
                        "WebSocket Error session_id={} error=({})",
                        session_id,
                        error
                    );
                    break;
                }
            };

            if msg.is_close() {
                tracing::info!("WebSocket closing for Session {}", session_id);
                break;
            }

            if msg.is_text() {
                let text = msg.to_str().unwrap();

                // Deserialize
                match serde_json::from_str::<IncomingMessage>(text) {
                    Ok(message) => {
                        tracing::info!("Received message: {:?}", message);
                    }
                    Err(e) => {
                        tracing::error!("Error deserializing message: {} {}", text, e);
                    }
                }
            }
        }
    };
    tokio::task::spawn(fut);
}

#[handler]
pub async fn download(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    let download_id = req
        .param::<String>("id")
        .expect("Download ID required to download file");

    let session_id =
        get_session_id(req, depot).expect("Session ID could not be found via request or depot");

    let store = &mut *STORE.lock().await;

    let session = store
        .sessions
        .get_mut(&session_id)
        .expect("Session not found");
    let executable = store
        .executables
        .get(&download_id as &str)
        .expect("Executable not found");

    // Create a download for the session
    let session_download = session.add_download(executable);
    tracing::info!(session_id, type = download_id, dl_token = session_download.token, "Download created");
    let data = executable.with_key(session_id.to_string().as_bytes());

    if let Err(e) = res.write_body(data) {
        tracing::error!("Error writing body: {}", e);
    }

    // TODO: Send the notify message via websocket

    res.headers.insert(
        "Content-Disposition",
        HeaderValue::from_str(
            format!("attachment; filename=\"{}\"", session_download.filename).as_str(),
        )
        .expect("Unable to create header"),
    );
    res.headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/octet-stream"),
    );
}

#[handler]
pub async fn get_session(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    let store = STORE.lock().await;

    let session_id = get_session_id(req, depot);
    if session_id.is_none() {
        res.status_code(StatusCode::BAD_REQUEST);
        return;
    }

    match store.sessions.get(&session_id.unwrap()) {
        Some(session) => {
            res.render(Json(&session));
        }
        None => {
            res.status_code(StatusCode::BAD_REQUEST);
        }
    }
}

// Acquires the session id from the request, preferring the depot
fn get_session_id(req: &Request, depot: &Depot) -> Option<usize> {
    if depot.contains_key("session_id") {
        return Some(*depot.get::<usize>("session_id").unwrap());
    }

    // Otherwise, just use whatever the Cookie might have
    match req.cookie("Session") {
        Some(cookie) => match cookie.value().parse::<usize>() {
            Ok(id) => Some(id),
            _ => None,
        },
        None => {
            tracing::warn!("Session was not provided in cookie or depot");
            None
        }
    }
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "5800".to_string());
    let addr = format!("0.0.0.0:{}", port);
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(format!(
            "info,dynamic_preauth={}",
            // Only log our message in debug mode
            match cfg!(debug_assertions) {
                true => "debug",
                false => "info",
            }
        )))
        .init();

    // Check if we are deployed on Railway
    let is_railway = env::var("RAILWAY_PROJECT_ID").is_ok();
    if is_railway {
        let build_logs = format!(
            "https://railway.com/project/{}/service/{}?environmentId={}&id={}#build",
            env::var("RAILWAY_PROJECT_ID").unwrap(),
            env::var("RAILWAY_SERVICE_ID").unwrap(),
            env::var("RAILWAY_ENVIRONMENT_ID").unwrap(),
            env::var("RAILWAY_DEPLOYMENT_ID").unwrap()
        );

        tracing::info!("Build logs available here: {}", build_logs);
    }

    // Add the executables to the store
    let mut store = STORE.lock().await;
    store.add_executable("windows", "./demo-windows.exe");
    store.add_executable("linux", "./demo-linux");
    drop(store); // critical: Drop the lock to avoid deadlock, otherwise the server will hang

    // Allow all origins if: debug mode or RAILWAY_PUBLIC_DOMAIN is not set
    let origin = if cfg!(debug_assertions) | env::var_os("RAILWAY_PUBLIC_DOMAIN").is_none() {
        "*".to_string()
    } else {
        format!(
            "https://{}",
            env::var_os("RAILWAY_PUBLIC_DOMAIN")
                .unwrap()
                .to_str()
                .unwrap()
        )
    };

    let cors = Cors::new()
        .allow_origin(&origin)
        .allow_methods(vec![Method::GET])
        .into_handler();
    tracing::debug!("CORS Allowed Origin: {}", &origin);

    let static_dir = StaticDir::new(["./public"]).defaults("index.html");

    // TODO: Move handlers to a separate file
    // TODO: Improved Token Generation
    // TODO: Advanded HMAC Verification
    // TODO: Session Purging

    let router = Router::new()
        .hoop(CatchPanic::new())
        .hoop(cors)
        .hoop(session_middleware)
        .push(Router::with_path("download/<id>").get(download))
        .push(Router::with_path("session").get(get_session))
        .push(Router::with_path("ws").goal(connect))
        .push(Router::with_path("<**path>").get(static_dir));

    let service = Service::new(router).hoop(Logger::new());

    let acceptor = TcpListener::new(addr).bind().await;
    Server::new(acceptor).serve(service).await;
}
