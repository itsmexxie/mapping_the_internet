use std::str::FromStr;

use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use ipnetwork::{IpNetwork, Ipv4Network};
use serde::Deserialize;

use crate::models::Address;
use crate::schema::Addresses;

use super::AppState;

#[derive(Deserialize)]
pub struct AddressQuery {
    pub target: Option<String>,
    pub start: Option<String>,
    pub end: Option<String>,
}

pub async fn address(
    Query(query): Query<AddressQuery>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());

    let conn = &mut mtilib::db::create_conn(
        &*state.settings.database.host,
        &*state.settings.database.username,
        &*state.settings.database.password,
        &*state.settings.database.database,
    );

    if let Some(address) = query.target {
        let target_address = IpNetwork::V4(match Ipv4Network::from_str(&address) {
            Ok(address) => address,
            Err(_) => return Err((headers, StatusCode::BAD_REQUEST)),
        });

        match Addresses::dsl::Addresses
            .find(target_address)
            .first::<Address>(conn)
        {
            Ok(res) => return Ok((headers, Json(vec![res]))),
            Err(error) => match error {
                diesel::result::Error::NotFound => return Err((headers, StatusCode::NOT_FOUND)),
                _ => return Err((headers, StatusCode::INTERNAL_SERVER_ERROR)),
            },
        }
    } else if query.start.is_some() && query.end.is_some() {
        let (start_address_uint, start_address) = match Ipv4Network::from_str(&query.start.unwrap())
        {
            Ok(address) => (address.ip().to_bits(), IpNetwork::V4(address)),
            Err(_) => return Err((headers, StatusCode::BAD_REQUEST)),
        };
        let (end_address_uint, end_address) = match Ipv4Network::from_str(&query.end.unwrap()) {
            Ok(address) => (address.ip().to_bits(), IpNetwork::V4(address)),
            Err(_) => return Err((headers, StatusCode::BAD_REQUEST)),
        };

        if start_address_uint > end_address_uint {
            return Err((headers, StatusCode::BAD_REQUEST));
        }

        if end_address_uint - start_address_uint > 255 {
            return Err((headers, StatusCode::FORBIDDEN));
        }

        match Addresses::dsl::Addresses
            .filter(Addresses::id.between(start_address, end_address))
            .select(Address::as_select())
            .load::<Address>(conn)
        {
            Ok(res) => return Ok((headers, Json(res))),
            Err(error) => match error {
                diesel::result::Error::NotFound => return Err((headers, StatusCode::NOT_FOUND)),
                _ => return Err((headers, StatusCode::INTERNAL_SERVER_ERROR)),
            },
        }
    }

    Err((headers, StatusCode::BAD_REQUEST))
}

pub async fn address_one(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::ACCESS_CONTROL_ALLOW_ORIGIN, "*".parse().unwrap());

    let conn = &mut mtilib::db::create_conn(
        &*state.settings.database.host,
        &*state.settings.database.username,
        &*state.settings.database.password,
        &*state.settings.database.database,
    );

    let address = IpNetwork::V4(match Ipv4Network::from_str(&address) {
        Ok(address) => address,
        Err(_) => return Err((headers, StatusCode::BAD_REQUEST)),
    });

    match Addresses::dsl::Addresses
        .find(address)
        .first::<Address>(conn)
    {
        Ok(res) => Ok((headers, Json(res))),
        Err(error) => match error {
            diesel::result::Error::NotFound => Err((headers, StatusCode::NOT_FOUND)),
            _ => Err((headers, StatusCode::INTERNAL_SERVER_ERROR)),
        },
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/", get(address))
        .route("/:address", get(address_one))
}
