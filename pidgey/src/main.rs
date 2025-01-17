use std::sync::Arc;

use diglett::Diglett;
use mtilib::{auth::JWTKeys, pokedex::Pokedex, Sprite};
use settings::Settings;
use tokio::{
    signal::{self, unix::SignalKind},
    sync::{oneshot, Mutex, Semaphore},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};
use url::Url;
use uuid::Uuid;

pub mod api;
pub mod diglett;
pub mod gust;
pub mod pidgeotto;
pub mod settings;

pub const MAX_WORKERS: usize = 64;

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
    // Tracing
    tracing_subscriber::fmt::init();

    // Sprite
    match Sprite::load("sprite.txt").await {
        Ok(mut sprite) => {
            if let Err(error) = sprite.print().await {
                error!("Failed to print sprite ({})", error);
            }
        }
        Err(error) => error!("Failed to load sprite ({})", error),
    }

    // Something for WSS
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Config
    let (config, settings) = mtilib::settings::deserialize_from_config("config.toml");
    let config = Arc::new(config);
    let settings: Arc<Settings> = Arc::new(settings);

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
    let pokedex = Arc::new(Mutex::new(
        Pokedex::login(
            &Url::parse(&settings.pokedex.address).expect("Failed to parse Pokedex url"),
            &settings.unit.username,
            &settings.unit.password,
        )
        .await
        .unwrap(),
    ));

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

    let worker_permits = Arc::new(Semaphore::new(settings.max_workers));

    // Graceful shutdown with cleanup
    let signal_task_tracker = task_tracker.clone();
    let signal_task_token = task_token.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(_) => {
                        info!("CTRL+C signal received, shutting down...");

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
                info!("Sigterm signal received, shutting down...");

                // Cancel all tasks
                signal_task_tracker.close();
                signal_task_token.cancel();
            }
        }
    });

    // Diglett setup
    let diglett = Arc::new(Diglett::new(settings.clone(), pokedex.clone()).await);

    // Ping client setup
    let ping_client = Arc::new(surge_ping::Client::new(&surge_ping::Config::new()).unwrap());

    // Axum API task
    let axum_config = config.clone();
    let axum_unit_uuid = unit_uuid.clone();
    let axum_worker_permits = worker_permits.clone();
    let axum_diglett = diglett.clone();
    let axum_ping_client = ping_client.clone();
    let axum_task_token = task_token.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(axum_config, axum_unit_uuid, jwt_keys, axum_worker_permits, axum_diglett, axum_ping_client) => {
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
            () = pidgeotto::run(settings, unit_uuid, worker_permits, pokedex, diglett, ping_client) => {
                info!("Pidgeotto task exited on its own!")
            }
            () = task_token.cancelled() => {
                info!("Pidgeotto task cancelled succesfully!");
            }
        }
    });

    task_tracker.wait().await;
}
