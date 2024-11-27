use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use super::{AppState, JWTClaims};
use crate::models::{NewServiceUnit, Service, ServiceUnit};
use crate::schema::{ServiceUnits, Services};

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
}

pub async fn login_index(
    Query(login_body): Query<LoginBody>,
    State(app_state): State<AppState>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let pg_conn = &mut crate::db::create_conn(&app_state.config);

    let service_query = Services::table
        .select(Service::as_select())
        .filter(Services::name.eq(&login_body.username))
        .first(pg_conn)
        .unwrap();

    if bcrypt::verify(&login_body.password, &service_query.password).unwrap() {
        if service_query.max_one {
            let service_units_query = ServiceUnits::table
                .select(ServiceUnit::as_select())
                .filter(ServiceUnits::service_id.eq(service_query.id))
                .load(pg_conn)
                .unwrap();

            if service_units_query.len() > 0 {
                return Err(StatusCode::FORBIDDEN);
            }
        }

        let unit_uuid = uuid::Uuid::new_v4();
        let system_time = SystemTime::now();

        let token_claims = JWTClaims {
            id: unit_uuid.to_string(),
            srv: login_body.username.clone(),
            exp: system_time.duration_since(UNIX_EPOCH).unwrap().as_secs() + 30 * 24 * 60 * 60,
        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::new(Algorithm::RS512),
            &token_claims,
            &EncodingKey::from_rsa_pem(app_state.jwt_keys.private.as_ref().unwrap()).unwrap(),
        )
        .unwrap();

        let new_service_unit = NewServiceUnit {
            id: &unit_uuid.to_string(),
            service_id: service_query.id, // service id as queried from the database
            address: login_body.address,
            port: login_body.port,
        };
        diesel::insert_into(ServiceUnits::table)
            .values(&new_service_unit)
            .returning(ServiceUnit::as_returning())
            .get_result(pg_conn)
            .unwrap();

        Ok(Json(LoginResponse {
            token: token.to_string(),
        }))
    } else {
        Err(StatusCode::UNAUTHORIZED)
    }
}

pub async fn logout_index(headers: HeaderMap, State(app_state): State<AppState>) -> StatusCode {
    if !headers.contains_key("authorization") {
        return StatusCode::BAD_REQUEST;
    }

    let header_token = headers
        .get("authorization")
        .unwrap()
        .to_str()
        .unwrap()
        .split(" ")
        .collect::<Vec<&str>>();
    if header_token.len() < 2 {
        return StatusCode::BAD_REQUEST;
    }

    match jsonwebtoken::decode::<JWTClaims>(
        header_token[1],
        &DecodingKey::from_rsa_pem(&app_state.jwt_keys.public).unwrap(),
        &Validation::new(Algorithm::RS512),
    ) {
        Ok(token) => {
            let pg_conn = &mut crate::db::create_conn(&app_state.config);
            diesel::delete(ServiceUnits::table.filter(ServiceUnits::id.eq(token.claims.id)))
                .execute(pg_conn)
                .unwrap();

            StatusCode::OK
        }
        Err(err) => {
            println!("{:?}", err);
            StatusCode::UNAUTHORIZED
        }
    }
}
