use axum::{
    extract::{Query, State},
    http::StatusCode,
    routing::post,
    Extension, Json, Router,
};
use jsonwebtoken::{Algorithm, EncodingKey, TokenData};
use mtilib::auth::JWTClaims;
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};
use tracing::error;
use uuid::Uuid;

use super::AppState;
use mtilib::db::models::Service;

#[derive(Debug, Deserialize)]
pub struct LoginBody {
    pub username: String,
    pub password: String,
    pub address: Option<String>,
    pub port: Option<i32>,
}

#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
    pub uuid: Uuid,
}

pub async fn login(
    Query(login_body): Query<LoginBody>,
    State(state): State<AppState>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let service = match sqlx::query_as::<_, Service>(
        r#"
		SELECT *
		FROM "Services"
		WHERE id = $1
		"#,
    )
    .bind(login_body.username.clone())
    .fetch_one(&mut *state.db_pool.acquire().await.unwrap())
    .await
    {
        Ok(service) => service,
        Err(error) => match error {
            sqlx::Error::RowNotFound => return Err(StatusCode::BAD_REQUEST),
            _ => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    };

    if bcrypt::verify(&login_body.password, &service.password).unwrap() {
        if service.max_one {
            let service_units: i64 = sqlx::query_scalar(
                r#"
				SELECT COUNT(*)
				FROM "ServiceUnits"
				WHERE service_id = $1
				"#,
            )
            .bind(service.id.clone())
            .fetch_one(&mut *state.db_pool.acquire().await.unwrap())
            .await
            .unwrap();

            if service_units > 0 {
                return Err(StatusCode::FORBIDDEN);
            }
        }

        let new_unit_uuid = uuid::Uuid::new_v4();
        let system_time = SystemTime::now();

        let token_claims = JWTClaims {
            id: new_unit_uuid,
            srv: login_body.username.clone(),
            exp: system_time.duration_since(UNIX_EPOCH).unwrap().as_secs()
                + state
                    .config
                    .get_int("api.jwt.expiration")
                    .unwrap_or(2592000) as u64,
        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::RS512),
            &token_claims,
            &EncodingKey::from_rsa_pem(state.jwt_keys.private.as_ref().unwrap()).unwrap(),
        )
        .unwrap();

        match sqlx::query(
            r#"
			INSERT INTO "ServiceUnits"
			VALUES ($1, $2, $3, $4)
			"#,
        )
        .bind(new_unit_uuid)
        .bind(service.id)
        .bind(login_body.address)
        .bind(login_body.port)
        .execute(&mut *state.db_pool.acquire().await.unwrap())
        .await
        {
            Ok(_) => Ok(Json(LoginResponse {
                token: token.to_string(),
                uuid: new_unit_uuid,
            })),
            Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
        }
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn logout(
    Extension(jwt): Extension<TokenData<JWTClaims>>,
    State(state): State<AppState>,
) -> StatusCode {
    match sqlx::query(
        r#"
		DELETE FROM "ServiceUnits"
		WHERE id = $1
		"#,
    )
    .bind(jwt.claims.id)
    .execute(&mut *state.db_pool.acquire().await.unwrap())
    .await
    {
        Ok(_) => StatusCode::OK,
        Err(error) => {
            error!("Error while logging out a unit: {}", error);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}

pub fn router(state: AppState) -> Router<AppState> {
    Router::new().route("/login", post(login)).route(
        "/logout",
        post(logout).layer(axum::middleware::from_fn_with_state(
            state.clone(),
            mtilib::auth::axum_middleware::<AppState>,
        )),
    )
}
