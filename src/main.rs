use std::env;
use std::sync::LazyLock;

use futures_util::{FutureExt, StreamExt};
use salvo::cors::Cors;
use salvo::http::{HeaderValue, Method, StatusCode, StatusError};
use salvo::logging::Logger;
use salvo::prelude::{
    handler, CatchPanic, Listener, Request, Response, Router, Server, Service, StaticDir,
    TcpListener, WebSocketUpgrade,
};
use salvo::websocket::WebSocket;
use salvo::writing::Json;
use salvo::Depot;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;

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
                        let id = store.new_session(res).await;
                        depot.insert("session_id", id);
                    }
                }
                Err(_) => {
                    let mut store = STORE.lock().await;
                    let id = store.new_session(res).await;

                    depot.insert("session_id", id);
                }
            }
        }
        None => {
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

async fn handle_socket(session_id: usize, ws: WebSocket) {
    // Split the socket into a sender and receive of messages.
    let (user_ws_tx, mut user_ws_rx) = ws.split();

    // Use an unbounded channel to handle buffering and flushing of messages
    // to the websocket...
    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);
    let fut = rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            tracing::error!(error = ?e, "websocket send error");
        }
    });
    tokio::task::spawn(fut);

    // Handle incoming messages
    let fut = async move {
        let mut store = STORE.lock().await;
        let session = store.sessions.get_mut(&session_id).unwrap();
        session.tx = Some(tx);
        drop(store);

        while let Some(result) = user_ws_rx.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(_) => {
                    // eprintln!("websocket error(uid={}): {}", my_id, e);
                    break;
                }
            };

            println!("Received message: {:?}", msg);
        }
    };
    tokio::task::spawn(fut);
}

#[handler]
pub async fn download(req: &mut Request, res: &mut Response) {
    let download_id = req.param::<String>("id").unwrap();

    let store = STORE.lock().await;
    let executable = store.executables.get(&download_id as &str).unwrap();
    let data = executable.with_key(b"test");

    if let Err(e) = res.write_body(data) {
        eprintln!("Error writing body: {}", e);
    }

    // TODO: Send the notify message via websocket

    res.headers.insert(
        "Content-Disposition",
        HeaderValue::from_str(format!("attachment; filename=\"{}\"", executable.filename).as_str())
            .unwrap(),
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

fn get_session_id(req: &Request, depot: &Depot) -> Option<usize> {
    match req.cookie("Session") {
        Some(cookie) => match cookie.value().parse::<usize>() {
            Ok(id) => Some(id),
            _ => None,
        },
        None => match depot.get::<usize>("session_id") {
            Ok(id) => Some(*id),
            _ => None,
        },
    }
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "5800".to_string());
    let addr = format!("0.0.0.0:{}", port);
    tracing_subscriber::fmt().init();

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

        println!("Build logs available here: {}", build_logs);
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

    let static_dir = StaticDir::new(["./public"]).defaults("index.html");

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
