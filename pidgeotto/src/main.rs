use std::sync::Arc;

use config::Config;
use mtilib::pokedex::{PokedexConfig, PokedexUnitConfig};
use tokio::signal;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

#[macro_use(concat_string)]
extern crate concat_string;

pub mod api;
pub mod jobs;

#[tokio::main]
async fn main() {
    // Tracing
    tracing_subscriber::fmt::init();

    // Config
    let config = Arc::new(
        Config::builder()
            .add_source(config::File::with_name("config.toml"))
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
    let pokedex_address = config.get_string("pokedex.address").unwrap();
    let pokedex_unit_config = PokedexUnitConfig::new(&unit_username, &unit_password);
    let pokedex_config = PokedexConfig::new(pokedex_unit_config, &pokedex_address);

    let jwt = match mtilib::pokedex::login(&pokedex_config).await {
        Ok(token) => {
            info!("Successfully logged into Pokedex!");
            token
        }
        Err(error) => return error!(error),
    };

    // Tokio
    let task_tracker = TaskTracker::new();
    let task_token = CancellationToken::new();

    // Axum API
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

    let signal_task_tracker = task_tracker.clone();
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(_) => {
                // Logout of Pokedex
                mtilib::pokedex::logout(&pokedex_config, &jwt).await;

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
