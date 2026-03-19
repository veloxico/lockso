use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Vault type entity.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct VaultType {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub allowed_users: serde_json::Value,
    pub allowed_groups: serde_json::Value,
    pub allowed_roles: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Lightweight view for API responses.
#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct VaultTypeView {
    pub id: Uuid,
    pub name: String,
    pub code: String,
}
