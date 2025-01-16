use ipnetwork::{IpNetwork, Ipv4Network};
use mtilib::db::models::NewAddress;
use mtilib::db::DbPool;
use mtilib::pidgey::{PidgeyCommand, PidgeyCommandPayload, PidgeyCommandResponsePayload};
use sqlx::QueryBuilder;
use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::pidgey::{Pidgey, PidgeyUnitRequest};
use crate::settings::Settings;

pub async fn run(settings: Arc<Settings>, db_pool: DbPool, pidgey: Arc<Pidgey>) {
    let task_permits = Arc::new(Semaphore::new(settings.scanner.max_tasks));

    info!(
        "Running scanner with batch size of {} and maximum number of tasks of {}!",
        settings.scanner.batch,
        task_permits.available_permits()
    );

    // Query the database for records in that range
    // Query pidgeys for the missing records
    // Write to database
    let mut curr_address = Ipv4Addr::from_str(&settings.scanner.start)
        .unwrap()
        .to_bits();

    loop {
        // Calculate current range for query
        let mut addresses_scanning = Vec::new();
        for i in curr_address..curr_address + settings.scanner.batch {
            addresses_scanning.push(IpNetwork::V4(
                Ipv4Network::new(Ipv4Addr::from_bits(i), 32).unwrap(),
            ));
        }

        debug!(
            "Trying range {:?} .. {:?}",
            addresses_scanning[0],
            addresses_scanning.last().unwrap()
        );

        // Check which addresses are already in our database
        let addresses_in_db = sqlx::query_scalar::<_, IpNetwork>(
            r#"
            SELECT id
            FROM "Addresses"
            WHERE id = ANY($1)
            "#,
        )
        .bind(addresses_scanning.clone())
        .fetch_all(&mut *db_pool.acquire().await.unwrap())
        .await
        .unwrap();

        let mut address_tasks = Vec::new();
        let query_results = Arc::new(Mutex::new(HashMap::new()));

        for address in addresses_scanning {
            if !addresses_in_db.contains(&address) {
                let cloned_task_permits = task_permits.clone();
                let cloned_pidgey = pidgey.clone();
                let cloned_query_results = query_results.clone();
                address_tasks.push(tokio::spawn(async move {
                    loop {
                        let _permit = cloned_task_permits.acquire().await.unwrap();
                        let unit = cloned_pidgey.get_unit().await;

                        let (job_tx, job_rx) =
                            tokio::sync::oneshot::channel::<PidgeyCommandResponsePayload>();

                        let job_uuid = Uuid::new_v4();
                        let ipaddr = match address.ip() {
                            std::net::IpAddr::V4(ipv4_addr) => ipv4_addr,
                            std::net::IpAddr::V6(_) => panic!("This should never happen"),
                        };

                        unit.tx
                            .send(PidgeyUnitRequest {
                                command: PidgeyCommand {
                                    id: job_uuid,
                                    payload: PidgeyCommandPayload::Query { address: ipaddr },
                                },
                                response: job_tx,
                            })
                            .await
                            .unwrap();

                        match job_rx.await {
                            Ok(response) => {
                                cloned_query_results.lock().await.insert(address, response);
                                break;
                            }
                            Err(_) => {
                                error!("Error while querying address {}, retrying...", address)
                            }
                        };
                    }
                }));
            }
        }

        // Wait for all queries to finish
        for task in address_tasks {
            task.await.unwrap();
        }

        let query_results = match Arc::try_unwrap(query_results) {
            Ok(results) => results.into_inner(),
            Err(_) => panic!("Failed to unwrap arc"),
        };

        // Create new records
        let new_addresses = query_results
            .into_iter()
            .map(|x| match x.1 {
                PidgeyCommandResponsePayload::Query {
                    allocation_state,
                    top_rir,
                    rir,
                    autsys,
                    country,
                    online,
                } => {
                    let top_rir_id = match top_rir {
                        Some(top_rir) => Some(top_rir.id().to_string()),
                        None => None,
                    };

                    let rir_id = match rir {
                        Some(rir) => Some(rir.id().to_string()),
                        None => None,
                    };

                    let mut routed = false;
                    let autsys_id = match autsys {
                        Some(autsys) => {
                            routed = true;
                            Some(autsys as i64)
                        }
                        None => None,
                    };

                    NewAddress {
                        id: x.0,
                        allocation_state_id: allocation_state.id().to_string(),
                        allocation_state_comment: None,
                        top_rir_id,
                        rir_id,
                        autsys_id,
                        routed,
                        online,
                        country,
                    }
                }
                _ => panic!("Should not be here!"),
            })
            .collect::<Vec<_>>();

        // Check which Autsyses are already in our database
        let new_autsyses = new_addresses
            .iter()
            .filter(|x| x.autsys_id.is_some())
            .map(|x| x.autsys_id.unwrap())
            .collect::<HashSet<i64>>();

        let autsyses_in_db = HashSet::<_>::from_iter(
            sqlx::query_scalar::<_, i64>(
                r#"
                SELECT id
                FROM "Autsyses"
                WHERE id = ANY($1)
                "#,
            )
            .bind(new_autsyses.iter().collect::<Vec<_>>())
            .fetch_all(&mut *db_pool.acquire().await.unwrap())
            .await
            .unwrap()
            .into_iter(),
        );

        sqlx::query(
            r#"
            INSERT INTO "Autsyses" (id)
            SELECT * FROM UNNEST($1)
            RETURNING id
            "#,
        )
        .bind(new_autsyses.difference(&autsyses_in_db).collect::<Vec<_>>())
        .execute(&mut *db_pool.acquire().await.unwrap())
        .await
        .unwrap();

        let mut addresses_qb = QueryBuilder::new(
            r#"INSERT INTO "Addresses" (id, allocation_state_id, allocation_state_comment, routed, online, top_rir_id, rir_id, autsys_id, country)"#,
        );

        addresses_qb.push_values(new_addresses, |mut b, new_address| {
            b.push_bind(new_address.id)
                .push_bind(new_address.allocation_state_id)
                .push_bind(new_address.allocation_state_comment)
                .push_bind(new_address.routed)
                .push_bind(new_address.online)
                .push_bind(new_address.top_rir_id)
                .push_bind(new_address.rir_id)
                .push_bind(new_address.autsys_id)
                .push_bind(new_address.country);
        });

        let addresses_query = addresses_qb.build();
        addresses_query
            .execute(&mut *db_pool.acquire().await.unwrap())
            .await
            .unwrap();

        curr_address += settings.scanner.batch;
    }
}
