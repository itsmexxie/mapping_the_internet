use std::str::FromStr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use diesel::{QueryDsl, RunQueryDsl};
use ipnetwork::{IpNetwork, Ipv4Network};

use crate::models::Address;
use crate::schema::Addresses;

use super::AppState;

pub async fn address(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, StatusCode> {
    let conn = &mut mtilib::db::create_conn(
        &*state.settings.database.host,
        &*state.settings.database.username,
        &*state.settings.database.password,
        &*state.settings.database.database,
    );

    let address = IpNetwork::V4(match Ipv4Network::from_str(&address) {
        Ok(address) => address,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    });

    match Addresses::dsl::Addresses
        .find(address)
        .first::<Address>(conn)
    {
        Ok(res) => Ok(Json(res)),
        Err(error) => match error {
            diesel::result::Error::NotFound => Err(StatusCode::NOT_FOUND),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/:address", get(address))
}
