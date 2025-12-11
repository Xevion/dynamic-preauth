use std::collections::HashMap;
use std::path;
use std::sync::LazyLock;

use salvo::{http::cookie::Cookie, Response};
use tokio::sync::Mutex;

use crate::models::{BuildLogs, Executable, ExecutableJson, Session};

pub static STORE: LazyLock<Mutex<State>> = LazyLock::new(|| Mutex::new(State::new()));

#[derive(Default)]
pub struct State {
    pub sessions: HashMap<u32, Session>,
    pub executables: HashMap<String, Executable>,
    pub build_logs: Option<BuildLogs>,
    pub build_log_url: Option<String>,
}

impl State {
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            executables: HashMap::new(),
            build_logs: None,
            build_log_url: None,
        }
    }

    pub fn add_executable(&mut self, exe_type: &str, exe_path: &str) {
        let data = std::fs::read(exe_path).expect("Unable to read file");

        let pattern = "a".repeat(1024);
        let key_start = Executable::search_pattern(&data, pattern.as_bytes(), 0).unwrap();
        let key_end = key_start + pattern.len();

        let path = path::Path::new(&exe_path);
        let name = path.file_stem().unwrap().to_str().unwrap();
        let extension = match path.extension() {
            Some(s) => s.to_str().unwrap(),
            None => "",
        };

        let exe = Executable {
            data,
            filename: path.file_name().unwrap().to_str().unwrap().to_string(),
            name: name.to_string(),
            extension: extension.to_string(),
            key_start,
            key_end,
        };

        self.executables.insert(exe_type.to_string(), exe);
    }

    pub async fn new_session(&mut self, res: &mut Response) -> u32 {
        let id: u32 = rand::random();

        let now = chrono::Utc::now();
        self.sessions.insert(
            id,
            Session {
                id,
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
                .partitioned(true)
                .secure(cfg!(debug_assertions) == false)
                .path("/")
                // Use SameSite=None only in development
                .same_site(if cfg!(debug_assertions) {
                    salvo::http::cookie::SameSite::None
                } else {
                    salvo::http::cookie::SameSite::Strict
                })
                .permanent()
                .build(),
        );

        id
    }

    pub fn executable_json(&self) -> Vec<ExecutableJson> {
        let mut executables = Vec::new();

        for (key, exe) in &self.executables {
            executables.push(ExecutableJson {
                id: key.to_string(),
                size: exe.data.len(),
                filename: exe.filename.clone(),
            });
        }

        executables
    }
}
