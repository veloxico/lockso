use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Session entity — represents an authenticated session.
#[derive(Debug, Clone, FromRow)]
pub struct Session {
    pub id: Uuid,
    pub user_id: Uuid,
    pub access_token_hash: String,
    pub refresh_token_hash: String,
    pub auth_method: String,
    pub is_two_factor_auth_required: bool,
    pub client_type: String,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub access_token_expired_at: DateTime<Utc>,
    pub refresh_token_expired_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub pin_code_hash: Option<String>,
    pub is_pin_code_required: bool,
    pub webauthn_challenge: Option<String>,
    pub last_authentications: serde_json::Value,
    pub attributes: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

/// Session info returned to the client.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionView {
    pub id: Uuid,
    pub auth_method: String,
    pub client_type: String,
    pub client_ip: Option<String>,
    pub user_agent: Option<String>,
    pub access_token_expired_at: DateTime<Utc>,
    pub last_activity_at: DateTime<Utc>,
    pub is_current: bool,
    pub created_at: DateTime<Utc>,
}

/// Login request payload.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginRequest {
    pub login: String,
    pub password: String,
    #[serde(default = "default_client_type")]
    pub client_type: String,
}

fn default_client_type() -> String {
    "Web".to_string()
}

/// Login response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expired_at: DateTime<Utc>,
    pub refresh_token_expired_at: DateTime<Utc>,
    pub user: super::user::UserView,
    pub is_two_factor_auth_required: bool,
    pub is_master_key_required: bool,
}

/// Refresh token request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshRequest {
    pub refresh_token: String,
}

/// Refresh token response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefreshResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub access_token_expired_at: DateTime<Utc>,
    pub refresh_token_expired_at: DateTime<Utc>,
}
