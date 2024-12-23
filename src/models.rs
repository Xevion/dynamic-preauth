use rand::Rng;
use salvo::{http::cookie::Cookie, websocket::Message, Response};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, path};
use tokio::sync::{mpsc::UnboundedSender, Mutex};

use crate::utility::search;

#[derive(Debug, Serialize, Clone)]
pub struct Session {
    pub downloads: Vec<SessionDownload>,

    pub first_seen: chrono::DateTime<chrono::Utc>,
    // The last time a request OR websocket message with this session was made
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
        let mut rng = rand::thread_rng();
        let token: u64 = rng.gen();

        let download = SessionDownload {
            token,
            filename: format!("{}-{:16x}{}", exe.name, token, exe.extension),
            last_used: chrono::Utc::now(),
            download_time: chrono::Utc::now(),
        };

        self.downloads.push(download);
        return self.downloads.last().unwrap();
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SessionDownload {
    pub token: u64,
    pub filename: String,
    pub last_used: chrono::DateTime<chrono::Utc>,
    pub download_time: chrono::DateTime<chrono::Utc>,
}

impl SessionDownload {}

#[derive(Clone, Debug)]
pub struct State<'a> {
    // A map of executables, keyed by their type/platform
    pub executables: HashMap<&'a str, Executable>,
    // A map of sessions, keyed by their identifier (a random number)
    pub sessions: HashMap<usize, Session>,
}

impl<'a> State<'a> {
    pub fn new() -> Mutex<Self> {
        Mutex::new(Self {
            executables: HashMap::new(),
            sessions: HashMap::new(),
        })
    }

    pub fn add_executable(&mut self, exe_type: &'a str, exe_path: &str) {
        let data = std::fs::read(&exe_path).expect("Unable to read file");

        let pattern = "a".repeat(1024);
        let key_start = search(&data, pattern.as_bytes(), 0).unwrap();
        let key_end = key_start + pattern.len();

        let filename = path::Path::new(&exe_path);
        let name = filename.file_stem().unwrap().to_str().unwrap();
        let extension = match filename.extension() {
            Some(s) => s.to_str().unwrap(),
            None => "",
        };

        let exe = Executable {
            data,
            name: name.to_string(),
            extension: extension.to_string(),
            key_start: key_start,
            key_end: key_end,
        };

        self.executables.insert(exe_type, exe);
    }

    pub async fn new_session(&mut self, res: &mut Response) -> usize {
        let mut rng = rand::thread_rng();
        let id: usize = rng.gen();

        let now = chrono::Utc::now();
        self.sessions.insert(
            id,
            Session {
                downloads: Vec::new(),
                last_seen: now,
                last_request: now,
                first_seen: now,
                tx: None,
            },
        );

        tracing::info!("New session created: {}", id);

        res.add_cookie(
            Cookie::build(("Session", id.to_string()))
                .http_only(true)
                .path("/")
                .same_site(salvo::http::cookie::SameSite::Lax)
                .permanent()
                .build(),
        );

        return id;
    }
}

#[derive(Default, Clone, Debug)]
pub struct Executable {
    pub data: Vec<u8>,     // the raw data of the executable
    pub name: String,      // the name before the extension
    pub extension: String, // may be empty string
    pub key_start: usize,  // the index of the byte where the key starts
    pub key_end: usize,    // the index of the byte where the key ends
}

impl Executable {
    pub fn with_key(&self, new_key: &[u8]) -> Vec<u8> {
        let mut data = self.data.clone();

        // Copy the key into the data
        for i in 0..new_key.len() {
            data[self.key_start + i] = new_key[i];
        }

        // If the new key is shorter than the old key, we just write over the remaining data
        if new_key.len() < self.key_end - self.key_start {
            for i in self.key_start + new_key.len()..self.key_end {
                data[i] = b' ';
            }
        }

        return data;
    }
}

#[derive(Debug, Deserialize)]
#[serde(tag = "type")]
pub enum IncomingMessage {
    // A request from the client to delete a session token
    DeleteSessionToken { id: u64 },
}

#[derive(Debug, Serialize)]
#[serde(tag = "type")]
pub enum OutgoingMessage {
    // An alert to the client that a session download has been used.
    TokenAlert { token: u64 },
    // A message describing the current session state
    State { session: Session },
    Executables { executables: Vec<ExecutableJson> },
}

#[derive(Debug, Serialize)]
pub struct ExecutableJson {
    pub id: String,
    pub size: usize,
    pub filename: String,
}
