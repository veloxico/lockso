use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Supported email providers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum EmailProvider {
    Smtp,
    Sendgrid,
    Ses,
    Resend,
    Mailgun,
    Postmark,
    Mandrill,
}

impl EmailProvider {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Smtp => "smtp",
            Self::Sendgrid => "sendgrid",
            Self::Ses => "ses",
            Self::Resend => "resend",
            Self::Mailgun => "mailgun",
            Self::Postmark => "postmark",
            Self::Mandrill => "mandrill",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "smtp" => Some(Self::Smtp),
            "sendgrid" => Some(Self::Sendgrid),
            "ses" => Some(Self::Ses),
            "resend" => Some(Self::Resend),
            "mailgun" => Some(Self::Mailgun),
            "postmark" => Some(Self::Postmark),
            "mandrill" => Some(Self::Mandrill),
            _ => None,
        }
    }
}

/// Email settings row from DB.
#[derive(Debug, Clone, FromRow)]
pub struct EmailSettings {
    pub id: Uuid,
    pub provider: String,
    pub is_enabled: bool,
    pub from_name: String,
    pub from_email: String,
    pub config_enc: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Email settings API response (no secrets).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EmailSettingsView {
    pub provider: String,
    pub is_enabled: bool,
    pub from_name: String,
    pub from_email: String,
    /// Config with sensitive fields masked.
    pub config: serde_json::Value,
    pub updated_at: DateTime<Utc>,
}

/// Update email settings DTO.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateEmailSettings {
    pub provider: String,
    pub is_enabled: bool,
    pub from_name: String,
    pub from_email: String,
    /// Provider-specific configuration (will be encrypted).
    pub config: serde_json::Value,
}

/// SMTP configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SmtpConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    #[serde(default)]
    pub use_tls: bool,
}

/// API key-based provider configuration (SendGrid, Resend, Mailgun, Postmark, Mandrill).
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ApiKeyConfig {
    pub api_key: String,
    /// Only for Mailgun — the sending domain.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub domain: String,
    /// Only for Mailgun — EU region.
    #[serde(default)]
    pub eu_region: bool,
}

/// Amazon SES configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SesConfig {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub region: String,
}

/// Test email request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SendTestEmail {
    pub to: String,
}
