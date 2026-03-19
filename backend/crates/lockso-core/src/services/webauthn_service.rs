//! WebAuthn/FIDO2 service — handles registration and authentication ceremonies.
//!
//! Supports ES256 (P-256/ECDSA with SHA-256) which is the most widely
//! supported algorithm by hardware authenticators.

use chrono::Utc;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppError;
use crate::models::webauthn::*;

/// Timeout for WebAuthn ceremonies (5 minutes).
const CEREMONY_TIMEOUT_MS: u64 = 300_000;

// ─── Registration ───────────────────────────────────────────────────────────

/// Generate registration options for the browser.
///
/// The challenge is stored in the session's `webauthn_challenge` field.
pub async fn begin_registration(
    pool: &PgPool,
    user_id: Uuid,
    user_login: &str,
    user_display_name: &str,
    rp_id: &str,
    rp_name: &str,
    session_id: Uuid,
) -> Result<RegistrationOptionsResponse, AppError> {
    // Get existing credentials to exclude
    let existing = list_credentials(pool, user_id).await?;
    let exclude: Vec<CredentialDescriptor> = existing
        .iter()
        .map(|c| CredentialDescriptor {
            cred_type: "public-key".into(),
            id: c.credential_id.clone(),
            transports: vec![],
        })
        .collect();

    // Generate random challenge (32 bytes)
    let challenge_bytes: [u8; 32] = rand::random();
    let challenge = base64_url::encode(&challenge_bytes);

    // Store challenge in session for verification
    sqlx::query("UPDATE sessions SET webauthn_challenge = $1 WHERE id = $2")
        .bind(&challenge)
        .bind(session_id)
        .execute(pool)
        .await?;

    let user_id_b64 = base64_url::encode(user_id.as_bytes());

    Ok(RegistrationOptionsResponse {
        challenge,
        rp: RelyingParty {
            name: rp_name.to_string(),
            id: rp_id.to_string(),
        },
        user: WebAuthnUser {
            id: user_id_b64,
            name: user_login.to_string(),
            display_name: user_display_name.to_string(),
        },
        pub_key_cred_params: vec![
            PubKeyCredParam {
                cred_type: "public-key".into(),
                alg: -7, // ES256
            },
        ],
        timeout: CEREMONY_TIMEOUT_MS,
        authenticator_selection: AuthenticatorSelection {
            authenticator_attachment: None,
            resident_key: "preferred".into(),
            require_resident_key: false,
            user_verification: "preferred".into(),
        },
        attestation: "none".into(),
        exclude_credentials: exclude,
    })
}

