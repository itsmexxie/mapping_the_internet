use axum::extract::{Path, State};
use axum::routing::get;
use axum::{http::StatusCode, Json, Router};
use mtilib::db::models::{Service, ServiceUnit};

use crate::api::AppState;

async fn get_services(State(state): State<AppState>) -> Result<Json<Vec<Service>>, StatusCode> {
    match sqlx::query_as::<_, Service>(
        r#"
		SELECT *
		FROM "Services"
		"#,
    )
    .fetch_all(&mut *state.db_pool.acquire().await.unwrap())
    .await
    {
        Ok(row) => Ok(Json(row)),
        Err(error) => match error {
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

async fn get_service(
    Path(service_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Service>, StatusCode> {
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
        Ok(row) => Ok(Json(row)),
        Err(error) => match error {
            sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

async fn get_service_units(
    Path(service_id): Path<i32>,
    State(state): State<AppState>,
) -> Result<Json<Vec<ServiceUnit>>, StatusCode> {
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
        Ok(rows) => Ok(Json(rows)),
        Err(error) => match error {
            sqlx::Error::RowNotFound => Err(StatusCode::NOT_FOUND),
            _ => Err(StatusCode::INTERNAL_SERVER_ERROR),
        },
    }
}

pub fn router(state: AppState) -> Router<AppState> {
    Router::new()
        .route("/", get(get_services))
        .route("/{service_id}", get(get_service))
        .route("/{service_id}/units", get(get_service_units))
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            mtilib::auth::axum_middleware::<AppState>,
        ))
}
