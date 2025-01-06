use concat_string::concat_string;
use config::Config;
use core::panic;
use mtilib::{
    pokedex::Pokedex,
    types::{AllocationState, Rir, ValueResponse},
};
use std::{net::Ipv4Addr, str::FromStr, sync::Arc};
use tokio::sync::Mutex;
use tracing::{error, info};
use url::Url;

pub struct Diglett {
    client: reqwest::Client,
    url: Url,
}

impl Diglett {
    pub async fn new(config: &Arc<Config> /*pokedex: Arc<Mutex<Pokedex>>*/) -> Self {
        let diglett_client = reqwest::Client::new();

        if let Ok(diglett_address) = config.get_string("diglett.address") {
            info!("diglett.address set, trying to connect...");

            match url::Url::parse(&diglett_address) {
                Ok(mut diglett_url) => {
                    if let Ok(port) = config.get_int("diglett.port") {
                        diglett_url.set_port(Some(port as u16)).unwrap()
                    }

                    match diglett_client.get(diglett_url.clone()).send().await {
                        Ok(_) => {
                            info!("Successfully connected to the configured diglett instance!");
                            return Diglett {
                                client: diglett_client,
                                url: diglett_url,
                            };
                        }
                        Err(err) => {
                            error!("Failed to connect to the configured diglett instance, trying units from Pokedex... ({})", err);
                        }
                    }
                }
                Err(_) => error!(
                    "Failed to parse configured diglett address, trying units from Pokedex..."
                ),
            };
        }

        // let services = pokedex
        //     .get_services()
        //     .await
        //     .expect("Failed to retrieve services from Pokedex!");

        // let diglett_service = services
        //     .iter()
        //     .find(|s| s.name == "diglett")
        //     .expect("Failed to find the diglett service!");

        // let mut units = pokedex
        //     .get_service_units(diglett_service.id)
        //     .await
        //     .expect("Failed to retrieve available diglett units!");

        // units.shuffle(&mut rand::thread_rng());
        // let mut i = 0;
        // while i < units.len() {
        //     if let Some(diglett_address) = &units[i].address {
        //         match url::Url::parse(diglett_address) {
        //             Ok(mut diglett_url) => {
        //                 if let Some(port) = units[i].port {
        //                     diglett_url.set_port(Some(port as u16)).unwrap()
        //                 }

        //                 match diglett_client.get(diglett_url.clone()).send().await {
        //                     Ok(_) => {
        //                         info!("Successfully connected to a diglett instance!");

        //                         return Diglett {
        //                             client: diglett_client,
        //                             url: diglett_url,
        //                         };
        //                     }
        //                     Err(_) => {
        //                         error!("Error while connecting to diglett, trying another...")
        //                     }
        //                 }
        //             }
        //             Err(_) => error!("Failed to parse diglett address, trying another..."),
        //         }
        //     }
        //     i += 1;
        // }

        panic!(
            "Failed to create Diglett client, no running or correctly configured instances found!"
        );
    }

    pub async fn allocation_state(
        &self,
        address: Ipv4Addr,
    ) -> Result<AllocationState, reqwest::StatusCode> {
        match self
            .client
            .get(concat_string!(self.url, address.to_string(), "/allocation"))
            .send()
            .await
        {
            Ok(res) => {
                let data: ValueResponse<String> = res.json().await.unwrap();
                Ok(AllocationState::from_str(&data.value).unwrap())
            }
            Err(_) => Err(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    pub async fn rir(
        &self,
        address: Ipv4Addr,
        top: bool,
    ) -> Result<Option<Rir>, reqwest::StatusCode> {
        let mut request_url = concat_string!(self.url, address.to_string(), "/rir");
        if top {
            request_url = concat_string!(request_url, "?top=true");
        }

        match self.client.get(request_url).send().await {
            Ok(res) => match res.status() {
                reqwest::StatusCode::OK => {
                    let data: ValueResponse<Option<String>> = res.json().await.unwrap();
                    match data.value {
                        Some(value) => Ok(Some(Rir::from_str(&value).unwrap())),
                        None => Ok(None),
                    }
                }
                reqwest::StatusCode::BAD_REQUEST => Err(reqwest::StatusCode::BAD_REQUEST),
                _ => Err(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
            },
            Err(_) => Err(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    pub async fn asn(&self, address: Ipv4Addr) -> Result<Option<u32>, reqwest::StatusCode> {
        match self
            .client
            .get(concat_string!(self.url, address.to_string(), "/asn"))
            .send()
            .await
        {
            Ok(res) => match res.status() {
                reqwest::StatusCode::OK => {
                    let data: ValueResponse<Option<u32>> = res.json().await.unwrap();
                    Ok(data.value)
                }
                reqwest::StatusCode::BAD_REQUEST => Err(reqwest::StatusCode::BAD_REQUEST),
                _ => Err(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
            },
            Err(_) => Err(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
        }
    }

    pub async fn country(&self, address: Ipv4Addr) -> Result<Option<String>, reqwest::StatusCode> {
        match self
            .client
            .get(concat_string!(self.url, address.to_string(), "/country"))
            .send()
            .await
        {
            Ok(res) => match res.status() {
                reqwest::StatusCode::OK => {
                    let data: ValueResponse<Option<String>> = res.json().await.unwrap();
                    Ok(data.value)
                }
                reqwest::StatusCode::BAD_REQUEST => Err(reqwest::StatusCode::BAD_REQUEST),
                _ => Err(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
            },
            Err(_) => Err(reqwest::StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
