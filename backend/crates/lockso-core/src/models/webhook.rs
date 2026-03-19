use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Webhook DB row.
#[derive(Debug, Clone, FromRow)]
pub struct Webhook {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub url_enc: String,
    pub events: serde_json::Value,
    pub is_enabled: bool,
    pub creator_id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Webhook view for API (URL masked).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookView {
    pub id: Uuid,
    pub name: String,
    pub provider: String,
    pub url_masked: String,
    pub events: Vec<String>,
    pub is_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Create webhook request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateWebhook {
    pub name: String,
    pub provider: String,
    pub url: String,
    pub events: Vec<String>,
}

/// Update webhook request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateWebhook {
    pub name: Option<String>,
    pub url: Option<String>,
    pub events: Option<Vec<String>>,
    pub is_enabled: Option<bool>,
}

/// Supported webhook providers.
pub const WEBHOOK_PROVIDERS: &[&str] = &["telegram", "slack", "discord", "custom"];

/// All subscribable event codes.
pub const WEBHOOK_EVENTS: &[&str] = &[
    "user.login",
    "user.login_failed",
    "user.register",
    "user.blocked",
    "item.created",
    "item.updated",
    "item.trashed",
    "item.restored",
    "vault.created",
    "vault.deleted",
    "send.created",
    "send.accessed",
    "trash.emptied",
    "settings.updated",
];