/// Verify the registration response from the browser and store the credential.
pub async fn finish_registration(
    pool: &PgPool,
    user_id: Uuid,
    session_id: Uuid,
    rp_id: &str,
    response: RegistrationResponse,
    strict_origin: bool,
) -> Result<WebAuthnCredentialView, AppError> {
    // Retrieve and clear challenge
    let stored_challenge = get_and_clear_challenge(pool, session_id).await?;

    // Decode client data JSON
    let client_data_bytes = base64_url::decode(&response.response.client_data_json)
        .map_err(|_| AppError::Validation("Invalid clientDataJSON encoding".into()))?;

    let client_data: serde_json::Value = serde_json::from_slice(&client_data_bytes)
        .map_err(|_| AppError::Validation("Invalid clientDataJSON".into()))?;

    // Verify client data
    let cd_type = client_data["type"].as_str().unwrap_or("");
    if cd_type != "webauthn.create" {
        return Err(AppError::Validation("Invalid ceremony type".into()));
    }

    let cd_challenge = client_data["challenge"].as_str().unwrap_or("");
    if cd_challenge != stored_challenge {
        return Err(AppError::Validation("Challenge mismatch".into()));
    }

    let cd_origin = client_data["origin"].as_str().unwrap_or("");
    // Origin must contain RP ID (e.g., https://lockso.example.com must contain "lockso.example.com")
    if !cd_origin.contains(rp_id) {
        if strict_origin {
            return Err(AppError::Validation("Origin does not match RP ID".into()));
        }
        tracing::warn!(origin = cd_origin, rp_id = rp_id, "Origin/RP ID mismatch — allowing for development");
    }

    // Parse attestation object (CBOR)
    let att_obj_bytes = base64_url::decode(&response.response.attestation_object)
        .map_err(|_| AppError::Validation("Invalid attestation object encoding".into()))?;

    // Extract authenticator data from attestation object
    // The attestation object is CBOR-encoded. We need to extract authData from it.
    // For "none" attestation, the structure is: {"fmt": "none", "attStmt": {}, "authData": bytes}
    let auth_data = extract_auth_data_from_attestation(&att_obj_bytes)?;

    // Parse authenticator data
    // Format: RP ID hash (32) | flags (1) | sign count (4) | [attested cred data] | [extensions]
    if auth_data.len() < 37 {
        return Err(AppError::Validation("Authenticator data too short".into()));
    }

    let rp_id_hash = &auth_data[0..32];
    let expected_rp_hash = Sha256::digest(rp_id.as_bytes());
    if rp_id_hash != expected_rp_hash.as_slice() {
        return Err(AppError::Validation("RP ID hash mismatch".into()));
    }

    let flags = auth_data[32];
    let user_present = flags & 0x01 != 0;
    if !user_present {
        return Err(AppError::Validation("User not present".into()));
    }

    let attested_cred_data = flags & 0x40 != 0;
    if !attested_cred_data {
        return Err(AppError::Validation("No attested credential data".into()));
    }

    let sign_count = u32::from_be_bytes([
        auth_data[33], auth_data[34], auth_data[35], auth_data[36],
    ]) as i64;

    // Parse attested credential data (starts at offset 37)
    // Format: AAGUID (16) | Credential ID Length (2) | Credential ID (L) | COSE Public Key
    if auth_data.len() < 55 {
        return Err(AppError::Validation("Auth data too short for attested credential".into()));
    }

    let aaguid = hex::encode(&auth_data[37..53]);
    let cred_id_len = u16::from_be_bytes([auth_data[53], auth_data[54]]) as usize;

    if auth_data.len() < 55 + cred_id_len {
        return Err(AppError::Validation("Auth data too short for credential ID".into()));
    }

    let cred_id_bytes = &auth_data[55..55 + cred_id_len];
    let credential_id = base64_url::encode(cred_id_bytes);

    // Verify this credential doesn't already exist
    let dup: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM webauthn_credentials WHERE credential_id = $1")
            .bind(&credential_id)
            .fetch_optional(pool)
            .await?;
    if dup.is_some() {
        return Err(AppError::Validation("Credential already registered".into()));
    }

    // Extract COSE public key (rest of auth_data after credential ID)
    let cose_key_bytes = &auth_data[55 + cred_id_len..];
    let public_key = base64_url::encode(cose_key_bytes);

    let transports = serde_json::to_value(&response.response.transports).unwrap_or_default();
    let device_name = if response.device_name.is_empty() {
        "Security Key".to_string()
    } else {
        response.device_name.chars().take(100).collect::<String>()
    };

    let id = Uuid::now_v7();
    let now = Utc::now();

    sqlx::query(
        r#"INSERT INTO webauthn_credentials (
            id, user_id, credential_id, public_key, sign_count,
            transports, device_name, aaguid, created_at
        ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)"#,
    )
    .bind(id)
    .bind(user_id)
    .bind(&credential_id)
    .bind(&public_key)
    .bind(sign_count)
    .bind(&transports)
    .bind(&device_name)
    .bind(&aaguid)
    .bind(now)
    .execute(pool)
    .await?;

    Ok(WebAuthnCredentialView {
        id,
        credential_id,
        device_name,
        backed_up: false,
        created_at: now,
        last_used_at: None,
    })
}

// ─── Authentication ─────────────────────────────────────────────────────────

