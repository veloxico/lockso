use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Vault entity — secure container for folders and items.
#[derive(Debug, Clone, FromRow)]
pub struct Vault {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub vault_type_id: Uuid,
    pub creator_id: Option<Uuid>,
    pub salt: String,
    pub color_code: i16,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Vault view for API responses.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct VaultView {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub vault_type_id: Uuid,
    pub creator_id: Option<Uuid>,
    pub color_code: i16,
    pub item_count: i64,
    pub folder_count: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Vault list item (lightweight).
#[derive(Debug, Serialize, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct VaultListItem {
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub vault_type_id: Uuid,
    pub color_code: i16,
    pub item_count: i64,
    pub folder_count: i64,
    pub created_at: DateTime<Utc>,
}

/// Create vault DTO.
///
/// `vault_type_id` accepts either a UUID or the string `"default"`,
/// which resolves to the Organization vault type.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateVault {
    pub name: String,
    pub description: Option<String>,
    pub vault_type_id: String,
    pub color_code: Option<i16>,
}

/// Update vault DTO.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateVault {
    pub name: Option<String>,
    pub description: Option<String>,
    pub color_code: Option<i16>,
}
