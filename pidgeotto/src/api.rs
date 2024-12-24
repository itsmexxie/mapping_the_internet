use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    extract::State,
    http::StatusCode,
    response::IntoResponse,
    routing::{any, get},
    Json, Router,
};
use config::Config;
use mtilib::auth::{GetJWTKeys, JWTKeys};
use serde::Serialize;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

use crate::pidgey::Pidgey;

pub mod ws;

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
    concat_string!("Diglett API, v", env!("CARGO_PKG_VERSION"))
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub unit_uuid: Arc<Uuid>,
    pub jwt_keys: Arc<JWTKeys>,
    pub pidgey: Arc<Pidgey>,
}

impl GetJWTKeys for AppState {
    fn get_jwt_keys(&self) -> impl AsRef<JWTKeys> {
        self.jwt_keys.to_owned()
    }
}

pub struct ApiOptions {
    pub config: Arc<Config>,
    pub unit_uuid: Arc<Uuid>,
    pub jwt_keys: Arc<JWTKeys>,
    pub pidgey: Arc<Pidgey>,
}

pub async fn run(options: ApiOptions) {
    let state = AppState {
        config: options.config.clone(),
        unit_uuid: options.unit_uuid,
        jwt_keys: options.jwt_keys,
        pidgey: options.pidgey,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/_unit", get(unit))
        .route("/_health", get(health))
        .route(
            "/ws",
            any(ws::ws_handler).layer(axum::middleware::from_fn_with_state(
                state.clone(),
                mtilib::auth::axum_middleware::<AppState>,
            )),
        )
        .with_state(state)
        .layer(TraceLayer::new_for_http());
    let app_port = options
        .config
        .get("api.port")
        .expect("api.port must be set!");

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        app_port,
    ))
    .await
    .unwrap();

    info!("Listening on port {}!", app_port);
    axum::serve(listener, app).await.unwrap();
}
