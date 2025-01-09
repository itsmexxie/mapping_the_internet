use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use mtilib::db::models::{Service, ServiceUnit};

use crate::api::AppState;

pub async fn services(State(state): State<AppState>) -> impl IntoResponse {
    match sqlx::query_as::<_, Service>(
        r#"
		SELECT *
		FROM "Services"
		"#,
    )
    .fetch_all(&mut *state.db_pool.acquire().await.unwrap())
    .await
    {
        Ok(rows) => Ok(Json(
            rows.iter()
                .map(|service| mtilib::pokedex::types::Service::from(service))
                .collect::<Vec<_>>(),
        )),
        Err(error) => match error {
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

pub async fn service(
    Path(service_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, Service>(
        r#"
		SELECT *
		FROM "Services"
        WHERE id = $1
		"#,
    )
    .bind(service_id)
    .fetch_one(&mut *state.db_pool.acquire().await.unwrap())
    .await
    {
        Ok(row) => Ok(Json(mtilib::pokedex::types::Service::from(row))),
        Err(error) => match error {
            sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

pub async fn service_units(
    Path(service_id): Path<String>,
    State(state): State<AppState>,
) -> impl IntoResponse {
    match sqlx::query_as::<_, ServiceUnit>(
        r#"
		SELECT *
		FROM "ServiceUnits"
        WHERE service_id = $1
		"#,
    )
    .bind(service_id)
    .fetch_all(&mut *state.db_pool.acquire().await.unwrap())
    .await
    {
        Ok(row) => Ok(Json(row)),
        Err(error) => match error {
            sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(services))
        .route("/{service_id}", get(service))
        .route("/{service_id}/units", get(service_units))
        .layer(axum::middleware::from_fn_with_state(
            state,
            mtilib::auth::axum_middleware::<AppState>,
        ))
}
