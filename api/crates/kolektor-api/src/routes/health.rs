use axum::{Json, extract::State};
use serde_json::{Value, json};
use sqlx::PgPool;

pub async fn health(State(pool): State<PgPool>) -> Json<Value> {
    let db_ok = sqlx::query("SELECT 1").execute(&pool).await.is_ok();
    Json(json!({
        "status": if db_ok { "ok" } else { "degraded" },
        "database": db_ok,
    }))
}
