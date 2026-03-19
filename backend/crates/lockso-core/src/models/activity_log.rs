use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Activity action constants — stored as VARCHAR(64) in the DB.
pub struct ActivityAction;

impl ActivityAction {
    // Auth
    pub const LOGIN: &str = "auth.login";
    pub const LOGIN_FAILED: &str = "auth.login_failed";
    pub const LOGOUT: &str = "auth.logout";
    pub const REGISTER: &str = "auth.register";

    // 2FA
    pub const TOTP_ENABLED: &str = "2fa.enabled";
    pub const TOTP_DISABLED: &str = "2fa.disabled";
    pub const TOTP_VERIFIED: &str = "2fa.verified";
    pub const TOTP_FAILED: &str = "2fa.failed";

    // Vault
    pub const VAULT_CREATED: &str = "vault.created";
    pub const VAULT_UPDATED: &str = "vault.updated";
    pub const VAULT_DELETED: &str = "vault.deleted";

    // Item
    pub const ITEM_CREATED: &str = "item.created";
    pub const ITEM_UPDATED: &str = "item.updated";
    pub const ITEM_DELETED: &str = "item.deleted";
    pub const ITEM_MOVED: &str = "item.moved";

    // Folder
    pub const FOLDER_CREATED: &str = "folder.created";
    pub const FOLDER_UPDATED: &str = "folder.updated";
    pub const FOLDER_DELETED: &str = "folder.deleted";

    // Sharing
    pub const SHARING_GRANTED: &str = "sharing.granted";
    pub const SHARING_REVOKED: &str = "sharing.revoked";
    pub const SHARING_UPDATED: &str = "sharing.updated";

    // Attachments
    pub const ATTACHMENT_UPLOADED: &str = "attachment.uploaded";
    pub const ATTACHMENT_DELETED: &str = "attachment.deleted";

    // User management (admin)
    pub const USER_ROLE_CHANGED: &str = "user.role_changed";
    pub const USER_BLOCKED: &str = "user.blocked";
    pub const USER_UNBLOCKED: &str = "user.unblocked";
    pub const USER_DELETED: &str = "user.deleted";

    // Groups
    pub const GROUP_CREATED: &str = "group.created";
    pub const GROUP_UPDATED: &str = "group.updated";
    pub const GROUP_DELETED: &str = "group.deleted";
    pub const GROUP_MEMBER_ADDED: &str = "group.member_added";
    pub const GROUP_MEMBER_REMOVED: &str = "group.member_removed";

    // Trash
    pub const ITEM_TRASHED: &str = "item.trashed";
    pub const ITEM_RESTORED: &str = "item.restored";
    pub const ITEM_PURGED: &str = "item.purged";
    pub const TRASH_EMPTIED: &str = "trash.emptied";

    // Sends
    pub const SEND_CREATED: &str = "send.created";
    pub const SEND_ACCESSED: &str = "send.accessed";
    pub const SEND_DELETED: &str = "send.deleted";

    // Settings
    pub const SETTINGS_UPDATED: &str = "settings.updated";
}

/// DB row.
#[derive(Debug, Clone, FromRow)]
pub struct ActivityLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub vault_id: Option<Uuid>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Joined row with user display name.
#[derive(Debug, Clone, FromRow)]
pub struct ActivityLogRow {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_name: Option<String>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub vault_id: Option<Uuid>,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// API response view.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityLogView {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub user_name: Option<String>,
    pub action: String,
    pub resource_type: Option<String>,
    pub resource_id: Option<Uuid>,
    pub vault_id: Option<Uuid>,
    pub client_ip: Option<String>,
    pub details: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Query parameters for listing activity logs.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActivityLogQuery {
    pub page: Option<u32>,
    pub per_page: Option<u32>,
    pub user_id: Option<Uuid>,
    pub action: Option<String>,
}

/// Paginated response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PaginatedActivityLogs {
    pub data: Vec<ActivityLogView>,
    pub total: i64,
    pub page: u32,
    pub per_page: u32,
}
