//! TOTP (Time-based One-Time Password) implementation per RFC 6238.
//!
//! Uses HMAC-SHA1 with 30-second time steps and 6-digit codes.

use hmac::{Hmac, Mac};
use sha1::Sha1;

use crate::error::CryptoError;

type HmacSha1 = Hmac<Sha1>;

/// TOTP parameters.
const TOTP_DIGITS: u32 = 6;
const TOTP_PERIOD: u64 = 30;
/// Allow codes from adjacent time steps (±1) to handle clock drift.
const TOTP_SKEW: u64 = 1;

/// Generate a random TOTP secret (20 bytes = 160 bits, standard for Google Authenticator).
pub fn generate_totp_secret() -> Result<Vec<u8>, CryptoError> {
    use rand::RngCore;
    let mut secret = vec![0u8; 20];
    rand::thread_rng()
        .try_fill_bytes(&mut secret)
        .map_err(|_| CryptoError::RandomGenerationFailed)?;
    Ok(secret)
}

/// Encode a TOTP secret as base32 (for QR codes and user display).
pub fn encode_secret_base32(secret: &[u8]) -> String {
    data_encoding::BASE32_NOPAD.encode(secret)
}

/// Decode a base32-encoded TOTP secret.
pub fn decode_secret_base32(encoded: &str) -> Result<Vec<u8>, CryptoError> {
    data_encoding::BASE32_NOPAD
        .decode(encoded.to_uppercase().as_bytes())
        .map_err(|_| CryptoError::InvalidKeyFormat)
}

/// Generate a TOTP code for the given secret and Unix timestamp.
pub fn generate_totp(secret: &[u8], timestamp: u64) -> Result<String, CryptoError> {
    let counter = timestamp / TOTP_PERIOD;
    let code = hotp(secret, counter)?;
    Ok(format!("{:0>width$}", code % 10u32.pow(TOTP_DIGITS), width = TOTP_DIGITS as usize))
}

/// Verify a TOTP code against the current time, allowing ±skew time steps.
pub fn verify_totp(secret: &[u8], code: &str, timestamp: u64) -> Result<bool, CryptoError> {
    if code.len() != TOTP_DIGITS as usize || !code.chars().all(|c| c.is_ascii_digit()) {
        return Ok(false);
    }

    let counter = timestamp / TOTP_PERIOD;

    for offset in 0..=TOTP_SKEW {
        // Check current and past time steps
        if let Ok(expected) = generate_totp(secret, (counter - offset) * TOTP_PERIOD) {
            if constant_time_eq(code.as_bytes(), expected.as_bytes()) {
                return Ok(true);
            }
        }
        // Check future time steps (except offset 0 which we already checked)
        if offset > 0 {
            if let Ok(expected) = generate_totp(secret, (counter + offset) * TOTP_PERIOD) {
                if constant_time_eq(code.as_bytes(), expected.as_bytes()) {
                    return Ok(true);
                }
            }
        }
    }

    Ok(false)
}

/// Build an otpauth:// URI for QR code generation.
///
/// Format: otpauth://totp/{issuer}:{account}?secret={base32}&issuer={issuer}&digits=6&period=30
pub fn build_otpauth_uri(secret_base32: &str, account: &str, issuer: &str) -> String {
    let issuer_encoded = urlencod(issuer);
    let account_encoded = urlencod(account);
    format!(
        "otpauth://totp/{issuer_encoded}:{account_encoded}?secret={secret_base32}&issuer={issuer_encoded}&digits={TOTP_DIGITS}&period={TOTP_PERIOD}"
    )
}

/// Generate recovery codes (8 codes, 8 chars each, alphanumeric).
pub fn generate_recovery_codes(count: usize) -> Result<Vec<String>, CryptoError> {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let charset = b"ABCDEFGHJKLMNPQRSTUVWXYZ23456789"; // No 0/O/1/I to avoid confusion

    let mut codes = Vec::with_capacity(count);
    for _ in 0..count {
        let code: String = (0..8)
            .map(|_| {
                let idx = rng.gen_range(0..charset.len());
                charset[idx] as char
            })
            .collect();
        codes.push(format!("{}-{}", &code[..4], &code[4..]));
    }
    Ok(codes)
}

// ─── Internal helpers ───

/// HOTP (HMAC-based One-Time Password) per RFC 4226.
fn hotp(secret: &[u8], counter: u64) -> Result<u32, CryptoError> {
    let mut mac =
        HmacSha1::new_from_slice(secret).map_err(|_| CryptoError::InvalidKeyFormat)?;

    mac.update(&counter.to_be_bytes());
    let result = mac.finalize().into_bytes();

    // Dynamic truncation
    let offset = (result[19] & 0x0f) as usize;
    let code = u32::from_be_bytes([
        result[offset] & 0x7f,
        result[offset + 1],
        result[offset + 2],
        result[offset + 3],
    ]);

    Ok(code)
}

/// Constant-time comparison to prevent timing attacks.
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// Minimal URL encoding for otpauth URI components.
fn urlencod(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => result.push(ch),
            ' ' => result.push_str("%20"),
            ':' => result.push_str("%3A"),
            '/' => result.push_str("%2F"),
            '@' => result.push_str("%40"),
            _ => {
                for byte in ch.to_string().as_bytes() {
                    result.push_str(&format!("%{byte:02X}"));
                }
            }
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_totp_generation() {
        // RFC 6238 test vector: secret = "12345678901234567890" (ASCII)
        let secret = b"12345678901234567890";
        let code = generate_totp(secret, 59).unwrap();
        assert_eq!(code.len(), 6);

        // Verify the code we just generated
        assert!(verify_totp(secret, &code, 59).unwrap());

        // Wrong code should fail
        assert!(!verify_totp(secret, "000000", 59).unwrap());
    }

    #[test]
    fn test_base32_roundtrip() {
        let secret = generate_totp_secret().unwrap();
        let encoded = encode_secret_base32(&secret);
        let decoded = decode_secret_base32(&encoded).unwrap();
        assert_eq!(secret, decoded);
    }

    #[test]
    fn test_recovery_codes() {
        let codes = generate_recovery_codes(8).unwrap();
        assert_eq!(codes.len(), 8);
        for code in &codes {
            assert_eq!(code.len(), 9); // 4 + '-' + 4
            assert!(code.contains('-'));
        }
    }

    #[test]
    fn test_otpauth_uri() {
        let uri = build_otpauth_uri("JBSWY3DPEHPK3PXP", "user@example.com", "Lockso");
        assert!(uri.starts_with("otpauth://totp/"));
        assert!(uri.contains("secret=JBSWY3DPEHPK3PXP"));
        assert!(uri.contains("issuer=Lockso"));
    }
}
