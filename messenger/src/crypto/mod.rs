//! Cryptographic module for Secure Telegram Client
//!
//! Provides:
//! - Hybrid post-quantum encryption (X25519 + Kyber1024 + ChaCha20-Poly1305)
//! - Double Ratchet for forward secrecy
//! - LSB steganography for plausible deniability
//! - Ed25519 digital signatures
//! - Argon2id password hashing
//!
//! SECURITY: требует аудита перед production
//! TODO: pentest перед release

pub mod constants;
pub mod double_ratchet;
pub mod hybrid;
pub mod steganography;

// Re-export main types
pub use hybrid::{
    decrypt, encrypt, sign, verify_signature, CryptoError, HybridCiphertext, HybridKeypair,
    PublicBundle,
};

pub use double_ratchet::{DoubleRatchetSession, RatchetCiphertext, RatchetError, RatchetSession};

pub use steganography::{capacity_bits, extract, hide, ImageRgb8, StegoError};
