use std::sync::Arc;

use api::ApiOptions;
use config::Config;
use mtilib::{
    auth::JWTKeys,
    pokedex::{config::PokedexConfig, Pokedex},
};
use pidgey::Pidgey;
use tokio::{
    signal::{self, unix::SignalKind},
    sync::Mutex,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

#[macro_use(concat_string)]
extern crate concat_string;

pub mod api;
pub mod db;
pub mod models;
pub mod pidgey;
pub mod scanner;
pub mod schema;

/*
 * 1. Tracing
 * 2. Config
 * 3. Load JWT keys
 * 3. Login to Pokedex
 * 4. Tokio setup
 * 5. Graceful shutdown task
 * 6. Scanner task
 * 7. Axum API task
 */
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

    // JWT keys
    let jwt_keys = Arc::new(
        JWTKeys::load_public(
            &config
                .get_string("api.jwt.public")
                .unwrap_or(String::from("jwt.key.pub")),
        )
        .await,
    );

    // Login to Pokedex
    let pokedex = Arc::new(Mutex::new(Pokedex::new(PokedexConfig::from_config(
        &config,
    ))));

    let unit_uuid = match pokedex.lock().await.login().await {
        Ok(res) => {
            info!("Successfully logged into Pokedex!");
            Arc::new(res.uuid)
        }
        Err(error) => {
            error!(error);
            panic!()
        }
    };

    // Tokio setup
    let task_tracker = TaskTracker::new();
    let task_token = CancellationToken::new();

    // Gracefull shutdown with cleanup
    let signal_task_tracker = task_tracker.clone();
    let signal_task_token = task_token.clone();
    let signal_task_pokedex = pokedex.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(_) => {
                        // Logout of Pokedex
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

    // Pidgey handler
    let pidgey = Arc::new(Pidgey::new());

    // Scanner
    let scanner_task_token = task_token.clone();
    let scanner_config = config.clone();
    let scanner_pidgey = pidgey.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = scanner::run(scanner_config, scanner_pidgey) => {
                info!("Scanner task exited on its own!");
            }
            () = scanner_task_token.cancelled() => {
                info!("Scanner task cancelled succesfully!");
            }
        }
    });

    // Axum API
    let axum_task_token = task_token.clone();
    let axum_config = config.clone();
    let axum_jwt_keys = jwt_keys.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(ApiOptions {
                config: axum_config,
                unit_uuid: unit_uuid,
                jwt_keys: axum_jwt_keys,
                pidgey: pidgey
            }) => {
                info!("Axum API task exited on its own!");
            },
            () = axum_task_token.cancelled() => {
                info!("API task cancelled succesfully!");
            }
        }
    });

    task_tracker.wait().await;
}
