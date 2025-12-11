use serde::{Deserialize, Serialize};

use super::executable::ExecutableJson;
use super::session::Session;

#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum IncomingMessage {
    // A request from the client to delete a download token
    DeleteDownloadToken { id: u32 },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "kebab-case")]
pub enum OutgoingMessage {
    // An alert to the client that a session download has been used.
    #[serde(rename = "notify")]
    TokenAlert {
        token: u32,
    },
    // A message describing the current session state
    State {
        session: Session,
    },
    Executables {
        build_log: Option<String>,
        executables: Vec<ExecutableJson>,
    },
}
