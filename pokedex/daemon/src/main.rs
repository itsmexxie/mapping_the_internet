use config::Config;
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use mtilib::auth::JWTKeys;
use std::sync::Arc;
use tokio::signal;
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};

#[macro_use(concat_string)]
extern crate concat_string;

pub mod api;
pub mod db;
pub mod models;
pub mod schema;

use models::{NewServiceUnit, Service, ServiceUnit};
use schema::{ServiceUnits, Services};

#[tokio::main]
async fn main() {
    // Logging
    tracing_subscriber::fmt::init();

    // Config
    let config = Arc::new(
        Config::builder()
            .add_source(config::File::with_name("daemon.config.toml"))
            .build()
            .unwrap(),
    );

    // JWT keys
    let jwt_keys = Arc::new(
        JWTKeys::load(
            &config
                .get_string("api.jwt.private")
                .unwrap_or(String::from("jwt.key")),
            &config
                .get_string("api.jwt.public")
                .unwrap_or(String::from("jwt.key.pub")),
        )
        .await,
    );

    // Register service with database
    let mut pg_conn = db::create_conn(&config);
    let service_query = Services::table
        .select(Service::as_select())
        .filter(Services::name.eq("pokedex"))
        .first(&mut pg_conn)
        .unwrap();

    let pokedex_unit_port = match config.get_string("unit.announce_port") {
        Ok(value) => match value.as_str() {
            "true" | "t" | "1" => {
                Some(config.get_int("api.port").expect("api.port must be set!") as i32)
            }
            "false" | "f" | "0" | _ => None,
        },
        Err(_) => None,
    };

    let pokedex_unit = diesel::insert_into(ServiceUnits::table)
        .values(&NewServiceUnit {
            id: &uuid::Uuid::new_v4().to_string(),
            service_id: service_query.id,
            address: Some(
                config
                    .get_string("unit.address")
                    .expect("Pokedex must have a unit address set!"),
            ),
            port: pokedex_unit_port,
        })
        .returning(ServiceUnit::as_returning())
        .get_result(&mut pg_conn)
        .expect("Failed to register unit with database!");
    info!("Successfully registered unit with database!");

    // Tokio setup
    let task_tracker = TaskTracker::new();
    let task_token = CancellationToken::new();

    // Axum API
    let axum_token = task_token.clone();
    let axum_config = config.clone();
    let axum_jwt_keys = jwt_keys.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(axum_config, axum_jwt_keys) => {
                info!("Axum API task exited on its own!");
            }
            () = axum_token.cancelled() => {
                info!("Axum API task cancelled successfully!");
            }
        }
    });

    let signal_task_tracker = task_tracker.clone();
    tokio::spawn(async move {
        match signal::ctrl_c().await {
            Ok(_) => {
                // Deregister pokedex with database
                diesel::delete(ServiceUnits::table.filter(ServiceUnits::id.eq(pokedex_unit.id)))
                    .execute(&mut pg_conn)
                    .unwrap();
                info!("Successfully deregistered unit with database!");

                // Cancel all tasks
                signal_task_tracker.close();
                task_token.cancel();
            }
            Err(err) => {
                error!("Unable to listen for shutdown signal: {}", err);
            }
        }
    });

    task_tracker.wait().await;
}
