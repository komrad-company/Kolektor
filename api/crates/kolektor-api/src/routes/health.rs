use axum::{Json, extract::State, http::StatusCode};
use serde_json::{Value, json};
use sqlx::PgPool;

pub async fn health(State(pool): State<PgPool>) -> (StatusCode, Json<Value>) {
    let db_ok = sqlx::query("SELECT 1").execute(&pool).await.is_ok();
    let status = if db_ok {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (
        status,
        Json(json!({
            "status": if db_ok { "ok" } else { "degraded" },
            "database": db_ok,
        })),
    )
}
