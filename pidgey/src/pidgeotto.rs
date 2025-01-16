use axum::http::HeaderValue;
use concat_string::concat_string;
use futures::{SinkExt, StreamExt};
use mtilib::pidgey::{
    PidgeyCommand, PidgeyCommandPayload, PidgeyCommandResponse, PidgeyCommandResponsePayload,
};
use mtilib::pokedex::Pokedex;
use mtilib::types::AllocationState;
use rand::random;
use rand::seq::SliceRandom;
use std::net::IpAddr;
use std::sync::Arc;
use surge_ping::{PingIdentifier, PingSequence, SurgeError};
use tokio::net::TcpStream;
use tokio::sync::{Mutex, Semaphore};
use tokio_tungstenite::tungstenite::client::IntoClientRequest;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream};
use tracing::{debug, error, info};
use url::Url;
use uuid::Uuid;

use crate::diglett::Diglett;
use crate::settings::Settings;

async fn try_connect(
    url: &Url,
    token: &String,
) -> Result<
    (
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        axum::http::Response<Option<Vec<u8>>>,
    ),
    tokio_tungstenite::tungstenite::Error,
> {
    let mut url = url.clone();
    url.set_path("/ws");

    // Create request
    let mut req = url.into_client_request().unwrap();
    let headers = req.headers_mut();
    headers.insert(
        "authorization",
        HeaderValue::from_str(&concat_string!("Bearer ", token)).unwrap(),
    );

    connect_async(req).await
}

#[derive(Debug)]
enum PidgeottoError {
    NotFound,
    Tungstenite(tokio_tungstenite::tungstenite::Error),
}

async fn connect(
    settings: Arc<Settings>,
    pokedex: Arc<Mutex<Pokedex>>,
) -> Result<
    (
        WebSocketStream<MaybeTlsStream<TcpStream>>,
        axum::http::Response<Option<Vec<u8>>>,
    ),
    PidgeottoError,
> {
    if let Some(pidgeotto_settings) = settings.pidgeotto.as_ref() {
        if let Some(pidgeotto_address) = pidgeotto_settings.address.as_ref() {
            if let Ok(pidgeotto_url) = Url::parse(&pidgeotto_address) {
                match try_connect(&pidgeotto_url, &pokedex.lock().await.get_token()).await {
                    Ok(ws_stream) => return Ok(ws_stream),
                    Err(error) => return Err(PidgeottoError::Tungstenite(error)),
                }
            }
        }
    }

    let mut units = pokedex.lock().await.get_service_units("pidgeotto").await;
    units.shuffle(&mut rand::thread_rng());

    let mut i = 0;
    while i < units.len() {
        match Url::parse(&units[i].address) {
            Ok(mut pidgeotto_url) => {
                if let Some(pidgeotto_port) = units[i].port {
                    pidgeotto_url.set_port(Some(pidgeotto_port as u16)).unwrap();
                }

                match try_connect(&pidgeotto_url, &pokedex.lock().await.get_token()).await {
                    Ok(ws_stream) => return Ok(ws_stream),
                    Err(error) => return Err(PidgeottoError::Tungstenite(error)),
                }
            }
            Err(_) => error!(
                "Failed to parse diglett address, trying another... ({})",
                units[i].address
            ),
        }
        i += 1;
    }

    Err(PidgeottoError::NotFound)
}

