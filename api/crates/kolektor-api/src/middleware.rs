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
use crate::state::AppState;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct AuthContext {
    pub token_id: Uuid,
    pub name: String,
    pub tenant_id: String,
}

pub async fn require_bearer_token(
    State(state): State<AppState>,
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

    let matched = match token_id_from_prefixed_token(&provided_token) {
        Some(token_id) => verify_token_by_id(&state.pool, token_id, &provided_token).await?,
        None => verify_legacy_token(&state.pool, &provided_token).await?,
    };

    let ctx = matched.ok_or(ApiError::Unauthorized)?;
    if let Some(expected_tenant) = &state.tenant_id
        && ctx.tenant_id != *expected_tenant
    {
        return Err(ApiError::Unauthorized);
    }

    let _ = sqlx::query("UPDATE kolektor.api_tokens SET last_used_at = now() WHERE id = $1")
        .bind(ctx.token_id)
        .execute(&state.pool)
        .await;

    request.extensions_mut().insert(ctx);
    Ok(next.run(request).await)
}

type TokenRow = (Uuid, String, String, String, Option<DateTime<Utc>>);

pub fn token_id_from_prefixed_token(token: &str) -> Option<Uuid> {
    let mut parts = token.splitn(3, '_');
    match (parts.next(), parts.next(), parts.next()) {
        (Some("klt"), Some(id), Some(secret)) if !secret.is_empty() => Uuid::parse_str(id).ok(),
        _ => None,
    }
}

async fn verify_token_by_id(
    pool: &PgPool,
    token_id: Uuid,
    provided_token: &str,
) -> Result<Option<AuthContext>, ApiError> {
    let row: Option<TokenRow> = sqlx::query_as(
        "SELECT id, name, token_hash, tenant_id, last_used_at \
         FROM kolektor.api_tokens WHERE id = $1",
    )
    .bind(token_id)
    .fetch_optional(pool)
    .await?;

    verify_row(row, provided_token).await
}

async fn verify_legacy_token(
    pool: &PgPool,
    provided_token: &str,
) -> Result<Option<AuthContext>, ApiError> {
    let candidates: Vec<TokenRow> = sqlx::query_as(
        "SELECT id, name, token_hash, tenant_id, last_used_at FROM kolektor.api_tokens",
    )
    .fetch_all(pool)
    .await?;

    for row in candidates {
        if let Some(ctx) = verify_row(Some(row), provided_token).await? {
            return Ok(Some(ctx));
        }
    }

    Ok(None)
}

async fn verify_row(
    row: Option<TokenRow>,
    provided_token: &str,
) -> Result<Option<AuthContext>, ApiError> {
    let Some((id, name, hash, tenant_id, _last_used)) = row else {
        return Ok(None);
    };

    let provided_clone = provided_token.to_string();
    let is_valid =
        tokio::task::spawn_blocking(move || bcrypt::verify(provided_clone, &hash).unwrap_or(false))
            .await
            .unwrap_or(false);

    if is_valid {
        Ok(Some(AuthContext {
            token_id: id,
            name,
            tenant_id,
        }))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_prefixed_token_id() {
        let id = Uuid::now_v7();
        let token = format!("klt_{}_secret", id.simple());
        assert_eq!(token_id_from_prefixed_token(&token), Some(id));
    }

    #[test]
    fn ignores_legacy_tokens() {
        assert_eq!(token_id_from_prefixed_token("plainlegacysecret"), None);
    }
}
