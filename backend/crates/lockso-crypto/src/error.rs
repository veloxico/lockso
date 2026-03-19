/// Cryptographic operation errors.
///
/// These errors are intentionally vague in production to prevent
/// information leakage (e.g., timing attacks, oracle attacks).
#[derive(Debug, thiserror::Error)]
pub enum CryptoError {
    #[error("encryption failed")]
    EncryptionFailed,

    #[error("decryption failed")]
    DecryptionFailed,

    #[error("invalid key length")]
    InvalidKeyLength,

    #[error("invalid nonce length")]
    InvalidNonceLength,

    #[error("password hashing failed")]
    HashingFailed,

    #[error("password verification failed")]
    VerificationFailed,

    #[error("key generation failed")]
    KeyGenerationFailed,

    #[error("invalid key format")]
    InvalidKeyFormat,

    #[error("random generation failed")]
    RandomGenerationFailed,
}
