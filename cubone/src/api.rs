use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use concat_string::concat_string;
use serde::Serialize;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

use crate::settings::Settings;

pub mod address;
pub mod map;

pub async fn access_control_header(req: Request<Body>, next: Next) -> impl IntoResponse {
    let mut res = next.run(req).await;
    res.headers_mut()
        .insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());
    res
}

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
    pub db_pool: mtilib::db::DbPool,
}

pub async fn run(settings: Arc<Settings>, unit_uuid: Arc<Uuid>, db_pool: mtilib::db::DbPool) {
    let state = AppState {
        settings: settings.clone(),
        unit_uuid,
        db_pool,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/_unit", get(unit))
        .route("/_health", get(health))
        .nest("/address", address::router())
        .nest("/map", map::router())
        .with_state(state)
        .layer(axum::middleware::from_fn(access_control_header))
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
