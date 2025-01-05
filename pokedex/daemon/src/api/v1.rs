use axum::Router;

use super::AppState;

pub mod auth;
pub mod services;

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/services", services::router(state.clone()))
        .nest("/auth", auth::router(state))
}
