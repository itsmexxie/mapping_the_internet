use mtilib::auth::JWTKeys;
use settings::Settings;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::signal::{self, unix::SignalKind};
use tokio_util::{sync::CancellationToken, task::TaskTracker};
use tracing::{error, info};
use uuid::Uuid;

pub mod api;
pub mod settings;

async fn shutdown(
    db_pool: mtilib::db::DbPool,
    task_tracker: TaskTracker,
    task_token: CancellationToken,
) {
    // Deregister all units from database
    sqlx::query(
        r#"
            DELETE FROM "ServiceUnits"
            "#,
    )
    .execute(&mut *db_pool.acquire().await.unwrap())
    .await
    .unwrap();
    info!("Successfully deregistered all units from the database!");

    // Cancel all tasks
    task_tracker.close();
    task_token.cancel();
}

/*
 * == STATIC ==
 * 1. Sprite
 * 2. Tracing
 * 3. Settings
 * 4. Load JWT keys
 * == POKEDEX ==
 * 5. Register unit with database
 * == TOKIO ==
 * 6. Tokio setup
 * 7. Graceful shutdown task
 * == RUNTIME ==
 * 8. Create database connection pool
 * 9. Axum API task
 */
#[tokio::main]
async fn main() {
    // Tracing
    tracing_subscriber::fmt::init();

    // Sprite
    // match Sprite::load("sprite.txt").await {
    //     Ok(mut sprite) => {
    //         if let Err(error) = sprite.print().await {
    //             error!("Failed to print sprite ({})", error);
    //         }
    //     }
    //     Err(error) => error!("Failed to load sprite ({})", error),
    // }

    // Settings
    let (config, settings) = mtilib::settings::deserialize_from_config("daemon.config.toml");
    let settings: Arc<Settings> = Arc::new(settings);

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

    // Register service with database
    let unit_uuid = Arc::new(Uuid::new_v4());
    let pokedex_unit_port = if settings.unit.announce_port {
        Some(settings.api.port as i32)
    } else {
        None
    };

    if let Err(error) = sqlx::query(
        r#"
		INSERT INTO "ServiceUnits"
		VALUES ($1, $2, $3, $4)
		"#,
    )
    .bind(*unit_uuid)
    .bind(&*settings.unit.username)
    .bind(&*settings.unit.address)
    .bind(pokedex_unit_port)
    .execute(&mut *db_pool.acquire().await.unwrap())
    .await
    {
        panic!("Failed to register the unit with the database! ({})", error);
    }

    info!("Successfully registered unit with database!");

    // Tokio setup
    let task_tracker = TaskTracker::new();
    let task_token = CancellationToken::new();

    // Gracefule shutdown with cleanup
    let signal_task_db_pool = db_pool.clone();
    let signal_task_tracker = task_tracker.clone();
    let signal_task_token = task_token.clone();
    tokio::spawn(async move {
        let mut sigterm = signal::unix::signal(SignalKind::terminate()).unwrap();
        tokio::select! {
            result = signal::ctrl_c() => {
                match result {
                    Ok(_) => {
                        info!("CTRL+C signal received, shutting down...");
                        shutdown(signal_task_db_pool, signal_task_tracker, signal_task_token).await;
                    }
                    Err(err) => {
                        error!("Unable to listen for shutdown signal: {}", err);
                    }
                }
            }
            _ = sigterm.recv() => {
                info!("Sigterm signal received, shutting down...");
                shutdown(signal_task_db_pool, signal_task_tracker, signal_task_token).await;
            }
        }
    });

    // Axum API
    let axum_db_pool = db_pool.clone();
    task_tracker.spawn(async move {
        tokio::select! {
            () = api::run(Arc::new(config), jwt_keys, unit_uuid, axum_db_pool) => {
                info!("Axum API task exited on its own!");
            }
            () = task_token.cancelled() => {
                info!("Axum API task cancelled successfully!");
            }
        }
    });

    task_tracker.wait().await;
    db_pool.close().await;
}
