use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
    sync::Arc,
};

use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};
use serde::{Deserialize, Serialize};
use surge_ping::SurgeError;
use tokio::sync::Mutex;

use crate::{api::AppState, gust::Gust, utils::ValueResponse};

pub async fn address_middleware(
    address: Path<String>,
    mut request: Request,
    next: Next,
) -> impl IntoResponse {
    match Ipv4Addr::from_str(&address) {
        Ok(address) => {
            request.extensions_mut().insert(address);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn address(State(state): State<AppState>) -> impl IntoResponse {}

pub async fn allocation(
    Extension(address): Extension<Ipv4Addr>,
    state: State<AppState>,
) -> Result<Json<ValueResponse<String>>, StatusCode> {
    match state.diglett.allocation_state(address).await {
        Ok(state) => Ok(Json(ValueResponse {
            value: state.to_string(),
        })),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

#[derive(Deserialize)]
pub struct RirQuery {
    #[serde(default)]
    pub top: bool,
}

pub async fn rir(
    Extension(address): Extension<Ipv4Addr>,
    query: Query<RirQuery>,
    state: State<AppState>,
) -> Result<Json<ValueResponse<Option<String>>>, StatusCode> {
    Ok(Json(ValueResponse {
        value: state.diglett.rir(address, query.top).await,
    }))
}

pub async fn asn(
    Extension(address): Extension<Ipv4Addr>,
    state: State<AppState>,
) -> Result<Json<ValueResponse<Option<u32>>>, StatusCode> {
    Ok(Json(ValueResponse {
        value: state.diglett.asn(address).await,
    }))
}

pub async fn country(
    Extension(address): Extension<Ipv4Addr>,
    state: State<AppState>,
) -> Result<Json<ValueResponse<Option<String>>>, StatusCode> {
    Ok(Json(ValueResponse {
        value: state.diglett.country(address).await,
    }))
}

#[derive(Serialize)]
pub struct OnlineResponse {
    value: bool,
    reason: Option<String>,
}

pub async fn online(
    Extension(address): Extension<Ipv4Addr>,
) -> Result<Json<OnlineResponse>, StatusCode> {
    let payload = [0; 8];
    match surge_ping::ping(IpAddr::V4(address), &payload).await {
        Ok(_) => Ok(Json(OnlineResponse {
            value: true,
            reason: None,
        })),
        Err(ping_error) => match ping_error {
            SurgeError::Timeout { seq: _ } => Ok(Json(OnlineResponse {
                value: false,
                reason: Some(String::from("timeout")),
            })),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

#[derive(Deserialize)]
pub struct PortRangeQuery {
    pub start: Option<u16>,
    pub end: Option<u16>,
}

pub async fn port_range(
    Extension(address): Extension<Ipv4Addr>,
    query: Query<PortRangeQuery>,
    state: State<AppState>,
) -> Result<Json<ValueResponse<HashMap<u16, bool>>>, StatusCode> {
    match Gust::new(address) {
        Ok(gust) => {
            let gust = Arc::new(gust);

            let gust_range_start = match query.start {
                Some(start) => start,
                None => match state.config.get_int("settings.gust.range.start") {
                    Ok(start) => start as u16,
                    Err(_) => 1,
                },
            };

            let gust_range_end = match query.end {
                Some(end) => end,
                None => match state.config.get_int("settings.gust.range.end") {
                    Ok(end) => end as u16,
                    Err(_) => 999,
                },
            };

            // Values from different ports
            // This is stored in a hashmap because we theoretically want to query nonconsecutive ports
            let ports = Arc::new(Mutex::new(HashMap::new()));
            let mut port_tasks = Vec::new();

            // Try the ports parallely
            for port in gust_range_start..=gust_range_end {
                let cloned_ports = ports.clone();
                let cloned_gust = gust.clone();
                let cloned_state = state.clone();
                port_tasks.push(tokio::spawn(async move {
                    let result = cloned_gust
                        .attack(
                            port,
                            cloned_state
                                .config
                                .get_int("settings.gust.timeout")
                                .unwrap_or(10) as u32,
                        )
                        .await;

                    cloned_ports.lock().await.insert(port, result);
                }));
            }

            for port_task in port_tasks {
                port_task.await.unwrap();
            }

            Ok(Json(ValueResponse {
                value: match Arc::try_unwrap(ports) {
                    Ok(ports) => ports.into_inner(),
                    Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                },
            }))
        }
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

pub async fn port(
    Path((address, port)): Path<(String, u16)>,
    state: State<AppState>,
) -> Result<Json<ValueResponse<bool>>, StatusCode> {
    match Ipv4Addr::from_str(&address) {
        Ok(address) => match Gust::new(address) {
            Ok(gust) => {
                let port_online = gust
                    .attack(
                        port,
                        state.config.get_int("settings.gust.timeout").unwrap_or(10) as u32,
                    )
                    .await;

                Ok(Json(ValueResponse { value: port_online }))
            }
            Err(_) => Err(StatusCode::BAD_REQUEST),
        },
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn index() -> impl IntoResponse {
    "Please specify an address!"
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(index))
        .route("/:address", get(address))
        .route("/:address/allocation", get(allocation))
        .route("/:address/rir", get(rir))
        .route("/:address/asn", get(asn))
        .route("/:address/country", get(country))
        .route("/:address/online", get(online))
        .route("/:address/port", get(port_range))
        .layer(axum::middleware::from_fn(address_middleware))
        .route("/:address/port/:port", get(port))
}
