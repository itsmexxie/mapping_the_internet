use std::sync::Arc;

use config::Config;
use tokio_tungstenite::connect_async;

pub async fn run(config: Arc<Config>) {
    let pidgeotto_address = config
        .get_string("pidgeotto.address")
        .expect("pidgeotto.address must be set!");

    let mut pidgeotto_url = String::from(pidgeotto_address);
    if let Ok(pidgeotto_port) = config.get_string("pidgeotto.port") {
        pidgeotto_url = concat_string!(pidgeotto_url, ":", &pidgeotto_port);
    }
    pidgeotto_url = concat_string!("ws://", pidgeotto_url, "/ws");

    let (ws_stream, _) = connect_async(&pidgeotto_url)
        .await
        .expect("Failed to establish a websocket connection to Pidgeotto!");
}
