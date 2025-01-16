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
use concat_string::concat_string;
use mtilib::auth::{GetJWTKeys, JWTKeys};
use serde::Serialize;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

use crate::{pidgey::Pidgey, settings::Settings};

pub mod ws;

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
    concat_string!("Diglett API, v", env!("CARGO_PKG_VERSION"))
}

#[derive(Clone)]
pub struct AppState {
    pub settings: Arc<Settings>,
    pub unit_uuid: Arc<Option<Uuid>>,
    pub jwt_keys: Arc<JWTKeys>,
    pub pidgey: Arc<Pidgey>,
}

impl GetJWTKeys for AppState {
    fn get_jwt_keys(&self) -> impl AsRef<JWTKeys> {
        self.jwt_keys.to_owned()
    }
}

pub async fn run(
    settings: Arc<Settings>,
    unit_uuid: Arc<Option<Uuid>>,
    jwt_keys: Arc<JWTKeys>,
    pidgey: Arc<Pidgey>,
) {
    let state = AppState {
        settings: settings.clone(),
        unit_uuid,
        jwt_keys,
        pidgey,
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

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        settings.api.port,
    ))
    .await
    .unwrap();

    info!("Listening on port {}!", settings.api.port);
    axum::serve(listener, app).await.unwrap();
}
