use std::{collections::HashMap, path};
use tokio::sync::Mutex;

use crate::utility::search;

#[derive(Default, Clone, Debug)]
pub(crate) struct State<'a> {
    pub executables: HashMap<&'a str, Executable>,
}

impl<'a> State<'a> {
    pub(crate) fn new() -> Mutex<Self> {
        Mutex::new(Self {
            executables: HashMap::new(),
        })
    }

    pub(crate) fn add_executable(&mut self, exe_type: &'a str, exe_path: &str) {
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
}

#[derive(Default, Clone, Debug)]
pub(crate) struct Executable {
    pub data: Vec<u8>,
    pub filename: String,
    pub key_start: usize,
    pub key_end: usize,
}

impl Executable {
    pub(crate) fn with_key(&self, new_key: &[u8]) -> Vec<u8> {
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
