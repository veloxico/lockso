use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Resource access level — defines what a user/group can do within a vault.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceAccess {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub permissions: serde_json::Value,
    pub priority: i32,
    pub is_access_override_allowed: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
