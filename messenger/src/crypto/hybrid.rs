//! Hybrid Post-Quantum Encryption
//!
//! Combines X25519 (classical ECDH) + Kyber1024 (post-quantum KEM)
//! for defense-in-depth against both classical and quantum adversaries.
//!
//! # Security Properties
//! - **Classical security**: X25519 ECDH provides 128-bit security
//! - **Post-quantum security**: Kyber1024 provides NIST Level 5 PQC security
//! - **Authentication**: Ed25519 signatures for ciphertext integrity
//! - **Key derivation**: HKDF-SHA3-256 for secure key expansion
//!
//! SECURITY: требует аудита перед production
//! TODO: pentest перед release

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use oqs::kem::{Algorithm, Ciphertext, Kem, PublicKey, SecretKey};
use rand::rngs::OsRng;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};

use crate::crypto::constants::{
    CHACHA20_KEY_SIZE, CHACHA20_NONCE_SIZE, ED25519_SIGNATURE_SIZE, HKDF_CHACHA20_INFO,
    X25519_KEY_SIZE,
};

// ============================================================================
// Error Types
// ============================================================================

/// Crypto error types
#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key generation failed: {0}")]
    KeyGenFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid signature")]
    InvalidSignature,

    #[error("Invalid key length: expected {expected}, got {actual}")]
    InvalidKeyLength { expected: usize, actual: usize },

    #[error("OQS error: {0}")]
    OqsError(String),
}

// ============================================================================
// Keypair Structure
// ============================================================================

/// Hybrid keypair: X25519 + Kyber1024 + Ed25519
///
/// SECURITY: приватные ключи НИКОГДА не покидают устройство
pub struct HybridKeypair {
    /// X25519 static secret (ECDH)
    pub x25519_secret: StaticSecret,
    /// X25519 public key
    pub x25519_public: X25519PublicKey,
    /// Kyber1024 secret key (PQC KEM)
    pub kyber_secret: SecretKey,
    /// Kyber1024 public key
    pub kyber_public: PublicKey,
    /// Ed25519 signing key
    pub ed25519_secret: SigningKey,
    /// Ed25519 verifying key
    pub ed25519_public: VerifyingKey,
}

impl HybridKeypair {
    /// Generate a new hybrid keypair
    ///
    /// # Returns
    /// - `HybridKeypair` with X25519 + Kyber1024 + Ed25519 keys
    /// - `CryptoError` if key generation fails
    pub fn generate() -> Result<Self, CryptoError> {
        // X25519 keypair (classical ECDH)
        let x25519_secret = StaticSecret::random_from_rng(OsRng);
        let x25519_public = X25519PublicKey::from(&x25519_secret);

        // Kyber1024 keypair (post-quantum KEM)
        oqs::init();
        let kyber_kem =
            Kem::new(Algorithm::Kyber1024).map_err(|e| CryptoError::KeyGenFailed(e.to_string()))?;
        let (kyber_public, kyber_secret) = kyber_kem
            .keypair()
            .map_err(|e| CryptoError::KeyGenFailed(e.to_string()))?;

        // Ed25519 keypair (signatures)
        let ed25519_secret = SigningKey::generate(&mut OsRng);
        let ed25519_public = VerifyingKey::from(&ed25519_secret);

        tracing::info!("Generated new hybrid keypair (X25519 + Kyber1024 + Ed25519)");

        Ok(HybridKeypair {
            x25519_secret,
            x25519_public,
            kyber_secret,
            kyber_public,
            ed25519_secret,
            ed25519_public,
        })
    }

    /// Get the public bundle for sharing with contacts
    pub fn public_bundle(&self) -> PublicBundle {
        PublicBundle {
            x25519_public: self.x25519_public.to_bytes(),
            kyber_public: self.kyber_public.as_ref().to_vec(),
            ed25519_public: self.ed25519_public.to_bytes(),
        }
    }
}

