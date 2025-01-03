use std::{collections::HashMap, str::FromStr, sync::Arc};

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
    Extension,
};
use futures::{SinkExt, StreamExt};
use jsonwebtoken::TokenData;
use mtilib::{auth::JWTClaims, pidgey::PidgeyCommandResponse};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::pidgey::{PidgeyUnit, PidgeyUnitRequest, PidgeyUnitResponse};

use super::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    Extension(token): Extension<TokenData<JWTClaims>>,
    State(state): State<AppState>,
) -> Response<Body> {
    ws.on_upgrade(move |socket| socket_handler(socket, token.claims, state))
}

pub async fn socket_handler(socket: WebSocket, jwt: JWTClaims, state: AppState) {
    let (mut ws_write, mut ws_read) = socket.split();

    let (unit_sender, mut unit_receiver) = tokio::sync::mpsc::channel::<PidgeyUnitRequest>(
        state
            .config
            .get_int("settings.scanner.max_tasks")
            .unwrap_or(512) as usize,
    );

    let jobs: Arc<Mutex<HashMap<Uuid, tokio::sync::oneshot::Sender<PidgeyUnitResponse>>>> =
        Arc::new(Mutex::new(HashMap::new()));

    let cloned_jwt = jwt.clone();
    let cloned_state = state.clone();
    let cloned_jobs = jobs.clone();
    let mut ws_recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_read.next().await {
            match msg {
                Message::Text(t) => {
                    let command = serde_json::from_str::<PidgeyCommandResponse>(&t).unwrap();
                    match command {
                        PidgeyCommandResponse::Register => {
                            let unit_uuid = Uuid::from_str(&cloned_jwt.id).unwrap();

                            if !cloned_state.pidgey.is_registered(&unit_uuid).await {
                                cloned_state
                                    .pidgey
                                    .register_unit(PidgeyUnit::new(
                                        Uuid::from_str(&cloned_jwt.id).unwrap(),
                                        unit_sender.clone(),
                                    ))
                                    .await;
                            }
                        }
                        PidgeyCommandResponse::Deregister => {
                            cloned_state
                                .pidgey
                                .deregister_unit(&Uuid::from_str(&cloned_jwt.id).unwrap())
                                .await;
                        }
                        PidgeyCommandResponse::Query {
                            id,
                            allocation_state,
                            top_rir,
                            rir,
                            asn,
                            country,
                            online,
                            ports,
                        } => cloned_jobs
                            .lock()
                            .await
                            .remove(&id)
                            .unwrap()
                            .send(PidgeyUnitResponse::Query {
                                allocation_state,
                                top_rir,
                                rir,
                                asn,
                                country,
                                online,
                                ports,
                            })
                            .unwrap(),
                        PidgeyCommandResponse::AllocationState { id, value } => cloned_jobs
                            .lock()
                            .await
                            .remove(&id)
                            .unwrap()
                            .send(PidgeyUnitResponse::AllocationState(value))
                            .unwrap(),
                        PidgeyCommandResponse::Rir { id, value } => cloned_jobs
                            .lock()
                            .await
                            .remove(&id)
                            .unwrap()
                            .send(PidgeyUnitResponse::Rir(value))
                            .unwrap(),
                        PidgeyCommandResponse::Asn { id, value } => cloned_jobs
                            .lock()
                            .await
                            .remove(&id)
                            .unwrap()
                            .send(PidgeyUnitResponse::Asn(value))
                            .unwrap(),
                        PidgeyCommandResponse::Country { id, value } => cloned_jobs
                            .lock()
                            .await
                            .remove(&id)
                            .unwrap()
                            .send(PidgeyUnitResponse::Country(value))
                            .unwrap(),
                        PidgeyCommandResponse::Online { id, value, reason } => cloned_jobs
                            .lock()
                            .await
                            .remove(&id)
                            .unwrap()
                            .send(PidgeyUnitResponse::Online { value, reason })
                            .unwrap(),
                        PidgeyCommandResponse::Port { id, value } => cloned_jobs
                            .lock()
                            .await
                            .remove(&id)
                            .unwrap()
                            .send(PidgeyUnitResponse::Port(value))
                            .unwrap(),
                        PidgeyCommandResponse::PortRange { id, value } => cloned_jobs
                            .lock()
                            .await
                            .remove(&id)
                            .unwrap()
                            .send(PidgeyUnitResponse::PortRange(value))
                            .unwrap(),
                    }
                }
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    });

    let mut unit_recv_task = tokio::spawn(async move {
        while let Some(message) = unit_receiver.recv().await {
            let mut lock = jobs.lock().await;
            lock.insert(message.id, message.response);
            drop(lock);

            ws_write
                .send(Message::Text(
                    serde_json::to_string(&message.command).unwrap(),
                ))
                .await
                .unwrap();
        }
    });

    tokio::select! {
        _ = &mut ws_recv_task => {
            state.pidgey.deregister_unit(&Uuid::from_str(&jwt.id).unwrap()).await;
            unit_recv_task.abort();
        }
        _ = &mut unit_recv_task => {
            ws_recv_task.abort();
        }
    }
}
