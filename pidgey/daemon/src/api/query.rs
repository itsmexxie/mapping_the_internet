use axum::{extract::Query, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr},
    str::FromStr,
};
use surge_ping::SurgeError;

#[derive(Deserialize)]
pub struct AddressQuery {
    pub address: String,
}

#[derive(Serialize)]
pub struct QueryResponse {
    pub online: QueryResponseOnline,
}

#[derive(Serialize)]
pub struct QueryResponseOnline {
    pub value: bool,
    pub reason: Option<String>,
}

pub async fn index(query: Query<AddressQuery>) -> Result<Json<QueryResponse>, StatusCode> {
    Ok(Json(QueryResponse {
        online: QueryResponseOnline {
            value: false,
            reason: Some("debug".to_string()),
        },
    }))
}

pub async fn online(query: Query<AddressQuery>) -> Result<Json<QueryResponseOnline>, StatusCode> {
    let ping_address = Ipv4Addr::from_str(&query.address);

    match ping_address {
        Ok(ping_address) => {
            let payload = [0; 8];
            match surge_ping::ping(IpAddr::V4(ping_address), &payload).await {
                Ok(_) => Ok(Json(QueryResponseOnline {
                    value: true,
                    reason: None,
                })),
                Err(ping_error) => match ping_error {
                    SurgeError::Timeout { seq: _ } => Ok(Json(QueryResponseOnline {
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
