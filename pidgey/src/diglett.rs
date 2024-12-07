use core::panic;
use std::{net::Ipv4Addr, str::FromStr, sync::Arc};

use config::Config;
use mtilib::{
    pokedex::Pokedex,
    types::{AllocationState, Rir},
};
use rand::seq::SliceRandom;
use tokio::sync::Mutex;
use tracing::{error, info};

use crate::utils::ValueResponse;

pub struct Diglett {
    client: reqwest::Client,
    url: String,
}

impl Diglett {
    pub async fn new(config: &Arc<Config>, pokedex: Arc<Mutex<Pokedex>>) -> Self {
        let diglett_client = reqwest::Client::new();

        if let Ok(mut diglett_address) = config.get_string("diglett.address") {
            info!("diglett.address set, trying to connect...");

            let diglett_port = match config.get_string("diglett.port") {
                Ok(port) => port,
                Err(_) => "".to_string(),
            };
            diglett_address = concat_string!(diglett_address, diglett_port);

            match diglett_client.get(&diglett_address).send().await {
                Ok(_) => {
                    info!("Successfully connected to a diglett instance!");
                    return Diglett {
                        client: diglett_client,
                        url: diglett_address,
                    };
                }
                Err(_) => {}
            }
        }

        let services = pokedex
            .lock()
            .await
            .get_services()
            .await
            .expect("Failed to retrieve services from Pokedex!");

        let diglett_service = services
            .iter()
            .find(|s| s.name == "diglett")
            .expect("Failed to find the diglett service!");

        let mut units = pokedex
            .lock()
            .await
            .get_service_units(diglett_service.id)
            .await
            .expect("Failed to retrieve available diglett units!");

        units.shuffle(&mut rand::thread_rng());
        let mut i = 0;
        while i < units.len() {
            if let Some(pidgeotto_address) = &units[i].address {
                match diglett_client.get(pidgeotto_address).send().await {
                    Ok(_) => {
                        info!("Successfully connected to a diglett instance!");
                        let mut url = pidgeotto_address.to_owned();
                        if !url.ends_with("/") {
                            url = concat_string!(url, "/");
                        }

                        return Diglett {
                            client: diglett_client,
                            url,
                        };
                    }
                    Err(_) => {
                        error!("Error while connecting to configured diglett, trying another...");
                    }
                }
            }
            i += 1;
        }

        panic!("Failed to create Diglett client, no running instances found!");
    }

    pub async fn allocation_state(
        &self,
        address: Ipv4Addr,
    ) -> Result<AllocationState, reqwest::StatusCode> {
        let diglett_client = reqwest::Client::new();

        match diglett_client
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
        let diglett_client = reqwest::Client::new();

        let mut request_url = concat_string!(self.url, address.to_string(), "/rir");
        if top {
            request_url = concat_string!(request_url, "?top=true");
        }

        match diglett_client.get(request_url).send().await {
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
        let diglett_client = reqwest::Client::new();

        match diglett_client
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
        let diglett_client = reqwest::Client::new();

        match diglett_client
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
