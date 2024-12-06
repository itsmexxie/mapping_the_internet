use std::sync::Arc;

use config::Config;
use diglett::Diglett;
use mtilib::pokedex::{PokedexConfig, PokedexUnitConfig};
use tokio::{
    signal::{self, unix::SignalKind},
    sync::Semaphore,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

#[macro_use(concat_string)]
extern crate concat_string;

pub mod api;
pub mod diglett;
pub mod gust;
pub mod pidgeotto;
pub mod utils;

pub const MAX_WORKERS: usize = 16;

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
    let pokedex_address = config
        .get_string("pokedex.address")
        .expect("pokedex.address must be set!");
    let pokedex_unit_config = PokedexUnitConfig::new(&unit_username, &unit_password);
    let pokedex_config = Arc::new(PokedexConfig::new(pokedex_unit_config, &pokedex_address));

    let jwt = match mtilib::pokedex::login(&pokedex_config).await {
        Ok(token) => {
            info!("Successfully logged into Pokedex!");
            Arc::new(token)
        }
        Err(error) => return error!(error),
    };

    // Tokio setup
    let task_tracker = TaskTracker::new();
    let task_token = CancellationToken::new();

    let max_workers = match config.get_int("settings.max_workers") {
        Ok(max) => max as usize,
        Err(_) => MAX_WORKERS,
    };
    let worker_permits = Arc::new(Semaphore::new(max_workers));

    // Graceful shutdown with cleanup
    let signal_task_tracker = task_tracker.clone();
    let signal_task_token = task_token.clone();
    let signal_pokedex_config = pokedex_config.clone();
    let signal_task_jwt = jwt.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(_) => {
                        // Logout of Pokedex
                        mtilib::pokedex::logout(&signal_pokedex_config, &signal_task_jwt).await;
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
                mtilib::pokedex::logout(&signal_pokedex_config, &signal_task_jwt).await;
                info!("Successfully logged out of Pokedex!");

                // Cancel all tasks
                signal_task_tracker.close();
                signal_task_token.cancel();
            }
        }
    });

    // Services setup
    let diglett = Arc::new(Diglett::new(&config, &pokedex_config).await);

    // Axum API task
    let axum_task_token = task_token.clone();
    let axum_config = config.clone();
    let axum_worker_permits = worker_permits.clone();
    let axum_diglett = diglett.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(axum_config, axum_worker_permits, axum_diglett) => {
                info!("Axum API task exited on its own!");
            },
            () = axum_task_token.cancelled() => {
                info!("Axum API task cancelled succesfully!");
            }
        }
    });

    // Pidgeotto connection task
    let pidgeotto_task_token = task_token.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = pidgeotto::run(config, worker_permits, jwt, diglett) => {
                info!("Pidgeotto task exited on its own!")
            }
            () = pidgeotto_task_token.cancelled() => {
                info!("Pidgeotto task cancelled succesfully!");
            }
        }
    });

    task_tracker.wait().await;
}
