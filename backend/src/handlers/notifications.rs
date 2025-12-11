use salvo::http::StatusCode;
use salvo::prelude::{handler, Request, Response};

use crate::models::OutgoingMessage;
use crate::state::STORE;

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
