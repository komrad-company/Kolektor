use std::net::SocketAddr;
use std::path::PathBuf;

use anyhow::Result;
use axum::{
    Router, middleware as axum_middleware,
    routing::{get, put},
};
use kolektor_common::db;
use tower_http::trace::TraceLayer;

use crate::config::ServeArgs;
use crate::middleware;
use crate::routes;
use crate::state::AppState;

pub async fn run(args: ServeArgs) -> Result<()> {
    let datasource_base = std::env::var("DATASOURCE_ID")
        .or_else(|_| std::env::var("DATASOURCE_ID_BASE"))
        .unwrap_or_else(|_| "ds".to_string());

    let pool = db::connect(&args.database_url, args.database_max_connections).await?;

    let state = AppState {
        pool: pool.clone(),
        datasource_base,
        vector_output: PathBuf::from(&args.vector_output),
    };

    let authed = Router::new()
        .route("/status", get(routes::status::get_status))
        .route("/parsers", get(routes::parsers::list))
        .route("/parsers/{category}/{name}", get(routes::parsers::get_one))
        .route(
            "/parsers/{category}/{name}/enabled",
            put(routes::parsers::put_enabled),
        )
        .layer(axum_middleware::from_fn_with_state(
            pool.clone(),
            middleware::require_bearer_token,
        ))
        .with_state(state);

    let public = Router::new()
        .route("/health", get(routes::health::health))
        .with_state(pool.clone());

    let api = Router::new().merge(public).merge(authed);

    let app = Router::new()
        .nest("/v1", api)
        .layer(TraceLayer::new_for_http());

    let addr: SocketAddr = args.listen_addr.parse()?;
    tracing::info!(%addr, output = %args.vector_output, "serve: listening");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

async fn shutdown_signal() {
    tokio::signal::ctrl_c()
        .await
        .expect("failed to install CTRL+C handler");
    tracing::info!("shutdown signal received");
}
