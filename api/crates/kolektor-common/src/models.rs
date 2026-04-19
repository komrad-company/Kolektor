use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Parser {
    pub id: Uuid,
    pub source_type: String,
    pub display_name: String,
    pub category: String,
    pub default_port: Option<i32>,
    pub ocsf_class_uid: Option<i32>,
    pub ocsf_category_uid: Option<i32>,
    pub ocsf_index: Option<String>,
    pub vector_toml: String,
    pub description: Option<String>,
    pub built_in: bool,
    pub enabled: bool,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct ApiToken {
    pub id: Uuid,
    pub name: String,
    pub token_hash: String,
    pub tenant_id: String,
    pub last_used_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}
