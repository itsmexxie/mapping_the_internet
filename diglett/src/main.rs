use api::ApiOptions;
use config::Config;
use mtilib::{
    auth::JWTKeys,
    pokedex::{config::PokedexConfig, Pokedex},
};
use providers::Providers;
use std::sync::Arc;
use tokio::{
    signal::{self, unix::SignalKind},
    sync::{Mutex, RwLock},
};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

pub mod api;
pub mod providers;
// pub mod scheduler;
pub mod utils;

#[macro_use(concat_string)]
extern crate concat_string;

/*
 * 1. Logging
 * 2. Config
 * 3. Load JWT keys
 * 3. Login to Pokedex
 * 4. Tokio setup
 * 5. Graceful shutdown task
 * 6. Load providers
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

	// Load JWT keys if api.auth is set to true
	let mut jwt_keys = None;
	if config.get_bool("api.auth").unwrap_or(true) {
		jwt_keys = Some(Arc::new(
			JWTKeys::load_public(
				&config
					.get_string("api.jwt")
					.unwrap_or(String::from("jwt.key.pub")),
			)
			.await,
		))
	}
]

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

    // Gracefule shutdown with cleanup
    let signal_task_tracker = task_tracker.clone();
    let signal_task_token = task_token.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(_) => {
                        pokedex.lock().await.logout().await;
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
                pokedex.lock().await.logout().await;
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
            () = api::run(ApiOptions {
                config,
                jwt,
                jwt_keys,
                providers,
            }) => {
                info!("Axum API task exited on its own!");
            }
            () = task_token.cancelled() => {
                info!("Axum API task cancelled succesfully!");
            }
        }
    });

    task_tracker.wait().await;
}
