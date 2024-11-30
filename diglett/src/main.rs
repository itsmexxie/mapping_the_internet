use std::sync::Arc;

use config::Config;
use mtilib::pokedex::{PokedexConfig, PokedexUnitConfig};
use providers::{arin, iana, thyme};
use serde::Deserialize;
use tokio::{signal, sync::RwLock};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

pub mod api;
pub mod providers;
pub mod utils;

#[macro_use(concat_string)]
extern crate concat_string;

pub fn get_config_value<T: for<'a> Deserialize<'a>>(config: &Config, id: &str) -> T {
    config.get::<T>(id).expect(&format!("{} must be set!", id))
}

#[tokio::main]
async fn main() {
    // Logging
    tracing_subscriber::fmt::init();

    // Config
    let config = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap();

    // Login to Pokedex
    let unit_username = config
        .get_string("unit.username")
        .expect("unit.username must be set!");
    let unit_password = config
        .get_string("unit.password")
        .expect("unit.password must be set!");
    let pokedex_address = config
        .get_string("pokedex.address")
        .expect("pokedex.address must be set!");
    let pokedex_unit_config = match config.get_string("unit.address") {
        Ok(unit_address) => match config.get_bool("unit.announce_port") {
            Ok(announce_port) => match announce_port {
                true => PokedexUnitConfig::with_address_and_port(
                    &unit_username,
                    &unit_password,
                    &unit_address,
                    config
                        .get_int("api.port")
                        .expect("api.port must be set!")
                        .try_into()
                        .unwrap(),
                ),
                false => {
                    PokedexUnitConfig::with_address(&unit_username, &unit_password, &unit_address)
                }
            },
            Err(_) => {
                PokedexUnitConfig::with_address(&unit_username, &unit_password, &unit_address)
            }
        },
        Err(_) => PokedexUnitConfig::new(&unit_username, &unit_password),
    };
    let pokedex_config = PokedexConfig::new(pokedex_unit_config, &pokedex_address);

    let jwt = match mtilib::pokedex::login(&pokedex_config).await {
        Ok(token) => {
            info!("Successfully logged into Pokedex!");
            token
        }
        Err(error) => return error!(error),
    };

    // Tokio setup
    let task_tracker = TaskTracker::new();
    let task_token = CancellationToken::new();

    // Gracefule shutdown with cleanup
    let signal_task_tracker = task_tracker.clone();
    let signal_task_token = task_token.clone();
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(_) => {
                // Logout of Pokedex
                mtilib::pokedex::logout(&pokedex_config, &jwt).await;
                info!("Successfully logged out of Pokedex!");

                // Cancel all tasks
                signal_task_tracker.close();
                signal_task_token.cancel();
            }
            Err(err) => {
                error!("Unable to listen for shutdown signal: {}", err);
            }
        }
    });

    // Load providers
    let providers = Arc::new(RwLock::new(providers::Providers {
        arin: arin::Providers {
            stats: arin::stats::load(&config).await,
        },
        iana: iana::Providers {
            reserved: iana::reserved::load(&config).await,
        },
        thyme: thyme::Providers {
            asn: thyme::asn_prefixes::load(&config).await,
            rir: thyme::rir_allocations::load(&config).await,
        },
    }));

    // Axum API
    let axum_token = task_token.clone();
    let axum_providers = providers.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(config, axum_providers) => {
                info!("Axum API task exited on its own!");
            }
            () = axum_token.cancelled() => {
                info!("Axum API task cancelled succesfully!");
            }
        }
    });

    task_tracker.wait().await;
}
