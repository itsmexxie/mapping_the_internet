use std::sync::Arc;

use config::Config;
use mtilib::{
    auth::JWTKeys,
    pokedex::{PokedexConfig, PokedexUnitConfig},
};
use pidgey::Pidgey;
use tokio::{
    signal::{self, unix::SignalKind},
    sync::RwLock,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

#[macro_use(concat_string)]
extern crate concat_string;

pub mod api;
pub mod pidgey;

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

    // Tokio setup
    let task_tracker = TaskTracker::new();
    let task_token = CancellationToken::new();

    // Gracefull shutdown with cleanup
    let signal_task_tracker = task_tracker.clone();
    let signal_task_token = task_token.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            result = signal::ctrl_c() => {
                match result {
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
            }
            _ = sigterm.recv() => {
                // Logout of Pokedex
                mtilib::pokedex::logout(&pokedex_config, &jwt).await;
                info!("Successfully logged out of Pokedex!");

                // Cancel all tasks
                signal_task_tracker.close();
                signal_task_token.cancel();
            }
        }
    });

    // Pidgey handler
    let pidgey = Arc::new(RwLock::new(Pidgey::new()));

    // Axum API
    let axum_task_token = task_token.clone();
    let axum_config = config.clone();
    let axum_jwt_keys = jwt_keys.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(axum_config, axum_jwt_keys, pidgey) => {
                info!("Axum API task exited on its own!");
            },
            () = axum_task_token.cancelled() => {
                info!("API task cancelled succesfully!");
            }
        }
    });

    task_tracker.wait().await;
}
