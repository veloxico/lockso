use rand_core::OsRng;
use rand::RngCore;

use crate::CryptoError;

/// Generate cryptographically secure random bytes.
pub fn secure_random_bytes(len: usize) -> Result<Vec<u8>, CryptoError> {
    let mut buf = vec![0u8; len];
    OsRng.fill_bytes(&mut buf);
    Ok(buf)
}

/// Generate a cryptographically secure random hex string.
pub fn secure_random_hex(byte_len: usize) -> Result<String, CryptoError> {
    let bytes = secure_random_bytes(byte_len)?;
    Ok(hex::encode(bytes))
}

/// Generate a cryptographically secure random base64 string (URL-safe, no padding).
pub fn secure_random_base64(byte_len: usize) -> Result<String, CryptoError> {
    use base64ct::{Base64UrlUnpadded, Encoding};
    let bytes = secure_random_bytes(byte_len)?;
    Ok(Base64UrlUnpadded::encode_string(&bytes))
}

/// Generate a secure access/refresh token (32 bytes → 64 hex chars).
pub fn generate_token() -> Result<String, CryptoError> {
    secure_random_hex(32)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_random_bytes_length() {
        let bytes = secure_random_bytes(32).unwrap();
        assert_eq!(bytes.len(), 32);
    }

    #[test]
    fn test_secure_random_bytes_uniqueness() {
        let a = secure_random_bytes(32).unwrap();
        let b = secure_random_bytes(32).unwrap();
        assert_ne!(a, b);
    }

    #[test]
    fn test_generate_token_length() {
        let token = generate_token().unwrap();
        assert_eq!(token.len(), 64); // 32 bytes = 64 hex chars
    }

    #[test]
    fn test_secure_random_hex() {
        let hex = secure_random_hex(16).unwrap();
        assert_eq!(hex.len(), 32);
        assert!(hex.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_secure_random_base64() {
        let b64 = secure_random_base64(32).unwrap();
        assert!(!b64.is_empty());
        // URL-safe base64 contains only these chars
        assert!(b64.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }
}
