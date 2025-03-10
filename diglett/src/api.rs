use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    middleware,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use mtilib::{
    auth::{GetJWTKeys, JWTKeys},
    types::{AllocationState, ValueResponse},
};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
};
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::info;
use uuid::Uuid;

use crate::{providers::Providers, settings::Settings};

async fn get_allocation(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ValueResponse<String>>, StatusCode> {
    match Ipv4Addr::from_str(address.trim()) {
        Ok(address) => {
            let address_bits: u32 = address.into();

            for entry in state.providers.read().await.iana.reserved.values.iter() {
                if entry.address_is_in(address_bits) {
                    return Ok(Json(ValueResponse {
                        value: AllocationState::Reserved.id().to_string(),
                    }));
                }
            }

            for entry in state.providers.read().await.stats.values.iter() {
                if entry.cidr.address_is_in(address_bits) {
                    return Ok(Json(ValueResponse {
                        value: entry.allocation_state.id().to_string(),
                    }));
                }
            }

            Ok(Json(ValueResponse {
                value: AllocationState::Unknown.id().to_string(),
            }))
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

#[derive(Deserialize)]
struct RirQuery {
    #[serde(default)]
    top: bool,
}

async fn get_rir(
    Path(address): Path<String>,
    Query(query): Query<RirQuery>,
    State(state): State<AppState>,
) -> Result<Json<ValueResponse<Option<String>>>, StatusCode> {
    match Ipv4Addr::from_str(address.trim()) {
        Ok(address) => {
            let address_bits: u32 = address.into();

            if query.top {
                // Use thyme allocations as top
                for entry in state
                    .providers
                    .read()
                    .await
                    .thyme
                    .rir_allocations
                    .values
                    .iter()
                {
                    if entry.cidr.address_is_in(address_bits) {
                        return Ok(Json(ValueResponse {
                            value: Some(entry.rir.id().to_string()),
                        }));
                    }
                }
            } else {
                // First look up the IANA recovered addresses
                for entry in state.providers.read().await.iana.recovered.values.iter() {
                    if address_bits >= entry.start.to_bits() && address_bits <= entry.end.to_bits()
                    {
                        return Ok(Json(ValueResponse {
                            value: Some(entry.rir.id().to_string()),
                        }));
                    }
                }

                // Then look up the ARIN stat files
                for entry in state.providers.read().await.stats.values.iter() {
                    if entry.cidr.address_is_in(address_bits) {
                        return Ok(Json(ValueResponse {
                            value: Some(entry.rir.id().to_string()),
                        }));
                    }
                }
            }

            Ok(Json(ValueResponse { value: None }))
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn get_asn(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ValueResponse<Option<u32>>>, StatusCode> {
    match Ipv4Addr::from_str(address.trim()) {
        Ok(address) => {
            let address_bits: u32 = address.into();

            for entry in state
                .providers
                .read()
                .await
                .thyme
                .asn_prefixes
                .values
                .iter()
            {
                if entry.cidr.address_is_in(address_bits) {
                    return Ok(Json(ValueResponse {
                        value: Some(entry.asn),
                    }));
                }
            }

            Ok(Json(ValueResponse { value: None }))
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn get_country(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ValueResponse<Option<String>>>, StatusCode> {
    match Ipv4Addr::from_str(address.trim()) {
        Ok(address) => {
            let address_bits: u32 = address.into();

            for entry in state.providers.read().await.stats.values.iter() {
                if entry.cidr.address_is_in(address_bits) {
                    return Ok(Json(ValueResponse {
                        value: entry.country.to_owned(),
                    }));
                }
            }

            Ok(Json(ValueResponse { value: None }))
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

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
struct AppState {
    unit_uuid: Arc<Option<Uuid>>,
    jwt_keys: Option<Arc<JWTKeys>>,
    providers: Arc<RwLock<Providers>>,
}

impl GetJWTKeys for AppState {
    fn get_jwt_keys(&self) -> impl AsRef<JWTKeys> {
        self.jwt_keys.to_owned().unwrap()
    }
}

pub async fn run(
    settings: Arc<Settings>,
    unit_uuid: Arc<Option<Uuid>>,
    jwt_keys: Option<Arc<JWTKeys>>,
    providers: Arc<RwLock<Providers>>,
) {
    let state = AppState {
        unit_uuid,
        jwt_keys,
        providers,
    };

    let mut address_router = Router::new()
        .route("/allocation", get(get_allocation))
        .route("/rir", get(get_rir))
        .route("/asn", get(get_asn))
        .route("/country", get(get_country));

    if settings.api.auth {
        address_router = address_router.layer(middleware::from_fn_with_state(
            state.clone(),
            mtilib::auth::axum_middleware::<AppState>,
        ))
    }

    let app = Router::new()
        .route("/", get(index))
        .route("/_unit", get(unit))
        .route("/_health", get(health))
        .nest("/{address}", address_router)
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
