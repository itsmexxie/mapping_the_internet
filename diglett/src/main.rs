use mtilib::{
    auth::JWTKeys,
    pokedex::{Pokedex, Url},
};
use providers::Providers;
use settings::Settings;
use std::sync::Arc;
use tokio::{
    fs::File,
    io,
    signal::{self, unix::SignalKind},
    sync::{oneshot, Mutex, RwLock},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};
use uuid::Uuid;

pub mod api;
pub mod providers;
pub mod settings;
pub mod utils;

#[macro_use(concat_string)]
extern crate concat_string;

/*
 * == STATIC ==
 * 1. Sprite
 * 2. Rustls
 * 3. Tracing
 * 4. Settings
 * 5. Load JWT keys
 * == POKEDEX ==
 * 6. Login to Pokedex
 * == TOKIO ==
 * 7. Tokio setup
 * 8. Graceful shutdown task
 * == RUNTIME ==
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

    // Something for WSS
    rustls::crypto::ring::default_provider()
        .install_default()
        .expect("Failed to install rustls crypto provider");

    // Tracing
    tracing_subscriber::fmt::init();

    // Settings
    let (config, settings) = mtilib::settings::deserialize_from_config("config.toml");
    let settings: Arc<Settings> = Arc::new(settings);

    // Load JWT keys if api.auth is set to true
    let mut jwt_keys = None;
    if settings.api.auth {
        jwt_keys = Some(Arc::new(
            JWTKeys::load_public(
                &config
                    .get_string("api.jwt")
                    .unwrap_or(String::from("jwt.key.pub")),
            )
            .await,
        ))
    }

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

    // Gracefule shutdown with cleanup
    let signal_task_tracker = task_tracker.clone();
    let signal_task_token = task_token.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(_) => {
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
                info!("Successfully logged out of Pokedex!");

                // Cancel all tasks
                signal_task_tracker.close();
                signal_task_token.cancel();
            }
        }
    });

    // Load providers
    let providers = Arc::new(RwLock::new(Providers::load(&config).await));

    // Scheduler task
    // let scheduler_providers = providers.clone();
    // let scheduler_token = task_token.clone();
    // task_tracker.spawn(async move {
    //     tokio::select! {
    //         _ = scheduler::run(scheduler_providers) => {
    //             info!("Scheduler task exited on its own!")
    //         }
    //         () = scheduler_token.cancelled() => {
    //             info!("Scheduler task cancelled succesfully!");
    //         }
    //     }
    // });

    // Axum API
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(settings, unit_uuid, jwt_keys, providers) => {
                info!("Axum API task exited on its own!");
            }
            () = task_token.cancelled() => {
                info!("Axum API task cancelled succesfully!");
            }
        }
    });

    task_tracker.wait().await;
}
