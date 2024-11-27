use std::sync::Arc;

use config::Config;
use serde::Deserialize;

pub mod api;
pub mod thyme;
pub mod utils;

#[macro_use(concat_string)]
extern crate concat_string;

pub fn get_config_value<T: for<'a> Deserialize<'a>>(config: &Config, id: &str) -> T {
    config.get::<T>(id).expect(&format!("{} must be set!", id))
}

#[tokio::main]
async fn main() {
    // Logging
    tracing_subscriber::fmt::init();

    // Config
    let config = Config::builder()
        .add_source(config::File::with_name("config.toml"))
        .build()
        .unwrap();

    if !thyme::check_file(&config, "thyme.asn.prefixes").await {
        thyme::download_file(&config, "thyme.asn.prefixes").await;
    }
    let asn_prefixes = Arc::new(thyme::asn_prefixes::load(&config).await);

    if !thyme::check_file(&config, "thyme.rir.allocations").await {
        thyme::download_file(&config, "thyme.rir.allocations").await;
    }
    let rir_allocations = Arc::new(thyme::rir_allocations::load(&config).await);

    let rir_allocations_api = rir_allocations.clone();
    let asn_prefixes_api = asn_prefixes.clone();
    let api_task = tokio::spawn(async move {
        api::run(config, rir_allocations_api, asn_prefixes_api).await;
    });

    api_task.await.unwrap();
}
