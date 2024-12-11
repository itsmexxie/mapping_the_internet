use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use config::Config;
use mtilib::types::AllocationState;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::providers;

#[derive(Serialize)]
struct ValueResponse<T>
where
    T: Serialize,
{
    value: T,
}

async fn get_allocation(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<ValueResponse<String>>, StatusCode> {
    match Ipv4Addr::from_str(address.trim()) {
        Ok(address) => {
            let address_bits: u32 = address.into();

            for entry in state.providers.read().await.iana.reserved.value.iter() {
                if entry.address_is_in(address_bits) {
                    return Ok(Json(ValueResponse {
                        value: AllocationState::Reserved.id(),
                    }));
                }
            }

            for entry in state.providers.read().await.arin.stats.value.iter() {
                if entry.cidr.address_is_in(address_bits) {
                    return Ok(Json(ValueResponse {
                        value: entry.allocation_state.id(),
                    }));
                }
            }

            Ok(Json(ValueResponse {
                value: AllocationState::Unknown.id(),
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
                for entry in state.providers.read().await.thyme.rir.iter() {
                    if entry.cidr.address_is_in(address_bits) {
                        return Ok(Json(ValueResponse {
                            value: Some(entry.rir.id().to_string()),
                        }));
                    }
                }
            } else {
                // First look up the IANA recovered addresses
                for entry in state.providers.read().await.iana.recovered.value.iter() {
                    if address_bits >= entry.start.to_bits() && address_bits <= entry.end.to_bits()
                    {
                        return Ok(Json(ValueResponse {
                            value: Some(entry.rir.id().to_string()),
                        }));
                    }
                }

                // Then look up the ARIN stat files
                for entry in state.providers.read().await.arin.stats.value.iter() {
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

            for entry in state.providers.read().await.thyme.asn.iter() {
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

            for entry in state.providers.read().await.arin.stats.value.iter() {
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

async fn index() -> &'static str {
    "Diglett API, v1.0.0"
}

#[derive(Clone)]
struct AppState {
    providers: Arc<RwLock<providers::Providers>>,
}

pub async fn run(config: Config, providers: Arc<RwLock<providers::Providers>>) {
    let state = AppState { providers };

    let app = Router::new()
        .route("/", get(index))
        .route("/:address/allocation", get(get_allocation))
        .route("/:address/rir", get(get_rir))
        .route("/:address/asn", get(get_asn))
        .route("/:address/country", get(get_country))
        .with_state(state)
        .layer(TraceLayer::new_for_http());
    let app_port = config.get("api.port").unwrap();

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        app_port,
    ))
    .await
    .unwrap();

    info!("Listening on port {}!", app_port);
    axum::serve(listener, app).await.unwrap();
}
