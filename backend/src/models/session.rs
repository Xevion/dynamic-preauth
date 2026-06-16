use std::collections::HashMap;

use salvo::websocket::Message;
use serde::Serialize;
use tokio::sync::mpsc::UnboundedSender;

use super::executable::Executable;
use super::messages::OutgoingMessage;

/// Sender type for WebSocket connections
pub type ConnectionSender = UnboundedSender<Result<Message, salvo::Error>>;

#[derive(Debug, Serialize, Clone)]
pub struct Session {
    pub id: u32,
    pub downloads: Vec<SessionDownload>,

    pub first_seen: chrono::DateTime<chrono::Utc>,
    // The last time a request OR websocket message from/to this session was made
    pub last_seen: chrono::DateTime<chrono::Utc>,
    // The last time a request was made with this session
    pub last_request: chrono::DateTime<chrono::Utc>,

    /// Multiple WebSocket connections per session (multi-tab support)
    /// Key is a random connection ID, value is the sender channel
    #[serde(skip_serializing)]
    pub connections: HashMap<u64, ConnectionSender>,
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

    /// Register a new WebSocket connection and return its ID
    pub fn add_connection(&mut self, tx: ConnectionSender) -> u64 {
        let connection_id: u64 = rand::random();
        self.connections.insert(connection_id, tx);
        tracing::debug!(
            "Added connection {} to session {} (total: {})",
            connection_id,
            self.id,
            self.connections.len()
        );
        connection_id
    }

    /// Remove a WebSocket connection by ID
    pub fn remove_connection(&mut self, connection_id: u64) {
        self.connections.remove(&connection_id);
        tracing::debug!(
            "Removed connection {} from session {} (remaining: {})",
            connection_id,
            self.id,
            self.connections.len()
        );
    }

    /// Broadcast a message to all connected WebSocket clients
    /// Returns the number of connections that received the message
    pub fn send_message(&mut self, message: OutgoingMessage) -> Result<usize, anyhow::Error> {
        if self.connections.is_empty() {
            return Err(anyhow::anyhow!("Session {} has no connections", self.id));
        }

        let json = serde_json::to_string(&message)?;
        let mut dead_connections = Vec::new();
        let mut sent_count = 0;

        for (&conn_id, tx) in &self.connections {
            match tx.send(Ok(Message::text(json.clone()))) {
                Ok(_) => sent_count += 1,
                Err(_) => {
                    // Channel closed, mark for removal
                    dead_connections.push(conn_id);
                }
            }
        }

        // Clean up dead connections
        for conn_id in dead_connections {
            self.connections.remove(&conn_id);
            tracing::debug!(
                "Cleaned up dead connection {} from session {}",
                conn_id,
                self.id
            );
        }

        if sent_count == 0 {
            return Err(anyhow::anyhow!(
                "Session {} has no active connections",
                self.id
            ));
        }

        Ok(sent_count)
    }

    pub fn send_state(&mut self) -> Result<usize, anyhow::Error> {
        let message = OutgoingMessage::State {
            session: self.clone(),
        };

        self.send_message(message)
    }

    /// Send a message to a single connection by ID, leaving the others untouched.
    /// Lets a freshly-connected tab catch up on state that the already-connected
    /// tabs already hold, so they don't receive a redundant update.
    pub fn send_message_to(
        &self,
        connection_id: u64,
        message: &OutgoingMessage,
    ) -> Result<(), anyhow::Error> {
        let tx = self.connections.get(&connection_id).ok_or_else(|| {
            anyhow::anyhow!(
                "Connection {} not found in session {}",
                connection_id,
                self.id
            )
        })?;

        let json = serde_json::to_string(message)?;
        tx.send(Ok(Message::text(json)))
            .map_err(|e| anyhow::anyhow!("Error sending message: {}", e))
    }

    /// Send the current session state to a single connection.
    pub fn send_state_to(&self, connection_id: u64) -> Result<(), anyhow::Error> {
        let message = OutgoingMessage::State {
            session: self.clone(),
        };

        self.send_message_to(connection_id, &message)
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SessionDownload {
    pub token: u32,
    pub filename: String,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub download_time: chrono::DateTime<chrono::Utc>,
}
