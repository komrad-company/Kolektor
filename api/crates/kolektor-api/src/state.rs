use std::path::PathBuf;

use axum::extract::FromRef;
use sqlx::PgPool;

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub datasource_base: String,
    pub vector_output: PathBuf,
    pub tenant_id: Option<String>,
}

impl FromRef<AppState> for PgPool {
    fn from_ref(state: &AppState) -> Self {
        state.pool.clone()
    }
}
