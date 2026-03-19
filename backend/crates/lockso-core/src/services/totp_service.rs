use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::encryption::{decrypt_field, encrypt_field};
use crate::error::AppError;
use lockso_crypto::totp;

/// Number of recovery codes to generate.
const RECOVERY_CODE_COUNT: usize = 8;

/// TOTP issuer name shown in authenticator apps.
const TOTP_ISSUER: &str = "Lockso";

/// 2FA setup response (returned when enabling 2FA).
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TotpSetupResponse {
    pub secret: String,
    pub otpauth_uri: String,
    pub recovery_codes: Vec<String>,
}

/// 2FA status response.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TotpStatus {
    pub is_enabled: bool,
    pub recovery_codes_remaining: u32,
}

/// Enable 2FA request (legacy — kept for reference only).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
pub struct EnableTotpRequest {
    pub secret: String,
    pub code: String,
}

/// Verify 2FA request (during login).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct VerifyTotpRequest {
    pub code: String,
}

/// Disable 2FA request.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DisableTotpRequest {
    pub code: String,
}

/// Begin 2FA setup: generate secret and otpauth URI.
///
/// The secret is NOT saved yet — user must verify with a code first.
pub async fn setup_totp(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<TotpSetupResponse, AppError> {
    // Check 2FA is not already enabled
    let status = get_totp_status(pool, user_id).await?;
    if status.is_enabled {
        return Err(AppError::Validation("2FA is already enabled".into()));
    }

    // Get user login for the otpauth URI
    let login: (String,) = sqlx::query_as("SELECT login FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await?;

    // Generate secret
    let secret_bytes = totp::generate_totp_secret()
        .map_err(|_| AppError::Internal("Failed to generate TOTP secret".into()))?;
    let secret_base32 = totp::encode_secret_base32(&secret_bytes);
    let uri = totp::build_otpauth_uri(&secret_base32, &login.0, TOTP_ISSUER);

    // Generate recovery codes
    let recovery_codes = totp::generate_recovery_codes(RECOVERY_CODE_COUNT)
        .map_err(|_| AppError::Internal("Failed to generate recovery codes".into()))?;

    Ok(TotpSetupResponse {
        secret: secret_base32,
        otpauth_uri: uri,
        recovery_codes,
    })
}

/// Enable 2FA request (verify initial code to confirm setup).
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EnableTotpRequestV2 {
    pub secret: String,
    pub code: String,
    pub recovery_codes: Vec<String>,
}

/// Confirm 2FA setup by verifying the first code.
///
/// Saves the encrypted secret and recovery codes to the user's auth_settings.
/// Recovery codes are passed from the client (same ones shown during setup)
/// to ensure the user sees and stores the exact codes that will work.
pub async fn enable_totp(
    pool: &PgPool,
    key: &[u8],
    user_id: Uuid,
    secret_base32: &str,
    code: &str,
    recovery_codes: &[String],
) -> Result<TotpStatus, AppError> {
    // Decode and verify the secret + code
    let secret_bytes = totp::decode_secret_base32(secret_base32)
        .map_err(|_| AppError::Validation("Invalid TOTP secret".into()))?;

    let now = current_timestamp();
    let valid = totp::verify_totp(&secret_bytes, code, now)
        .map_err(|_| AppError::Internal("TOTP verification failed".into()))?;

    if !valid {
        return Err(AppError::Validation("Invalid verification code".into()));
    }

    // Validate recovery codes were provided
    if recovery_codes.len() != RECOVERY_CODE_COUNT {
        return Err(AppError::Validation("Invalid recovery codes".into()));
    }

    // Encrypt the secret before storing
    let encrypted_secret = encrypt_field(key, secret_base32)?;

    // Hash recovery codes for storage (store hashes, not plaintext)
    let recovery_hashes: Vec<String> = recovery_codes
        .iter()
        .map(|code| lockso_crypto::search_hash::hash_token(&code.replace('-', "")))
        .collect();

    // Update user's auth_settings with 2FA data
    let totp_data = serde_json::json!({
        "totpEnabled": true,
        "totpSecretEnc": encrypted_secret,
        "recoveryCodeHashes": recovery_hashes,
    });

    sqlx::query(
        "UPDATE users SET auth_settings = auth_settings || $1, updated_at = NOW() WHERE id = $2",
    )
    .bind(&totp_data)
    .bind(user_id)
    .execute(pool)
    .await?;

    tracing::info!(user_id = %user_id, "2FA enabled");

    Ok(TotpStatus {
        is_enabled: true,
        recovery_codes_remaining: recovery_codes.len() as u32,
    })
}

/// Verify a TOTP code (during login or sensitive operations).
///
/// Uses Redis to prevent TOTP code replay within the validity window.
pub async fn verify_totp_code(
    pool: &PgPool,
    redis: &redis::aio::ConnectionManager,
    key: &[u8],
    user_id: Uuid,
    code: &str,
) -> Result<bool, AppError> {
    let auth_settings = get_auth_settings(pool, user_id).await?;

    let totp_enabled = auth_settings
        .get("totpEnabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !totp_enabled {
        // Only pass through if called from a context where 2FA is not expected.
        // When called from disable_totp, this is guarded by the caller checking status first.
        return Err(AppError::Validation("2FA is not enabled".into()));
    }

    // First try as a TOTP code
    let encrypted_secret = auth_settings
        .get("totpSecretEnc")
        .and_then(|v| v.as_str())
        .ok_or_else(|| AppError::Internal("TOTP secret not found".into()))?;

    let secret_base32 = decrypt_field(key, encrypted_secret)?;
    let secret_bytes = totp::decode_secret_base32(&secret_base32)
        .map_err(|_| AppError::Internal("Failed to decode TOTP secret".into()))?;

    let now = current_timestamp();
    let valid = totp::verify_totp(&secret_bytes, code, now)
        .map_err(|_| AppError::Internal("TOTP verification failed".into()))?;

    if valid {
        // Prevent replay: check if this code was already used within the window
        // Hash the code in the key to avoid exposing TOTP codes in Redis
        let code_hash = lockso_crypto::search_hash::hash_token(code);
        let replay_key = format!("totp_used:{}:{}", user_id, &code_hash[..16]);
        let mut conn = redis.clone();
        let already_used: bool = redis::cmd("SET")
            .arg(&replay_key)
            .arg("1")
            .arg("NX") // Only set if not exists
            .arg("EX")
            .arg(90_u64) // Expire after 90s (TOTP window)
            .query_async(&mut conn)
            .await
            .map(|v: Option<String>| v.is_none()) // None means key already existed
            .unwrap_or(true); // Fail closed: treat Redis error as replay

        if already_used {
            tracing::warn!(user_id = %user_id, "TOTP code replay attempt blocked");
            return Ok(false);
        }

        return Ok(true);
    }

    // Try as a recovery code
    let code_normalized = code.replace('-', "");
    let code_hash = lockso_crypto::search_hash::hash_token(&code_normalized);

    let recovery_hashes: Vec<String> = auth_settings
        .get("recoveryCodeHashes")
        .and_then(|v| serde_json::from_value(v.clone()).ok())
        .unwrap_or_default();

    if let Some(idx) = recovery_hashes.iter().position(|h| h == &code_hash) {
        // Remove the used recovery code
        let mut updated_hashes = recovery_hashes;
        updated_hashes.remove(idx);

        let update_data = serde_json::json!({
            "recoveryCodeHashes": updated_hashes,
        });

        sqlx::query(
            "UPDATE users SET auth_settings = auth_settings || $1, updated_at = NOW() WHERE id = $2",
        )
        .bind(&update_data)
        .bind(user_id)
        .execute(pool)
        .await?;

        tracing::warn!(user_id = %user_id, remaining = updated_hashes.len(), "Recovery code used");
        return Ok(true);
    }

    Ok(false)
}

/// Disable 2FA for a user.
pub async fn disable_totp(
    pool: &PgPool,
    redis: &redis::aio::ConnectionManager,
    key: &[u8],
    user_id: Uuid,
    code: &str,
) -> Result<TotpStatus, AppError> {
    // Check 2FA is actually enabled before verifying
    let status = get_totp_status(pool, user_id).await?;
    if !status.is_enabled {
        return Err(AppError::Validation("2FA is not enabled".into()));
    }

    // Verify the code first
    let valid = verify_totp_code(pool, redis, key, user_id, code).await?;
    if !valid {
        return Err(AppError::Validation("Invalid verification code".into()));
    }

    // Remove TOTP data from auth_settings
    sqlx::query(
        r#"UPDATE users SET
            auth_settings = auth_settings - 'totpEnabled' - 'totpSecretEnc' - 'recoveryCodeHashes',
            updated_at = NOW()
        WHERE id = $1"#,
    )
    .bind(user_id)
    .execute(pool)
    .await?;

    tracing::info!(user_id = %user_id, "2FA disabled");

    Ok(TotpStatus {
        is_enabled: false,
        recovery_codes_remaining: 0,
    })
}

/// Get 2FA status for a user.
pub async fn get_totp_status(pool: &PgPool, user_id: Uuid) -> Result<TotpStatus, AppError> {
    let auth_settings = get_auth_settings(pool, user_id).await?;

    let is_enabled = auth_settings
        .get("totpEnabled")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    let recovery_remaining = auth_settings
        .get("recoveryCodeHashes")
        .and_then(|v| v.as_array())
        .map(|arr| arr.len() as u32)
        .unwrap_or(0);

    Ok(TotpStatus {
        is_enabled,
        recovery_codes_remaining: if is_enabled {
            recovery_remaining
        } else {
            0
        },
    })
}

/// Check if user has 2FA enabled.
pub async fn is_totp_enabled(pool: &PgPool, user_id: Uuid) -> Result<bool, AppError> {
    let status = get_totp_status(pool, user_id).await?;
    Ok(status.is_enabled)
}

// ─── Helpers ───

async fn get_auth_settings(pool: &PgPool, user_id: Uuid) -> Result<serde_json::Value, AppError> {
    let row: Option<(serde_json::Value,)> =
        sqlx::query_as("SELECT auth_settings FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    row.map(|(v,)| v)
        .ok_or(AppError::UserNotFound)
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
