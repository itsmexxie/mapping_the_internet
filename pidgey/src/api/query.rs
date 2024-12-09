use super::AppState;
use axum::{
    extract::{Request, State},
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::IntoResponse,
    Router,
};
use jsonwebtoken::{Algorithm, DecodingKey, Validation};
use mtilib::auth::JWTClaims;

pub mod address;

// All queries should be behind an auth check
pub async fn query_auth(
    headers: HeaderMap,
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<impl IntoResponse, StatusCode> {
    if !headers.contains_key("authorization") {
        return Err(StatusCode::UNAUTHORIZED);
    }

    let header_token = headers
        .get("authorization")
        .unwrap()
        .to_str()
        .unwrap()
        .split(" ")
        .collect::<Vec<&str>>();
    if header_token.len() < 2 {
        return Err(StatusCode::UNAUTHORIZED);
    }

    match jsonwebtoken::decode::<JWTClaims>(
        header_token[1],
        &DecodingKey::from_rsa_pem(&state.jwt_keys.public).unwrap(),
        &Validation::new(Algorithm::RS512),
    ) {
        Ok(token) => {
            request.extensions_mut().insert(token);
            Ok(next.run(request).await)
        }
        Err(_) => Err(StatusCode::UNAUTHORIZED),
    }
}

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
        .layer(middleware::from_fn_with_state(state.clone(), query_auth))
}
