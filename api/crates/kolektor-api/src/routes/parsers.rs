use axum::{
    Json,
    extract::{Path, Query, State},
};
use kolektor_common::models::Parser;
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

use crate::config_writer;
use crate::error::ApiError;
use crate::state::AppState;

#[derive(Debug, Deserialize, Default)]
pub struct ListQuery {
    pub enabled: Option<bool>,
    pub category: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ParserSummary {
    pub id: Uuid,
    pub source_type: String,
    pub display_name: String,
    pub category: String,
    pub default_port: Option<i32>,
    pub ocsf_class_uid: Option<i32>,
    pub ocsf_category_uid: Option<i32>,
    pub ocsf_index: Option<String>,
    pub description: Option<String>,
    pub built_in: bool,
    pub enabled: bool,
    pub version: i32,
}

impl From<&Parser> for ParserSummary {
    fn from(p: &Parser) -> Self {
        Self {
            id: p.id,
            source_type: p.source_type.clone(),
            display_name: p.display_name.clone(),
            category: p.category.clone(),
            default_port: p.default_port,
            ocsf_class_uid: p.ocsf_class_uid,
            ocsf_category_uid: p.ocsf_category_uid,
            ocsf_index: p.ocsf_index.clone(),
            description: p.description.clone(),
            built_in: p.built_in,
            enabled: p.enabled,
            version: p.version,
        }
    }
}

pub async fn list(
    State(state): State<AppState>,
    Query(q): Query<ListQuery>,
) -> Result<Json<Vec<ParserSummary>>, ApiError> {
    let mut sql = String::from("SELECT * FROM kolektor.parsers WHERE 1=1");
    if q.enabled.is_some() {
        sql.push_str(" AND enabled = $1");
    }
    if q.category.is_some() {
        if q.enabled.is_some() {
            sql.push_str(" AND category = $2");
        } else {
            sql.push_str(" AND category = $1");
        }
    }
    sql.push_str(" ORDER BY category, source_type");

    let mut query = sqlx::query_as::<_, Parser>(&sql);
    if let Some(e) = q.enabled {
        query = query.bind(e);
    }
    if let Some(c) = q.category {
        query = query.bind(c);
    }
    let parsers = query.fetch_all(&state.pool).await?;

    Ok(Json(parsers.iter().map(ParserSummary::from).collect()))
}

pub async fn get_one(
    State(state): State<AppState>,
    Path((category, name)): Path<(String, String)>,
) -> Result<Json<Parser>, ApiError> {
    let source_type = format!("{category}/{name}");
    let parser =
        sqlx::query_as::<_, Parser>("SELECT * FROM kolektor.parsers WHERE source_type = $1")
            .bind(&source_type)
            .fetch_optional(&state.pool)
            .await?
            .ok_or(ApiError::NotFound)?;

    Ok(Json(parser))
}

#[derive(Debug, Deserialize)]
pub struct EnabledBody {
    pub enabled: bool,
}

pub async fn put_enabled(
    State(state): State<AppState>,
    Path((category, name)): Path<(String, String)>,
    Json(body): Json<EnabledBody>,
) -> Result<Json<Parser>, ApiError> {
    let source_type = format!("{category}/{name}");

    let mut tx = state.pool.begin().await?;

    let current: Option<Parser> = sqlx::query_as::<_, Parser>(
        "SELECT * FROM kolektor.parsers WHERE source_type = $1 FOR UPDATE",
    )
    .bind(&source_type)
    .fetch_optional(&mut *tx)
    .await?;

    let current = current.ok_or(ApiError::NotFound)?;

    if current.enabled == body.enabled {
        tx.commit().await?;
        return Ok(Json(current));
    }

    let updated: Parser = sqlx::query_as::<_, Parser>(
        "UPDATE kolektor.parsers SET enabled = $1, updated_at = now() \
         WHERE source_type = $2 RETURNING *",
    )
    .bind(body.enabled)
    .bind(&source_type)
    .fetch_one(&mut *tx)
    .await?;

    let active: Vec<Parser> = sqlx::query_as::<_, Parser>(
        "SELECT * FROM kolektor.parsers WHERE enabled = true ORDER BY source_type",
    )
    .fetch_all(&mut *tx)
    .await?;

    let event_type = if body.enabled {
        "parser_enabled"
    } else {
        "parser_disabled"
    };
    sqlx::query(
        "INSERT INTO kolektor.sync_events (id, event_type, parser_id, payload) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(Uuid::now_v7())
    .bind(event_type)
    .bind(updated.id)
    .bind(json!({ "source_type": source_type }))
    .execute(&mut *tx)
    .await?;

    sqlx::query(
        "INSERT INTO kolektor.sync_events (id, event_type, payload) VALUES ($1, 'config_written', $2)",
    )
    .bind(Uuid::now_v7())
    .bind(json!({ "active_count": active.len() }))
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    let content = config_writer::assemble_toml(&active, &state.datasource_base);
    config_writer::write_atomic(&state.vector_output, &content).await?;

    tracing::info!(%source_type, enabled = body.enabled, "parser updated, vector config rewritten");

    Ok(Json(updated))
}
