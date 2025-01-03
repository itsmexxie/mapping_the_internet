use std::collections::HashMap;
use std::net::Ipv4Addr;
use std::str::FromStr;

use axum::extract::{Path, State};
use axum::{http::StatusCode, response::IntoResponse, routing::get, Router};
use diesel::query_dsl::methods::FindDsl;
use diesel::{PgConnection, RunQueryDsl};
use ipnetwork::{IpNetwork, Ipv4Network};
use mtilib::types::AllocationState;
use tracing::debug;

use crate::models::{Address, AddressMap, NewAddressMap};
use crate::schema::{AddressMaps, Addresses};

use super::AppState;

// This shit NEEDS to be parallelized
// *Ahem* where is my academic language, sorry...
// This procedure is very slow indeed, and in serious need of parallelization due to serial calls to the database *adjusts bowtie*
fn get_block_average(network: Ipv4Network, conn: &mut PgConnection) -> AllocationState {
    match AddressMaps::dsl::AddressMaps
        .find(IpNetwork::V4(network))
        .first::<AddressMap>(conn)
    {
        Ok(map) => AllocationState::from_str(&map.allocation_state_id).unwrap(),
        Err(_) => {
            debug!("cached map for {} not found, calculating...", network);
            let start = network.network().to_bits();
            let mut state_occurence = HashMap::new();

            if network.prefix() == 24 {
                for i in 0..=u8::MAX {
                    let new_address = Ipv4Addr::from_bits(start + i as u32);

                    match Addresses::dsl::Addresses
                        .find(IpNetwork::V4(new_address.into()))
                        .first::<Address>(conn)
                    {
                        Ok(db_address) => {
                            *state_occurence
                                .entry(
                                    AllocationState::from_str(&db_address.allocation_state_id)
                                        .unwrap(),
                                )
                                .or_insert(0) += 1
                        }
                        Err(_) => {
                            *state_occurence.entry(AllocationState::Unknown).or_insert(0) += 1
                        }
                    }
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
                        .entry(get_block_average(new_network, conn))
                        .or_insert(0) += 1;
                }
            }

            let max = state_occurence
                .iter()
                .max_by(|a, b| a.1.cmp(&b.1))
                .map(|(k, _v)| k)
                .unwrap();

            diesel::insert_into(AddressMaps::table)
                .values(&NewAddressMap {
                    id: IpNetwork::V4(network),
                    allocation_state_id: max.id().to_string(),
                })
                .execute(conn)
                .unwrap();

            return max.clone();
        }
    }
}

pub async fn map(
    Path((address, prefix)): Path<(String, u8)>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    if ![8, 16, 24].contains(&prefix) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let mut conn = mtilib::db::create_conn(
        &*state.settings.database.host,
        &*state.settings.database.username,
        &*state.settings.database.password,
        &*state.settings.database.database,
    );

    // Double parsing network because we want all zeros on host bits
    // Fix by making get_average_block return an error when DB refuses the address since
    // that's the reason we are doing this...
    let network = match Ipv4Network::new(
        match Ipv4Network::new(
            match Ipv4Addr::from_str(&address) {
                Ok(addr) => addr,
                Err(_) => return Err(StatusCode::BAD_REQUEST),
            },
            prefix,
        ) {
            Ok(network) => network.network(),
            Err(_) => return Err(StatusCode::BAD_REQUEST),
        },
        prefix,
    ) {
        Ok(network) => network,
        Err(_) => return Err(StatusCode::BAD_REQUEST),
    };

    println!("{}", network);

    Ok(get_block_average(network, &mut conn).id().to_string())
}

pub fn router() -> Router<AppState> {
    Router::new().route("/{address}/{prefix}", get(map))
}
