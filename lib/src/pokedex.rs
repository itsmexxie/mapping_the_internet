use core::panic;

use concat_string::concat_string;
use futures::{SinkExt, StreamExt};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use tokio::sync::oneshot;
use tokio_tungstenite::connect_async;
use tracing::error;
use types::Service;
pub use url::Url;
use uuid::Uuid;

use crate::db::models::ServiceUnit;

pub mod types;
pub mod ws;

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
    // TODO: Make a separate client which handles communication with Pokedex ws
    // Probably will spawn a task and return a wrapper around a mpsc::Sender for sending commands
    // Not sure about receiving
    pub async fn register<S: AsRef<str>>(
        &self,
        address: S,
        port: Option<u16>,
        callback: oneshot::Sender<Uuid>,
    ) {
        #[cfg(feature = "rustls")]
        rustls::crypto::ring::default_provider()
            .install_default()
            .expect("Failed to install rustls crypto provider");

        let mut register_address = self.address.clone();
        match register_address.scheme() {
            "https" => register_address.set_scheme("wss").unwrap(),
            _ => register_address.set_scheme("ws").unwrap(),
        };
        register_address.set_path("/v2/ws");

        let (ws_stream, _) = connect_async(register_address).await.unwrap();
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

    pub fn get_token(&self) -> String {
        self.token.clone()
    }

    // TODO: Proper error handling
    pub async fn get_services(&self) -> Vec<Service> {
        let mut services_url = self.address.clone();
        services_url.set_path("/v2/services");

        match reqwest::Client::new()
            .get(services_url)
            .bearer_auth(self.token.clone())
            .send()
            .await
        {
            Ok(res) => match res.status() {
                StatusCode::OK => res.json().await.unwrap(),
                status => panic!("Error while getting Pokedex services! (status: {})", status),
            },
            Err(error) => panic!("Error while getting Pokedex services! ({})", error),
        }
    }

    pub async fn get_service(&self, service_id: String) -> Service {
        let mut service_url = self.address.clone();
        service_url.set_path(&concat_string!("/v2/service/", service_id));

        match reqwest::Client::new()
            .get(service_url)
            .bearer_auth(self.token.clone())
            .send()
            .await
        {
            Ok(res) => match res.status() {
                StatusCode::OK => res.json().await.unwrap(),
                status => panic!("Error while getting Pokedex services! (status: {})", status),
            },
            Err(error) => panic!("Error while getting Pokedex service! ({})", error),
        }
    }

    pub async fn get_service_units(&self, service_id: String) -> Vec<ServiceUnit> {
        let mut service_units_url = self.address.clone();
        service_units_url.set_path(&concat_string!("/v2/service/", service_id, "/units"));

        match reqwest::Client::new()
            .get(service_units_url)
            .bearer_auth(self.token.clone())
            .send()
            .await
        {
            Ok(res) => match res.status() {
                StatusCode::OK => res.json().await.unwrap(),
                status => panic!("Error while getting Pokedex services! (status: {})", status),
            },
            Err(error) => panic!("Error while getting Pokedex service! ({})", error),
        }
    }
}