/// Public key bundle for exchange with contacts
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PublicBundle {
    /// X25519 public key (32 bytes)
    pub x25519_public: [u8; X25519_KEY_SIZE],
    /// Kyber1024 public key
    pub kyber_public: Vec<u8>,
    /// Ed25519 verifying key (32 bytes)
    pub ed25519_public: [u8; ED25519_SIGNATURE_SIZE / 2],
}

impl PublicBundle {
    /// Serialize to bytes for network transmission
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.x25519_public);
        bytes.extend_from_slice(&(self.kyber_public.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.kyber_public);
        bytes.extend_from_slice(&self.ed25519_public);
        bytes
    }

    /// Deserialize from bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        let min_len = X25519_KEY_SIZE + 4 + ED25519_SIGNATURE_SIZE / 2;
        if bytes.len() < min_len {
            return Err(CryptoError::InvalidKeyLength {
                expected: min_len,
                actual: bytes.len(),
            });
        }

        let mut x25519_public = [0u8; X25519_KEY_SIZE];
        x25519_public.copy_from_slice(&bytes[0..X25519_KEY_SIZE]);

        let kyber_len = u32::from_le_bytes(
            bytes[X25519_KEY_SIZE..X25519_KEY_SIZE + 4]
                .try_into()
                .unwrap(),
        ) as usize;
        let kyber_start = X25519_KEY_SIZE + 4;
        let kyber_public = bytes[kyber_start..kyber_start + kyber_len].to_vec();

        let ed_start = kyber_start + kyber_len;
        let mut ed25519_public = [0u8; ED25519_SIGNATURE_SIZE / 2];
        ed25519_public.copy_from_slice(&bytes[ed_start..ed_start + ED25519_SIGNATURE_SIZE / 2]);

        Ok(PublicBundle {
            x25519_public,
            kyber_public,
            ed25519_public,
        })
    }
}

// ============================================================================
// Ciphertext Structure
// ============================================================================

/// Encrypted message structure
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct HybridCiphertext {
    /// Ephemeral X25519 public key
    pub ephemeral_x25519: [u8; X25519_KEY_SIZE],
    /// Kyber1024 ciphertext
    pub kyber_ciphertext: Ciphertext,
    /// ChaCha20 nonce (12 bytes)
    pub nonce: Vec<u8>,
    /// ChaCha20-Poly1305 ciphertext
    pub ciphertext: Vec<u8>,
    /// Ed25519 signature over ciphertext
    pub signature: Vec<u8>,
}

impl HybridCiphertext {
    /// Serialize to JSON bytes
    pub fn to_bytes(&self) -> Result<Vec<u8>, CryptoError> {
        serde_json::to_vec(self).map_err(|e| CryptoError::EncryptionFailed(e.to_string()))
    }

    /// Deserialize from JSON bytes
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        serde_json::from_slice(bytes).map_err(|e| CryptoError::DecryptionFailed(e.to_string()))
    }
}

// ============================================================================
// Encryption / Decryption
// ============================================================================

