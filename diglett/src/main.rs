use std::sync::Arc;

use config::Config;
use mtilib::pokedex::{config::PokedexConfig, Pokedex};
use providers::{arin, iana, thyme};
use serde::Deserialize;
use tokio::{
    signal::{self, unix::SignalKind},
    sync::{Mutex, RwLock},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

pub mod api;
pub mod providers;
pub mod utils;

#[macro_use(concat_string)]
extern crate concat_string;

pub fn get_config_value<T: for<'a> Deserialize<'a>>(config: &Config, id: &str) -> T {
    config
        .get::<T>(id)
        .unwrap_or_else(|_| panic!("{} must be set!", id))
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
    let pokedex = Arc::new(Mutex::new(Pokedex::new(PokedexConfig::from_config(
        &config,
    ))));

    let jwt = match pokedex.lock().await.login().await {
        Ok(token) => {
            info!("Successfully logged into Pokedex!");
            Arc::new(token)
        }
        Err(error) => {
            error!(error);
            panic!()
        }
    };

    // Tokio setup
    let task_tracker = TaskTracker::new();
    let task_token = CancellationToken::new();

    // Gracefule shutdown with cleanup
    let signal_task_tracker = task_tracker.clone();
    let signal_task_token = task_token.clone();
    let signal_task_pokedex = pokedex.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(_) => {
                        signal_task_pokedex.lock().await.logout(&jwt).await;
                        info!("Successfully logged out of Pokedex!");

                        // Cancel all tasks
                        signal_task_tracker.close();
                        signal_task_token.cancel();
                    }
                    Err(err) => {
                        error!("Unable to listen for shutdown signal: {}", err);
                    }
                }
            }
            _ = sigterm.recv() => {
                // Logout of Pokedex
                signal_task_pokedex.lock().await.logout(&jwt).await;
                info!("Successfully logged out of Pokedex!");

                // Cancel all tasks
                signal_task_tracker.close();
                signal_task_token.cancel();
            }
        }
    });

    // Load providers
    let providers = Arc::new(RwLock::new(providers::Providers {
        arin: arin::Providers {
            stats: arin::stats::load(&config).await,
        },
        iana: iana::Providers {
            recovered: iana::recovered::load(&config).await,
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
