use core::panic;
use std::{net::Ipv4Addr, sync::Arc};

use config::Config;
use mtilib::pokedex::PokedexConfig;
use rand::seq::SliceRandom;
use reqwest::StatusCode;
use tracing::{error, info};

pub struct Diglett {
    url: String,
}

impl Diglett {
    pub async fn new(config: &Arc<Config>, pokedex_config: &PokedexConfig) -> Self {
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
                        url: diglett_address,
                    };
                }
                Err(_) => {
                    error!("Error while connecting to configured diglett, trying to find one from Pokedex...");
                }
            }
        }

        let services = mtilib::pokedex::get_services(&pokedex_config)
            .await
            .expect("Failed to retrieve services from Pokedex!");

        let diglett_service = services
            .iter()
            .find(|s| s.name == "diglett")
            .expect("Failed to find the diglett service!");

        let mut units = mtilib::pokedex::get_service_units(&pokedex_config, diglett_service.id)
            .await
            .expect("Failed to retrieve available diglett units!");

        units.shuffle(&mut rand::thread_rng());
        let mut i = 0;
        while i < units.len() {
            if let Some(pidgeotto_address) = &units[i].address {
                match diglett_client.get(pidgeotto_address).send().await {
                    Ok(_) => {
                        info!("Successfully connected to a diglett instance!");
                        return Diglett {
                            url: pidgeotto_address.to_owned(),
                        };
                    }
                    Err(_) => {
                        error!("Error while connecting to configured diglett, trying another...");
                    }
                }
            }
            i += 1;
        }

        panic!("Failed to create Diglett, no running instances found!");
    }

    pub async fn rir(&self, address: Ipv4Addr) -> Option<String> {
        let diglett_client = reqwest::Client::new();

        match diglett_client
            .get(concat_string!(
                self.url,
                "/rir?address=",
                address.to_string()
            ))
            .send()
            .await
        {
            Ok(res) => match res.status() {
                StatusCode::OK => Some(res.text().await.unwrap()),
                StatusCode::NOT_FOUND => None,
                _ => None,
            },
            Err(_) => None,
        }
    }

    pub async fn asn(&self, address: Ipv4Addr) -> Option<u32> {
        let diglett_client = reqwest::Client::new();

        match diglett_client
            .get(concat_string!(
                self.url,
                "/asn?address=",
                address.to_string()
            ))
            .send()
            .await
        {
            Ok(res) => match res.status() {
                StatusCode::OK => Some(res.text().await.unwrap().parse::<u32>().unwrap()),
                StatusCode::NOT_FOUND => None,
                _ => None,
            },
            Err(_) => None,
        }
    }
}