pub async fn run(
    settings: Arc<Settings>,
    mut unit_uuid: Arc<Option<Uuid>>,
    worker_permits: Arc<Semaphore>,
    pokedex: Arc<Mutex<Pokedex>>,
    diglett: Arc<Diglett>,
    ping_client: Arc<surge_ping::Client>,
) {
    if let Some(pidgeotto_settings) = settings.pidgeotto.as_ref() {
        if !pidgeotto_settings.connect {
            info!("pidgeotto.connect set to false, not connecting!");
            return;
        }
    }

    // If we haven't registered to Pokedex, make a UUID on the spot
    if unit_uuid.is_none() {
        unit_uuid = Arc::new(Some(Uuid::new_v4()));
    };

    // Connect to Pidgeotto
    let (ws_stream, _) = match connect(settings.clone(), pokedex).await {
        Ok(stream) => {
            info!("Successfully established a websocket connection to a Pidgeotto instance!");
            stream
        }
        Err(error) => match error {
            PidgeottoError::NotFound => panic!("No available Pidgeotto units found!"),
            PidgeottoError::Tungstenite(error) => panic!(
                "Failed to establish a websocket connection to Pidgeotto! ({})",
                error
            ),
        },
    };

    let (mut ws_write, mut ws_read) = ws_stream.split();

    ws_write
        .send(tokio_tungstenite::tungstenite::Message::Text(
            serde_json::to_string(&PidgeyCommandResponse {
                id: Uuid::new_v4(),
                payload: PidgeyCommandResponsePayload::Register {
                    unit_uuid: unit_uuid.unwrap(),
                },
            })
            .unwrap()
            .into(),
        ))
        .await
        .unwrap();

    let (response_tx, mut response_rx) =
        tokio::sync::mpsc::channel::<PidgeyCommandResponse>(settings.max_workers);

    // Websocket write task
    // Can't clone the resulting ws_write from tungstenite, so only this task writes to the websocket
    // while other tasks use tokio::sync::mpsc channels to communicate with this task
    tokio::spawn(async move {
        while let Some(response) = response_rx.recv().await {
            ws_write
                .send(Message::Text(
                    serde_json::to_string(&response).unwrap().into(),
                ))
                .await
                .unwrap();
        }
    });

    while let Some(Ok(message)) = ws_read.next().await {
        let cloned_worker_permits = worker_permits.clone();
        let cloned_diglett = diglett.clone();
        let cloned_ping_client = ping_client.clone();
        let cloned_response_tx = response_tx.clone();
        tokio::spawn(async move {
            debug!("Received message {}", message);

            if let tokio_tungstenite::tungstenite::Message::Text(t) = message {
                match serde_json::from_str::<PidgeyCommand>(&t) {
                    Ok(command) => match command.payload {
                        PidgeyCommandPayload::Query { address } => {
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
                                    address, status
                                ),
                            };
                            let autsys = match cloned_diglett.asn(address).await {
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
                                    .send(PidgeyCommandResponse {
                                        id: command.id,
                                        payload:
                                            mtilib::pidgey::PidgeyCommandResponsePayload::Query {
                                                allocation_state: alloc_state,
                                                top_rir,
                                                rir,
                                                autsys,
                                                country,
                                                online: false,
                                            },
                                    })
                                    .await
                                    .unwrap();
                            } else {
                                let payload = [0; 8];
                                let mut pinger = cloned_ping_client
                                    .pinger(IpAddr::V4(address), PingIdentifier(random()))
                                    .await;
                                let online = pinger.ping(PingSequence(0), &payload).await.is_ok();

                                cloned_response_tx
                                    .send(PidgeyCommandResponse {
                                        id: command.id,
                                        payload:
                                            mtilib::pidgey::PidgeyCommandResponsePayload::Query {
                                                allocation_state: match online {
                                                    true => AllocationState::Allocated, // If the address is online then the state must be allocated
                                                    false => alloc_state,
                                                },
                                                top_rir,
                                                rir,
                                                autsys,
                                                country,
                                                online,
                                            },
                                    })
                                    .await
                                    .unwrap();
                            }
                        }
                        PidgeyCommandPayload::AllocationState { address } => {
                            let _permit = cloned_worker_permits.acquire().await.unwrap();
                            cloned_response_tx
                                .send(PidgeyCommandResponse {
                                    id: command.id,
                                    payload: mtilib::pidgey::PidgeyCommandResponsePayload::AllocationState {
                                        value: cloned_diglett.allocation_state(address).await.unwrap(),
                                    }
                                })
                                .await
                                .unwrap()
                        }
                        PidgeyCommandPayload::Rir { address, top } => {
                            let _permit = cloned_worker_permits.acquire().await.unwrap();
                            cloned_response_tx
                                .send(PidgeyCommandResponse {
                                    id: command.id,
                                    payload: mtilib::pidgey::PidgeyCommandResponsePayload::Rir {
                                        value: cloned_diglett.rir(address, top).await.unwrap(),
                                    },
                                })
                                .await
                                .unwrap()
                        }
                        PidgeyCommandPayload::Autsys { address } => {
                            let _permit = cloned_worker_permits.acquire().await.unwrap();
                            cloned_response_tx
                                .send(PidgeyCommandResponse {
                                    id: command.id,
                                    payload: mtilib::pidgey::PidgeyCommandResponsePayload::Autsys {
                                        value: cloned_diglett.asn(address).await.unwrap(),
                                    },
                                })
                                .await
                                .unwrap()
                        }
                        PidgeyCommandPayload::Country { address } => {
                            let _permit = cloned_worker_permits.acquire().await.unwrap();
                            cloned_response_tx
                                .send(PidgeyCommandResponse {
                                    id: command.id,
                                    payload:
                                        mtilib::pidgey::PidgeyCommandResponsePayload::Country {
                                            value: cloned_diglett.country(address).await.unwrap(),
                                        },
                                })
                                .await
                                .unwrap()
                        }
                        PidgeyCommandPayload::Online { address } => {
                            let _permit = cloned_worker_permits.acquire().await.unwrap();
                            let payload = [0; 8];
                            let cmd = match surge_ping::ping(IpAddr::V4(address), &payload).await {
                                Ok(_) => PidgeyCommandResponse {
                                    id: command.id,
                                    payload: mtilib::pidgey::PidgeyCommandResponsePayload::Online {
                                        value: true,
                                        reason: None,
                                    },
                                },
                                Err(ping_error) => match ping_error {
                                    SurgeError::Timeout { seq: _ } => PidgeyCommandResponse {
                                        id: command.id,
                                        payload: PidgeyCommandResponsePayload::Online {
                                            value: false,
                                            reason: Some(String::from("timeout")),
                                        },
                                    },
                                    _ => PidgeyCommandResponse {
                                        id: command.id,
                                        payload: PidgeyCommandResponsePayload::Online {
                                            value: false,
                                            reason: Some(String::from("unknown")),
                                        },
                                    },
                                },
                            };

                            cloned_response_tx.send(cmd).await.unwrap()
                        }
                        _ => {}
                    },
                    Err(error) => error!("{}", error),
                }
            }
        });
    }
}
