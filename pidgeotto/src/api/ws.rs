use std::{borrow::Cow, net::Ipv4Addr, str::FromStr, sync::Arc};

use axum::{
    body::Body,
    extract::{
        ws::{close_code, CloseFrame, Message, WebSocket},
        State, WebSocketUpgrade,
    },
    http::{HeaderMap, StatusCode},
    response::Response,
};
use futures::{SinkExt, StreamExt};
use jsonwebtoken::{DecodingKey, TokenData, Validation};
use mtilib::{auth::JWTClaims, pidgey::PidgeyCommand};
use tracing::error;
use uuid::Uuid;

use crate::pidgey::PidgeyUnit;

use super::AppState;

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
    State(state): State<AppState>,
) -> Result<Response<Body>, StatusCode> {
    if !headers.contains_key("authorization") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let header_token = headers
        .get("authorization")
        .unwrap()
        .to_str()
        .unwrap()
        .split(" ")
        .collect::<Vec<&str>>();
    if header_token.len() < 2 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match jsonwebtoken::decode::<JWTClaims>(
        header_token[1],
        &DecodingKey::from_rsa_pem(&state.jwt_keys.public).unwrap(),
        &Validation::new(jsonwebtoken::Algorithm::RS512),
    ) {
        Ok(token) => Ok(ws.on_upgrade(move |socket| socket_handler(socket, token.claims, state))),
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

pub async fn socket_handler(socket: WebSocket, jwt: JWTClaims, state: AppState) {
    let (mut ws_write, mut ws_read) = socket.split();

    let (unit_sender, mut unit_receiver) = tokio::sync::mpsc::channel::<PidgeyCommand>(32);

    let cloned_jwt = jwt.clone();
    let cloned_state = state.clone();
    let mut ws_recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = ws_read.next().await {
            match msg {
                Message::Text(t) => match serde_json::from_str::<PidgeyCommand>(&t).unwrap() {
                    PidgeyCommand::Register => {
                        let unit_uuid = Uuid::from_str(&cloned_jwt.id).unwrap();

                        if !cloned_state.pidgey.read().await.is_registered(&unit_uuid) {
                            cloned_state
                                .pidgey
                                .write()
                                .await
                                .register_unit(PidgeyUnit::new(
                                    Uuid::from_str(&cloned_jwt.id).unwrap(),
                                    unit_sender.clone(),
                                ));
                        }

                        ws_write
                            .send(Message::Text(
                                match serde_json::to_string(&PidgeyCommand::AllocationState {
                                    address: Ipv4Addr::from_str("1.1.1.1").unwrap(),
                                }) {
                                    Ok(command) => command,
                                    Err(error) => {
                                        error!("{}", error);
                                        "".to_string()
                                    }
                                },
                            ))
                            .await
                            .unwrap();
                    }
                    _ => {}
                },
                Message::Close(_) => {
                    break;
                }
                _ => {}
            }
        }
    });

    let mut unit_recv_task =
        tokio::spawn(async move { while let Some(message) = unit_receiver.recv().await {} });

    tokio::select! {
        _ = &mut ws_recv_task => {
            state.pidgey.write().await.deregister_unit(&Uuid::from_str(&jwt.id).unwrap());
            unit_recv_task.abort();
        }
        _ = &mut unit_recv_task => {
            ws_recv_task.abort();
        }
    }

    // sender
    //     .send(Message::Close(Some(CloseFrame {
    //         code: close_code::NORMAL,
    //         reason: Cow::from("Goodbye!"),
    //     })))
    //     .await
    //     .unwrap();
}
