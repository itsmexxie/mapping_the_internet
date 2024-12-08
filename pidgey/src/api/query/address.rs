use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};

use axum::{
    extract::{Path, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::IntoResponse,
    routing::get,
    Extension, Json, Router,
};
use mtilib::types::ValueResponse;
use rand::random;
use serde::{Deserialize, Serialize};
use surge_ping::{PingIdentifier, PingSequence, SurgeError};

use crate::{api::AppState, gust::Gust};

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
            value: state.id().to_string(),
        })),
        Err(status) => Err(status),
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
    match state.diglett.rir(address, query.top).await {
        Ok(rir) => match rir {
            Some(rir) => Ok(Json(ValueResponse {
                value: Some(rir.id().to_string()),
            })),
            None => Ok(Json(ValueResponse { value: None })),
        },
        Err(status) => Err(status),
    }
}

pub async fn asn(
    Extension(address): Extension<Ipv4Addr>,
    state: State<AppState>,
) -> Result<Json<ValueResponse<Option<u32>>>, StatusCode> {
    match state.diglett.asn(address).await {
        Ok(asn) => Ok(Json(ValueResponse { value: asn })),
        Err(status) => Err(status),
    }
}

pub async fn country(
    Extension(address): Extension<Ipv4Addr>,
    state: State<AppState>,
) -> Result<Json<ValueResponse<Option<String>>>, StatusCode> {
    match state.diglett.country(address).await {
        Ok(country) => Ok(Json(ValueResponse { value: country })),
        Err(status) => Err(status),
    }
}

#[derive(Serialize)]
pub struct OnlineResponse {
    value: bool,
    reason: Option<String>,
}

pub async fn online(
    Extension(address): Extension<Ipv4Addr>,
    State(state): State<AppState>,
) -> Result<Json<OnlineResponse>, StatusCode> {
    let payload = [0; 8];
    let mut pinger = state
        .ping_client
        .pinger(IpAddr::V4(address), PingIdentifier(random()))
        .await;
    match pinger.ping(PingSequence(0), &payload).await {
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
                        state.config.get_int("settings.gust.timeout").unwrap_or(5) as u32,
                    )
                    .await;

                Ok(Json(ValueResponse { value: port_online }))
            }
            Err(_) => Err(StatusCode::BAD_REQUEST),
        },
        Err(_) => Err(StatusCode::BAD_REQUEST),
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
    Err(StatusCode::NOT_IMPLEMENTED)
    // match Gust::new(address) {
    //     Ok(gust) => {
    //         let gust = Arc::new(gust);

    //         let gust_range_start = match query.start {
    //             Some(start) => start,
    //             None => match state.config.get_int("settings.gust.range.start") {
    //                 Ok(start) => start as u16,
    //                 Err(_) => 1,
    //             },
    //         };

    //         let gust_range_end = match query.end {
    //             Some(end) => end,
    //             None => match state.config.get_int("settings.gust.range.end") {
    //                 Ok(end) => end as u16,
    //                 Err(_) => 999,
    //             },
    //         };

    //         Ok(Json(ValueResponse {
    //             value: gust
    //                 .attack_range(
    //                     gust_range_start..=gust_range_end,
    //                     state.config.get_int("settings.gust.timeout").unwrap_or(10) as u32,
    //                 )
    //                 .await
    //                 .unwrap(),
    //         }))
    //     }
    //     Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    // }
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
