use std::sync::Arc;

use config::Config;

#[macro_use(concat_string)]
extern crate concat_string;

pub mod api;
pub mod db;
pub mod models;
pub mod schema;

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

    // Axum API
    let api_config = config.clone();
    let api_task = tokio::spawn(async move {
        api::run(api_config).await;
    });

    api_task.await.unwrap();
}
