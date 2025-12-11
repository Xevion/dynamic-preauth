use salvo::http::StatusCode;
use salvo::prelude::{handler, Request, Response};
use salvo::writing::Json;
use salvo::Depot;

use crate::state::STORE;

#[handler]
pub async fn session_middleware(req: &mut Request, res: &mut Response, depot: &mut Depot) {
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
pub fn get_session_id(req: &Request, depot: &Depot) -> Option<u32> {
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
