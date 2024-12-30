use std::sync::Arc;

use mtilib::pokedex::{config::PokedexConfig, Pokedex};
use settings::Settings;
use tokio::{
    fs::File,
    io,
    signal::{self, unix::SignalKind},
    sync::Mutex,
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

pub mod api;
pub mod models;
pub mod schema;
pub mod settings;

#[tokio::main]
async fn main() {
    // Sprite
    let sprite_file = File::open("sprite.txt")
        .await
        .expect("Failed to read sprite file!");
    let mut reader = io::BufReader::new(sprite_file);
    io::copy(&mut reader, &mut io::stdout())
        .await
        .expect("Failed to copy sprite to stdout!");

    // Tracing
    tracing_subscriber::fmt::init();

    // Settings
    let (config, settings) = Settings::load();
    let settings = Arc::new(settings);

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

    // Axum API task
    let axum_task_token = task_token.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(settings, unit_uuid) => {
                info!("Axum API task exited on its own!");
            },
            () = axum_task_token.cancelled() => {
                info!("Axum API task cancelled succesfully!");
            }
        }
    });

    task_tracker.wait().await;
}