/// Generate authentication options.
pub async fn begin_authentication(
    pool: &PgPool,
    user_id: Uuid,
    rp_id: &str,
    session_id: Uuid,
) -> Result<AuthenticationOptionsResponse, AppError> {
    let creds = sqlx::query_as::<_, WebAuthnCredential>(
        "SELECT * FROM webauthn_credentials WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    if creds.is_empty() {
        return Err(AppError::Validation("No registered credentials".into()));
    }

    let challenge_bytes: [u8; 32] = rand::random();
    let challenge = base64_url::encode(&challenge_bytes);

    sqlx::query("UPDATE sessions SET webauthn_challenge = $1 WHERE id = $2")
        .bind(&challenge)
        .bind(session_id)
        .execute(pool)
        .await?;

    let allow_credentials: Vec<CredentialDescriptor> = creds
        .iter()
        .map(|c| {
            let transports: Vec<String> =
                serde_json::from_value(c.transports.clone()).unwrap_or_default();
            CredentialDescriptor {
                cred_type: "public-key".into(),
                id: c.credential_id.clone(),
                transports,
            }
        })
        .collect();

    Ok(AuthenticationOptionsResponse {
        challenge,
        timeout: CEREMONY_TIMEOUT_MS,
        rp_id: rp_id.to_string(),
        allow_credentials,
        user_verification: "preferred".into(),
    })
}

/// Verify the authentication response.
pub async fn finish_authentication(
    pool: &PgPool,
    user_id: Uuid,
    session_id: Uuid,
    rp_id: &str,
    response: AuthenticationResponse,
) -> Result<(), AppError> {
    let stored_challenge = get_and_clear_challenge(pool, session_id).await?;

    // Find the credential
    let cred = sqlx::query_as::<_, WebAuthnCredential>(
        "SELECT * FROM webauthn_credentials WHERE credential_id = $1 AND user_id = $2",
    )
    .bind(&response.id)
    .bind(user_id)
    .fetch_optional(pool)
    .await?
    .ok_or(AppError::Validation("Unknown credential".into()))?;

    // Decode client data JSON
    let client_data_bytes = base64_url::decode(&response.response.client_data_json)
        .map_err(|_| AppError::Validation("Invalid clientDataJSON".into()))?;

    let client_data: serde_json::Value = serde_json::from_slice(&client_data_bytes)
        .map_err(|_| AppError::Validation("Invalid clientDataJSON".into()))?;

    if client_data["type"].as_str().unwrap_or("") != "webauthn.get" {
        return Err(AppError::Validation("Invalid ceremony type".into()));
    }
    if client_data["challenge"].as_str().unwrap_or("") != stored_challenge {
        return Err(AppError::Validation("Challenge mismatch".into()));
    }

    // Decode authenticator data
    let auth_data = base64_url::decode(&response.response.authenticator_data)
        .map_err(|_| AppError::Validation("Invalid authenticator data".into()))?;

    if auth_data.len() < 37 {
        return Err(AppError::Validation("Auth data too short".into()));
    }

    // Verify RP ID hash
    let rp_id_hash = &auth_data[0..32];
    let expected_rp_hash = Sha256::digest(rp_id.as_bytes());
    if rp_id_hash != expected_rp_hash.as_slice() {
        return Err(AppError::Validation("RP ID hash mismatch".into()));
    }

    // Check user present flag
    if auth_data[32] & 0x01 == 0 {
        return Err(AppError::Validation("User not present".into()));
    }

    // Verify signature
    let client_data_hash = Sha256::digest(&client_data_bytes);
    let mut signed_data = auth_data.clone();
    signed_data.extend_from_slice(&client_data_hash);

    let signature_bytes = base64_url::decode(&response.response.signature)
        .map_err(|_| AppError::Validation("Invalid signature encoding".into()))?;

    let public_key_bytes = base64_url::decode(&cred.public_key)
        .map_err(|_| AppError::Validation("Invalid stored public key".into()))?;

    verify_es256_signature(&public_key_bytes, &signed_data, &signature_bytes)?;

    // Update sign count
    let new_sign_count = u32::from_be_bytes([
        auth_data[33], auth_data[34], auth_data[35], auth_data[36],
    ]) as i64;

    // Sign count check (if non-zero, must be greater than stored)
    if cred.sign_count > 0 && new_sign_count > 0 && new_sign_count <= cred.sign_count {
        tracing::warn!(
            credential_id = %cred.credential_id,
            stored = cred.sign_count,
            received = new_sign_count,
            "Possible cloned authenticator detected"
        );
        return Err(AppError::Validation("Possible cloned authenticator".into()));
    }

    sqlx::query(
        "UPDATE webauthn_credentials SET sign_count = $1, last_used_at = $2 WHERE id = $3",
    )
    .bind(new_sign_count)
    .bind(Utc::now())
    .bind(cred.id)
    .execute(pool)
    .await?;

    Ok(())
}

