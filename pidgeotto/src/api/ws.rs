use std::borrow::Cow;

use axum::{
    body::Body,
    extract::{
        ws::{close_code, CloseFrame, Message, WebSocket},
        WebSocketUpgrade,
    },
    http::{HeaderMap, StatusCode},
    response::Response,
};

pub async fn ws_handler(
    ws: WebSocketUpgrade,
    headers: HeaderMap,
) -> Result<Response<Body>, StatusCode> {
    if !headers.contains_key("authorization") {
        return Err(StatusCode::BAD_REQUEST);
    }

    let header_token = headers
        .get("authorization")
        .unwrap()
        .to_str()
        .unwrap()
        .split(" ")
        .collect::<Vec<&str>>();
    if header_token.len() < 2 {
        return Err(StatusCode::BAD_REQUEST);
    }

    Ok(ws.on_upgrade(move |socket| socket_handler(socket)))
}

pub async fn socket_handler(mut socket: WebSocket) {
    socket
        .send(Message::Close(Some(CloseFrame {
            code: close_code::NORMAL,
            reason: Cow::from("Goodbye!"),
        })))
        .await
        .unwrap();
}
