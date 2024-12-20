use axum::{routing::get, Router};
use config::Config;
use mtilib::auth::{GetJWTKeys, JWTKeys};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::Semaphore;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::diglett::Diglett;

pub mod query;

async fn index() -> &'static str {
    "Pidgey API, v0.1.0"
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub jwt_keys: Arc<JWTKeys>,
    pub worker_permits: Arc<Semaphore>,
    pub diglett: Arc<Diglett>,
    pub ping_client: Arc<surge_ping::Client>,
}

impl GetJWTKeys for AppState {
    fn get_jwt_keys(&self) -> impl AsRef<JWTKeys> {
        self.jwt_keys.clone()
    }
}

pub async fn run(
    config: Arc<Config>,
    jwt_keys: Arc<JWTKeys>,
    worker_permits: Arc<Semaphore>,
    diglett: Arc<Diglett>,
    ping_client: Arc<surge_ping::Client>,
) {
    let state = AppState {
        config: config.clone(),
        jwt_keys,
        worker_permits,
        diglett,
        ping_client,
    };

    let app = Router::new()
        .route("/", get(index))
        .nest("/query", query::router(state.clone()))
        .with_state(state)
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
