use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use diesel::{ExpressionMethods, QueryDsl, RunQueryDsl, SelectableHelper};
use jsonwebtoken::{DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use super::{ApiState, JWTClaims};
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
    State(api_state): State<ApiState>,
    Query(login_body): Query<LoginBody>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let pg_conn = &mut crate::db::create_conn(&api_state.config);

    let password_query = Services::table
        .select(Service::as_select())
        .filter(Services::name.eq(&login_body.username))
        .first(pg_conn)
        .unwrap();

    if bcrypt::verify(&login_body.password, &password_query.password).unwrap() {
        let unit_uuid = uuid::Uuid::new_v4();
        let system_time = SystemTime::now();

        let token_claims = JWTClaims {
            id: unit_uuid.to_string(),
            srv: login_body.username.clone(),
            exp: system_time.duration_since(UNIX_EPOCH).unwrap().as_secs() + 30 * 24 * 60 * 60,
        };
        let token = jsonwebtoken::encode(
            &jsonwebtoken::Header::default(),
            &token_claims,
            &jsonwebtoken::EncodingKey::from_secret(
                api_state
                    .config
                    .get_string("api.jwtKey")
                    .expect("api.jwtKey must be set!")
                    .as_ref(),
            ),
        )
        .unwrap();

        let service_unit_address = match login_body.address {
            Some(address) => match address.parse::<ipnetwork::IpNetwork>() {
                Ok(inet_address) => Some(inet_address),
                Err(_) => return Err(StatusCode::BAD_REQUEST),
            },
            None => None,
        };

        let new_service_unit = NewServiceUnit {
            id: &unit_uuid.to_string(),
            service_id: password_query.id, // service id as queried from the database
            address: service_unit_address,
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

pub async fn logout_index(headers: HeaderMap, State(api_state): State<ApiState>) -> StatusCode {
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
        &DecodingKey::from_secret(
            api_state
                .config
                .get_string("api.jwtKey")
                .expect("api.jwtKey must be set!")
                .as_ref(),
        ),
        &Validation::default(),
    ) {
        Ok(token) => {
            let pg_conn = &mut crate::db::create_conn(&api_state.config);
            println!("logout token ok");
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
