use std::sync::LazyLock;

use salvo::http::HeaderValue;
use salvo::logging::Logger;
use salvo::prelude::{
    handler, Listener, Request, Response, Router, Server, Service, StaticDir, TcpListener,
};
use salvo::writing::Json;
use tokio::sync::Mutex;

use crate::models::State;

static STORE: LazyLock<Mutex<State>> = LazyLock::new(State::new);

mod models;
mod utility;

#[handler]
async fn session_middleware(req: &mut Request, res: &mut Response) {
    match req.cookie("Session") {
        Some(cookie) => {
            // Check if the session exists
            match cookie.value().parse::<usize>() {
                Ok(session_id) => {
                    let mut store = STORE.lock().await;
                    if !store.sessions.contains_key(&session_id) {
                        store.new_session(res).await;
                    }
                }
                Err(_) => {
                    let mut store = STORE.lock().await;
                    store.new_session(res).await;
                }
            }
        }
        None => {
            let mut store = STORE.lock().await;
            store.new_session(res).await;
        }
    }
}

#[handler]
pub async fn download(req: &mut Request, res: &mut Response) {
    let article_id = req.param::<String>("id").unwrap();

    let store = STORE.lock().await;
    let executable = store.executables.get(&article_id as &str).unwrap();
    let data = executable.with_key(b"test");

    if let Err(e) = res.write_body(data) {
        eprintln!("Error writing body: {}", e);
    }

    res.headers.insert(
        "Content-Disposition",
        HeaderValue::from_str(format!("attachment; filename=\"{}\"", executable.filename).as_str())
            .unwrap(),
    );
    res.headers.insert(
        "Content-Type",
        HeaderValue::from_static("application/octet-stream"),
    );
}

#[handler]
pub async fn get_session(req: &mut Request, res: &mut Response) {
    let store = STORE.lock().await;

    let session_id = req
        .cookie("Session")
        .unwrap()
        .value()
        .parse::<usize>()
        .unwrap();
    let session = store.sessions.get(&session_id).unwrap();

    res.render(Json(&session));
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "5800".to_string());
    let addr = format!("0.0.0.0:{}", port);
    tracing_subscriber::fmt().init();

    let mut store = STORE.lock().await;
    store.add_executable("windows", "./demo-windows.exe");
    store.add_executable("linux", "./demo-linux");
    drop(store);

    let static_dir = StaticDir::new(["./public"]).defaults("index.html");

    let router = Router::new()
        .hoop(session_middleware)
        .push(Router::with_path("download/<id>").get(download))
        .push(Router::with_path("session").get(get_session))
        .push(Router::with_path("<**path>").get(static_dir));

    let service = Service::new(router).hoop(Logger::new());

    let acceptor = TcpListener::new(addr).bind().await;
    Server::new(acceptor).serve(service).await;
}
