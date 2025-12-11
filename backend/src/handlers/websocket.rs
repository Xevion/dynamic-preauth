use futures_util::{FutureExt, StreamExt};
use salvo::http::StatusError;
use salvo::prelude::{handler, Request, Response, WebSocketUpgrade};
use salvo::websocket::WebSocket;
use salvo::Depot;
use tokio::sync::mpsc;
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::models::{IncomingMessage, OutgoingMessage};
use crate::state::STORE;

use super::session::get_session_id;

#[handler]
pub async fn connect(
    req: &mut Request,
    res: &mut Response,
    depot: &Depot,
) -> Result<(), StatusError> {
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
