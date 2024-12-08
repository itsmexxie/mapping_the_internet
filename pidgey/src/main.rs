use std::sync::Arc;

use config::Config;
use diglett::Diglett;
use mtilib::pokedex::{config::PokedexConfig, Pokedex};
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

pub const MAX_WORKERS: usize = 16;

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
    let signal_task_jwt = jwt.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(_) => {
                        signal_task_pokedex.lock().await.logout(&signal_task_jwt).await;
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
                signal_task_pokedex.lock().await.logout(&signal_task_jwt).await;
                info!("Successfully logged out of Pokedex!");

                // Cancel all tasks
                signal_task_tracker.close();
                signal_task_token.cancel();
            }
        }
    });

    // Diglett setup
    let diglett = Arc::new(Diglett::new(&config, pokedex).await);

    // Create ping client
    let ping_client = Arc::new(surge_ping::Client::new(&surge_ping::Config::new()).unwrap());
    // This method only creates the number of sockets equal to the number of maximum workers defined in config
    // and doesn't waste system resources
    // let ping_pool = Arc::new(Pool::new(max_workers));
    // for _ in 0..max_workers {
    //     ping_pool
    //         .add(
    //             surge_ping::Client::new(&surge_ping::Config::default())
    //                 .expect("Failed to create surge ping client!"),
    //         )
    //         .await
    //         .unwrap();
    // }

    // Axum API task
    let axum_task_token = task_token.clone();
    let axum_config = config.clone();
    let axum_worker_permits = worker_permits.clone();
    let axum_diglett = diglett.clone();
    let axum_ping_client = ping_client.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(axum_config, axum_worker_permits, axum_diglett, axum_ping_client) => {
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

// #[cfg(test)]
// mod tests {
//     use std::{
//         net::{IpAddr, Ipv4Addr},
//         str::FromStr,
//         sync::Arc,
//         time::{SystemTime, UNIX_EPOCH},
//     };

//     use rand::random;
//     use surge_ping::{PingIdentifier, PingSequence};

//     #[tokio::test]
//     async fn test_parallel_ping() {
//         let client = Arc::new(surge_ping::Client::new(&surge_ping::Config::default()).unwrap());

//         let addresses = vec![
//             "1.1.1.2", "1.1.1.3", "1.1.1.4", "1.1.1.5", "1.1.1.6", "1.1.1.7", "1.1.1.8", "0.0.0.0",
//         ];
//         let mut tasks = Vec::new();

//         let start = SystemTime::now();
//         let since_the_epoch = start
//             .duration_since(UNIX_EPOCH)
//             .expect("Time went backwards");
//         println!("{}", since_the_epoch.as_millis());

//         for address in addresses {
//             let cloned_client = client.clone();
//             tasks.push(tokio::spawn(async move {
//                 let mut pinger = cloned_client
//                     .pinger(
//                         IpAddr::V4(Ipv4Addr::from_str(address).unwrap()),
//                         PingIdentifier(random()),
//                     )
//                     .await;

//                 match pinger
//                     .ping(PingSequence(0), &[0, 0, 0, 0, 0, 0, 0, 0])
//                     .await
//                 {
//                     Ok(_) => {
//                         let start = SystemTime::now();
//                         let since_the_epoch = start
//                             .duration_since(UNIX_EPOCH)
//                             .expect("Time went backwards");
//                         println!("{} {} {}", since_the_epoch.as_millis(), address, true)
//                     }
//                     Err(_) => println!("{} {}", address, false),
//                 }
//             }));
//         }

//         for task in tasks {
//             task.await.unwrap();
//         }

//         assert_eq!(false, true);
//     }
// }
