use axum::Router;

use super::AppState;

pub mod auth;
pub mod services;
pub mod ws;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::router())
        .nest("/ws", ws::router())
        .nest("/services", services::router(state))
}
