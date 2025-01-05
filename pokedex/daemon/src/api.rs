use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{http::StatusCode, Json, Router};
use concat_string::concat_string;
use config::Config;
use mtilib::auth::{GetJWTKeys, JWTKeys};
use mtilib::db::DbPool;
use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

use mtilib::db::models::{Service, ServiceUnit};

pub mod auth;

async fn get_services(State(state): State<AppState>) -> Result<Json<Vec<Service>>, StatusCode> {
    match sqlx::query_as::<_, Service>(
        r#"
		SELECT *
		FROM "Services"
		"#,
    )
    .fetch_all(&mut *state.db_pool.acquire().await.unwrap())
    .await
    {
        Ok(row) => Ok(Json(row)),
        Err(error) => match error {
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

async fn get_service(
    Path(service_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Service>, StatusCode> {
    match sqlx::query_as::<_, Service>(
        r#"
		SELECT *
		FROM "Services"
		WHERE id = $1
		"#,
    )
    .bind(service_id)
    .fetch_one(&mut *state.db_pool.acquire().await.unwrap())
    .await
    {
        Ok(row) => Ok(Json(row)),
        Err(error) => match error {
            sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

async fn get_service_units(
    Path(service_id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ServiceUnit>>, StatusCode> {
    match sqlx::query_as::<_, ServiceUnit>(
        r#"
		SELECT *
		FROM "ServiceUnits"
		WHERE service_id = $1
		"#,
    )
    .bind(service_id)
    .fetch_all(&mut *state.db_pool.acquire().await.unwrap())
    .await
    {
        Ok(rows) => Ok(Json(rows)),
        Err(error) => match error {
            sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
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
    concat_string!("Pokedex API, v", env!("CARGO_PKG_VERSION"))
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
        .nest(
            "/services",
            Router::new()
                .route("/", get(get_services))
                .route("/{service_id}", get(get_service))
                .route("/{service_id}/units", get(get_service_units))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    mtilib::auth::axum_middleware::<AppState>,
                )),
        )
        .nest("/auth", auth::router(state.clone()))
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
