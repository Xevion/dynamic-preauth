use salvo::prelude::*;

#[handler]
async fn hello() -> &'static str {
    "Hello World"
}

#[tokio::main]
async fn main() {
    let port = std::env::var("PORT").unwrap_or_else(|_| "5800".to_string());
    let addr = format!("0.0.0.0:{}", port);
    tracing_subscriber::fmt().init();

    let router = Router::new().get(hello);
    let acceptor = TcpListener::new(addr).bind().await;
    Server::new(acceptor).serve(router).await;
}
