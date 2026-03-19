use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// Note: Serialize is used by UserView and CreateUser, not by User.

/// User entity — core authentication subject.
///
/// Note: Serialize is intentionally NOT derived on User to prevent
/// accidental serialization of sensitive fields (password_hash, keys, etc.)
/// in logs or responses. Always use UserView for API responses.
#[derive(Debug, Clone, FromRow, Deserialize)]
pub struct User {
    pub id: Uuid,
    pub login: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub full_name: String,
    pub master_key_options: serde_json::Value,
    pub master_key_hash: Option<String>,
    pub keys_public: Option<String>,
    pub keys_private_encrypted: Option<String>,
    pub signup_type: String,
    pub role_id: Uuid,
    pub auth_settings: serde_json::Value,
    pub blocked_ips: serde_json::Value,
    pub interface_settings: serde_json::Value,
    pub client_settings: serde_json::Value,
    pub password_hash_history: serde_json::Value,
    pub is_blocked: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User creation DTO (for registration).
#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub login: String,
    pub password: String,
    pub email: Option<String>,
    pub full_name: Option<String>,
    pub master_key_hash: Option<String>,
    pub keys_public: Option<String>,
    pub keys_private_encrypted: Option<String>,
}

/// Public user view (safe to return in API responses).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserView {
    pub id: Uuid,
    pub login: String,
    pub email: Option<String>,
    pub full_name: String,
    pub signup_type: String,
    pub role_id: Uuid,
    pub is_blocked: bool,
    pub last_login_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<User> for UserView {
    fn from(u: User) -> Self {
        Self {
            id: u.id,
            login: u.login,
            email: u.email,
            full_name: u.full_name,
            signup_type: u.signup_type,
            role_id: u.role_id,
            is_blocked: u.is_blocked,
            last_login_at: u.last_login_at,
            created_at: u.created_at,
            updated_at: u.updated_at,
        }
    }
}
