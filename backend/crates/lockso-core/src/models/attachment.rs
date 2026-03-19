use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Database row for an attachment.
#[derive(Debug, Clone, FromRow)]
pub struct Attachment {
    pub id: Uuid,
    pub item_id: Uuid,
    pub vault_id: Uuid,
    pub uploader_id: Uuid,
    pub name_enc: String,
    pub storage_key: String,
    pub size_bytes: i64,
    pub mime_type: String,
    pub created_at: DateTime<Utc>,
}

/// API response for an attachment (decrypted filename).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentView {
    pub id: Uuid,
    pub item_id: Uuid,
    pub name: String,
    pub size_bytes: i64,
    pub mime_type: String,
    pub uploader_id: Uuid,
    pub created_at: DateTime<Utc>,
}

/// Request body for deleting an attachment (empty — ID is in the URL path).
/// Upload is handled via multipart form, not JSON.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachmentUploadMeta {
    pub item_id: Uuid,
}
