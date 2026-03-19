use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Send row in the database.
#[derive(Debug, Clone, FromRow)]
pub struct Send {
    pub id: Uuid,
    pub creator_id: Uuid,
    pub access_id: String,
    pub ciphertext_b64: String,
    pub passphrase_hash: Option<String>,
    pub max_views: i16,
    pub view_count: i16,
    pub expires_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Owner's list view.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendListEntry {
    pub id: Uuid,
    pub access_id: String,
    pub has_passphrase: bool,
    pub max_views: i16,
    pub view_count: i16,
    pub expires_at: DateTime<Utc>,
    pub is_expired: bool,
    pub is_consumed: bool,
    pub created_at: DateTime<Utc>,
}

/// Create send request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSend {
    pub ciphertext_b64: String,
    pub passphrase: Option<String>,
    pub max_views: Option<i16>,
    pub ttl_hours: Option<i32>,
}

/// Create send response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSendResponse {
    pub id: Uuid,
    pub access_id: String,
}

/// Public send metadata (returned before passphrase check).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendPublicMeta {
    pub has_passphrase: bool,
}

/// Public send access response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SendAccessView {
    pub ciphertext_b64: String,
}

/// Passphrase submission.
#[derive(Debug, Deserialize)]
pub struct SendPassphraseRequest {
    pub passphrase: String,
}
