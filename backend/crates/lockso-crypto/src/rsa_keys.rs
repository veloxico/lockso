use rsa::{
    RsaPrivateKey, RsaPublicKey,
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey, EncodeRsaPrivateKey, EncodeRsaPublicKey},
};
use rand_core::OsRng;

use crate::CryptoError;

/// RSA key pair for client-side encryption (CSE).
///
/// Each user has an RSA key pair:
/// - Public key: stored in plaintext in DB, used to encrypt vault master keys
/// - Private key: encrypted with user's master password, stored in DB
pub struct RsaKeyPair {
    pub public_key_pem: String,
    pub private_key_pem: String,
}

/// Generate a new RSA key pair.
///
/// Key size should be 2048 or 4096 bits.
/// 4096 is recommended for long-term security.
pub fn generate_keypair(bits: usize) -> Result<RsaKeyPair, CryptoError> {
    if bits != 2048 && bits != 4096 {
        return Err(CryptoError::KeyGenerationFailed);
    }

    let private_key =
        RsaPrivateKey::new(&mut OsRng, bits).map_err(|_| CryptoError::KeyGenerationFailed)?;

    let public_key = RsaPublicKey::from(&private_key);

    let private_key_pem = private_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .map_err(|_| CryptoError::KeyGenerationFailed)?
        .to_string();

    let public_key_pem = public_key
        .to_pkcs1_pem(rsa::pkcs1::LineEnding::LF)
        .map_err(|_| CryptoError::KeyGenerationFailed)?;

    Ok(RsaKeyPair {
        public_key_pem,
        private_key_pem,
    })
}

/// Validate that a PEM-encoded RSA public key is well-formed.
pub fn validate_public_key(pem: &str) -> Result<(), CryptoError> {
    RsaPublicKey::from_pkcs1_pem(pem).map_err(|_| CryptoError::InvalidKeyFormat)?;
    Ok(())
}

/// Validate that a PEM-encoded RSA private key is well-formed.
pub fn validate_private_key(pem: &str) -> Result<(), CryptoError> {
    RsaPrivateKey::from_pkcs1_pem(pem).map_err(|_| CryptoError::InvalidKeyFormat)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair_2048() {
        let kp = generate_keypair(2048).unwrap();
        assert!(kp.public_key_pem.contains("BEGIN RSA PUBLIC KEY"));
        assert!(kp.private_key_pem.contains("BEGIN RSA PRIVATE KEY"));
    }

    // 4096-bit key generation is too slow for unit tests
    // #[test]
    // fn test_generate_keypair_4096() { ... }

    #[test]
    fn test_invalid_key_size() {
        assert!(generate_keypair(1024).is_err());
        assert!(generate_keypair(3072).is_err());
    }

    #[test]
    fn test_validate_keys() {
        let kp = generate_keypair(2048).unwrap();
        assert!(validate_public_key(&kp.public_key_pem).is_ok());
        assert!(validate_private_key(&kp.private_key_pem).is_ok());
    }

    #[test]
    fn test_validate_invalid_pem() {
        assert!(validate_public_key("not a pem").is_err());
        assert!(validate_private_key("not a pem").is_err());
    }
}
