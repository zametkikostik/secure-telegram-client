//! Authentication module
//!
//! Provides:
//! - OS keychain integration for secure key storage
//! - Argon2id password hashing
//! - Secure password buffer with zeroize
//!
//! SECURITY: требует аудита перед production
//! TODO: pentest перед release

pub mod keychain;

pub use keychain::{Keychain, KeychainError, SecurePassword};
