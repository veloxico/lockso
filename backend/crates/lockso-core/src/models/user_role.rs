use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// User role entity with permissions.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct UserRole {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub permissions: serde_json::Value,
    pub auth_settings: serde_json::Value,
    pub manageable_user_roles: serde_json::Value,
    pub offline_access: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Role view for API responses.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserRoleView {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub permissions: serde_json::Value,
    pub auth_settings: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRole> for UserRoleView {
    fn from(r: UserRole) -> Self {
        Self {
            id: r.id,
            name: r.name,
            code: r.code,
            permissions: r.permissions,
            auth_settings: r.auth_settings,
            created_at: r.created_at,
            updated_at: r.updated_at,
        }
    }
}
