use axum::Router;

use super::AppState;

pub mod auth;

pub fn router() -> Router<AppState> {
    Router::new().nest("/auth", auth::router())
}
