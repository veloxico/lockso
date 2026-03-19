use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

/// Stored WebAuthn credential.
#[derive(Debug, Clone, FromRow)]
pub struct WebAuthnCredential {
    pub id: Uuid,
    pub user_id: Uuid,
    pub credential_id: String,
    pub public_key: String,
    pub sign_count: i64,
    pub transports: serde_json::Value,
    pub device_name: String,
    pub aaguid: String,
    pub backed_up: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Credential view for API responses.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WebAuthnCredentialView {
    pub id: Uuid,
    pub credential_id: String,
    pub device_name: String,
    pub backed_up: bool,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
}

/// Registration options response (sent to browser to start registration).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationOptionsResponse {
    pub challenge: String,
    pub rp: RelyingParty,
    pub user: WebAuthnUser,
    pub pub_key_cred_params: Vec<PubKeyCredParam>,
    pub timeout: u64,
    pub authenticator_selection: AuthenticatorSelection,
    pub attestation: String,
    pub exclude_credentials: Vec<CredentialDescriptor>,
}

/// Authentication options response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationOptionsResponse {
    pub challenge: String,
    pub timeout: u64,
    pub rp_id: String,
    pub allow_credentials: Vec<CredentialDescriptor>,
    pub user_verification: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RelyingParty {
    pub name: String,
    pub id: String,
}

#[derive(Debug, Serialize)]
pub struct WebAuthnUser {
    pub id: String,
    pub name: String,
    #[serde(rename = "displayName")]
    pub display_name: String,
}

#[derive(Debug, Serialize)]
pub struct PubKeyCredParam {
    #[serde(rename = "type")]
    pub cred_type: String,
    pub alg: i32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatorSelection {
    pub authenticator_attachment: Option<String>,
    pub resident_key: String,
    pub require_resident_key: bool,
    pub user_verification: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CredentialDescriptor {
    #[serde(rename = "type")]
    pub cred_type: String,
    pub id: String,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub transports: Vec<String>,
}

/// Client registration response (from browser).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistrationResponse {
    pub id: String,
    pub raw_id: String,
    pub response: AuthenticatorAttestationResponse,
    #[serde(rename = "type")]
    pub cred_type: String,
    #[serde(default)]
    pub device_name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatorAttestationResponse {
    pub client_data_json: String,
    pub attestation_object: String,
    #[serde(default)]
    pub transports: Vec<String>,
}

/// Client authentication response (from browser).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticationResponse {
    pub id: String,
    pub raw_id: String,
    pub response: AuthenticatorAssertionResponse,
    #[serde(rename = "type")]
    pub cred_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthenticatorAssertionResponse {
    pub client_data_json: String,
    pub authenticator_data: String,
    pub signature: String,
    #[serde(default)]
    pub user_handle: Option<String>,
}
