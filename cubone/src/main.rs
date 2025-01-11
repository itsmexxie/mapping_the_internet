use std::sync::Arc;

use mtilib::pokedex::{Pokedex, Url};
use settings::Settings;
use sqlx::PgPool;
use tokio::{
    fs::File,
    io,
    signal::{self, unix::SignalKind},
    sync::{oneshot, Mutex},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};
use uuid::Uuid;

pub mod api;
pub mod models;
pub mod settings;

/*
 * == STATIC ==
 * 1. Sprite
 * 2. Tracing
 * 3. Settings
 * 4. Load JWT keys
 * == POKEDEX ==
 * 5. Login to Pokedex
 * == TOKIO ==
 * 6. Tokio setup
 * 7. Graceful shutdown task
 * == RUNTIME ==
 * 8. Create database connection pool
 * 9. Load providers
 * 10. Axum API task
 */
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
    let (_, settings) = mtilib::settings::deserialize_from_config("config.toml");
    let settings: Arc<Settings> = Arc::new(settings);

    // Login to Pokedex
    let pokedex = Arc::new(Mutex::new(
        Pokedex::login(
            &Url::parse(&settings.pokedex.address).expect("Failed to parse Pokedex url"),
            &settings.unit.username,
            &settings.unit.password,
        )
        .await
        .unwrap(),
    ));
    info!("Successfully logged into Pokedex!");

    let unit_uuid = Arc::new(match settings.unit.address.as_ref() {
        Some(unit_address) => {
            let (register_tx, register_rx) = oneshot::channel::<Uuid>();

            let unit_port = match settings.unit.announce_port {
                true => Some(settings.api.port),
                false => None,
            };

            pokedex
                .lock()
                .await
                .register(unit_address, unit_port, register_tx)
                .await;

            info!("Successfully registered the unit to Pokedex!");
            Some(register_rx.await.unwrap())
        }
        None => None,
    });

    // Tokio setup
    let task_tracker = TaskTracker::new();
    let task_token = CancellationToken::new();

    // Graceful shutdown with cleanup
    let signal_task_tracker = task_tracker.clone();
    let signal_task_token = task_token.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(_) => {
                        // Cancel all tasks
                        signal_task_tracker.close();
                        signal_task_token.cancel();
                        println!("shutting down");
                    }
                    Err(err) => {
                        error!("Unable to listen for shutdown signal: {}", err);
                    }
                }
            }
            _ = sigterm.recv() => {
                // Cancel all tasks
                signal_task_tracker.close();
                signal_task_token.cancel();
            }
        }
    });

    // Create database connection pool
    let db_pool = Arc::new(
        PgPool::connect(&mtilib::db::url(
            &*settings.database.hostname,
            &*settings.database.username,
            &*settings.database.password,
            &*settings.database.database,
        ))
        .await
        .unwrap(),
    );

    // Axum API task
    let axum_task_token = task_token.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(settings, unit_uuid, db_pool) => {
                info!("Axum API task exited on its own!");
            },
            () = axum_task_token.cancelled() => {
                info!("Axum API task cancelled succesfully!");
            }
        }
    });

    task_tracker.wait().await;
}
