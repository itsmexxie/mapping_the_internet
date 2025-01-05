use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio_tungstenite::connect_async;
use tracing::error;
pub use url::Url;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum UnitMessage {
    Register {
        token: String,
        address: String,
        port: Option<u16>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PokedexMessage {
    Registered { uuid: Uuid },
    Error { message: String },
}

pub struct Pokedex {
    address: Url,
    token: String,
}

impl Pokedex {
    pub async fn login<S: AsRef<str>>(
        address: &Url,
        username: S,
        password: S,
    ) -> Result<Self, reqwest::Error> {
        let mut login_address = address.clone();
        login_address.set_path("/v2/auth/login");

        let mut query = login_address.query_pairs_mut();
        query.append_pair("username", username.as_ref());
        query.append_pair("password", password.as_ref());

        drop(query);

        let client = reqwest::Client::new();
        let res = client.post(login_address.clone()).send().await?;

        Ok(Pokedex {
            address: address.clone(),
            token: res.text().await.unwrap(),
        })
    }

    // TODO: Proper error handling
    // TODO: Expose communication with Pokedex through WS
    pub async fn register<S: AsRef<str>>(
        &self,
        address: S,
        port: Option<u16>,
        callback: oneshot::Sender<Uuid>,
    ) {
        let mut register_address = self.address.clone();
        match register_address.scheme() {
            "https" => register_address.set_scheme("wss").unwrap(),
            _ => register_address.set_scheme("ws").unwrap(),
        };
        register_address.set_path("/v2/ws");

        let (ws_stream, _) = connect_async(register_address.as_str()).await.unwrap();
        let (mut ws_write, mut ws_read) = ws_stream.split();

        tokio::spawn(async move {
            let mut callback = Some(callback);
            while let Some(Ok(message)) = ws_read.next().await {
                match message {
                    tokio_tungstenite::tungstenite::Message::Text(utf8_bytes) => {
                        match serde_json::from_str::<PokedexMessage>(&utf8_bytes.as_str()) {
                            Ok(message) => match message {
                                PokedexMessage::Registered { uuid } => {
                                    if let Some(callback) = callback.take() {
                                        callback.send(uuid).unwrap();
                                    }
                                }
                                PokedexMessage::Error { message } => {
                                    error!("Error response from Pokedex: {}", message)
                                }
                            },
                            Err(error) => {
                                error!("Error while parsing response from Pokedex: {}", error);
                            }
                        }
                    }
                    _ => {}
                }
            }
        });

        ws_write
            .send(tokio_tungstenite::tungstenite::Message::Text(
                serde_json::to_string(&UnitMessage::Register {
                    token: self.token.clone(),
                    address: address.as_ref().to_string(),
                    port: port,
                })
                .unwrap()
                .into(),
            ))
            .await
            .unwrap();
    }
}
