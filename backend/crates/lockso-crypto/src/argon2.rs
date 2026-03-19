use argon2::{
    Algorithm, Argon2, Params, Version,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand_core::OsRng;

use crate::CryptoError;

/// Argon2id configuration parameters.
///
/// Defaults match OWASP 2024 recommendations for password hashing:
/// - Memory: 19 MiB (19456 KiB)
/// - Iterations: 2
/// - Parallelism: 1
/// - Output length: 32 bytes
#[derive(Debug, Clone)]
pub struct Argon2Config {
    pub memory_kib: u32,
    pub iterations: u32,
    pub parallelism: u32,
    pub output_len: usize,
}

impl Default for Argon2Config {
    fn default() -> Self {
        Self {
            memory_kib: 19456,
            iterations: 2,
            parallelism: 1,
            output_len: 32,
        }
    }
}

/// Hash a password using Argon2id.
///
/// Returns a PHC-formatted string (e.g., `$argon2id$v=19$m=19456,t=2,p=1$...`).
/// The salt is randomly generated and embedded in the output.
pub fn hash_password(password: &str, config: &Argon2Config) -> Result<String, CryptoError> {
    let params = Params::new(
        config.memory_kib,
        config.iterations,
        config.parallelism,
        Some(config.output_len),
    )
    .map_err(|_| CryptoError::HashingFailed)?;

    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let salt = SaltString::generate(&mut OsRng);

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .map_err(|_| CryptoError::HashingFailed)?;

    Ok(hash.to_string())
}

/// Verify a password against an Argon2id hash (PHC format).
///
/// Performs constant-time comparison to prevent timing attacks.
pub fn verify_password(password: &str, hash: &str) -> Result<bool, CryptoError> {
    let parsed = PasswordHash::new(hash).map_err(|_| CryptoError::VerificationFailed)?;

    // Extract params from the hash to use the same config for verification
    let argon2 = Argon2::default();

    match argon2.verify_password(password.as_bytes(), &parsed) {
        Ok(()) => Ok(true),
        Err(argon2::password_hash::Error::Password) => Ok(false),
        Err(_) => Err(CryptoError::VerificationFailed),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_and_verify() {
        let config = Argon2Config::default();
        let password = "correct_horse_battery_staple";

        let hash = hash_password(password, &config).unwrap();
        assert!(hash.starts_with("$argon2id$"));

        assert!(verify_password(password, &hash).unwrap());
        assert!(!verify_password("wrong_password", &hash).unwrap());
    }

    #[test]
    fn test_unique_salts() {
        let config = Argon2Config::default();
        let password = "same_password";

        let hash1 = hash_password(password, &config).unwrap();
        let hash2 = hash_password(password, &config).unwrap();

        // Same password produces different hashes (different salts)
        assert_ne!(hash1, hash2);
        // But both verify correctly
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }

    #[test]
    fn test_empty_password() {
        let config = Argon2Config::default();
        let hash = hash_password("", &config).unwrap();
        assert!(verify_password("", &hash).unwrap());
        assert!(!verify_password("not_empty", &hash).unwrap());
    }
}
