use axum::{Json, extract::State};
use chrono::{DateTime, Utc};
use serde::Serialize;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Serialize)]
pub struct StatusResponse {
    pub parsers_total: i64,
    pub parsers_enabled: i64,
    pub fetchers_total: i64,
    pub fetchers_enabled: i64,
    pub last_reload_at: Option<DateTime<Utc>>,
    pub datasource_base: String,
    pub vector_output: String,
}

pub async fn get_status(State(state): State<AppState>) -> Result<Json<StatusResponse>, ApiError> {
    let total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kolektor.parsers")
        .fetch_one(&state.pool)
        .await?;

    let enabled: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM kolektor.parsers WHERE enabled = true")
            .fetch_one(&state.pool)
            .await?;

    let last_reload: Option<DateTime<Utc>> = sqlx::query_scalar(
        "SELECT created_at FROM kolektor.sync_events WHERE event_type = 'config_written' \
         ORDER BY created_at DESC LIMIT 1",
    )
    .fetch_optional(&state.pool)
    .await?
    .flatten();

    let fetchers_total: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM kolektor.fetchers")
        .fetch_one(&state.pool)
        .await?;

    let fetchers_enabled: i64 =
        sqlx::query_scalar("SELECT COUNT(*) FROM kolektor.fetchers WHERE enabled = true")
            .fetch_one(&state.pool)
            .await?;

    Ok(Json(StatusResponse {
        parsers_total: total,
        parsers_enabled: enabled,
        fetchers_total,
        fetchers_enabled,
        last_reload_at: last_reload,
        datasource_base: state.datasource_base.clone(),
        vector_output: state.vector_output.display().to_string(),
    }))
}
