use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use config::Config;
use mtilib::types::{AllocationState, Rir};
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::providers;

#[derive(Deserialize)]
struct AddressQuery {
    address: String,
}

#[derive(Serialize)]
struct ValueResponse<T>
where
    T: Serialize,
{
    value: T,
}

#[derive(Serialize)]
struct AllocationResponse {
    value: String,
}

async fn get_allocation(
    Query(query): Query<AddressQuery>,
    State(state): State<AppState>,
) -> Result<Json<AllocationResponse>, StatusCode> {
    match Ipv4Addr::from_str(query.address.trim()) {
        Ok(address) => {
            let address_bits: u32 = address.into();

            for entry in state.providers.read().await.iana.reserved.iter() {
                if entry.address_is_in(address_bits) {
                    return Ok(Json(AllocationResponse {
                        value: AllocationState::Reserved.to_string(),
                    }));
                }
            }

            for entry in state.providers.read().await.arin.stats.iter() {
                if entry.cidr.address_is_in(address_bits) {
                    return Ok(Json(AllocationResponse {
                        value: entry.allocation_state.to_string(),
                    }));
                }
            }

            Ok(Json(AllocationResponse {
                value: AllocationState::Unknown.to_string(),
            }))
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn get_rir(
    Query(query): Query<AddressQuery>,
    State(state): State<AppState>,
) -> Result<Json<ValueResponse<Option<String>>>, StatusCode> {
    match Ipv4Addr::from_str(query.address.trim()) {
        Ok(address) => {
            let address_bits: u32 = address.into();

            // First look up the ARIN stat files
            for entry in state.providers.read().await.arin.stats.iter() {
                if entry.cidr.address_is_in(address_bits) {
                    return Ok(Json(ValueResponse {
                        value: Some(entry.rir.to_string()),
                    }));
                }
            }

            // Use thyme allocations as fallback
            for entry in state.providers.read().await.thyme.rir.iter() {
                if entry.cidr.address_is_in(address_bits) {
                    return Ok(Json(ValueResponse {
                        value: Some(entry.rir.to_string()),
                    }));
                }
            }

            Ok(Json(ValueResponse { value: None }))
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn get_asn(
    Query(query): Query<AddressQuery>,
    State(state): State<AppState>,
) -> Result<Json<ValueResponse<Option<u32>>>, StatusCode> {
    match Ipv4Addr::from_str(query.address.trim()) {
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
    Query(query): Query<AddressQuery>,
    State(state): State<AppState>,
) -> Result<Json<ValueResponse<Option<String>>>, StatusCode> {
    match Ipv4Addr::from_str(query.address.trim()) {
        Ok(address) => {
            let address_bits: u32 = address.into();

            for entry in state.providers.read().await.arin.stats.iter() {
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
    "Diglett API, v0.1.0"
}

#[derive(Clone)]
struct AppState {
    providers: Arc<RwLock<providers::Providers>>,
}

pub async fn run(config: Config, providers: Arc<RwLock<providers::Providers>>) {
    let state = AppState { providers };

    let app = Router::new()
        .route("/", get(index))
        .route("/allocation", get(get_allocation))
        .route("/rir", get(get_rir))
        .route("/asn", get(get_asn))
        .route("/country", get(get_country))
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
