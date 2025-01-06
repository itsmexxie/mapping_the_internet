use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use concat_string::concat_string;
use config::Config;
use mtilib::auth::{GetJWTKeys, JWTKeys};
use serde::Serialize;
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};
use tokio::sync::Semaphore;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

use crate::diglett::Diglett;

#[derive(Serialize)]
struct UnitResponse {
    uuid: Option<Uuid>,
}

async fn unit(State(state): State<AppState>) -> Json<UnitResponse> {
    Json(UnitResponse {
        uuid: *state.unit_uuid,
    })
}

async fn health() -> impl IntoResponse {
    StatusCode::OK
}

async fn index() -> impl IntoResponse {
    concat_string!("Pidgey API, v", env!("CARGO_PKG_VERSION"))
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub unit_uuid: Arc<Option<Uuid>>,
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
    unit_uuid: Arc<Option<Uuid>>,
    jwt_keys: Arc<JWTKeys>,
    worker_permits: Arc<Semaphore>,
    diglett: Arc<Diglett>,
    ping_client: Arc<surge_ping::Client>,
) {
    let state = AppState {
        config: config.clone(),
        unit_uuid,
        jwt_keys,
        worker_permits,
        diglett,
        ping_client,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/_unit", get(unit))
        .route("/_health", get(health))
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
