use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Item (password) entity — encrypted record stored in a vault.
///
/// Fields ending in `_enc` are AES-256-GCM encrypted (server-side).
/// The plaintext is only accessible through the service layer which
/// handles encryption/decryption transparently.
#[derive(Debug, Clone, FromRow)]
pub struct Item {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub creator_id: Option<Uuid>,
    pub name_enc: String,
    pub login_enc: String,
    pub password_enc: String,
    pub url_enc: String,
    pub description_enc: String,
    pub customs_enc: String,
    pub tags: serde_json::Value,
    pub search_hashes: serde_json::Value,
    pub color_code: i16,
    pub password_changed_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub deleted_by: Option<Uuid>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Trash list entry — decrypted name/login for display across all vaults.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashListEntry {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub name: String,
    pub login: String,
    pub url: String,
    pub color_code: i16,
    pub vault_name: String,
    pub deleted_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

/// Decrypted item view for API responses.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemView {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub creator_id: Option<Uuid>,
    pub name: String,
    pub login: String,
    pub password: String,
    pub url: String,
    pub description: String,
    pub customs: Vec<CustomField>,
    pub tags: Vec<String>,
    pub color_code: i16,
    pub is_favorite: bool,
    pub password_changed_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Item list entry (name + metadata, no password).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ItemListEntry {
    pub id: Uuid,
    pub vault_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub name: String,
    pub login: String,
    pub url: String,
    pub tags: Vec<String>,
    pub color_code: i16,
    pub is_favorite: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Custom field on an item.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CustomField {
    pub name: String,
    pub value: String,
    /// "text", "password", "url", "email", "totp"
    #[serde(rename = "type")]
    pub field_type: String,
}

/// Create item DTO.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateItem {
    pub vault_id: Uuid,
    pub folder_id: Option<Uuid>,
    pub name: String,
    pub login: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub customs: Option<Vec<CustomField>>,
    pub tags: Option<Vec<String>>,
    pub color_code: Option<i16>,
}

/// Update item DTO.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateItem {
    pub name: Option<String>,
    pub login: Option<String>,
    pub password: Option<String>,
    pub url: Option<String>,
    pub description: Option<String>,
    pub customs: Option<Vec<CustomField>>,
    pub tags: Option<Vec<String>>,
    pub color_code: Option<i16>,
    pub folder_id: Option<Uuid>,
}

/// Move item DTO.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MoveItem {
    pub folder_id: Option<Uuid>,
    pub vault_id: Option<Uuid>,
}

/// Search request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub query: String,
    pub vault_id: Option<Uuid>,
}
