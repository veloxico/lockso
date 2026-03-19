//! Lockso cryptographic primitives.
//!
//! - Argon2id password hashing
//! - AES-256-GCM symmetric encryption (server-side)
//! - XChaCha20-Poly1305 (client-side encryption support)
//! - RSA key pair generation and management
//! - SHA-512 blind search hashing
//! - Secure random generation (CSPRNG)

pub mod aes_gcm;
pub mod argon2;
pub mod chacha20;
pub mod error;
pub mod random;
pub mod rsa_keys;
pub mod search_hash;
pub mod totp;

pub use error::CryptoError;
