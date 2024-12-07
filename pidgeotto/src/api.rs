use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{routing::any, Router};
use config::Config;
use mtilib::auth::JWTKeys;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::pidgey::Pidgey;

pub mod ws;

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub jwt_keys: Arc<JWTKeys>,
    pub pidgey: Arc<Pidgey>,
}

pub async fn run(config: Arc<Config>, jwt_keys: Arc<JWTKeys>, pidgey: Arc<Pidgey>) {
    let app_state = AppState {
        config: config.clone(),
        jwt_keys,
        pidgey,
    };

    let app = Router::new()
        .route("/ws", any(ws::ws_handler))
        .with_state(app_state)
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
