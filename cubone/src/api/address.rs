use std::{net::Ipv4Addr, str::FromStr};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use ipnetwork::{IpNetwork, Ipv4Network};
use tracing::error;

use crate::models;

use super::AppState;

pub async fn address_one(
    Path(address): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let target_address = IpNetwork::V4(match Ipv4Network::from_str(&address) {
        Ok(network) => network,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    });

    let mut db_conn = state.db_pool.acquire().await.unwrap();

    match sqlx::query_as::<_, models::Address>(
        r#"
		SELECT *
		FROM "Addresses"
		WHERE id = $1
		"#,
    )
    .bind(target_address)
    .fetch_one(&mut *db_conn)
    .await
    {
        Ok(row) => Ok(Json(vec![row])),
        Err(error) => match error {
            sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
            _ => {
                error!("{}", error);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
    }
}

pub async fn address_network(
    Path((address, prefix_length)): Path<(String, u8)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let target_address = IpNetwork::V4(
        match Ipv4Network::new(
            match Ipv4Addr::from_str(&address) {
                Ok(addr) => addr,
                Err(_) => return Err(StatusCode::BAD_REQUEST),
            },
            prefix_length,
        ) {
            Ok(network) => network,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        },
    );

    let mut db_conn = state.db_pool.acquire().await.unwrap();

    match sqlx::query_as::<_, models::Address>(
        r#"
		SELECT *
		FROM "Addresses"
		WHERE id << $1
		"#,
    )
    .bind(target_address)
    .fetch_all(&mut *db_conn)
    .await
    {
        Ok(rows) => Ok(Json(rows)),
        Err(error) => match error {
            sqlx::Error::RowNotFound => Err(StatusCode::BAD_REQUEST),
            _ => {
                error!("{}", error);
                Err(StatusCode::INTERNAL_SERVER_ERROR)
            }
        },
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        // .route("/", get(address))
        .route("/{address}", get(address_one))
        .route("/{address}/{prefix_length}", get(address_network))
}
