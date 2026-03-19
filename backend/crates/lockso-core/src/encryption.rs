//! Server-side encryption helpers.
//!
//! All sensitive item fields are encrypted with AES-256-GCM using a
//! server-wide encryption key before storage. The key is loaded from
//! the `LOCKSO_ENCRYPTION_KEY` environment variable (64 hex chars = 32 bytes).
//!
//! Encrypted fields are stored as base64 in the database.

use base64ct::{Base64, Encoding};
use lockso_crypto::aes_gcm;

use crate::error::AppError;

/// Encrypt a plaintext string and return base64-encoded ciphertext.
/// Returns empty string for empty input (no encryption needed).
pub fn encrypt_field(key: &[u8], plaintext: &str) -> Result<String, AppError> {
    if plaintext.is_empty() {
        return Ok(String::new());
    }
    let ciphertext = aes_gcm::encrypt(key, plaintext.as_bytes())
        .map_err(|_| AppError::Internal("encryption failed".into()))?;
    Ok(Base64::encode_string(&ciphertext))
}

/// Decrypt a base64-encoded ciphertext and return plaintext string.
/// Returns empty string for empty input.
pub fn decrypt_field(key: &[u8], ciphertext_b64: &str) -> Result<String, AppError> {
    if ciphertext_b64.is_empty() {
        return Ok(String::new());
    }
    let ciphertext = Base64::decode_vec(ciphertext_b64)
        .map_err(|_| AppError::Internal("invalid base64 in encrypted field".into()))?;
    let plaintext = aes_gcm::decrypt(key, &ciphertext)
        .map_err(|_| AppError::Internal("decryption failed".into()))?;
    String::from_utf8(plaintext)
        .map_err(|_| AppError::Internal("decrypted data is not valid UTF-8".into()))
}

/// Load the server encryption key from environment.
/// Must be exactly 64 hex characters (32 bytes).
pub fn load_encryption_key() -> Result<Vec<u8>, String> {
    let hex_key = std::env::var("LOCKSO_ENCRYPTION_KEY").map_err(|_| {
        "LOCKSO_ENCRYPTION_KEY environment variable is required".to_string()
    })?;

    if hex_key.len() != 64 {
        return Err(format!(
            "LOCKSO_ENCRYPTION_KEY must be exactly 64 hex chars (32 bytes), got {}",
            hex_key.len()
        ));
    }

    let key = hex::decode(&hex_key).map_err(|_| {
        "LOCKSO_ENCRYPTION_KEY must be valid hex".to_string()
    })?;

    // Validate key by performing a round-trip test
    aes_gcm::validate_key(&key).map_err(|_| {
        "LOCKSO_ENCRYPTION_KEY validation failed".to_string()
    })?;

    Ok(key)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Vec<u8> {
        vec![0x42u8; 32]
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = "hello, lockso! Привет!";
        let encrypted = encrypt_field(&key, plaintext).unwrap();
        assert_ne!(encrypted, plaintext);
        let decrypted = decrypt_field(&key, &encrypted).unwrap();
        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_empty_field() {
        let key = test_key();
        assert_eq!(encrypt_field(&key, "").unwrap(), "");
        assert_eq!(decrypt_field(&key, "").unwrap(), "");
    }

    #[test]
    fn test_different_encryptions() {
        let key = test_key();
        let e1 = encrypt_field(&key, "same").unwrap();
        let e2 = encrypt_field(&key, "same").unwrap();
        // Different nonces = different ciphertext
        assert_ne!(e1, e2);
        // But both decrypt to same
        assert_eq!(decrypt_field(&key, &e1).unwrap(), "same");
        assert_eq!(decrypt_field(&key, &e2).unwrap(), "same");
    }
}
