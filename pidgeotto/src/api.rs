use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{routing::any, Router};
use config::Config;
use tower_http::trace::TraceLayer;
use tracing::info;

pub mod ws;

pub async fn run(config: Arc<Config>) {
    let app = Router::new()
        .route("/ws", any(ws::ws_handler))
        .layer(TraceLayer::new_for_http());
    let app_port = config.get("api.port").expect("api.port must be set!");

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        app_port,
    ))
    .await
    .unwrap();

    info!("Listening on port {}!", app_port);
    axum::serve(listener, app).await.unwrap();
}