/// Encrypt message using hybrid X25519 + Kyber1024 scheme
///
/// # Algorithm
/// 1. Generate ephemeral X25519 keypair
/// 2. X25519 ECDH with recipient's public key
/// 3. Kyber1024 encapsulation with recipient's public key
/// 4. HKDF-SHA3-256 to combine shared secrets
/// 5. ChaCha20-Poly1305 encryption with random nonce
/// 6. Ed25519 signature over ciphertext
///
/// SECURITY: nonce is generated from OsRng for each message
pub fn encrypt(
    plaintext: &[u8],
    recipient_x25519: &X25519PublicKey,
    recipient_kyber: &PublicKey,
    sender_signing_key: &SigningKey,
) -> Result<HybridCiphertext, CryptoError> {
    // 1. Ephemeral X25519 keypair
    let ephemeral_secret = StaticSecret::random_from_rng(OsRng);
    let ephemeral_public = X25519PublicKey::from(&ephemeral_secret);

    // 2. X25519 ECDH shared secret
    let x25519_shared = ephemeral_secret.diffie_hellman(recipient_x25519);

    // 3. Kyber1024 encapsulation
    oqs::init();
    let kyber_kem =
        Kem::new(Algorithm::Kyber1024).map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;
    let (kyber_ciphertext, kyber_shared) = kyber_kem
        .encapsulate(recipient_kyber)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // 4. Combine shared secrets via HKDF-SHA3-256
    let mut combined = Vec::with_capacity(x25519_shared.as_bytes().len() + kyber_shared.len());
    combined.extend_from_slice(x25519_shared.as_bytes());
    combined.extend_from_slice(kyber_shared.as_ref());

    // 5. Derive ChaCha20-Poly1305 key
    let mut key_bytes = [0u8; CHACHA20_KEY_SIZE];
    let hk = hkdf::Hkdf::<sha3::Sha3_256>::new(None, &combined);
    hk.expand(HKDF_CHACHA20_INFO, &mut key_bytes)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // 6. Generate cryptographically random nonce
    let mut nonce_bytes = [0u8; CHACHA20_NONCE_SIZE];
    OsRng.fill_bytes(&mut nonce_bytes);

    // 7. Encrypt with ChaCha20-Poly1305
    let key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(key);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, plaintext)
        .map_err(|e| CryptoError::EncryptionFailed(e.to_string()))?;

    // 8. Sign the ciphertext with sender's Ed25519 key
    let signature = sender_signing_key.sign(&ciphertext);

    tracing::debug!("Message encrypted with hybrid X25519+Kyber1024 scheme");

    Ok(HybridCiphertext {
        ephemeral_x25519: ephemeral_public.to_bytes(),
        kyber_ciphertext,
        nonce: nonce_bytes.to_vec(),
        ciphertext,
        signature: signature.to_bytes().to_vec(),
    })
}

/// Decrypt message using hybrid X25519 + Kyber1024 scheme
///
/// # Algorithm
/// 1. Reconstruct ephemeral X25519 public key
/// 2. X25519 ECDH with our secret key
/// 3. Kyber1024 decapsulation with our secret key
/// 4. HKDF-SHA3-256 to derive ChaCha20 key
/// 5. Verify Ed25519 signature
/// 6. ChaCha20-Poly1305 decryption
pub fn decrypt(
    ciphertext: &HybridCiphertext,
    my_x25519_secret: &StaticSecret,
    my_kyber_secret: &SecretKey,
    sender_verifying_key: &VerifyingKey,
) -> Result<Vec<u8>, CryptoError> {
    // 1. Reconstruct ephemeral public key
    let ephemeral_public = X25519PublicKey::from(ciphertext.ephemeral_x25519);

    // 2. X25519 ECDH shared secret
    let x25519_shared = my_x25519_secret.diffie_hellman(&ephemeral_public);

    // 3. Kyber1024 decapsulation
    oqs::init();
    let kyber_kem =
        Kem::new(Algorithm::Kyber1024).map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;
    let kyber_shared = kyber_kem
        .decapsulate(my_kyber_secret, &ciphertext.kyber_ciphertext)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    // 4. Combine shared secrets via HKDF-SHA3-256
    let mut combined = Vec::with_capacity(x25519_shared.as_bytes().len() + kyber_shared.len());
    combined.extend_from_slice(x25519_shared.as_bytes());
    combined.extend_from_slice(kyber_shared.as_ref());

    // 5. Derive ChaCha20-Poly1305 key
    let mut key_bytes = [0u8; CHACHA20_KEY_SIZE];
    let hk = hkdf::Hkdf::<sha3::Sha3_256>::new(None, &combined);
    hk.expand(HKDF_CHACHA20_INFO, &mut key_bytes)
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    // 6. Verify Ed25519 signature
    let signature = Signature::from_bytes(
        ciphertext
            .signature
            .as_slice()
            .try_into()
            .map_err(|_| CryptoError::InvalidSignature)?,
    );
    sender_verifying_key
        .verify(&ciphertext.ciphertext, &signature)
        .map_err(|_| CryptoError::InvalidSignature)?;

    // 7. Decrypt with ChaCha20-Poly1305
    let key = Key::from_slice(&key_bytes);
    let cipher = ChaCha20Poly1305::new(key);
    let nonce = Nonce::from_slice(&ciphertext.nonce);

    let plaintext = cipher
        .decrypt(nonce, ciphertext.ciphertext.as_ref())
        .map_err(|e| CryptoError::DecryptionFailed(e.to_string()))?;

    tracing::debug!("Message decrypted successfully");

    Ok(plaintext)
}

