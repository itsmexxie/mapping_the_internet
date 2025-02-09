use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    response::IntoResponse,
    routing::any,
    Router,
};
use futures::{SinkExt, StreamExt};
use mtilib::{
    auth::JWTClaims,
    pokedex::{PokedexMessage, UnitMessage},
};
use tracing::info;
use uuid::Uuid;

use crate::api::AppState;

pub async fn register(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| unit_handler(socket, state))
}

async fn unit_handler(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut unit_uuid = None;

    while let Some(Ok(message)) = receiver.next().await {
        match message {
            axum::extract::ws::Message::Text(utf8_bytes) => {
                match serde_json::from_str::<UnitMessage>(&utf8_bytes.as_str()) {
                    Ok(message) => match message {
                        UnitMessage::Register {
                            token,
                            address,
                            port,
                        } => {
                            if unit_uuid.is_some() {
                                sender
                                    .send(axum::extract::ws::Message::Text(
                                        serde_json::to_string(&PokedexMessage::Error {
                                            message: String::from("Already registered!"),
                                        })
                                        .unwrap()
                                        .into(),
                                    ))
                                    .await
                                    .unwrap();
                                continue;
                            }

                            match jsonwebtoken::decode::<JWTClaims>(
                                &token,
                                &jsonwebtoken::DecodingKey::from_rsa_pem(&state.jwt_keys.public)
                                    .unwrap(),
                                &jsonwebtoken::Validation::new(jsonwebtoken::Algorithm::RS512),
                            ) {
                                Ok(token) => {
                                    unit_uuid = Some(Uuid::new_v4());
                                    let unit_port = match port {
                                        Some(port) => Some(port as i32),
                                        None => None,
                                    };

                                    info!("Registered unit {:?}", unit_uuid.unwrap());

                                    sqlx::query(
                                        r#"
                                            INSERT INTO "ServiceUnits"
                                            VALUES ($1, $2, $3, $4)
                                            "#,
                                    )
                                    .bind(unit_uuid.unwrap())
                                    .bind(token.claims.sub)
                                    .bind(address)
                                    .bind(unit_port)
                                    .execute(&mut *state.db_pool.acquire().await.unwrap())
                                    .await
                                    .unwrap();

                                    sender
                                        .send(axum::extract::ws::Message::Text(
                                            serde_json::to_string(&PokedexMessage::Registered {
                                                uuid: unit_uuid.unwrap(),
                                            })
                                            .unwrap()
                                            .into(),
                                        ))
                                        .await
                                        .unwrap();
                                }
                                Err(_) => {
                                    // Begone unauthorized units and other connections!
                                    return sender
                                        .send(axum::extract::ws::Message::Close(None))
                                        .await
                                        .unwrap();
                                }
                            }
                        }
                    },
                    Err(_) => sender
                        .send(axum::extract::ws::Message::Text(
                            serde_json::to_string(&PokedexMessage::Error {
                                message: String::from("Failed to parse message!"),
                            })
                            .unwrap()
                            .into(),
                        ))
                        .await
                        .unwrap(),
                }
            }
            _ => {}
        }
    }

    if let Some(unit_uuid) = unit_uuid {
        info!("Deregistered unit {:?}", unit_uuid);

        sqlx::query(
            r#"
				DELETE FROM "ServiceUnits"
				WHERE id = $1
				"#,
        )
        .bind(unit_uuid)
        .execute(&mut *state.db_pool.acquire().await.unwrap())
        .await
        .unwrap();
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/", any(register))
}
