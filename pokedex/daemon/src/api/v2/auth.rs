use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::IntoResponse,
    routing::post,
    Router,
};
use mtilib::{auth::JWTClaims, db::models::Service};
use serde::Deserialize;

use crate::api::AppState;

#[derive(Debug, Deserialize)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
}

pub async fn login(
    Query(query): Query<LoginBody>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    let service = match sqlx::query_as::<_, Service>(
        r#"
		SELECT *
		FROM "Services"
		WHERE id = $1
		"#,
    )
    .bind(query.username.clone())
    .fetch_one(&mut *state.db_pool.acquire().await.unwrap())
    .await
    {
        Ok(service) => service,
        Err(error) => match error {
            sqlx::Error::RowNotFound => return Err(StatusCode::BAD_REQUEST),
            _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    };

    if bcrypt::verify(&query.password, &service.password).unwrap() {
        let system_time = SystemTime::now();

        let token_claims = JWTClaims {
            sub: service.id.clone(),
            exp: system_time.duration_since(UNIX_EPOCH).unwrap().as_secs()
                + state.config.get_int("api.jwt.expiration").unwrap_or(3600) as u64,
        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(jsonwebtoken::Algorithm::RS512),
            &token_claims,
            &jsonwebtoken::EncodingKey::from_rsa_pem(state.jwt_keys.private.as_ref().unwrap())
                .unwrap(),
        )
        .unwrap();

        Ok(token)
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub fn router() -> Router<AppState> {
    Router::new().route("/login", post(login))
}
