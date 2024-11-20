use config::Config;

pub mod api;

#[tokio::main]
async fn main() {
    // Logging
    tracing_subscriber::fmt::init();

    // Config
    let config = Config::builder()
        .add_source(config::File::with_name("daemon.config.toml"))
        .build()
        .unwrap();

    // Axum API
    let api_task = tokio::spawn(async move {
        api::run(config).await;
    });

    api_task.await.unwrap();
}
