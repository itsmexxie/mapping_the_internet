use axum::extract::State;
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{http::StatusCode, Json, Router};
use config::Config;
use mtilib::auth::{GetJWTKeys, JWTKeys};
use mtilib::db::DbPool;
use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

pub mod v1;
pub mod v2;

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
    format!("Pokedex API, v{}", env!("CARGO_PKG_VERSION"))
}

#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub jwt_keys: Arc<JWTKeys>,
    pub unit_uuid: Arc<Uuid>,
    pub db_pool: DbPool,
}

impl GetJWTKeys for AppState {
    fn get_jwt_keys(&self) -> impl AsRef<JWTKeys> {
        self.jwt_keys.to_owned()
    }
}

pub async fn run(
    config: Arc<Config>,
    jwt_keys: Arc<JWTKeys>,
    unit_uuid: Arc<Uuid>,
    db_pool: DbPool,
) {
    let state = AppState {
        config: config.clone(),
        jwt_keys,
        unit_uuid,
        db_pool,
    };

    let app = Router::new()
        .nest("/v1", v1::router(state.clone()))
        .nest("/v2", v2::router())
        .route("/_unit", get(unit))
        .route("/_health", get(health))
        .route("/", get(index))
        .with_state(state)
        .layer(TraceLayer::new_for_http());

    let app_port = config.get("api.port").expect("api.port must be set!");

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        app_port,
    ))
    .await
    .unwrap();

    info!("Listening on port {}", app_port);
    axum::serve(listener, app).await.unwrap();
}
