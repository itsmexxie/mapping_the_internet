use super::AppState;
use axum::{
    extract::{Request, State},
    middleware::{self, Next},
    response::IntoResponse,
    Router,
};

pub mod address;

// Queries should be limited to the configured number of max "workers" (defaults to 16 if left unconfigured).
// Clients shouldn't be disconnected when no permits are available but they should wait until a permit becomes
// available and the requests completes
pub async fn query_limiter(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> impl IntoResponse {
    let _permit = state.worker_permits.acquire().await.unwrap();

    next.run(request).await
}

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .nest("/address", address::router())
        .layer(middleware::from_fn_with_state(state.clone(), query_limiter))
        // All queries should be behind an auth check
        .layer(middleware::from_fn_with_state(
            state.clone(),
            mtilib::auth::axum_middleware::<AppState>,
        ))
}
