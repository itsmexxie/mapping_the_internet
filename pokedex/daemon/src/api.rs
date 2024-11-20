use axum::extract::{Path, Query, State};
use axum::http::HeaderMap;
use axum::routing::get;
use axum::{http::StatusCode, routing::post, Json, Router};
use config::Config;
use diesel::query_dsl::methods::{FilterDsl, SelectDsl};
use diesel::{pg, ExpressionMethods, RunQueryDsl, SelectableHelper};
use jsonwebtoken::{decode, encode, DecodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tower_http::trace::TraceLayer;
use tracing::{debug, info};

use crate::models::{NewServiceUnit, Service, ServiceUnit};
use crate::schema::{ServiceUnits, Services};

#[derive(Deserialize)]
struct LoginBody {
    username: String,
    password: String,
    address: Option<String>,
    port: Option<i32>,
}

#[derive(Debug, Serialize, Deserialize)]
struct JWTClaims {
    id: String,
    srv: String,
    exp: u64,
}

#[derive(Serialize)]
struct LoginResponse {
    token: String,
}

async fn login(
    State(api_state): State<ApiState>,
    Query(login_body): Query<LoginBody>,
) -> Result<Json<LoginResponse>, StatusCode> {
    let pg_conn = &mut crate::db::create_conn(&api_state.config);

    let password_query = Services::table
        .select(Service::as_select())
        .filter(Services::name.eq(&login_body.username))
        .first(pg_conn)
        .unwrap();

    if bcrypt::verify(login_body.password, &password_query.password).unwrap() {
        let unit_uuid = uuid::Uuid::new_v4();
        let system_time = SystemTime::now();

        let token_claims = JWTClaims {
            id: unit_uuid.to_string(),
            srv: login_body.username,
            exp: system_time.duration_since(UNIX_EPOCH).unwrap().as_secs() + 30 * 24 * 60 * 60,
        };
        let token = encode(
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

        let new_service_unit = NewServiceUnit {
            id: &unit_uuid.to_string(),
            service_id: password_query.id, // service id as queried from the database
            address: login_body.address.as_deref(),
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

async fn logout(headers: HeaderMap, State(api_state): State<ApiState>) -> StatusCode {
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

    match decode::<JWTClaims>(
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

#[derive(Serialize)]
struct ApiService {
    id: i32,
    name: String,
}

async fn get_services(
    State(api_state): State<ApiState>,
) -> Result<Json<Vec<ApiService>>, StatusCode> {
    let pg_conn = &mut crate::db::create_conn(&api_state.config);

    let query_results = Services::dsl::Services
        .select(Service::as_select())
        .load(pg_conn)
        .unwrap();

    let mut api_results = Vec::new();
    for query_result in query_results {
        api_results.push(ApiService {
            id: query_result.id,
            name: query_result.name,
        });
    }

    Ok(Json(api_results))
}

async fn get_service(
    Path(service_id): Path<i32>,
    State(api_state): State<ApiState>,
) -> Result<Json<ApiService>, StatusCode> {
    let pg_conn = &mut crate::db::create_conn(&api_state.config);

    let query_result = Services::dsl::Services
        .filter(Services::id.eq(service_id))
        .select(Service::as_select())
        .first(pg_conn)
        .unwrap();

    Ok(Json(ApiService {
        id: query_result.id,
        name: query_result.name,
    }))
}

#[derive(Clone)]
struct ApiState {
    config: Arc<Config>,
}

pub async fn run(config: Arc<Config>) {
    let api_state = ApiState {
        config: config.clone(),
    };

    let app = Router::new()
        .route("/login", post(login))
        .route("/logout", post(logout))
        .route("/services", get(get_services))
        .route("/services/:service_id", get(get_service))
        .with_state(api_state)
        .layer(TraceLayer::new_for_http());
    let app_port = config.get("api.port").unwrap();

    let listener = tokio::net::TcpListener::bind(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)),
        app_port,
    ))
    .await
    .unwrap();

    info!("[API] Listening on port {}", app_port);
    axum::serve(listener, app).await.unwrap();
}
