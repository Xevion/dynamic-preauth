use rand::{distributions::Alphanumeric, Rng};
use salvo::{http::cookie::Cookie, Response};
use serde::Serialize;
use std::{collections::HashMap, path};
use tokio::sync::Mutex;

use crate::utility::search;

#[derive(Clone, Debug, Serialize)]
pub struct Session {
    pub tokens: Vec<String>,
    pub last_seen: chrono::DateTime<chrono::Utc>,
    pub first_seen: chrono::DateTime<chrono::Utc>,
}

#[derive(Default, Clone, Debug)]
pub struct State<'a> {
    pub executables: HashMap<&'a str, Executable>,
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

        let filename = path::Path::new(&exe_path)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();

        let exe = Executable {
            data,
            filename,
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
                tokens: vec![],
                last_seen: now,
                first_seen: now,
            },
        );

        res.add_cookie(
            Cookie::build(("Session", id.to_string()))
                .permanent()
                .build(),
        );

        return id;
    }
}

#[derive(Default, Clone, Debug)]
pub struct Executable {
    pub data: Vec<u8>,
    pub filename: String,
    pub key_start: usize,
    pub key_end: usize,
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
