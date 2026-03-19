use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// API key DB row.
#[derive(Debug, Clone, FromRow)]
pub struct ApiKey {
    pub id: Uuid,
    pub name: String,
    pub key_hash: String,
    pub key_prefix: String,
    pub user_id: Uuid,
    pub permission: String,
    pub vault_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
}

/// API key view (no hash exposed).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyView {
    pub id: Uuid,
    pub name: String,
    pub key_prefix: String,
    pub user_id: Uuid,
    pub permission: String,
    pub vault_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
}

/// Response when creating a key (shows the raw key ONCE).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyCreated {
    pub id: Uuid,
    pub name: String,
    /// The raw API key — shown only once, never stored.
    pub key: String,
    pub key_prefix: String,
    pub permission: String,
    pub vault_id: Option<Uuid>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

/// Create API key request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateApiKey {
    pub name: String,
    /// "read" or "read_write"
    pub permission: String,
    /// Optional vault restriction.
    pub vault_id: Option<Uuid>,
    /// Optional expiration (ISO 8601).
    pub expires_at: Option<DateTime<Utc>>,
}

pub const VALID_PERMISSIONS: &[&str] = &["read", "read_write"];
