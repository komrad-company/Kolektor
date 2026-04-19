use std::path::PathBuf;

use anyhow::Result;
use kolektor_common::{db, models::Parser};

use crate::config::InitArgs;
use crate::config_writer;

pub async fn run(args: InitArgs) -> Result<()> {
    let datasource_base = std::env::var("DATASOURCE_ID")
        .or_else(|_| std::env::var("DATASOURCE_ID_BASE"))
        .unwrap_or_else(|_| "ds".to_string());

    tracing::info!(%datasource_base, catalog_dir = %args.catalog_dir, output = %args.vector_output, "init: starting");

    let pool = db::connect(&args.database_url, 2).await?;

    tracing::info!("running migrations");
    sqlx::migrate!("../../migrations").run(&pool).await?;

    tracing::info!("seeding catalog");
    let catalog_dir = PathBuf::from(&args.catalog_dir);
    let report = kolektor_seed::seed(&pool, &catalog_dir).await?;
    tracing::info!(
        inserted = report.inserted,
        updated = report.updated,
        "seed done"
    );

    tracing::info!("writing initial Vector config");
    let enabled: Vec<Parser> = sqlx::query_as::<_, Parser>(
        "SELECT * FROM kolektor.parsers WHERE enabled = true ORDER BY source_type",
    )
    .fetch_all(&pool)
    .await?;
    let content = config_writer::assemble_toml(&enabled, &datasource_base);
    let output = PathBuf::from(&args.vector_output);
    config_writer::write_atomic(&output, &content).await?;

    tracing::info!(enabled_count = enabled.len(), output = %output.display(), "init done");
    Ok(())
}
