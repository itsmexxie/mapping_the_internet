use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::Router;
use config::Config;
use tracing::info;

#[tokio::main]
async fn main() {
    // Logging
    tracing_subscriber::fmt::init();

    // Config
    let config = Config::builder()
        .add_source(config::File::with_name("daemon.config.toml"))
        .build()
        .unwrap();

    // Axum API
    let app = Router::new();
    let app_port = config.get("api.port").unwrap();

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        app_port,
    ))
    .await
    .unwrap();

    info!("[API] Listening on port {}", app_port);
    axum::serve(listener, app).await.unwrap();
}
