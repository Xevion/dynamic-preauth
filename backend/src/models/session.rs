use salvo::websocket::Message;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;

use super::executable::Executable;
use super::messages::OutgoingMessage;

#[derive(Debug, Serialize, Clone)]
pub struct Session {
    pub id: u32,
    pub downloads: Vec<SessionDownload>,

    pub first_seen: chrono::DateTime<chrono::Utc>,
    // The last time a request OR websocket message from/to this session was made
    pub last_seen: chrono::DateTime<chrono::Utc>,
    // The last time a request was made with this session
    pub last_request: chrono::DateTime<chrono::Utc>,

    // The sender for the websocket connection
    #[serde(skip_serializing)]
    pub tx: Option<UnboundedSender<Result<Message, salvo::Error>>>,
}

impl Session {
    // Update the last seen time(s) for the session
    pub fn seen(&mut self, socket: bool) {
        self.last_seen = chrono::Utc::now();
        if !socket {
            self.last_request = chrono::Utc::now();
        }
    }

    // Add a download to the session
    pub fn add_download(&mut self, exe: &Executable) -> &SessionDownload {
        let token: u32 = rand::random();

        let download = SessionDownload {
            token,
            filename: format!(
                "{}-{:08x}{}{}",
                exe.name,
                token,
                if !exe.extension.is_empty() { "." } else { "" },
                exe.extension
            ),
            last_used: chrono::Utc::now(),
            download_time: chrono::Utc::now(),
        };

        self.downloads.push(download);
        self.downloads.last().unwrap()
    }

    // Delete a download from the session
    // Returns true if the download was deleted, false if it was not found
    pub fn delete_download(&mut self, token: u32) -> bool {
        if let Some(index) = self.downloads.iter().position(|d| d.token == token) {
            self.downloads.remove(index);
            true
        } else {
            tracing::warn!("Attempted to delete non-existent download token: {}", token);
            false
        }
    }

    // This function's failure is not a failure to transmit the message, but a failure to buffer it into the channel (or any preceding steps).
    pub fn send_message(&mut self, message: OutgoingMessage) -> Result<(), anyhow::Error> {
        if self.tx.is_none() {
            return Err(anyhow::anyhow!("Session {} has no sender", self.id));
        }

        // TODO: Error handling
        let tx = self.tx.as_ref().unwrap();
        let result = tx.send(Ok(Message::text(serde_json::to_string(&message).unwrap())));

        match result {
            Ok(_) => Ok(()),
            Err(e) => Err(anyhow::anyhow!("Error sending message: {}", e)),
        }
    }

    pub fn send_state(&mut self) -> Result<(), anyhow::Error> {
        let message = OutgoingMessage::State {
            session: self.clone(),
        };

        self.send_message(message)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SessionDownload {
    pub token: u32,
    pub filename: String,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub download_time: chrono::DateTime<chrono::Utc>,
}
