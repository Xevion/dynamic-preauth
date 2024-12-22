use std::collections::HashMap;

use salvo::http::HeaderValue;
use salvo::logging::Logger;
use salvo::prelude::{
    affix_state, handler, Depot, Listener, Request, Response, Router, Server, Service, StaticDir,
    TcpListener,
};

use crate::models::State;

mod models;
mod utility;

#[handler]
async fn download(depot: &mut Depot, req: &mut Request, res: &mut Response) {
    let article_id = req.param::<String>("id").unwrap();

    let state = depot.obtain::<State>().unwrap();
    let executable = state.executables.get(&article_id as &str).unwrap();
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

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "5800".to_string());
    let addr = format!("0.0.0.0:{}", port);
    tracing_subscriber::fmt().init();

    let mut state = State {
        executables: HashMap::new(),
    };

    state.add_executable("windows", "./demo-windows.exe");
    state.add_executable("linux", "./demo-linux");

    let static_dir = StaticDir::new(["./public"]).defaults("index.html");

    let router = Router::new()
        .hoop(affix_state::inject(state))
        .push(Router::with_path("download/<id>").get(download))
        .push(Router::with_path("<**path>").get(static_dir));

    let service = Service::new(router).hoop(Logger::new());

    let acceptor = TcpListener::new(addr).bind().await;
    Server::new(acceptor).serve(service).await;
}
