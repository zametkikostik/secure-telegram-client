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
pub mod hybrid;
pub mod double_ratchet;
pub mod steganography;

// Re-export main types
pub use hybrid::{
    HybridKeypair,
    PublicBundle,
    HybridCiphertext,
    CryptoError,
    encrypt,
    decrypt,
    sign,
    verify_signature,
};

pub use double_ratchet::{
    DoubleRatchetSession,
    RatchetSession,
    RatchetCiphertext,
    RatchetError,
};

pub use steganography::{
    hide,
    extract,
    capacity_bits,
    ImageRgb8,
    StegoError,
};
