use std::collections::HashMap;
use std::net::IpAddr;
use std::sync::Arc;

use axum::http::HeaderValue;
use config::Config;
use futures::{SinkExt, StreamExt};
use mtilib::pidgey::{PidgeyCommand, PidgeyCommandResponse};
use mtilib::types::AllocationState;
use rand::random;
use surge_ping::{PingIdentifier, PingSequence, SurgeError};
use tokio::sync::Semaphore;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info};

use crate::diglett::Diglett;
use crate::gust::Gust;

pub async fn run(
    config: Arc<Config>,
    worker_permits: Arc<Semaphore>,
    jwt: Arc<String>,
    diglett: Arc<Diglett>,
    ping_client: Arc<surge_ping::Client>,
) {
    if !config.get_bool("pidgeotto.connect").unwrap_or(true) {
        info!("pidgeotto.connect set to false, not connecting!");
        return;
    }

    // Get address
    let mut pidgeotto_url = url::Url::parse(
        &config
            .get_string("pidgeotto.address")
            .expect("pidgeotto.address must be set!"),
    )
    .expect("Failed to parse configured pidgeotto address!");

    if let Ok(port) = config.get_int("pidgeotto.port") {
        pidgeotto_url.set_port(Some(port as u16)).unwrap()
    }

    pidgeotto_url.set_path("/ws");

    // Create request
    let mut pidgeotto_req = pidgeotto_url.into_client_request().unwrap();
    let headers = pidgeotto_req.headers_mut();
    headers.insert(
        "authorization",
        HeaderValue::from_str(&concat_string!("Bearer ", *jwt)).unwrap(),
    );

    // Connect to Pidgeotto
    let (ws_stream, _) = match connect_async(pidgeotto_req).await {
        Ok(a) => {
            info!("Successfully established a websocket connection to a Pidgeotto instance!");
            a
        }
        Err(err) => panic!(
            "Failed to establish a websocket connection to Pidgeotto! ({})",
            err
        ),
    };

    let (mut ws_write, mut ws_read) = ws_stream.split();

    ws_write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::to_string(&PidgeyCommandResponse::Register).unwrap(),
        ))
        .await
        .unwrap();

    let max_workers = match config.get_int("settings.max_workers") {
        Ok(max) => max as usize,
        Err(_) => crate::MAX_WORKERS,
    };
    let (response_tx, mut response_rx) =
        tokio::sync::mpsc::channel::<PidgeyCommandResponse>(max_workers);

    // Websocket write task
    // Can't clone the resulting ws_write from tungstenite, so only this task writes to the websocket
    // while other tasks use tokio::sync::mpsc channels to communicate with this task
    tokio::spawn(async move {
        while let Some(response) = response_rx.recv().await {
            ws_write
                .send(Message::Text(serde_json::to_string(&response).unwrap()))
                .await
                .unwrap();
        }
    });

    while let Some(Ok(message)) = ws_read.next().await {
        let cloned_config = config.clone();
        let cloned_worker_permits = worker_permits.clone();
        let cloned_diglett = diglett.clone();
        let cloned_ping_client = ping_client.clone();
        let cloned_response_tx = response_tx.clone();
        tokio::spawn(async move {
            debug!("Received message {}", message);

            if let tokio_tungstenite::tungstenite::Message::Text(t) = message {
                                match serde_json::from_str::<PidgeyCommand>(&t) {
                                    Ok(command) => match command {
                                        PidgeyCommand::Register => {}
                                        PidgeyCommand::Deregister => {}
                                        PidgeyCommand::Query {
                                            id,
                                            address,
                                            ports_start: _,
                                            ports_end: _,
                                        } => {
                                            let _permit = cloned_worker_permits.acquire().await.unwrap();

                                            let alloc_state = match cloned_diglett.allocation_state(address).await {
            									Ok(alloc_state) => alloc_state,
            									Err(status) => panic!(
                                                    "Panicked while retrieving allocation state for address {}! (status: {})",
                                                    address,
            										status
                                                ),
            								};
                                            let top_rir = match cloned_diglett.rir(address, true).await {
            									Ok(rir) => rir,
            									Err(status) => panic!(
                                                    "Panicked while retrieving top RIR for address {}! (status: {})",
                                                    address,
            										status
                                                ),
            								};
                                            let rir = match cloned_diglett.rir(address, false).await {
            									Ok(rir) => rir,
            									Err(status) => panic!(
                                                    "Panicked while retrieving rir for address {}! (status: {})",
                                                    address,
            										status
                                                ),
            								};
                                            let asn = match cloned_diglett.asn(address).await {
            									Ok(asn) => asn,
            									Err(status) => panic!(
                                                    "Panicked while retrieving AS number for address {}! (status: {})",
                                                    address,
            										status
                                                ),
            								};
                                            let country = match cloned_diglett.country(address).await {
            									Ok(country) => country,
            									Err(status) => panic!(
                                                    "Panicked while retrieving country for address {}! (status: {})",
                                                    address,
            										status
                                                ),
            								};

                                            if alloc_state == AllocationState::Reserved
                                                || alloc_state == AllocationState::Unallocated
                                            {
                                                cloned_response_tx
                                                    .send(PidgeyCommandResponse::Query {
                                                        id,
                                                        allocation_state: alloc_state,
                                                        top_rir,
                                                        rir,
                                                        asn,
                                                        country,
                                                        online: false,
                                                        ports: None,
                                                    })
                                                    .await
                                                    .unwrap();
                                            } else {
                                                let payload = [0; 8];
                                                let mut pinger = cloned_ping_client
                                                    .pinger(IpAddr::V4(address), PingIdentifier(random()))
                                                    .await;
                                                let online =
                                                    pinger.ping(PingSequence(0), &payload).await.is_ok();

                                                // let gust = Arc::new(Gust::new(address).unwrap());

                                                // let gust_range_start = match ports_start {
                                                //     Some(start) => start,
                                                //     None => match cloned_config
                                                //         .get_int("settings.gust.range.start")
                                                //     {
                                                //         Ok(start) => start as u16,
                                                //         Err(_) => 1,
                                                //     },
                                                // };

                                                // let gust_range_end = match ports_end {
                                                //     Some(end) => end,
                                                //     None => {
                                                //         match cloned_config.get_int("settings.gust.range.end") {
                                                //             Ok(end) => end as u16,
                                                //             Err(_) => 999,
                                                //         }
                                                //     }
                                                // };

                                                // let ports = gust
                                                //     .attack_range(
                                                //         gust_range_start..=gust_range_end,
                                                //         cloned_config
                                                //             .get_int("settings.gust.timeout")
                                                //             .unwrap_or(10)
                                                //             as u32,
                                                //     )
                                                //     .await
                                                //     .unwrap();

                                                cloned_response_tx
                                                    .send(PidgeyCommandResponse::Query {
                                                        id,
                                                        allocation_state: match online {
                                                            true => AllocationState::Allocated, // If the address is online then the state must be allocated
                                                            false => alloc_state,
                                                        },
                                                        top_rir,
                                                        rir,
                                                        asn,
                                                        country,
                                                        online,
                                                        ports: Some(HashMap::new()),
                                                    })
                                                    .await
                                                    .unwrap();
                                            }
                                        }
                                        PidgeyCommand::AllocationState { id, address } => {
                                            let _permit = cloned_worker_permits.acquire().await.unwrap();
                                            cloned_response_tx
                                                .send(PidgeyCommandResponse::AllocationState {
                                                    id,
                                                    value: cloned_diglett
                                                        .allocation_state(address)
                                                        .await
                                                        .unwrap(),
                                                })
                                                .await
                                                .unwrap()
                                        }
                                        PidgeyCommand::Rir { id, address, top } => {
                                            let _permit = cloned_worker_permits.acquire().await.unwrap();
                                            cloned_response_tx
                                                .send(PidgeyCommandResponse::Rir {
                                                    id,
                                                    value: cloned_diglett.rir(address, top).await.unwrap(),
                                                })
                                                .await
                                                .unwrap()
                                        }
                                        PidgeyCommand::Asn { id, address } => {
                                            let _permit = cloned_worker_permits.acquire().await.unwrap();
                                            cloned_response_tx
                                                .send(PidgeyCommandResponse::Asn {
                                                    value: cloned_diglett.asn(address).await.unwrap(),
                                                    id,
                                                })
                                                .await
                                                .unwrap()
                                        }
                                        PidgeyCommand::Country { id, address } => {
                                            let _permit = cloned_worker_permits.acquire().await.unwrap();
                                            cloned_response_tx
                                                .send(PidgeyCommandResponse::Country {
                                                    value: cloned_diglett.country(address).await.unwrap(),
                                                    id,
                                                })
                                                .await
                                                .unwrap()
                                        }
                                        PidgeyCommand::Online { id, address } => {
                                            let _permit = cloned_worker_permits.acquire().await.unwrap();
                                            let payload = [0; 8];
                                            let cmd =
                                                match surge_ping::ping(IpAddr::V4(address), &payload).await {
                                                    Ok(_) => PidgeyCommandResponse::Online {
                                                        id,
                                                        value: true,
                                                        reason: None,
                                                    },
                                                    Err(ping_error) => match ping_error {
                                                        SurgeError::Timeout { seq: _ } => {
                                                            PidgeyCommandResponse::Online {
                                                                id,
                                                                value: false,
                                                                reason: Some(String::from("timeout")),
                                                            }
                                                        }
                                                        _ => PidgeyCommandResponse::Online {
                                                            id,
                                                            value: false,
                                                            reason: Some(String::from("unknown")),
                                                        },
                                                    },
                                                };

                                            cloned_response_tx.send(cmd).await.unwrap()
                                        }
                                        PidgeyCommand::Port { id, address, port } => {
                                            let _permit = cloned_worker_permits.acquire().await.unwrap();
                                            let gust = Gust::new(address).unwrap();
                                            let port_online = gust
                                                .attack(
                                                    port,
                                                    cloned_config.get_int("settings.gust.timeout").unwrap_or(5)
                                                        as u32,
                                                )
                                                .await;

                                            cloned_response_tx
                                                .send(PidgeyCommandResponse::Port {
                                                    value: port_online,
                                                    id,
                                                })
                                                .await
                                                .unwrap()
                                        }
                                        PidgeyCommand::PortRange {
                                            id,
                                            address,
                                            start,
                                            end,
                                        } => {
                                            let _permit = cloned_worker_permits.acquire().await.unwrap();
                                            let gust = Arc::new(Gust::new(address).unwrap());

                                            let gust_range_start = match start {
                                                Some(start) => start,
                                                None => {
                                                    match cloned_config.get_int("settings.gust.range.start") {
                                                        Ok(start) => start as u16,
                                                        Err(_) => 1,
                                                    }
                                                }
                                            };

                                            let gust_range_end = match end {
                                                Some(end) => end,
                                                None => {
                                                    match cloned_config.get_int("settings.gust.range.end") {
                                                        Ok(end) => end as u16,
                                                        Err(_) => 999,
                                                    }
                                                }
                                            };

                                            let ports_online = gust
                                                .attack_range(
                                                    gust_range_start..=gust_range_end,
                                                    cloned_config.get_int("settings.gust.timeout").unwrap_or(10)
                                                        as u32,
                                                )
                                                .await
                                                .unwrap();

                                            cloned_response_tx
                                                .send(PidgeyCommandResponse::PortRange {
                                                    id,
                                                    value: ports_online,
                                                })
                                                .await
                                                .unwrap()
                                        }
                                    },
                                    Err(error) => error!("{}", error),
                                }
                            }
        });
    }
}
