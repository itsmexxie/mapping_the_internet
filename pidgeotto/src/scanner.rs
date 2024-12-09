use std::collections::{HashMap, HashSet};
use std::net::Ipv4Addr;
use std::str::FromStr;
use std::sync::Arc;

use config::Config;
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use diesel::{ExpressionMethods, RunQueryDsl, SelectableHelper};
use ipnetwork::{IpNetwork, Ipv4Network};
use mtilib::pidgey::PidgeyCommand;
use tokio::sync::{Mutex, Semaphore};
use tracing::{debug, error, info};
use uuid::Uuid;

use crate::models::{Address, Asn};
use crate::pidgey::{Pidgey, PidgeyUnitRequest, PidgeyUnitResponse};
use crate::schema::{Addresses, Asns};

pub async fn run(config: Arc<Config>, pidgey: Arc<Pidgey>) {
    let pg_conn = &mut crate::db::create_conn(&config);
    let batch = config.get_int("settings.scanner.batch").unwrap_or(1024) as u32;
    let task_permits = Arc::new(Semaphore::new(
        config.get_int("settings.scanner.max_tasks").unwrap_or(512) as usize,
    ));

    info!(
        "Running scanner with batch size of {} and maximum number of tasks of {}!",
        batch,
        task_permits.available_permits()
    );

    // Query the database for records in that range
    // Query pidgeys for the missing records
    // Write to database
    let mut curr_address: u32 = Ipv4Addr::from_str(
        &config
            .get_string("settings.scanner.start")
            .unwrap_or("0.0.0.0".to_string()),
    )
    .unwrap()
    .to_bits();

    loop {
        // Calculate current range for query
        let mut addresses_scanning = Vec::new();
        for i in curr_address..curr_address + batch {
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
        let addresses_query: Vec<Address> = Addresses::table
            .select(Address::as_select())
            .filter(Addresses::id.eq_any(&addresses_scanning))
            .load(pg_conn)
            .unwrap();
        let addresses_in_db = addresses_query
            .into_iter()
            .map(|x| x.id)
            .collect::<Vec<_>>();

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
                            tokio::sync::oneshot::channel::<PidgeyUnitResponse>();

                        let uuid = Uuid::new_v4();
                        let ipaddr = match address.ip() {
                            std::net::IpAddr::V4(ipv4_addr) => ipv4_addr,
                            std::net::IpAddr::V6(_) => panic!("This should never happen"),
                        };

                        unit.tx
                            .send(PidgeyUnitRequest {
                                id: uuid,
                                command: PidgeyCommand::Query {
                                    id: uuid,
                                    address: ipaddr,
                                    ports_start: Some(1),
                                    ports_end: Some(999),
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
                PidgeyUnitResponse::Query {
                    allocation_state,
                    top_rir,
                    rir,
                    asn,
                    country,
                    online,
                    ports,
                } => {
                    let mut routed = false;
                    let asn = match asn {
                        Some(asn) => {
                            routed = true;
                            Some(asn as i32)
                        }
                        None => None,
                    };

                    let ports = match ports {
                        Some(ports) => ports
                            .into_iter()
                            .filter(|p| p.1)
                            .map(|x| x.0 as i32)
                            .collect::<Vec<_>>(),
                        None => Vec::new(),
                    };

                    Address {
                        id: x.0,
                        allocation_state_id: allocation_state.id(),
                        allocation_state_comment: None,
                        top_rir_id: top_rir,
                        rir_id: rir,
                        asn_id: asn,
                        routed,
                        online,
                        ports,
                        country,
                    }
                }
                _ => panic!("Should not be here!"),
            })
            .collect::<Vec<_>>();

        if !config.get_bool("settings.dry_run").unwrap_or(false) {
            // Check which ASs are already in our database
            // Unfortunate naming scheme ¯\_(ツ)_/¯
            let curr_autsyses = new_addresses
                .iter()
                .filter(|x| x.asn_id.is_some())
                .map(|x| x.asn_id.unwrap())
                .collect::<HashSet<i32>>();

            let autsyses_db = Asns::table
                .select(Asn::as_select())
                .filter(Asns::id.eq_any(&curr_autsyses))
                .load(pg_conn)
                .unwrap()
                .iter()
                .map(|x| x.id)
                .collect::<Vec<_>>();

            let mut new_autsyses = Vec::new();
            for curr_autsys in curr_autsyses {
                if !autsyses_db.contains(&curr_autsys) {
                    new_autsyses.push(Asn { id: curr_autsys });
                }
            }

            diesel::insert_into(Asns::dsl::Asns)
                .values(new_autsyses)
                .execute(pg_conn)
                .unwrap();

            diesel::insert_into(Addresses::dsl::Addresses)
                .values(new_addresses)
                .execute(pg_conn)
                .unwrap();
        }

        curr_address += batch;
    }
}
