use axum::extract::{Path, State};
use axum::response::IntoResponse;
use axum::routing::get;
use axum::{http::StatusCode, routing::post, Json, Router};
use config::Config;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use mtilib::auth::{GetJWTKeys, JWTKeys};
use serde::Serialize;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

use crate::models::{Service, ServiceUnit};
use crate::schema::{ServiceUnits, Services};

pub mod auth;

#[derive(Serialize)]
struct ApiService {
    id: i32,
    name: String,
}

#[derive(Serialize)]
struct ApiServiceUnit {
    id: String,
    service_id: i32,
    address: Option<String>,
    port: Option<i32>,
}

async fn get_services(
    State(api_state): State<AppState>,
) -> Result<Json<Vec<ApiService>>, StatusCode> {
    let pg_conn = &mut crate::db::create_conn(&api_state.config);

    let query_results = Services::dsl::Services
        .select(Service::as_select())
        .load(pg_conn)
        .unwrap();

    let mut api_results = Vec::new();
    for query_result in query_results {
        api_results.push(ApiService {
            id: query_result.id,
            name: query_result.name,
        });
    }

    Ok(Json(api_results))
}

async fn get_service(
    Path(service_id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<ApiService>, StatusCode> {
    let pg_conn = &mut crate::db::create_conn(&state.config);

    let query_result = Services::dsl::Services
        .filter(Services::id.eq(service_id))
        .select(Service::as_select())
        .first(pg_conn)
        .unwrap();

    Ok(Json(ApiService {
        id: query_result.id,
        name: query_result.name,
    }))
}

async fn get_service_units(
    Path(service_id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ApiServiceUnit>>, StatusCode> {
    let pg_conn = &mut crate::db::create_conn(&state.config);

    let query_results: Vec<ServiceUnit> = ServiceUnits::dsl::ServiceUnits
        .filter(ServiceUnits::service_id.eq(service_id))
        .select(ServiceUnit::as_select())
        .load(pg_conn)
        .unwrap();

    let mut api_results = Vec::new();
    for result in query_results {
        api_results.push(ApiServiceUnit {
            id: result.id,
            service_id: result.service_id,
            address: result.address,
            port: result.port,
        })
    }

    Ok(Json(api_results))
}

#[derive(Serialize)]
struct UnitResponse {
    uuid: Uuid,
}

async fn unit(State(state): State<AppState>) -> Json<UnitResponse> {
    Json(UnitResponse { uuid: *state.uuid })
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
    pub uuid: Arc<Uuid>,
}

impl GetJWTKeys for AppState {
    fn get_jwt_keys(&self) -> impl AsRef<JWTKeys> {
        self.jwt_keys.to_owned()
    }
}

pub async fn run(config: Arc<Config>, jwt_keys: Arc<JWTKeys>, uuid: Arc<Uuid>) {
    let state = AppState {
        config: config.clone(),
        jwt_keys,
        uuid,
    };

    let app = Router::new()
        .nest(
            "/services",
            Router::new()
                .route("/", get(get_services))
                .route("/:service_id", get(get_service))
                .route("/:service_id/units", get(get_service_units))
                .layer(axum::middleware::from_fn_with_state(
                    state.clone(),
                    mtilib::auth::axum_middleware::<AppState>,
                )),
        )
        .nest(
            "/auth",
            Router::new()
                .route("/login", post(auth::login_index))
                .route(
                    "/logout",
                    post(auth::logout_index).layer(axum::middleware::from_fn_with_state(
                        state.clone(),
                        mtilib::auth::axum_middleware::<AppState>,
                    )),
                ),
        )
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
