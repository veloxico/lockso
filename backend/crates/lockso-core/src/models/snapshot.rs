use chrono::{DateTime, Utc};
use serde::Serialize;
use sqlx::FromRow;
use uuid::Uuid;

use super::item::CustomField;

/// Snapshot entity — immutable point-in-time record of an item's state.
/// Auto-created on every item create/update.
#[derive(Debug, Clone, FromRow)]
pub struct Snapshot {
    pub id: Uuid,
    pub item_id: Uuid,
    pub vault_id: Uuid,
    pub name_enc: String,
    pub login_enc: String,
    pub password_enc: String,
    pub url_enc: String,
    pub description_enc: String,
    pub customs_enc: String,
    pub tags: serde_json::Value,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Decrypted snapshot view.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotView {
    pub id: Uuid,
    pub item_id: Uuid,
    pub name: String,
    pub login: String,
    pub password: String,
    pub url: String,
    pub description: String,
    pub customs: Vec<CustomField>,
    pub tags: Vec<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Snapshot list entry (no decrypted password for list view).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SnapshotListEntry {
    pub id: Uuid,
    pub item_id: Uuid,
    pub name: String,
    pub login: String,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}
