use std::net::IpAddr;
use std::sync::Arc;

use axum::http::HeaderValue;
use config::Config;
use futures::{SinkExt, StreamExt};
use mtilib::pidgey::PidgeyCommand;
use surge_ping::SurgeError;
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tracing::{debug, error, info};

use crate::diglett::Diglett;
use crate::gust::Gust;

pub async fn run(config: Arc<Config>, jwt: Arc<String>, diglett: Arc<Diglett>) {
    if !config.get_bool("pidgeotto.connect").unwrap() {
        info!("pidgeotto.connect set to false, not connecting!");
        return;
    }

    // Get address
    let mut pidgeotto_address = config
        .get_string("pidgeotto.address")
        .expect("pidgeotto.address must be set!");

    if let Ok(pidgeotto_port) = config.get_string("pidgeotto.port") {
        pidgeotto_address = concat_string!(pidgeotto_address, ":", &pidgeotto_port);
    }
    pidgeotto_address = concat_string!("ws://", pidgeotto_address, "/ws");

    // Create request
    let pidgeotto_url: url::Url = url::Url::parse(&pidgeotto_address).unwrap();
    let mut pidgeotto_req = pidgeotto_url.into_client_request().unwrap();
    let headers = pidgeotto_req.headers_mut();
    headers.insert(
        "authorization",
        HeaderValue::from_str(&concat_string!("Bearer ", *jwt)).unwrap(),
    );

    // Connect to Pidgeotto
    let (ws_stream, _) = connect_async(pidgeotto_req)
        .await
        .expect("Failed to establish a websocket connection to Pidgeotto!");

    let (mut ws_write, mut ws_read) = ws_stream.split();

    ws_write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::to_string(&PidgeyCommand::Register).unwrap(),
        ))
        .await
        .unwrap();

    tokio::spawn(async move {
        while let Some(Ok(message)) = ws_read.next().await {
            debug!("Received message {}", message);
            match message {
                tokio_tungstenite::tungstenite::Message::Text(t) => {
                    match serde_json::from_str::<PidgeyCommand>(&t) {
                        Ok(command) => match command {
                            PidgeyCommand::AllocationState { address } => ws_write
                                .send(Message::Text(
                                    serde_json::to_string(&PidgeyCommand::AllocationStateRes {
                                        value: diglett
                                            .allocation_state(address)
                                            .await
                                            .unwrap()
                                            .to_string(),
                                    })
                                    .unwrap(),
                                ))
                                .await
                                .unwrap(),
                            PidgeyCommand::Rir { address, top } => ws_write
                                .send(Message::Text(
                                    serde_json::to_string(&PidgeyCommand::RirRes {
                                        value: diglett.rir(address, top).await.unwrap(),
                                    })
                                    .unwrap(),
                                ))
                                .await
                                .unwrap(),
                            PidgeyCommand::Asn { address } => ws_write
                                .send(Message::Text(
                                    serde_json::to_string(&PidgeyCommand::AsnRes {
                                        value: diglett.asn(address).await.unwrap(),
                                    })
                                    .unwrap(),
                                ))
                                .await
                                .unwrap(),
                            PidgeyCommand::Country { address } => ws_write
                                .send(Message::Text(
                                    serde_json::to_string(&PidgeyCommand::CountryRes {
                                        value: diglett.country(address).await.unwrap(),
                                    })
                                    .unwrap(),
                                ))
                                .await
                                .unwrap(),
                            PidgeyCommand::Online { address } => {
                                let payload = [0; 8];
                                let cmd =
                                    match surge_ping::ping(IpAddr::V4(address), &payload).await {
                                        Ok(_) => &PidgeyCommand::OnlineRes {
                                            value: true,
                                            reason: None,
                                        },
                                        Err(ping_error) => match ping_error {
                                            SurgeError::Timeout { seq: _ } => {
                                                &PidgeyCommand::OnlineRes {
                                                    value: false,
                                                    reason: Some(String::from("timeout")),
                                                }
                                            }
                                            _ => &PidgeyCommand::OnlineRes {
                                                value: false,
                                                reason: Some(String::from("unknown")),
                                            },
                                        },
                                    };

                                ws_write
                                    .send(Message::Text(serde_json::to_string(&cmd).unwrap()))
                                    .await
                                    .unwrap()
                            }
                            PidgeyCommand::PortRange {
                                address,
                                start,
                                end,
                            } => {
                                let gust = Arc::new(Gust::new(address).unwrap());

                                let gust_range_start = match start {
                                    Some(start) => start,
                                    None => match config.get_int("settings.gust.range.start") {
                                        Ok(start) => start as u16,
                                        Err(_) => 1,
                                    },
                                };

                                let gust_range_end = match end {
                                    Some(end) => end,
                                    None => match config.get_int("settings.gust.range.end") {
                                        Ok(end) => end as u16,
                                        Err(_) => 999,
                                    },
                                };

                                let ports_online = gust
                                    .attack_range(
                                        gust_range_start..=gust_range_end,
                                        config.get_int("settings.gust.timeout").unwrap_or(10)
                                            as u32,
                                    )
                                    .await
                                    .unwrap();

                                ws_write
                                    .send(Message::Text(
                                        serde_json::to_string(&PidgeyCommand::PortRangeRes {
                                            value: ports_online,
                                        })
                                        .unwrap(),
                                    ))
                                    .await
                                    .unwrap()
                            }
                            PidgeyCommand::Port { address, port } => {
                                let gust = Gust::new(address).unwrap();
                                let port_online = gust
                                    .attack(
                                        port,
                                        config.get_int("settings.gust.timeout").unwrap_or(5) as u32,
                                    )
                                    .await;

                                ws_write
                                    .send(Message::Text(
                                        serde_json::to_string(&PidgeyCommand::PortRes {
                                            value: port_online,
                                        })
                                        .unwrap(),
                                    ))
                                    .await
                                    .unwrap()
                            }
                            _ => {}
                        },
                        Err(error) => error!("{}", error),
                    }
                }
                tokio_tungstenite::tungstenite::Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    })
    .await
    .unwrap();
}
