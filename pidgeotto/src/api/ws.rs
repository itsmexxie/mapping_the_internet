use std::{collections::HashMap, sync::Arc};

use axum::{
    body::Body,
    extract::{
        ws::{Message, WebSocket},
        State, WebSocketUpgrade,
    },
    response::Response,
};
use futures::{SinkExt, StreamExt};
use mtilib::pidgey::{PidgeyCommandResponse, PidgeyCommandResponsePayload};
use tokio::sync::Mutex;
use uuid::Uuid;

use crate::pidgey::{PidgeyUnit, PidgeyUnitRequest};

use super::AppState;

pub async fn ws_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response<Body> {
    ws.on_upgrade(move |socket| socket_handler(socket, state))
}

pub async fn socket_handler(socket: WebSocket, state: AppState) {
    let (mut ws_write, mut ws_read) = socket.split();

    let (unit_sender, mut unit_receiver) =
        tokio::sync::mpsc::channel::<PidgeyUnitRequest>(state.settings.scanner.max_tasks);

    let jobs: Arc<
        Mutex<HashMap<Uuid, tokio::sync::oneshot::Sender<PidgeyCommandResponsePayload>>>,
    > = Arc::new(Mutex::new(HashMap::new()));

    let mut unit_uuid = None;

    let cloned_state = state.clone();
    let cloned_jobs = jobs.clone();
    let mut ws_recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_read.next().await {
            match msg {
                Message::Text(t) => {
                    let command_res = serde_json::from_str::<PidgeyCommandResponse>(&t).unwrap();
                    match command_res.payload {
                        PidgeyCommandResponsePayload::Register {
                            unit_uuid: new_unit_uuid,
                        } => {
                            if !cloned_state.pidgey.is_registered(&new_unit_uuid).await {
                                unit_uuid = Some(new_unit_uuid);
                                cloned_state
                                    .pidgey
                                    .register_unit(PidgeyUnit::new(
                                        new_unit_uuid,
                                        unit_sender.clone(),
                                    ))
                                    .await;
                            }
                        }
                        PidgeyCommandResponsePayload::Deregister => {
                            if unit_uuid.is_some() {
                                cloned_state
                                    .pidgey
                                    .deregister_unit(&unit_uuid.unwrap())
                                    .await;
                            }
                        }
                        _ => cloned_jobs
                            .lock()
                            .await
                            .remove(&command_res.id)
                            .unwrap()
                            .send(command_res.payload)
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
            lock.insert(message.command.id, message.response);
            drop(lock);

            ws_write
                .send(Message::Text(
                    serde_json::to_string(&message.command).unwrap().into(),
                ))
                .await
                .unwrap();
        }
    });

    tokio::select! {
        _ = &mut ws_recv_task => {
            if unit_uuid.is_some() {
                state.pidgey.deregister_unit(&unit_uuid.unwrap()).await;
            }
            unit_recv_task.abort();
        }
        _ = &mut unit_recv_task => {
            ws_recv_task.abort();
        }
    }
}
