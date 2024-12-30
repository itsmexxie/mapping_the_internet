use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{extract::State, http::StatusCode, response::IntoResponse, routing::get, Json, Router};
use concat_string::concat_string;
use serde::Serialize;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

use crate::settings::Settings;

pub mod address;

#[derive(Serialize)]
struct UnitResponse {
    uuid: Uuid,
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
    concat_string!("Cubone API, v", env!("CARGO_PKG_VERSION"))
}

#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub unit_uuid: Arc<Uuid>,
}

pub async fn run(settings: Arc<Settings>, unit_uuid: Arc<Uuid>) {
    let state = AppState {
        settings: settings.clone(),
        unit_uuid,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/_unit", get(unit))
        .route("/_health", get(health))
        .nest("/address", address::router())
        .with_state(state)
        .layer(TraceLayer::new_for_http());
    let app_port = settings.api.port;

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        app_port,
    ))
    .await
    .unwrap();

    info!("Listening on port {}!", app_port);
    axum::serve(listener, app).await.unwrap();
}
