use std::{collections::HashMap, future::Future, net::Ipv4Addr, pin::Pin, str::FromStr};

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Router,
};
use ipnetwork::{IpNetwork, Ipv4Network};
use mtilib::types::AllocationState;
use sqlx::Error;
use tracing::debug;

use crate::models::{Address, AddressMap};

use super::AppState;

pub fn get_network_average(
    network: Ipv4Network,
    db_pool: mtilib::db::DbPool,
) -> Pin<Box<dyn Future<Output = Result<AllocationState, Error>> + Send>> {
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
            Ok(state) => Ok(AllocationState::from_str(&state.allocation_state_id).unwrap()),
            Err(error) => match error {
                Error::RowNotFound => {
                    debug!("cached map for {} not found, calculating...", network);
                    let start = network.network().to_bits();
                    let mut state_occurence: HashMap<AllocationState, u32> = HashMap::new();

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

                            *state_occurence
                                .entry(
                                    match get_network_average(new_network, db_pool.clone()).await {
                                        Ok(average) => average,
                                        Err(error) => return Err(error),
                                    },
                                )
                                .or_insert(0) += 1;
                        }
                    }

                    let max = state_occurence
                        .iter()
                        .max_by(|a, b| a.1.cmp(b.1))
                        .map(|(k, _v)| k)
                        .unwrap_or(&AllocationState::Unknown);

                    sqlx::query(
                        r#"
						INSERT INTO "AddressMaps" (id, allocation_state_id)
						VALUES ($1, $2)
						"#,
                    )
                    .bind(wrapped_network)
                    .bind(max.id())
                    .execute(&mut *db_pool.acquire().await.unwrap())
                    .await
                    .unwrap();

                    Ok(max.clone())
                }
                _ => Err(error),
            },
        }
    })
}

pub async fn map_one(
    Path((address, prefix_length)): Path<(String, u8)>,
    State(state): State<AppState>,
) -> Result<String, StatusCode> {
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
        return match sqlx::query_scalar::<_, String>(
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
            Ok(alloc_state) => Ok(alloc_state),
            Err(error) => match error {
                sqlx::Error::RowNotFound => Ok(AllocationState::Unknown.id().to_string()),
                _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
            },
        };
    }

    match get_network_average(target_network, state.db_pool.clone()).await {
        Ok(average) => Ok(average.id().to_string()),
        Err(error) => {
            println!("{}", error);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/{address}/{prefix_length}", get(map_one))
}