// ─── Credential management ──────────────────────────────────────────────────

pub async fn list_credentials(
    pool: &PgPool,
    user_id: Uuid,
) -> Result<Vec<WebAuthnCredentialView>, AppError> {
    let rows = sqlx::query_as::<_, WebAuthnCredential>(
        "SELECT * FROM webauthn_credentials WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| WebAuthnCredentialView {
            id: r.id,
            credential_id: r.credential_id,
            device_name: r.device_name,
            backed_up: r.backed_up,
            created_at: r.created_at,
            last_used_at: r.last_used_at,
        })
        .collect())
}

pub async fn delete_credential(
    pool: &PgPool,
    credential_id: Uuid,
    user_id: Uuid,
) -> Result<(), AppError> {
    let result = sqlx::query(
        "DELETE FROM webauthn_credentials WHERE id = $1 AND user_id = $2",
    )
    .bind(credential_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Credential not found".into()));
    }

    Ok(())
}

pub async fn rename_credential(
    pool: &PgPool,
    credential_id: Uuid,
    user_id: Uuid,
    name: &str,
) -> Result<(), AppError> {
    if name.is_empty() || name.len() > 100 {
        return Err(AppError::Validation(
            "Credential name must be 1-100 characters".into(),
        ));
    }

    let result = sqlx::query(
        "UPDATE webauthn_credentials SET device_name = $1 WHERE id = $2 AND user_id = $3",
    )
    .bind(name)
    .bind(credential_id)
    .bind(user_id)
    .execute(pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound("Credential not found".into()));
    }

    Ok(())
}

