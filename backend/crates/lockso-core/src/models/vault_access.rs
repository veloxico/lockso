use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Vault-user access link (DB row).
#[derive(Debug, Clone, FromRow)]
pub struct VaultUserAccess {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub user_id: Uuid,
    pub resource_access_id: Uuid,
    pub granted_by: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Vault member info returned in API responses.
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultMember {
    pub id: Uuid,
    pub user_id: Uuid,
    pub login: String,
    pub full_name: String,
    pub email: Option<String>,
    pub access_code: String,
    pub access_name: String,
    pub resource_access_id: Uuid,
    pub granted_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Request to share a vault with a user.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShareVaultRequest {
    pub user_id: Uuid,
    pub resource_access_id: Uuid,
}

/// Request to update a member's access level.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAccessRequest {
    pub resource_access_id: Uuid,
}
