use axum::{
    extract::{Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
};
use chrono::{DateTime, Utc};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::ApiError;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthContext {
    pub token_id: Uuid,
    pub name: String,
    pub tenant_id: String,
}

pub async fn require_bearer_token(
    State(pool): State<PgPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, ApiError> {
    let header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok());

    let provided_token = match header.and_then(|h| h.strip_prefix("Bearer ")) {
        Some(t) if !t.is_empty() => t.trim().to_string(),
        _ => return Err(ApiError::Unauthorized),
    };

    type TokenRow = (Uuid, String, String, String, Option<DateTime<Utc>>);
    let candidates: Vec<TokenRow> = sqlx::query_as(
        "SELECT id, name, token_hash, tenant_id, last_used_at FROM kolektor.api_tokens",
    )
    .fetch_all(&pool)
    .await?;

    let mut matched: Option<AuthContext> = None;
    for (id, name, hash, tenant_id, _last_used) in candidates {
        let provided_clone = provided_token.clone();
        let hash_clone = hash.clone();
        let is_valid = tokio::task::spawn_blocking(move || {
            bcrypt::verify(provided_clone, &hash_clone).unwrap_or(false)
        })
        .await
        .unwrap_or(false);

        if is_valid {
            matched = Some(AuthContext {
                token_id: id,
                name,
                tenant_id,
            });
            break;
        }
    }

    let ctx = matched.ok_or(ApiError::Unauthorized)?;

    let _ = sqlx::query("UPDATE kolektor.api_tokens SET last_used_at = now() WHERE id = $1")
        .bind(ctx.token_id)
        .execute(&pool)
        .await;

    request.extensions_mut().insert(ctx);
    Ok(next.run(request).await)
}
