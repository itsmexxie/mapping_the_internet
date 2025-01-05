use axum::Router;

use super::AppState;

pub mod auth;
pub mod ws;

pub fn router() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/ws", ws::router())
}
