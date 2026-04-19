use anyhow::{Context, Result};
use base64::Engine;
use chrono::{DateTime, Utc};
use kolektor_common::db;
use rand::RngCore;
use uuid::Uuid;

use crate::config::{TokenArgs, TokenCommand, TokenCreateArgs, TokenListArgs};

pub async fn run(args: TokenArgs) -> Result<()> {
    match args.command {
        TokenCommand::Create(a) => create(a).await,
        TokenCommand::List(a) => list(a).await,
    }
}

async fn create(args: TokenCreateArgs) -> Result<()> {
    let pool = db::connect(&args.database_url, 2).await?;

    let mut bytes = [0u8; 32];
    rand::thread_rng().fill_bytes(&mut bytes);
    let token = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes);

    let hash = bcrypt::hash(&token, bcrypt::DEFAULT_COST).context("hashing token with bcrypt")?;

    let id = Uuid::now_v7();
    sqlx::query(
        "INSERT INTO kolektor.api_tokens (id, name, token_hash, tenant_id) \
         VALUES ($1, $2, $3, $4)",
    )
    .bind(id)
    .bind(&args.name)
    .bind(&hash)
    .bind(&args.tenant_id)
    .execute(&pool)
    .await?;

    println!(
        "Token créé : name={}, tenant_id={}",
        args.name, args.tenant_id
    );
    println!("Token ID  : {id}");
    println!();
    println!("Secret à utiliser en header (ne sera plus affiché) :");
    println!("  Authorization: Bearer {token}");

    Ok(())
}

async fn list(args: TokenListArgs) -> Result<()> {
    let pool = db::connect(&args.database_url, 2).await?;

    type TokenListRow = (Uuid, String, String, Option<DateTime<Utc>>, DateTime<Utc>);
    let rows: Vec<TokenListRow> = sqlx::query_as(
        "SELECT id, name, tenant_id, last_used_at, created_at \
         FROM kolektor.api_tokens ORDER BY created_at DESC",
    )
    .fetch_all(&pool)
    .await?;

    println!(
        "{:<38} {:<20} {:<15} {:<25} created_at",
        "id", "name", "tenant_id", "last_used_at"
    );
    for (id, name, tenant_id, last_used, created_at) in rows {
        let last = last_used
            .map(|d| d.to_rfc3339())
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<38} {:<20} {:<15} {:<25} {}",
            id,
            name,
            tenant_id,
            last,
            created_at.to_rfc3339()
        );
    }

    Ok(())
}
