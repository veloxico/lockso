use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Application settings — singleton row with 17 JSONB category columns.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub id: Uuid,
    pub session: serde_json::Value,
    pub email: serde_json::Value,
    pub sso: serde_json::Value,
    pub search: serde_json::Value,
    pub favicon: serde_json::Value,
    pub interface: serde_json::Value,
    pub notification: serde_json::Value,
    pub custom_banner: serde_json::Value,
    pub user_lockout: serde_json::Value,
    pub activity_log: serde_json::Value,
    pub browser_extension: serde_json::Value,
    pub auth_password_complexity: serde_json::Value,
    pub master_password_complexity: serde_json::Value,
    pub vault: serde_json::Value,
    pub task: serde_json::Value,
    pub user: serde_json::Value,
    pub internal: serde_json::Value,
    pub trash: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Session settings category.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionSettings {
    /// Access token TTL in seconds (default: 3600 = 1 hour).
    pub access_token_ttl: i64,
    /// Refresh token TTL in seconds (default: 2592000 = 30 days).
    pub refresh_token_ttl: i64,
    /// Inactivity TTL in seconds, Web clients only (default: 1800 = 30 min).
    /// Set to 0 to disable.
    pub inactivity_ttl: i64,
    /// CSRF token TTL in seconds (default: 3600).
    pub csrf_token_ttl: i64,
}

impl Default for SessionSettings {
    fn default() -> Self {
        Self {
            access_token_ttl: 3600,
            refresh_token_ttl: 2_592_000,
            inactivity_ttl: 1800,
            csrf_token_ttl: 3600,
        }
    }
}

/// Password complexity rules.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PasswordComplexity {
    pub min_length: u32,
    pub require_uppercase: bool,
    pub require_lowercase: bool,
    pub require_digits: bool,
    pub require_special: bool,
}

impl Default for PasswordComplexity {
    fn default() -> Self {
        Self {
            min_length: 8,
            require_uppercase: true,
            require_lowercase: true,
            require_digits: true,
            require_special: false,
        }
    }
}

/// Master password complexity rules (stricter defaults).
impl PasswordComplexity {
    pub fn master_default() -> Self {
        Self {
            min_length: 12,
            require_uppercase: true,
            require_lowercase: true,
            require_digits: true,
            require_special: true,
        }
    }
}

/// Trash settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrashSettings {
    /// Retention period in days before auto-purge.
    pub retention_days: u32,
    /// Whether auto-purge is enabled.
    pub auto_empty_enabled: bool,
}

impl Default for TrashSettings {
    fn default() -> Self {
        Self {
            retention_days: 30,
            auto_empty_enabled: true,
        }
    }
}

/// User lockout settings.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UserLockoutSettings {
    /// Whether brute-force protection is enabled.
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Max failed login attempts before lockout.
    pub max_attempts: u32,
    /// Window in seconds to count failures.
    pub window_seconds: u64,
    /// Lockout duration in seconds.
    pub lockout_seconds: u64,
}

fn default_true() -> bool {
    true
}

impl Default for UserLockoutSettings {
    fn default() -> Self {
        Self {
            enabled: true,
            max_attempts: 7,
            window_seconds: 180,
            lockout_seconds: 60,
        }
    }
}
