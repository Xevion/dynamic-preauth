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
use salvo::websocket::WebSocket;
use salvo::writing::Json;
use salvo::Depot;
use tokio::sync::{mpsc, Mutex};
use tokio_stream::wrappers::UnboundedReceiverStream;
use tracing_subscriber::EnvFilter;

use crate::config::Config;
use crate::models::State;

static STORE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State::new()));

mod config;
mod models;
mod railway;
mod utility;

#[handler]
async fn session_middleware(req: &mut Request, res: &mut Response, depot: &mut Depot) {
    match req.cookie("Session") {
        Some(cookie) => {
            // Check if the session exists
            match cookie.value().parse::<u32>() {
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
                    } else {
                        store.sessions.get_mut(&session_id).unwrap().seen(false);
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

async fn handle_socket(session_id: u32, websocket: WebSocket) {
    // Split the socket into a sender and receive of messages.
    let (socket_tx, mut socket_rx) = websocket.split();

    // Use an unbounded channel to handle buffering and flushing of messages to the websocket...
    let (tx_channel, tx_channel_rx) = mpsc::unbounded_channel();
    let transmit = UnboundedReceiverStream::new(tx_channel_rx);
    let fut_handle_tx_buffer = transmit
        .then(|message| async {
            match message {
                Ok(ref message) => {
                    tracing::debug!(message = ?message, "Outgoing Message");
                }
                Err(ref e) => {
                    tracing::error!(error = ?e, "Outgoing Message Error");
                }
            }
            message
        })
        .forward(socket_tx)
        .map(|result| {
            tracing::debug!("WebSocket send result: {:?}", result);
            if let Err(e) = result {
                tracing::error!(error = ?e, "websocket send error");
            }
        });
    tokio::task::spawn(fut_handle_tx_buffer);

    let store = &mut *STORE.lock().await;

    // Create the executable message first, borrow issues
    let executable_message = OutgoingMessage::Executables {
        executables: store.executable_json(),
        build_log: if store.build_logs.is_some() {
            Some("/build-logs".to_string())
        } else {
            None
        },
    };

    let session = store
        .sessions
        .get_mut(&session_id)
        .expect("Unable to get session");
    session.tx = Some(tx_channel);

    session
        .send_state()
        .expect("Failed to buffer state message");
    session
        .send_message(executable_message)
        .expect("Failed to buffer executables message");

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
                        tracing::debug!(message = ?message, "Received message");

                        match message {
                            IncomingMessage::DeleteDownloadToken { id } => {
                                let store = &mut *STORE.lock().await;
                                let session = store
                                    .sessions
                                    .get_mut(&session_id)
                                    .expect("Session not found");

                                if session.delete_download(id) {
                                    session
                                        .send_state()
                                        .expect("Failed to buffer state message");
                                }
                            }
                        }
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
pub async fn get_build_logs(req: &mut Request, res: &mut Response, _depot: &mut Depot) {
    let store = STORE.lock().await;

    if let Some(build_logs) = &store.build_logs {
        // Use pre-computed hash for ETag
        let etag = format!("\"{:x}\"", build_logs.content_hash);

        // Check If-None-Match header
        if let Some(if_none_match) = req.headers().get("If-None-Match") {
            if if_none_match == &etag {
                res.status_code(StatusCode::NOT_MODIFIED);
                return;
            }
        }

        // Check If-Modified-Since header
        if let Some(if_modified_since) = req.headers().get("If-Modified-Since") {
            if let Ok(if_modified_since_str) = if_modified_since.to_str() {
                if let Ok(if_modified_since_time) =
                    chrono::DateTime::parse_from_rfc2822(if_modified_since_str)
                {
                    if build_logs.fetched_at <= if_modified_since_time {
                        res.status_code(StatusCode::NOT_MODIFIED);
                        return;
                    }
                }
            }
        }

        res.headers_mut().insert("ETag", etag.parse().unwrap());
        res.headers_mut()
            .insert("Content-Type", "text/plain; charset=utf-8".parse().unwrap());
        res.headers_mut()
            .insert("Cache-Control", "public, max-age=300".parse().unwrap());
        res.headers_mut().insert(
            "Last-Modified",
            build_logs.fetched_at.to_rfc2822().parse().unwrap(),
        );

        res.render(&build_logs.content);
    } else {
        res.status_code(StatusCode::NOT_FOUND);
        res.render("Build logs not available");
    }
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
    let data = executable.with_key(session_download.token.to_string().as_bytes());

    if let Err(e) = res.write_body(data) {
        tracing::error!("Error writing body: {}", e);
    }

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

    // Don't try to send state if somehow the session has not connected
    if session.tx.is_some() {
        session
            .send_state()
            .expect("Failed to buffer state message");
    } else {
        tracing::warn!("Download being made without any connection websocket");
    }
}

#[handler]
pub async fn notify(req: &mut Request, res: &mut Response) {
    let key = req.query::<String>("key");

    if key.is_none() {
        res.status_code(StatusCode::BAD_REQUEST);
        return;
    }

    let key = key.unwrap();

    if !key.starts_with("0x") {
        res.status_code(StatusCode::BAD_REQUEST);
        return;
    }

    // Parse key into u32
    let key = match u32::from_str_radix(key.trim_start_matches("0x"), 16) {
        Ok(k) => k,
        Err(e) => {
            tracing::error!("Error parsing key: {}", e);
            res.status_code(StatusCode::BAD_REQUEST);
            return;
        }
    };

    let store = &mut *STORE.lock().await;

    let target_session = store
        .sessions
        .iter_mut()
        .find(|(_, session)| session.downloads.iter().any(|d| d.token == key));

    match target_session {
        Some((_, session)) => {
            let message = OutgoingMessage::TokenAlert { token: key };

            if let Err(e) = session.send_message(message) {
                tracing::warn!(
                    error = e.to_string(),
                    "Session did not have a receiving WebSocket available, notify ignored.",
                );
                res.status_code(StatusCode::NOT_MODIFIED);
                return;
            }

            res.render("Notification sent");
        }
        None => {
            tracing::warn!("Session not found for key while attempting notify: {}", key);
            res.status_code(StatusCode::UNAUTHORIZED);
            return;
        }
    }
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
fn get_session_id(req: &Request, depot: &Depot) -> Option<u32> {
    if depot.contains_key("session_id") {
        return Some(*depot.get::<u32>("session_id").unwrap());
    }

    // Otherwise, just use whatever the Cookie might have
    match req.cookie("Session") {
        Some(cookie) => cookie.value().parse::<u32>().ok(),
        None => {
            tracing::warn!("Session was not provided in cookie or depot");
            None
        }
    }
}

#[tokio::main]
async fn main() {
    // Load environment variables from .env file (development only)
    #[cfg(debug_assertions)]
    dotenvy::dotenv().ok();

    // Parse configuration from environment
    let config: Config = envy::from_env().expect("Failed to parse environment configuration");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(format!(
            "info,dynamic_preauth={}",
            if cfg!(debug_assertions) { "debug" } else { "info" }
        )))
        .init();

    // Add the build log & executables to the store
    let mut store = STORE.lock().await;

    // Check if we are deployed on Railway
    if config.railway.is_railway() {
        if let Some(build_logs_url) = config.railway.build_logs_url() {
            tracing::info!("Build logs available here: {}", build_logs_url);
            store.build_log_url = Some(build_logs_url);
        }

        // Try to fetch actual build logs using Railway API
        if config.railway.has_token() {
            match crate::railway::fetch_build_logs().await {
                Ok(build_logs) => {
                    tracing::info!(
                        "Successfully fetched build logs ({} bytes)",
                        build_logs.content.len()
                    );
                    store.build_logs = Some(build_logs);
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch build logs from Railway API: {}", e);
                }
            }
        } else {
            tracing::warn!("RAILWAY_TOKEN not set, skipping build log fetch");
        }
    }

    store.add_executable("Windows", "./demo.exe");
    store.add_executable("Linux", "./demo-linux");
    // store.add_executable("MacOS", "./demo-macos");

    drop(store); // critical: Drop the lock to avoid deadlock, otherwise the server will hang

    let origin = config.railway.cors_origin();
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
        // /notify does not need a session, nor should it have one
        .push(Router::with_path("notify").post(notify))
        // /build-logs does not need a session
        .push(Router::with_path("build-logs").get(get_build_logs))
        .push(
            Router::new()
                .hoop(session_middleware)
                .push(Router::with_path("download/<id>").get(download))
                .push(Router::with_path("session").get(get_session))
                // websocket /ws
                .push(Router::with_path("ws").goal(connect))
                // static files
                .push(Router::with_path("<**path>").get(static_dir)),
        );

    let service = Service::new(router).hoop(cors).hoop(Logger::new());

    let acceptor = TcpListener::new(config.bind_addr()).bind().await;
    Server::new(acceptor).serve(service).await;
}
