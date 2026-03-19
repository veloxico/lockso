use chacha20poly1305::{XChaCha20Poly1305, XNonce, aead::{Aead, KeyInit}};

use crate::{CryptoError, random::secure_random_bytes};

/// XChaCha20-Poly1305 key size in bytes.
pub const KEY_SIZE: usize = 32;
/// XChaCha20-Poly1305 nonce size in bytes (extended nonce).
pub const NONCE_SIZE: usize = 24;

/// Encrypt plaintext using XChaCha20-Poly1305.
///
/// Output format: `[24-byte nonce][ciphertext+tag]`
///
/// This is used for client-side encryption (CSE) operations.
/// XChaCha20-Poly1305 is preferred over AES-GCM for CSE because:
/// - 24-byte nonce eliminates nonce reuse risk even at scale
/// - Software-only (no AES-NI dependency on client devices)
/// - IETF standard (RFC 8439 extended)
pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if key.len() != KEY_SIZE {
        return Err(CryptoError::InvalidKeyLength);
    }

    let cipher =
        XChaCha20Poly1305::new_from_slice(key).map_err(|_| CryptoError::InvalidKeyLength)?;

    let nonce_bytes = secure_random_bytes(NONCE_SIZE)?;
    let nonce = XNonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| CryptoError::EncryptionFailed)?;

    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypt ciphertext using XChaCha20-Poly1305.
///
/// Expects input format: `[24-byte nonce][ciphertext+tag]`
pub fn decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if key.len() != KEY_SIZE {
        return Err(CryptoError::InvalidKeyLength);
    }

    if ciphertext.len() < NONCE_SIZE + 16 {
        return Err(CryptoError::DecryptionFailed);
    }

    let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_SIZE);

    let cipher =
        XChaCha20Poly1305::new_from_slice(key).map_err(|_| CryptoError::InvalidKeyLength)?;
    let nonce = XNonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, encrypted)
        .map_err(|_| CryptoError::DecryptionFailed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Vec<u8> {
        vec![0x55u8; KEY_SIZE]
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"client-side encrypted data";

        let ciphertext = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_nonce_size_in_output() {
        let key = test_key();
        let plaintext = b"test";

        let ciphertext = encrypt(&key, plaintext).unwrap();
        // nonce (24) + plaintext (4) + tag (16) = 44
        assert_eq!(ciphertext.len(), NONCE_SIZE + plaintext.len() + 16);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key = test_key();
        let wrong_key = vec![0xAAu8; KEY_SIZE];
        let plaintext = b"secret";

        let ciphertext = encrypt(&key, plaintext).unwrap();
        assert!(decrypt(&wrong_key, &ciphertext).is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = test_key();
        let plaintext = b"integrity";

        let mut ciphertext = encrypt(&key, plaintext).unwrap();
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0xFF;

        assert!(decrypt(&key, &ciphertext).is_err());
    }
}
