use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// A single permission grant — links a resource (vault/folder/item) to a grantee (user/group).
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessGrant {
    pub id: Uuid,
    pub vault_id: Option<Uuid>,
    pub folder_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub resource_access_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Grant view with resolved names for API responses.
#[derive(Debug, Clone, FromRow, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccessGrantView {
    pub id: Uuid,
    pub vault_id: Option<Uuid>,
    pub folder_id: Option<Uuid>,
    pub item_id: Option<Uuid>,
    pub user_id: Option<Uuid>,
    pub group_id: Option<Uuid>,
    pub grantee_name: String,
    pub grantee_type: String,
    pub access_code: String,
    pub access_name: String,
    pub resource_access_id: Uuid,
    pub granted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

/// Request to grant access.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GrantAccessRequest {
    /// "user" or "group"
    pub grantee_type: String,
    pub grantee_id: Uuid,
    pub resource_access_id: Uuid,
}
