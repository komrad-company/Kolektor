use axum::{
    Json,
    extract::{Path, Query, State},
};
use kolektor_common::models::Fetcher;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::error::ApiError;
use crate::state::AppState;

const ALLOWED_PROVIDERS: &[&str] = &["microsoft_graph", "microsoft365_management", "s3"];

#[derive(Debug, Deserialize, Default)]
pub struct ListQuery {
    pub enabled: Option<bool>,
    pub provider: Option<String>,
    pub parser_source_type: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CreateFetcherBody {
    pub name: String,
    pub provider: String,
    pub parser_source_type: String,
    pub enabled: Option<bool>,
    pub interval_seconds: Option<i32>,
    pub output_path: String,
    pub config: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateFetcherBody {
    pub name: Option<String>,
    pub provider: Option<String>,
    pub parser_source_type: Option<String>,
    pub enabled: Option<bool>,
    pub interval_seconds: Option<i32>,
    pub output_path: Option<String>,
    pub config: Option<Value>,
    pub state: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct EnabledBody {
    pub enabled: bool,
}

#[derive(Debug, Serialize)]
pub struct FetcherSummary {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub parser_source_type: String,
    pub enabled: bool,
    pub interval_seconds: i32,
    pub output_path: String,
    pub config: Value,
    pub state: Value,
    pub last_attempt_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_success_at: Option<chrono::DateTime<chrono::Utc>>,
    pub last_error: Option<String>,
    pub version: i32,
}

impl From<Fetcher> for FetcherSummary {
    fn from(f: Fetcher) -> Self {
        Self {
            id: f.id,
            name: f.name,
            provider: f.provider,
            parser_source_type: f.parser_source_type,
            enabled: f.enabled,
            interval_seconds: f.interval_seconds,
            output_path: f.output_path,
            config: f.config,
            state: f.state,
            last_attempt_at: f.last_attempt_at,
            last_success_at: f.last_success_at,
            last_error: f.last_error,
            version: f.version,
        }
    }
}

pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<FetcherSummary>>, ApiError> {
    let mut sql = String::from("SELECT * FROM kolektor.fetchers WHERE 1=1");
    let mut idx = 1;
    if q.enabled.is_some() {
        sql.push_str(&format!(" AND enabled = ${idx}"));
        idx += 1;
    }
    if q.provider.is_some() {
        sql.push_str(&format!(" AND provider = ${idx}"));
        idx += 1;
    }
    if q.parser_source_type.is_some() {
        sql.push_str(&format!(" AND parser_source_type = ${idx}"));
    }
    sql.push_str(" ORDER BY provider, name");

    let mut query = sqlx::query_as::<_, Fetcher>(&sql);
    if let Some(enabled) = q.enabled {
        query = query.bind(enabled);
    }
    if let Some(provider) = q.provider {
        query = query.bind(provider);
    }
    if let Some(parser_source_type) = q.parser_source_type {
        query = query.bind(parser_source_type);
    }

    let fetchers = query.fetch_all(&state.pool).await?;
    Ok(Json(
        fetchers.into_iter().map(FetcherSummary::from).collect(),
    ))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<FetcherSummary>, ApiError> {
    let fetcher = fetcher_by_id(&state, id).await?;
    Ok(Json(fetcher.into()))
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateFetcherBody>,
) -> Result<Json<FetcherSummary>, ApiError> {
    validate_provider(&body.provider)?;
    validate_interval(body.interval_seconds.unwrap_or(300))?;

    ensure_parser_exists(&state, &body.parser_source_type).await?;

    let fetcher: Fetcher = sqlx::query_as(
        "INSERT INTO kolektor.fetchers (
            id, name, provider, parser_source_type, enabled, interval_seconds, output_path, config
         ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING *",
    )
    .bind(Uuid::now_v7())
    .bind(body.name)
    .bind(body.provider)
    .bind(body.parser_source_type)
    .bind(body.enabled.unwrap_or(false))
    .bind(body.interval_seconds.unwrap_or(300))
    .bind(body.output_path)
    .bind(body.config.unwrap_or_else(|| json!({})))
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(fetcher.into()))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateFetcherBody>,
) -> Result<Json<FetcherSummary>, ApiError> {
    let current = fetcher_by_id(&state, id).await?;

    let provider = body.provider.unwrap_or(current.provider);
    validate_provider(&provider)?;

    let interval_seconds = body.interval_seconds.unwrap_or(current.interval_seconds);
    validate_interval(interval_seconds)?;

    let parser_source_type = body
        .parser_source_type
        .unwrap_or(current.parser_source_type);
    ensure_parser_exists(&state, &parser_source_type).await?;

    let updated: Fetcher = sqlx::query_as(
        "UPDATE kolektor.fetchers SET
            name = $2,
            provider = $3,
            parser_source_type = $4,
            enabled = $5,
            interval_seconds = $6,
            output_path = $7,
            config = $8,
            state = $9,
            version = version + 1,
            updated_at = now()
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(body.name.unwrap_or(current.name))
    .bind(provider)
    .bind(parser_source_type)
    .bind(body.enabled.unwrap_or(current.enabled))
    .bind(interval_seconds)
    .bind(body.output_path.unwrap_or(current.output_path))
    .bind(body.config.unwrap_or(current.config))
    .bind(body.state.unwrap_or(current.state))
    .fetch_one(&state.pool)
    .await?;

    Ok(Json(updated.into()))
}

pub async fn put_enabled(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(body): Json<EnabledBody>,
) -> Result<Json<FetcherSummary>, ApiError> {
    let updated: Fetcher = sqlx::query_as(
        "UPDATE kolektor.fetchers
         SET enabled = $2, version = version + 1, updated_at = now()
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(body.enabled)
    .fetch_optional(&state.pool)
    .await?
    .ok_or(ApiError::NotFound)?;

    Ok(Json(updated.into()))
}

pub async fn delete_one(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Value>, ApiError> {
    let rows = sqlx::query("DELETE FROM kolektor.fetchers WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await?
        .rows_affected();

    if rows == 0 {
        return Err(ApiError::NotFound);
    }

    Ok(Json(json!({ "deleted": true, "id": id })))
}

async fn fetcher_by_id(state: &AppState, id: Uuid) -> Result<Fetcher, ApiError> {
    sqlx::query_as::<_, Fetcher>("SELECT * FROM kolektor.fetchers WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await?
        .ok_or(ApiError::NotFound)
}

async fn ensure_parser_exists(state: &AppState, source_type: &str) -> Result<(), ApiError> {
    let exists: bool =
        sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM kolektor.parsers WHERE source_type = $1)")
            .bind(source_type)
            .fetch_one(&state.pool)
            .await?;
    if exists {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "parser_source_type {source_type:?} does not exist"
        )))
    }
}

fn validate_provider(provider: &str) -> Result<(), ApiError> {
    if ALLOWED_PROVIDERS.contains(&provider) {
        Ok(())
    } else {
        Err(ApiError::BadRequest(format!(
            "unsupported provider {provider:?}"
        )))
    }
}

fn validate_interval(interval_seconds: i32) -> Result<(), ApiError> {
    if interval_seconds >= 30 {
        Ok(())
    } else {
        Err(ApiError::BadRequest(
            "interval_seconds must be >= 30".to_string(),
        ))
    }
}
