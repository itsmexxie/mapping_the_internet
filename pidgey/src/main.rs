use std::sync::Arc;

use config::Config;
use diglett::Diglett;
use mtilib::{
    auth::JWTKeys,
    pokedex::{config::PokedexConfig, Pokedex},
};
use tokio::{
    signal::{self, unix::SignalKind},
    sync::{Mutex, Semaphore},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

#[macro_use(concat_string)]
extern crate concat_string;

pub mod api;
pub mod diglett;
pub mod gust;
pub mod pidgeotto;

pub const MAX_WORKERS: usize = 64;

#[tokio::main]
async fn main() {
    // Tracing
    tracing_subscriber::fmt::init();

    // Something for WSS
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Config
    let config = Arc::new(
        Config::builder()
            .add_source(config::File::with_name("config.toml"))
            .build()
            .unwrap(),
    );

    let max_workers = match config.get_int("settings.max_workers") {
        Ok(max) => max as usize,
        Err(_) => MAX_WORKERS,
    };

    // JWT keys
    let jwt_keys = Arc::new(
        JWTKeys::load_public(
            &config
                .get_string("api.jwt")
                .unwrap_or(String::from("jwt.key.pub")),
        )
        .await,
    );

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

    let worker_permits = Arc::new(Semaphore::new(max_workers));

    // Graceful shutdown with cleanup
    let signal_task_tracker = task_tracker.clone();
    let signal_task_token = task_token.clone();
    let signal_task_pokedex = pokedex.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(_) => {
                        signal_task_pokedex.lock().await.logout().await;
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
                signal_task_pokedex.lock().await.logout().await;
                info!("Successfully logged out of Pokedex!");

                // Cancel all tasks
                signal_task_tracker.close();
                signal_task_token.cancel();
            }
        }
    });

    // Diglett setup
    let diglett = Arc::new(Diglett::new(&config, pokedex).await);

    // Ping client setup
    let ping_client = Arc::new(surge_ping::Client::new(&surge_ping::Config::new()).unwrap());

    // Axum API task
    let axum_task_token = task_token.clone();
    let axum_config = config.clone();
    let axum_worker_permits = worker_permits.clone();
    let axum_diglett = diglett.clone();
    let axum_ping_client = ping_client.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(axum_config, jwt_keys, axum_worker_permits, axum_diglett, axum_ping_client) => {
                info!("Axum API task exited on its own!");
            },
            () = axum_task_token.cancelled() => {
                info!("Axum API task cancelled succesfully!");
            }
        }
    });

    // Pidgeotto connection task
    task_tracker.spawn(async move {
        tokio::select! {
            () = pidgeotto::run(config, worker_permits, jwt, diglett, ping_client) => {
                info!("Pidgeotto task exited on its own!")
            }
            () = task_token.cancelled() => {
                info!("Pidgeotto task cancelled succesfully!");
            }
        }
    });

    task_tracker.wait().await;
}
