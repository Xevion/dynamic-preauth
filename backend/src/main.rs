use dynamic_preauth::config::Config;
use dynamic_preauth::handlers::{
    connect, download, get_build_logs, get_session, notify, session_middleware,
};
use dynamic_preauth::railway;
use dynamic_preauth::state::STORE;

use salvo::cors::Cors;
use salvo::http::Method;
use salvo::logging::Logger;
use salvo::prelude::{CatchPanic, Listener, Router, Server, Service, StaticDir, TcpListener};
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() {
    // Load environment variables from .env file (development only)
    #[cfg(debug_assertions)]
    dotenvy::dotenv().ok();

    // Parse configuration from environment
    let config: Config = envy::from_env().expect("Failed to parse environment configuration");

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::new(format!(
            "info,dynamic_preauth={}",
            if cfg!(debug_assertions) {
                "debug"
            } else {
                "info"
            }
        )))
        .init();

    // Add the build log & executables to the store
    let mut store = STORE.lock().await;

    // Check if we are deployed on Railway
    if config.railway.is_railway() {
        if let Some(build_logs_url) = config.railway.build_logs_url() {
            tracing::info!("Build logs available here: {}", build_logs_url);
            store.build_log_url = Some(build_logs_url);
        }

        // Try to fetch actual build logs using Railway API
        if config.railway.has_token() {
            match railway::fetch_build_logs().await {
                Ok(build_logs) => {
                    tracing::info!(
                        "Successfully fetched build logs ({} bytes)",
                        build_logs.content.len()
                    );
                    store.build_logs = Some(build_logs);
                }
                Err(e) => {
                    tracing::warn!("Failed to fetch build logs from Railway API: {}", e);
                }
            }
        } else {
            tracing::warn!("RAILWAY_TOKEN not set, skipping build log fetch");
        }
    }

    for (exe_type, exe_path) in [
        ("Windows", "./demo.exe"),
        ("Linux", "./demo-linux"),
        // ("MacOS", "./demo-macos"),
    ] {
        if let Err(e) = store.add_executable(exe_type, exe_path) {
            // In debug mode, allow missing Windows executable for dev convenience
            if cfg!(debug_assertions) && exe_type == "Windows" {
                tracing::warn!(
                    "Windows executable not found at {} (skipping - cross-compilation not set up)",
                    exe_path
                );
                tracing::warn!("To enable Windows builds: rustup target add x86_64-pc-windows-gnu && sudo apt install mingw-w64");
                continue;
            }

            tracing::error!("{}", e);
            std::process::exit(1);
        }
    }

    drop(store); // critical: Drop the lock to avoid deadlock, otherwise the server will hang

    let origin = config.railway.cors_origin();
    let cors = Cors::new()
        .allow_origin(&origin)
        .allow_methods(vec![Method::GET])
        .into_handler();
    tracing::debug!("CORS Allowed Origin: {}", &origin);

    let static_dir = StaticDir::new(["./public"]).defaults("index.html");

    // TODO: Improved Token Generation
    // TODO: Advanced HMAC Verification
    // TODO: Session Purging

    let router = Router::new()
        .hoop(CatchPanic::new())
        // /notify does not need a session, nor should it have one
        .push(Router::with_path("notify").post(notify))
        // /build-logs does not need a session
        .push(Router::with_path("build-logs").get(get_build_logs))
        .push(
            Router::new()
                .hoop(session_middleware)
                .push(Router::with_path("download/<id>").get(download))
                .push(Router::with_path("session").get(get_session))
                // websocket /ws
                .push(Router::with_path("ws").goal(connect))
                // static files
                .push(Router::with_path("<**path>").get(static_dir)),
        );

    let service = Service::new(router).hoop(cors).hoop(Logger::new());

    let bind_addr = config.bind_addr();
    tracing::info!("Server starting on http://{}", bind_addr);
    tracing::info!("WebSocket endpoint: ws://{}/ws", bind_addr);

    if cfg!(debug_assertions) {
        tracing::info!("Development mode - CORS allows all origins");
        tracing::info!("Access the app at http://localhost:4321 (Astro dev server)");
    }

    let acceptor = TcpListener::new(&bind_addr).bind().await;
    Server::new(acceptor).serve(service).await;
}