pub async fn has_credentials(pool: &PgPool, user_id: Uuid) -> Result<bool, AppError> {
    let row: Option<(i64,)> = sqlx::query_as(
        "SELECT COUNT(*) FROM webauthn_credentials WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_one(pool)
    .await
    .map(Some)?;

    Ok(row.map(|(c,)| c > 0).unwrap_or(false))
}

// ─── Helpers ────────────────────────────────────────────────────────────────

async fn get_and_clear_challenge(
    pool: &PgPool,
    session_id: Uuid,
) -> Result<String, AppError> {
    let row: Option<(Option<String>,)> = sqlx::query_as(
        "SELECT webauthn_challenge FROM sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    let challenge = row
        .and_then(|(c,)| c)
        .ok_or(AppError::Validation("No active WebAuthn challenge".into()))?;

    // Clear challenge (one-time use)
    sqlx::query("UPDATE sessions SET webauthn_challenge = NULL WHERE id = $1")
        .bind(session_id)
        .execute(pool)
        .await?;

    Ok(challenge)
}

/// Extract authData from a CBOR-encoded attestation object.
/// Minimal CBOR parser — only handles the "none" attestation format.
fn extract_auth_data_from_attestation(data: &[u8]) -> Result<Vec<u8>, AppError> {
    // Simple CBOR map parser for attestation object
    // Expected structure: map(3) { "fmt": "none", "attStmt": {}, "authData": bytes }
    // We just need to find the "authData" key and extract the bytes value.

    // Look for the "authData" string in CBOR
    // CBOR text string "authData" = 68 61 75 74 68 44 61 74 61 (length 8 = 0x68)
    let auth_data_marker = b"\x68authData";

    let pos = data
        .windows(auth_data_marker.len())
        .position(|w| w == auth_data_marker)
        .ok_or(AppError::Validation("authData not found in attestation".into()))?;

    let after = pos + auth_data_marker.len();
    if after >= data.len() {
        return Err(AppError::Validation("Truncated attestation object".into()));
    }

    // Next byte should be a CBOR byte string header
    let header = data[after];
    let (len, start) = if header >= 0x40 && header <= 0x57 {
        // Short byte string (length 0-23 encoded in the header)
        ((header - 0x40) as usize, after + 1)
    } else if header == 0x58 {
        // Byte string with 1-byte length
        if after + 1 >= data.len() {
            return Err(AppError::Validation("Truncated CBOR length".into()));
        }
        (data[after + 1] as usize, after + 2)
    } else if header == 0x59 {
        // Byte string with 2-byte length
        if after + 2 >= data.len() {
            return Err(AppError::Validation("Truncated CBOR length".into()));
        }
        (
            u16::from_be_bytes([data[after + 1], data[after + 2]]) as usize,
            after + 3,
        )
    } else {
        return Err(AppError::Validation(format!(
            "Unexpected CBOR type for authData: 0x{:02x}",
            header
        )));
    };

    if start + len > data.len() {
        return Err(AppError::Validation("authData extends beyond attestation object".into()));
    }

    Ok(data[start..start + len].to_vec())
}

/// Verify an ES256 (P-256/ECDSA with SHA-256) signature.
fn verify_es256_signature(
    cose_key_bytes: &[u8],
    signed_data: &[u8],
    signature: &[u8],
) -> Result<(), AppError> {
    use ecdsa::signature::Verifier;
    use p256::ecdsa::{Signature, VerifyingKey};

    // Extract x, y coordinates from COSE key (CBOR map).
    // COSE key for ES256 contains: kty(1)=2(EC2), alg(3)=-7, crv(-1)=1(P-256), x(-2)=bytes, y(-3)=bytes
    let (x, y) = extract_ec2_coords(cose_key_bytes)?;

    // Build uncompressed point: 0x04 || x || y
    let mut point = Vec::with_capacity(65);
    point.push(0x04);
    point.extend_from_slice(&x);
    point.extend_from_slice(&y);

    let verifying_key = VerifyingKey::from_sec1_bytes(&point)
        .map_err(|e| AppError::Validation(format!("Invalid EC public key: {e}")))?;

    let sig = Signature::from_der(signature)
        .map_err(|e| AppError::Validation(format!("Invalid DER signature: {e}")))?;

    // The verifier hashes signed_data with SHA-256 internally (ES256 = ECDSA + SHA-256)
    verifying_key
        .verify(signed_data, &sig)
        .map_err(|_| AppError::Validation("Signature verification failed".into()))
}

/// Extract x and y coordinates from a COSE EC2 key (CBOR-encoded).
fn extract_ec2_coords(data: &[u8]) -> Result<(Vec<u8>, Vec<u8>), AppError> {
    // Look for the x coordinate marker: CBOR negative integer -2 = 0x21, followed by byte string
    // Look for the y coordinate marker: CBOR negative integer -3 = 0x22, followed by byte string

    let x = extract_cbor_bytes_after_key(data, 0x21)
        .ok_or(AppError::Validation("Missing x coordinate in COSE key".into()))?;
    let y = extract_cbor_bytes_after_key(data, 0x22)
        .ok_or(AppError::Validation("Missing y coordinate in COSE key".into()))?;

    if x.len() != 32 || y.len() != 32 {
        return Err(AppError::Validation(format!(
            "Invalid coordinate lengths: x={}, y={}",
            x.len(),
            y.len()
        )));
    }

    Ok((x, y))
}

/// Find a CBOR negative integer key and extract the following byte string value.
fn extract_cbor_bytes_after_key(data: &[u8], key_byte: u8) -> Option<Vec<u8>> {
    for i in 0..data.len() {
        if data[i] == key_byte {
            let after = i + 1;
            if after >= data.len() {
                return None;
            }
            let header = data[after];
            if header == 0x58 && after + 1 < data.len() {
                // 1-byte length
                let len = data[after + 1] as usize;
                let start = after + 2;
                if start + len <= data.len() {
                    return Some(data[start..start + len].to_vec());
                }
            } else if header >= 0x40 && header <= 0x57 {
                // Short byte string
                let len = (header - 0x40) as usize;
                let start = after + 1;
                if start + len <= data.len() {
                    return Some(data[start..start + len].to_vec());
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_cbor_bytes() {
        // CBOR: 0x21 (key -2), 0x58 0x20 (byte string, 32 bytes), then 32 zero bytes
        let mut data = vec![0x21, 0x58, 0x20];
        data.extend_from_slice(&[0u8; 32]);
        let result = extract_cbor_bytes_after_key(&data, 0x21);
        assert!(result.is_some());
        assert_eq!(result.unwrap().len(), 32);
    }
}
