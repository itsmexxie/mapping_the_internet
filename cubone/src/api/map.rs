use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use ipnetwork::{IpNetwork, Ipv4Network};
use mtilib::{
    db::models::{Address, AddressMap},
    types::AllocationState,
};
use serde::Serialize;
use sqlx::Error;
use std::{collections::HashMap, future::Future, net::Ipv4Addr, pin::Pin, str::FromStr};
use tracing::debug;

use super::AppState;

pub fn get_network_average(
    network: Ipv4Network,
    db_pool: mtilib::db::DbPool,
) -> Pin<Box<dyn Future<Output = Result<(AllocationState, bool, bool), Error>> + Send>> {
    Box::pin(async move {
        let wrapped_network = IpNetwork::V4(network);

        match sqlx::query_as::<_, AddressMap>(
            r#"
			SELECT *
			FROM "AddressMaps"
			WHERE id = $1
			"#,
        )
        .bind(wrapped_network)
        .fetch_one(&mut *db_pool.acquire().await.unwrap())
        .await
        {
            Ok(map) => Ok((
                AllocationState::from_str(&map.allocation_state_id).unwrap(),
                map.routed,
                map.online,
            )),
            Err(error) => match error {
                Error::RowNotFound => {
                    debug!("cached map for {} not found, calculating...", network);
                    let start = network.network().to_bits();
                    let mut state_occurence: HashMap<AllocationState, u32> = HashMap::new();
                    let mut routed_occurence: HashMap<bool, u32> = HashMap::new();
                    let mut online_occurence: HashMap<bool, u32> = HashMap::new();

                    if network.prefix() == 24 {
                        match sqlx::query_as::<_, Address>(
                            r#"
							SELECT *
							FROM "Addresses"
							WHERE id << $1
							"#,
                        )
                        .bind(wrapped_network)
                        .fetch_all(&mut *db_pool.acquire().await.unwrap())
                        .await
                        {
                            Ok(rows) => {
                                let mut i = 0;
                                for row in rows {
                                    *state_occurence
                                        .entry(
                                            AllocationState::from_str(&row.allocation_state_id)
                                                .unwrap(),
                                        )
                                        .or_insert(0) += 1;
                                    *routed_occurence.entry(row.routed).or_insert(0) += 1;
                                    *online_occurence.entry(row.online).or_insert(0) += 1;

                                    i += 1;
                                }

                                *state_occurence.entry(AllocationState::Unknown).or_insert(0) +=
                                    256 - i;
                            }
                            Err(error) => return Err(error),
                        }
                    } else {
                        let shift = 24 - network.prefix();
                        let new_prefix = network.prefix() + 8;

                        for i in 0..=u8::MAX {
                            let new_network = Ipv4Network::new(
                                Ipv4Addr::from_bits(start + ((i as u32) << shift)),
                                new_prefix,
                            )
                            .unwrap();

                            let average = get_network_average(new_network, db_pool.clone()).await?;
                            *state_occurence.entry(average.0).or_insert(0) += 1;
                            *routed_occurence.entry(average.1).or_insert(0) += 1;
                            *online_occurence.entry(average.2).or_insert(0) += 1;
                        }
                    }

                    let average_state = state_occurence
                        .iter()
                        .max_by(|a, b| a.1.cmp(b.1))
                        .map(|(k, _v)| k)
                        .unwrap_or(&AllocationState::Unknown);
                    let (average_routed, average_online) = match average_state {
                        AllocationState::Allocated => {
                            let routed = routed_occurence
                                .iter()
                                .max_by(|a, b| a.1.cmp(b.1))
                                .map(|(k, _v)| k)
                                .unwrap_or(&false);
                            let online = online_occurence
                                .iter()
                                .max_by(|a, b| a.1.cmp(b.1))
                                .map(|(k, _v)| k)
                                .unwrap_or(&false);
                            (routed, online)
                        }
                        _ => (&false, &false),
                    };

                    sqlx::query(
                        r#"
						INSERT INTO "AddressMaps" (id, allocation_state_id, routed, online)
						VALUES ($1, $2, $3, $4)
						"#,
                    )
                    .bind(wrapped_network)
                    .bind(average_state.id())
                    .bind(average_routed)
                    .bind(average_online)
                    .execute(&mut *db_pool.acquire().await.unwrap())
                    .await
                    .unwrap();

                    Ok((average_state.clone(), false, false))
                }
                _ => Err(error),
            },
        }
    })
}

#[derive(Serialize)]
struct MapOneResponse {
    allocation_state: String,
    routed: bool,
    online: bool,
}

pub async fn map_one(
    Path((address, prefix_length)): Path<(String, u8)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if ![8, 16, 24, 32].contains(&prefix_length) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let target_network = match Ipv4Network::new(
        match Ipv4Addr::from_str(&address) {
            Ok(addr) => addr,
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        },
        prefix_length,
    ) {
        Ok(network) => network,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    if target_network.prefix() == 32 {
        return match sqlx::query_as::<_, Address>(
            r#"
            SELECT allocation_state_id
            FROM "Addresses"
            WHERE id = $1
            "#,
        )
        .bind(IpNetwork::V4(target_network))
        .fetch_one(&mut *state.db_pool.acquire().await.unwrap())
        .await
        {
            Ok(address) => Ok(Json(MapOneResponse {
                allocation_state: address.allocation_state_id,
                routed: address.routed,
                online: address.online,
            })),
            Err(error) => match error {
                sqlx::Error::RowNotFound => Ok(Json(MapOneResponse {
                    allocation_state: AllocationState::Unknown.id().to_string(),
                    routed: false,
                    online: false,
                })),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            },
        };
    }

    match get_network_average(target_network, state.db_pool.clone()).await {
        Ok(average) => Ok(Json(MapOneResponse {
            allocation_state: average.0.id().to_string(),
            routed: average.1,
            online: average.2,
        })),
        Err(error) => {
            println!("Error while getting network average: {}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/{address}/{prefix_length}", get(map_one))
}