/// Sign data with Ed25519
pub fn sign(message: &[u8], secret: &SigningKey) -> Signature {
    secret.sign(message)
}

/// Verify Ed25519 signature
pub fn verify_signature(
    signature: &Signature,
    message: &[u8],
    public: &VerifyingKey,
) -> Result<(), CryptoError> {
    public
        .verify(message, signature)
        .map_err(|_| CryptoError::InvalidSignature)
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hybrid_encrypt_decrypt() {
        let keypair = HybridKeypair::generate().unwrap();
        let message = b"Hello, Post-Quantum World!";

        let ciphertext = encrypt(
            message,
            &keypair.x25519_public,
            &keypair.kyber_public,
            &keypair.ed25519_secret,
        )
        .unwrap();

        let decrypted = decrypt(
            &ciphertext,
            &keypair.x25519_secret,
            &keypair.kyber_secret,
            &keypair.ed25519_public,
        )
        .unwrap();

        assert_eq!(message, &decrypted[..]);
    }

    #[test]
    fn test_sign_verify() {
        let keypair = HybridKeypair::generate().unwrap();
        let message = b"Test message";

        let signature = sign(message, &keypair.ed25519_secret);
        assert!(verify_signature(&signature, message, &keypair.ed25519_public).is_ok());
    }

    #[test]
    fn test_public_bundle_serialization() {
        let keypair = HybridKeypair::generate().unwrap();
        let bundle = keypair.public_bundle();
        let bytes = bundle.to_bytes();
        let restored = PublicBundle::from_bytes(&bytes).unwrap();

        assert_eq!(bundle.x25519_public, restored.x25519_public);
        assert_eq!(bundle.kyber_public, restored.kyber_public);
        assert_eq!(bundle.ed25519_public, restored.ed25519_public);
    }

    #[test]
    fn test_ciphertext_serialization() {
        let keypair = HybridKeypair::generate().unwrap();
        let message = b"Serialization test";

        let ciphertext = encrypt(
            message,
            &keypair.x25519_public,
            &keypair.kyber_public,
            &keypair.ed25519_secret,
        )
        .unwrap();

        let bytes = ciphertext.to_bytes().unwrap();
        let restored = HybridCiphertext::from_bytes(&bytes).unwrap();

        assert_eq!(ciphertext.ephemeral_x25519, restored.ephemeral_x25519);
        assert_eq!(ciphertext.kyber_ciphertext, restored.kyber_ciphertext);
        assert_eq!(ciphertext.nonce, restored.nonce);
        assert_eq!(ciphertext.ciphertext, restored.ciphertext);
        assert_eq!(ciphertext.signature, restored.signature);
    }

    #[test]
    fn test_tampered_ciphertext_fails() {
        let keypair = HybridKeypair::generate().unwrap();
        let message = b"Tamper test";

        let mut ciphertext = encrypt(
            message,
            &keypair.x25519_public,
            &keypair.kyber_public,
            &keypair.ed25519_secret,
        )
        .unwrap();

        // Tamper with the ciphertext
        ciphertext.ciphertext[0] ^= 0xFF;

        let result = decrypt(
            &ciphertext,
            &keypair.x25519_secret,
            &keypair.kyber_secret,
            &keypair.ed25519_public,
        );

        assert!(result.is_err());
    }
}
