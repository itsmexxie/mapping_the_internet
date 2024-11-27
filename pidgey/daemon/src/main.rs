use std::sync::Arc;

use config::Config;
use mtilib::pokedex::{PokedexConfig, PokedexUnitConfig};
use serde::Serialize;
use tokio::signal;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

#[macro_use(concat_string)]
extern crate concat_string;

pub mod api;
pub mod pidgeotto;

#[derive(Serialize)]
struct ApiServiceUnit {
    id: String,
    service_id: i32,
    address: Option<String>,
    port: Option<i32>,
}

#[tokio::main]
async fn main() {
    // Tracing
    tracing_subscriber::fmt::init();

    // Config
    let config = Arc::new(
        Config::builder()
            .add_source(config::File::with_name("daemon.config.toml"))
            .build()
            .unwrap(),
    );

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
    let pokedex_unit_config = PokedexUnitConfig::new(&unit_username, &unit_password);
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

    // Axum API task
    let axum_task_token = task_token.clone();
    let axum_config = config.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(axum_config) => {
                info!("Axum API task exited on its own!");
            },
            () = axum_task_token.cancelled() => {
                info!("API task cancelled succesfully!");
            }
        }
    });

    // Pidgeotto connection task
    let pidgeotto_task_token = task_token.clone();
    let pidgeotto_config = config.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = pidgeotto::run(pidgeotto_config) => {
                info!("Pidgeotto task exited on its own!")
            }
            () = pidgeotto_task_token.cancelled() => {
                info!("Pidgeotto task cancelled succesfully!");
            }
        }
    });

    // Graceful shutdown with cleanup
    let signal_task_tracker = task_tracker.clone();
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(_) => {
                // Logout of Pokedex
                mtilib::pokedex::logout(&pokedex_config, &jwt).await;
                info!("Successfully logged out of Pokedex!");

                // Cancel all tasks
                signal_task_tracker.close();
                task_token.cancel();
            }
            Err(err) => {
                error!("Unable to listen for shutdown signal: {}", err);
            }
        }
    });

    task_tracker.wait().await;
}
