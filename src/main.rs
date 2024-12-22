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
    let data = state.with_key(b"test"); // Clone the data

    if let Err(e) = res.write_body(data) {
        eprintln!("Error writing body: {}", e);
    }

    res.headers.insert(
        "Content-Disposition",
        HeaderValue::from_static("attachment; filename=demo"),
    );
    res.headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/octet-stream"),
    );
}

#[allow(dead_code)]
#[derive(Default, Clone, Debug)]
struct State {
    data: Vec<u8>,
    key_start: usize,
    key_end: usize,
}

impl State {
    fn with_key(&self, new_key: &[u8]) -> Vec<u8> {
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

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "5800".to_string());
    let addr = format!("0.0.0.0:{}", port);
    tracing_subscriber::fmt().init();

    let path = "./demo/target/release/demo";
    let data = std::fs::read(&path).expect("Unable to read file");

    let pattern = "a".repeat(1024);
    let key_start = search(&data, pattern.as_bytes(), 0).unwrap();
    let key_end = key_start + pattern.len();

    let state = State {
        data,
        key_start: key_start,
        key_end: key_end,
    };

    let router = Router::new()
        .hoop(affix_state::inject(state))
        .get(hello)
        .push(Router::with_path("download").get(download));

    let service = Service::new(router).hoop(Logger::new());

    let acceptor = TcpListener::new(addr).bind().await;
    Server::new(acceptor).serve(service).await;
}
