use sha2::{Digest, Sha512};

/// Generate a blind search hash for an encrypted field value.
///
/// Uses SHA-512 with a per-vault salt, truncated to 20 chars of base64.
/// This allows searching encrypted data without decryption:
/// 1. Client sends `sha512(value + vault_salt)` truncated
/// 2. Server compares against stored search hashes
/// 3. Server never sees plaintext
///
/// The truncation to 20 chars provides ~120 bits of entropy,
/// sufficient for search while keeping index size manageable.
pub fn blind_search_hash(value: &str, salt: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.update(value.as_bytes());
    hasher.update(salt.as_bytes());
    let hash = hasher.finalize();

    use base64ct::{Base64UrlUnpadded, Encoding};
    let full = Base64UrlUnpadded::encode_string(&hash);
    full[..20].to_string()
}

/// Hash a token (access/refresh) for secure storage.
///
/// We store SHA-512 hashes of tokens, never the raw tokens themselves.
/// This way, even if the database is compromised, tokens cannot be extracted.
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha512::new();
    hasher.update(token.as_bytes());
    let hash = hasher.finalize();
    hex::encode(hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blind_search_hash_deterministic() {
        let h1 = blind_search_hash("password123", "vault_salt_abc");
        let h2 = blind_search_hash("password123", "vault_salt_abc");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_blind_search_hash_length() {
        let h = blind_search_hash("test", "salt");
        assert_eq!(h.len(), 20);
    }

    #[test]
    fn test_different_salts_different_hashes() {
        let h1 = blind_search_hash("same_value", "salt_a");
        let h2 = blind_search_hash("same_value", "salt_b");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_different_values_different_hashes() {
        let h1 = blind_search_hash("value_a", "same_salt");
        let h2 = blind_search_hash("value_b", "same_salt");
        assert_ne!(h1, h2);
    }

    #[test]
    fn test_hash_token() {
        let token = "abc123def456";
        let hash = hash_token(token);
        // SHA-512 produces 128 hex chars
        assert_eq!(hash.len(), 128);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_token_deterministic() {
        let h1 = hash_token("same_token");
        let h2 = hash_token("same_token");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_hash_token_different() {
        let h1 = hash_token("token_a");
        let h2 = hash_token("token_b");
        assert_ne!(h1, h2);
    }
}
