use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
    sync::Arc,
};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::get,
    Router,
};
use config::Config;
use serde::Deserialize;
use tower_http::trace::TraceLayer;
use tracing::info;

use crate::thyme::{asn_prefixes::AsnPrefixEntry, rir_allocations::RirAllocationEntry};

#[derive(Deserialize)]
struct RirQuery {
    address: String,
}

async fn get_rir(
    Query(query): Query<RirQuery>,
    State(state): State<AppState>,
) -> Result<String, StatusCode> {
    match Ipv4Addr::from_str(&query.address.trim()) {
        Ok(address) => {
            let address_bits: u32 = address.into();

            for entry in state.rir_allocations.iter() {
                if entry.prefix.address_is_in(address_bits) {
                    return Ok(entry.rir.to_string());
                }
            }

            Err(StatusCode::NOT_FOUND)
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

#[derive(Deserialize)]
struct AsnQuery {
    address: String,
}

async fn get_asn(
    Query(query): Query<AsnQuery>,
    State(state): State<AppState>,
) -> Result<String, StatusCode> {
    match Ipv4Addr::from_str(&query.address.trim()) {
        Ok(address) => {
            let address_bits: u32 = address.into();

            for entry in state.asn_prefixes.iter() {
                if entry.prefix.address_is_in(address_bits) {
                    return Ok(entry.asn.to_string());
                }
            }

            Err(StatusCode::NOT_FOUND)
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

async fn index() -> &'static str {
    "Diglett API, v0.1.0"
}

#[derive(Clone)]
struct AppState {
    rir_allocations: Arc<Vec<RirAllocationEntry>>,
    asn_prefixes: Arc<Vec<AsnPrefixEntry>>,
}

pub async fn run(
    config: Config,
    rir_allocations: Arc<Vec<RirAllocationEntry>>,
    asn_prefixes: Arc<Vec<AsnPrefixEntry>>,
) {
    let state = AppState {
        rir_allocations,
        asn_prefixes,
    };

    let app = Router::new()
        .route("/", get(index))
        .route("/rir", get(get_rir))
        .route("/asn", get(get_asn))
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
