use aes_gcm::{
    Aes256Gcm, Key, Nonce,
    aead::{Aead, KeyInit},
};

use crate::{CryptoError, random::secure_random_bytes};

/// AES-256-GCM key size in bytes.
pub const KEY_SIZE: usize = 32;
/// AES-256-GCM nonce size in bytes.
pub const NONCE_SIZE: usize = 12;

/// Encrypt plaintext using AES-256-GCM.
///
/// Output format: `[12-byte nonce][ciphertext+tag]`
///
/// The nonce is prepended to the ciphertext so that it can be
/// extracted during decryption. A fresh random nonce is generated
/// for each encryption operation.
pub fn encrypt(key: &[u8], plaintext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if key.len() != KEY_SIZE {
        return Err(CryptoError::InvalidKeyLength);
    }

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);

    let nonce_bytes = secure_random_bytes(NONCE_SIZE)?;
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|_| CryptoError::EncryptionFailed)?;

    // Prepend nonce to ciphertext
    let mut output = Vec::with_capacity(NONCE_SIZE + ciphertext.len());
    output.extend_from_slice(&nonce_bytes);
    output.extend_from_slice(&ciphertext);

    Ok(output)
}

/// Decrypt ciphertext using AES-256-GCM.
///
/// Expects input format: `[12-byte nonce][ciphertext+tag]`
pub fn decrypt(key: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>, CryptoError> {
    if key.len() != KEY_SIZE {
        return Err(CryptoError::InvalidKeyLength);
    }

    if ciphertext.len() < NONCE_SIZE + 16 {
        // Minimum: nonce (12) + tag (16), empty plaintext
        return Err(CryptoError::DecryptionFailed);
    }

    let (nonce_bytes, encrypted) = ciphertext.split_at(NONCE_SIZE);

    let key = Key::<Aes256Gcm>::from_slice(key);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(nonce_bytes);

    cipher
        .decrypt(nonce, encrypted)
        .map_err(|_| CryptoError::DecryptionFailed)
}

/// Validate an encryption key by performing a round-trip encrypt/decrypt.
///
/// This is used at startup to verify the server encryption key is valid.
pub fn validate_key(key: &[u8]) -> Result<(), CryptoError> {
    let test_data = b"lockso_key_validation_test";
    let encrypted = encrypt(key, test_data)?;
    let decrypted = decrypt(key, &encrypted)?;
    if decrypted != test_data {
        return Err(CryptoError::InvalidKeyFormat);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_key() -> Vec<u8> {
        vec![0x42u8; KEY_SIZE]
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        let key = test_key();
        let plaintext = b"hello, lockso!";

        let ciphertext = encrypt(&key, plaintext).unwrap();
        let decrypted = decrypt(&key, &ciphertext).unwrap();

        assert_eq!(decrypted, plaintext);
    }

    #[test]
    fn test_ciphertext_format() {
        let key = test_key();
        let plaintext = b"test";

        let ciphertext = encrypt(&key, plaintext).unwrap();
        // nonce (12) + plaintext (4) + tag (16) = 32
        assert_eq!(ciphertext.len(), NONCE_SIZE + plaintext.len() + 16);
    }

    #[test]
    fn test_different_nonces() {
        let key = test_key();
        let plaintext = b"same data";

        let c1 = encrypt(&key, plaintext).unwrap();
        let c2 = encrypt(&key, plaintext).unwrap();

        // Different nonces produce different ciphertexts
        assert_ne!(c1, c2);
        // But both decrypt to the same plaintext
        assert_eq!(decrypt(&key, &c1).unwrap(), plaintext);
        assert_eq!(decrypt(&key, &c2).unwrap(), plaintext);
    }

    #[test]
    fn test_wrong_key_fails() {
        let key = test_key();
        let wrong_key = vec![0x13u8; KEY_SIZE];
        let plaintext = b"secret";

        let ciphertext = encrypt(&key, plaintext).unwrap();
        assert!(decrypt(&wrong_key, &ciphertext).is_err());
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let key = test_key();
        let plaintext = b"integrity test";

        let mut ciphertext = encrypt(&key, plaintext).unwrap();
        // Flip a byte in the ciphertext (after nonce)
        let last = ciphertext.len() - 1;
        ciphertext[last] ^= 0xFF;

        assert!(decrypt(&key, &ciphertext).is_err());
    }

    #[test]
    fn test_invalid_key_length() {
        let short_key = vec![0u8; 16];
        assert!(encrypt(&short_key, b"test").is_err());
        assert!(decrypt(&short_key, &[0u8; 40]).is_err());
    }

    #[test]
    fn test_empty_plaintext() {
        let key = test_key();
        let ciphertext = encrypt(&key, b"").unwrap();
        let decrypted = decrypt(&key, &ciphertext).unwrap();
        assert!(decrypted.is_empty());
    }

    #[test]
    fn test_validate_key() {
        let key = test_key();
        assert!(validate_key(&key).is_ok());
        assert!(validate_key(&[0u8; 16]).is_err()); // wrong length
    }
}
