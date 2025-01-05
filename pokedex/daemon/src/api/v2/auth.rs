use axum::{
    extract::{ws::WebSocket, State, WebSocketUpgrade},
    http::StatusCode,
    response::IntoResponse,
    routing::any,
    Router,
};
use futures::{SinkExt, StreamExt};
use mtilib::db::models::Service;
use serde::{Deserialize, Serialize};
use tracing::info;
use uuid::Uuid;

use crate::api::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnitMessage {
    Register {
        username: String,
        password: String,
        address: Option<String>,
        port: Option<i32>,
    },
    Error {
        message: String,
    },
}

pub async fn login(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| unit_handler(socket, state))
}

pub async fn unit_handler(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let mut unit_uuid = None;

    while let Some(Ok(message)) = receiver.next().await {
        match message {
            axum::extract::ws::Message::Text(utf8_bytes) => {
                match serde_json::from_str::<UnitMessage>(&utf8_bytes.as_str()) {
                    Ok(message) => match message {
                        UnitMessage::Register {
                            username,
                            password,
                            address,
                            port,
                        } => {
                            if unit_uuid.is_some() {
                                continue;
                            }

                            let service = match sqlx::query_as::<_, Service>(
                                r#"
                            	SELECT *
                            	FROM "Services"
                            	WHERE id = $1
                            	"#,
                            )
                            .bind(username.clone())
                            .fetch_one(&mut *state.db_pool.acquire().await.unwrap())
                            .await
                            {
                                Ok(service) => service,
                                Err(_) => {
                                    sender
                                        .send(axum::extract::ws::Message::Text(
                                            StatusCode::INTERNAL_SERVER_ERROR.to_string().into(),
                                        ))
                                        .await
                                        .unwrap();
                                    continue;
                                }
                            };

                            if bcrypt::verify(&password, &service.password).unwrap() {
                                unit_uuid = Some(Uuid::new_v4());

                                info!("Registered unit {:?}", unit_uuid.unwrap());

                                sqlx::query(
                                    r#"
									INSERT INTO "ServiceUnits"
									VALUES ($1, $2, $3, $4)
									"#,
                                )
                                .bind(unit_uuid.unwrap())
                                .bind(service.id)
                                .bind(address)
                                .bind(port)
                                .execute(&mut *state.db_pool.acquire().await.unwrap())
                                .await
                                .unwrap();
                            } else {
                                return sender
                                    .send(axum::extract::ws::Message::Close(None))
                                    .await
                                    .unwrap();
                            }
                        }
                        _ => {}
                    },
                    Err(_) => sender
                        .send(axum::extract::ws::Message::Text(
                            String::from("Failed to parse message").into(),
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
    Router::new().route("/login", any(login))
}
