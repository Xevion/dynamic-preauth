use std::collections::HashMap;
use std::iter::Map;
use std::path;

use salvo::http::HeaderValue;
use salvo::logging::Logger;

use salvo::prelude::*;

fn search(buf: &[u8], pattern: &[u8], start_index: usize) -> Option<usize> {
    let mut i = start_index;

    // If the buffer is empty, the pattern is too long
    if pattern.len() > buf.len() {
        return None;
    }

    // If the pattern is empty
    if pattern.len() == 0 {
        return None;
    }

    // If the starting index is too high
    if start_index >= buf.len() {
        return None;
    }

    while i < buf.len() {
        for j in 0..pattern.len() {
            // If the pattern is too long to fit in the buffer anymore
            if i + j >= buf.len() {
                return None;
            }

            // If the pattern stops matching
            if buf[i + j] != pattern[j] {
                break;
            }

            // If the pattern is found
            if j == pattern.len() - 1 {
                return Some(i);
            }
        }

        i += 1;
    }
    None
}

#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

#[handler]
async fn download(depot: &mut Depot, req: &mut Request, res: &mut Response) {
    let state = depot.obtain::<State>().unwrap();
    let data = state.exe_with_key("windows", b"test"); // Clone the data

    if let Err(e) = res.write_body(data) {
        eprintln!("Error writing body: {}", e);
    }

    res.headers.insert(
        "Content-Disposition",
        HeaderValue::from_str(format!("attachment; filename=\"{}\"", "demo.exe").as_str()).unwrap(),
    );
    res.headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/octet-stream"),
    );
}
#[derive(Default, Clone, Debug)]
struct State {
    executables: HashMap<String, Executable>,
}

#[derive(Default, Clone, Debug)]
struct Executable {
    data: Vec<u8>,
    name: String,
    key_start: usize,
    key_end: usize,
}

impl State {
    fn add_executable(&mut self, exe_type: String, exe_path: String) {
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
            name: filename,
            key_start: key_start,
            key_end: key_end,
        };

        self.executables.insert(exe_type, exe);
    }
    fn exe_with_key(&self, executable_type: &str, new_key: &[u8]) -> Vec<u8> {
        let exe = self.executables.get(executable_type).unwrap();

        let mut data = exe.data.clone();

        // Copy the key into the data
        for i in 0..new_key.len() {
            data[exe.key_start + i] = new_key[i];
        }

        // If the new key is shorter than the old key, we just write over the remaining data
        if new_key.len() < exe.key_end - exe.key_start {
            for i in exe.key_start + new_key.len()..exe.key_end {
                data[i] = b' ';
            }
        }

        return data;
    }
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "5800".to_string());
    let addr = format!("0.0.0.0:{}", port);
    tracing_subscriber::fmt().init();

    let mut state = State {
        executables: HashMap::new(),
    };

    state.add_executable(
        "windows".to_string(),
        "./demo/target/x86_64-pc-windows-gnu/release/demo.exe".to_string(),
    );
    state.add_executable(
        "linux".to_string(),
        "./demo/target/release/demo".to_string(),
    );

    let router = Router::new()
        .hoop(affix_state::inject(state))
        .get(hello)
        .push(Router::with_path("download").get(download));

    let service = Service::new(router).hoop(Logger::new());

    let acceptor = TcpListener::new(addr).bind().await;
    Server::new(acceptor).serve(service).await;
}
