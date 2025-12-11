use salvo::http::HeaderValue;
use salvo::prelude::{handler, Request, Response};
use salvo::Depot;

use crate::state::STORE;

use super::session::get_session_id;

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
