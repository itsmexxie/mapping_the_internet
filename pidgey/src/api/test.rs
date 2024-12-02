use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Serialize;
use std::{
    collections::BTreeMap,
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};
use surge_ping::SurgeError;

use crate::gust::Gust;

use super::AppState;

#[derive(Serialize)]
pub struct IndexResponse {
    pub allocation_state: String,
    pub top_rir: Option<String>,
    pub rir: Option<String>,
    pub asn: Option<u32>,
    pub online: OnlineResponse,
    pub ports: Vec<PortResponse>,
}

pub async fn index(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<IndexResponse>, StatusCode> {
    let _permit = state.worker_permits.acquire().await.unwrap();

    let query_address = Ipv4Addr::from_str(&query.address);

    match query_address {
        Ok(query_address) => {
            let rir = state.diglett.rir(query_address).await;
            let asn = state.diglett.asn(query_address).await;

            let payload = [0; 8];
            let online_response = match surge_ping::ping(IpAddr::V4(query_address), &payload).await
            {
                Ok(_) => OnlineResponse {
                    value: true,
                    reason: None,
                },
                Err(ping_error) => match ping_error {
                    SurgeError::Timeout { seq: _ } => OnlineResponse {
                        value: false,
                        reason: Some(String::from("timeout")),
                    },
                    _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
                },
            };

            let ports = match Gust::new(&query.address) {
                Ok(gust) => {
                    let gust_range_start = state
                        .config
                        .get_int("settings.gust.range.start")
                        .unwrap_or(1) as u16;
                    let gust_range_end = state
                        .config
                        .get_int("settings.gust.range.end")
                        .unwrap_or(1000) as u16;

                    let mut ports = BTreeMap::new();

                    for port in gust_range_start..=gust_range_end {
                        ports.insert(
                            port,
                            gust.attack(
                                port,
                                state.config.get_int("settings.gust.timeout").unwrap_or(10) as u32,
                            )
                            .await,
                        );
                    }

                    let mut ports_vec = ports
                        .into_iter()
                        .map(|(port, value)| PortResponse { port, value })
                        .collect::<Vec<PortResponse>>();
                    ports_vec.sort_by(|a, b| a.port.cmp(&b.port));

                    ports_vec
                }
                Err(_) => return Err(StatusCode::BAD_REQUEST),
            };

            Ok(Json(IndexResponse {
                allocation_state: String::from("Allocated"),
                online: online_response,
                rir,
                asn,
                ports,
            }))
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn allocation_state(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<String, StatusCode> {
    let _permit = state.worker_permits.acquire().await.unwrap();

    Err(StatusCode::NOT_IMPLEMENTED)
}

#[derive(Serialize)]
pub struct RirResponse {
    pub value: Option<String>,
}

pub async fn rir(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<RirResponse>, StatusCode> {
    let _permit = state.worker_permits.acquire().await;

    let rir_address = Ipv4Addr::from_str(&query.address);

    match rir_address {
        Ok(rir_address) => {
            let rir = state.diglett.rir(rir_address).await;

            match rir {
                Some(rir) => Ok(Json(RirResponse { value: Some(rir) })),
                None => Ok(Json(RirResponse { value: None })),
            }
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

#[derive(Serialize)]
pub struct AsnResponse {
    pub value: Option<u32>,
}

pub async fn asn(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<AsnResponse>, StatusCode> {
    let _permit = state.worker_permits.acquire().await;

    let asn_address = Ipv4Addr::from_str(&query.address);

    match asn_address {
        Ok(asn_address) => {
            let asn = state.diglett.asn(asn_address).await;

            match asn {
                Some(asn) => Ok(Json(AsnResponse { value: Some(asn) })),
                None => Ok(Json(AsnResponse { value: None })),
            }
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

#[derive(Serialize)]
pub struct OnlineResponse {
    pub value: bool,
    pub reason: Option<String>,
}

pub async fn online(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<OnlineResponse>, StatusCode> {
    let _permit = state.worker_permits.acquire().await.unwrap();

    let ping_address = Ipv4Addr::from_str(&query.address);

    match ping_address {
        Ok(ping_address) => {
            let payload = [0; 8];
            match surge_ping::ping(IpAddr::V4(ping_address), &payload).await {
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
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

#[derive(Serialize)]
pub struct PortResponse {
    port: u16,
    value: bool,
}

pub async fn ports(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<PortResponse>>, StatusCode> {
    let _permit = state.worker_permits.acquire().await.unwrap();

    match Gust::new(&query.address) {
        Ok(gust) => {
            let gust_range_start = state
                .config
                .get_int("settings.gust.range.start")
                .unwrap_or(1) as u16;
            let gust_range_end = state
                .config
                .get_int("settings.gust.range.end")
                .unwrap_or(1000) as u16;

            let mut ports = BTreeMap::new();

            for port in gust_range_start..=gust_range_end {
                ports.insert(
                    port,
                    gust.attack(
                        port,
                        state.config.get_int("settings.gust.timeout").unwrap_or(10) as u32,
                    )
                    .await,
                );
            }

            let mut ports_vec = ports
                .into_iter()
                .map(|(port, value)| PortResponse { port, value })
                .collect::<Vec<PortResponse>>();
            ports_vec.sort_by(|a, b| a.port.cmp(&b.port));

            Ok(Json(ports_vec))
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}
