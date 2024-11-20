use axum::{extract::Query, http::StatusCode, routing::get, Json, Router};
use config::Config;
use serde::{de::value, Deserialize, Serialize};
use std::{
    net::{IpAddr, Ipv4Addr, SocketAddr},
    str::FromStr,
};
use surge_ping::SurgeError;
use tower_http::trace::TraceLayer;
use tracing::{debug, info};

#[derive(Deserialize)]
struct QueryQuery {
    address: String,
}

#[derive(Serialize)]
struct QueryResponse {
    online: bool,
}

async fn query(query_query: Query<QueryQuery>) -> Result<Json<QueryResponse>, StatusCode> {
    debug!("querying address {}", query_query.address);

    Ok(Json(QueryResponse { online: false }))
}

#[derive(Deserialize)]
struct PingQuery {
    address: String,
}

#[derive(Serialize)]
struct PingResponse {
    online: bool,
    reason: Option<String>,
}

async fn ping(ping_query: Query<PingQuery>) -> Result<Json<PingResponse>, StatusCode> {
    debug!("pinging address {}", ping_query.address);

    let ping_address = Ipv4Addr::from_str(&ping_query.address);

    match ping_address {
        Ok(ping_address) => {
            let payload = [0; 8];
            match surge_ping::ping(IpAddr::V4(ping_address), &payload).await {
                Ok(_) => Ok(Json(PingResponse {
                    online: true,
                    reason: None,
                })),
                Err(ping_error) => match ping_error {
                    SurgeError::Timeout { seq: _ } => Ok(Json(PingResponse {
                        online: false,
                        reason: Some(String::from("timeout")),
                    })),
                    _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
                },
            }
        }
        Err(_) => Err(StatusCode::BAD_REQUEST),
    }
}

pub async fn run(config: Config) {
    let app = Router::new()
        .route("/ping", get(ping))
        .layer(TraceLayer::new_for_http());
    let app_port = config.get("api.port").unwrap();

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        app_port,
    ))
    .await
    .unwrap();

    info!("Listening on port {}", app_port);
    axum::serve(listener, app).await.unwrap();
}
