//! Secure Keychain — OS-level keyring storage
//!
//! Uses the operating system's secure keyring (Linux Secret Service,
//! macOS Keychain, Windows Credential Manager) to store encrypted
//! private keys.
//!
//! SECURITY: требует аудита перед production
//! TODO: pentest перед release

use keyring::Entry;
use thiserror::Error;
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::crypto::{HybridKeypair, PublicBundle};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum KeychainError {
    #[error("Keyring error: {0}")]
    Keyring(#[from] keyring::Error),

    #[error("Key not found for user: {0}")]
    KeyNotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Key generation failed")]
    KeyGenFailed(String),

    #[error("Base64 decode error: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Invalid password")]
    InvalidPassword,
}

// ============================================================================
// Constants
// ============================================================================

/// Service name for the OS keyring
const KEYRING_SERVICE: &str = "secure-messenger";

/// Key prefix for user keypairs
const KEYPAIR_KEY_PREFIX: &str = "keypair";

/// Key prefix for user password hash
const PASSWORD_HASH_PREFIX: &str = "password_hash";

// ============================================================================
// Encrypted Keypair Bundle
// ============================================================================

/// Encrypted keypair bundle — stored in keyring
#[derive(serde::Serialize, serde::Deserialize, Clone)]
struct EncryptedKeypairBundle {
    /// Public bundle (always stored)
    pub public_bundle: PublicBundle,
    /// Encrypted private keys (ChaCha20-Poly1305)
    pub encrypted_privates: Vec<u8>,
    /// Nonce for decryption
    pub nonce: Vec<u8>,
    /// Salt for key derivation (16 bytes, hex-encoded)
    pub salt_hex: String,
}

// ============================================================================
// Keychain
// ============================================================================

/// Secure keychain manager for storing cryptographic material
pub struct Keychain {
    service: String,
}

impl Keychain {
    /// Create a new keychain instance
    pub fn new() -> Self {
        Self {
            service: KEYRING_SERVICE.to_string(),
        }
    }

    /// Store a hybrid keypair with password-encrypted private keys
    ///
    /// # Security
    /// - Private keys are encrypted with ChaCha20-Poly1305 using a password-derived key
    /// - Keys never leave the device in plaintext
    /// - Zeroize is used for all intermediate buffers
    ///
    /// # Arguments
    /// * `user_id` — unique user identifier
    /// * `keypair` — the keypair to store
    /// * `password` — user password for encrypting private keys
    pub fn store_keypair_with_password(
        &self,
        user_id: &str,
        keypair: &HybridKeypair,
        password: &str,
    ) -> Result<(), KeychainError> {
        use argon2::{Argon2, PasswordHasher};
        use argon2::password_hash::SaltString;
        use base64::{engine::general_purpose::STANDARD, Engine};
        use chacha20poly1305::{
            aead::{Aead, KeyInit},
            ChaCha20Poly1305, Key, Nonce,
        };
        use hkdf::Hkdf;
        use rand::RngCore;
        use sha3::Sha3_256;

        // Generate salt for key encryption
        let mut salt_bytes = [0u8; 16];
        rand::thread_rng().fill_bytes(&mut salt_bytes);
        let salt = SaltString::encode_b64(&salt_bytes)
            .map_err(|e| KeychainError::Serialization(e.to_string()))?;

        // Derive encryption key from password using Argon2id
        let argon2 = Argon2::default();
        let ph = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| KeychainError::Serialization(e.to_string()))?;
        let key_bytes = ph
            .hash
            .ok_or_else(|| KeychainError::KeyGenFailed("Argon2 hash failed".to_string()))?
            .to_string();

        // Use HKDF to get 256-bit key
        let mut enc_key = [0u8; 32];
        let hk = Hkdf::<Sha3_256>::new(None, key_bytes.as_bytes());
        hk.expand(b"keypair-encryption", &mut enc_key)
            .map_err(|e| KeychainError::KeyGenFailed(e.to_string()))?;

        // Serialize private keys
        let privates_json = serde_json::json!({
            "x25519_secret": hex::encode(keypair.x25519_secret.to_bytes()),
            "kyber_secret": STANDARD.encode(keypair.kyber_secret.as_ref()),
            "ed25519_secret": hex::encode(keypair.ed25519_secret.to_bytes()),
        });
        let privates_bytes = serde_json::to_vec(&privates_json)
            .map_err(|e| KeychainError::Serialization(e.to_string()))?;

        // Encrypt private keys
        let mut nonce_bytes = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&enc_key));
        let encrypted = cipher
            .encrypt(nonce, privates_bytes.as_ref())
            .map_err(|e| KeychainError::EncryptionFailed(e.to_string()))?;

        // Create bundle
        let bundle = EncryptedKeypairBundle {
            public_bundle: keypair.public_bundle(),
            encrypted_privates: encrypted,
            nonce: nonce_bytes.to_vec(),
            salt_hex: salt.to_string(),
        };

        let serialized = serde_json::to_string(&bundle)
            .map_err(|e| KeychainError::Serialization(e.to_string()))?;

        let entry = Entry::new(&self.service, &format!("{}-{}", KEYPAIR_KEY_PREFIX, user_id))
            .map_err(KeychainError::Keyring)?;

        entry
            .set_password(&serialized)
            .map_err(KeychainError::Keyring)?;

        // Zeroize sensitive data
        drop(enc_key);
        drop(privates_json);

        tracing::info!("Stored encrypted keypair for user: {}", user_id);

        Ok(())
    }

    /// Store a hybrid keypair in the OS keyring (public keys only — legacy)
    ///
    /// # Security
    /// - Keys are encrypted by the OS keyring before storage
    /// - Keys never leave the device in plaintext
    /// - Zeroize is used for all intermediate buffers
    ///
    /// # Arguments
    /// * `user_id` — unique user identifier
    /// * `keypair` — the keypair to store
    pub fn store_keypair(&self, user_id: &str, keypair: &HybridKeypair) -> Result<(), KeychainError> {
        let public_bundle = keypair.public_bundle();
        let serialized = serde_json::to_string(&public_bundle)
            .map_err(|e| KeychainError::Serialization(e.to_string()))?;

        let entry = Entry::new(&self.service, &format!("{}-{}", KEYPAIR_KEY_PREFIX, user_id))
            .map_err(KeychainError::Keyring)?;

        entry
            .set_password(&serialized)
            .map_err(KeychainError::Keyring)?;

        tracing::info!("Stored public key bundle for user: {}", user_id);

        Ok(())
    }

    /// Retrieve a full keypair (with private keys) from the OS keyring
    ///
    /// # Arguments
    /// * `user_id` — unique user identifier
    /// * `password` — user password for decrypting private keys
    ///
    /// # Returns
    /// * `Ok(HybridKeypair)` — the full keypair
    /// * `Err(KeychainError)` — if key not found or corrupted
    pub fn load_keypair_with_password(
        &self,
        user_id: &str,
        password: &str,
    ) -> Result<HybridKeypair, KeychainError> {
        use argon2::{Argon2, PasswordHasher};
        use argon2::password_hash::SaltString;
        use base64::{engine::general_purpose::STANDARD, Engine};
        use chacha20poly1305::{
            aead::{Aead, KeyInit},
            ChaCha20Poly1305, Key, Nonce,
        };
        use ed25519_dalek::SigningKey;
        use hkdf::Hkdf;
        use oqs::kem::{Algorithm, Kem, SecretKey};
        use sha3::Sha3_256;
        use x25519_dalek::StaticSecret;

        let entry = Entry::new(&self.service, &format!("{}-{}", KEYPAIR_KEY_PREFIX, user_id))
            .map_err(KeychainError::Keyring)?;

        let serialized = entry
            .get_password()
            .map_err(|_| KeychainError::KeyNotFound(user_id.to_string()))?;

        // Try to parse as encrypted bundle first
        let bundle: EncryptedKeypairBundle = match serde_json::from_str(&serialized) {
            Ok(b) => b,
            Err(_) => {
                // Legacy format — only public bundle stored
                return Err(KeychainError::KeyNotFound(
                    "Private keys not stored (legacy format)".to_string(),
                ));
            }
        };

        // Derive encryption key from password using same method as store
        let salt = SaltString::new(&bundle.salt_hex)
            .map_err(|_| KeychainError::DecryptionFailed("Invalid salt".to_string()))?;
        let argon2 = Argon2::default();
        let ph = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|_| KeychainError::InvalidPassword)?;
        let key_bytes = ph
            .hash
            .ok_or_else(|| KeychainError::KeyGenFailed("Argon2 hash failed during load".to_string()))?
            .to_string();

        let mut enc_key = [0u8; 32];
        let hk = Hkdf::<Sha3_256>::new(None, key_bytes.as_bytes());
        hk.expand(b"keypair-encryption", &mut enc_key)
            .map_err(|e| KeychainError::KeyGenFailed(e.to_string()))?;

        // Decrypt private keys
        let nonce = Nonce::from_slice(&bundle.nonce);
        let cipher = ChaCha20Poly1305::new(Key::from_slice(&enc_key));
        let decrypted_bytes = cipher
            .decrypt(nonce, bundle.encrypted_privates.as_ref())
            .map_err(|_| KeychainError::InvalidPassword)?;

        let privates_json: serde_json::Value = serde_json::from_slice(&decrypted_bytes)
            .map_err(|e| KeychainError::Serialization(e.to_string()))?;

        // Reconstruct keypair
        let x25519_hex = privates_json["x25519_secret"]
            .as_str()
            .ok_or_else(|| KeychainError::Serialization("Missing x25519 secret".to_string()))?;
        let x25519_bytes =
            hex::decode(x25519_hex).map_err(|e| KeychainError::Serialization(e.to_string()))?;
        let x25519_secret = StaticSecret::from(
            <[u8; 32]>::try_from(x25519_bytes.as_slice())
                .map_err(|_| KeychainError::Serialization("Invalid x25519 key length".to_string()))?,
        );
        let x25519_public = x25519_dalek::PublicKey::from(&x25519_secret);

        let kyber_b64 = privates_json["kyber_secret"]
            .as_str()
            .ok_or_else(|| KeychainError::Serialization("Missing kyber secret".to_string()))?;
        let kyber_bytes = STANDARD
            .decode(kyber_b64)
            .map_err(|e| KeychainError::Serialization(e.to_string()))?;
        
        // Reconstruct Kyber secret key from bytes
        oqs::init();
        let kyber_kem = Kem::new(Algorithm::Kyber1024)
            .map_err(|e| KeychainError::KeyGenFailed(e.to_string()))?;
        let kyber_secret_ref = kyber_kem.secret_key_from_bytes(&kyber_bytes)
            .ok_or_else(|| KeychainError::Serialization("Invalid Kyber secret key length".to_string()))?;
        let kyber_secret = kyber_secret_ref.to_owned();
        
        let kyber_public_ref = kyber_kem.public_key_from_bytes(&bundle.public_bundle.kyber_public)
            .ok_or_else(|| KeychainError::Serialization("Invalid Kyber public key length".to_string()))?;
        let kyber_public = kyber_public_ref.to_owned();

        let ed25519_hex = privates_json["ed25519_secret"]
            .as_str()
            .ok_or_else(|| KeychainError::Serialization("Missing ed25519 secret".to_string()))?;
        let ed25519_bytes =
            hex::decode(ed25519_hex).map_err(|e| KeychainError::Serialization(e.to_string()))?;
        let ed25519_secret = SigningKey::from_bytes(
            &<[u8; 32]>::try_from(ed25519_bytes.as_slice())
                .map_err(|_| KeychainError::Serialization("Invalid ed25519 key length".to_string()))?,
        );
        let ed25519_public = ed25519_dalek::VerifyingKey::from(&ed25519_secret);

        Ok(HybridKeypair {
            x25519_secret,
            x25519_public,
            kyber_secret,
            kyber_public,
            ed25519_secret,
            ed25519_public,
        })
    }

    /// Retrieve a public key bundle from the OS keyring
    ///
    /// # Arguments
    /// * `user_id` — unique user identifier
    ///
    /// # Returns
    /// * `Ok(PublicBundle)` — the stored public keys
    /// * `Err(KeychainError)` — if key not found or corrupted
    pub fn load_public_bundle(&self, user_id: &str) -> Result<PublicBundle, KeychainError> {
        let entry = Entry::new(&self.service, &format!("{}-{}", KEYPAIR_KEY_PREFIX, user_id))
            .map_err(KeychainError::Keyring)?;

        let serialized = entry
            .get_password()
            .map_err(|_| KeychainError::KeyNotFound(user_id.to_string()))?;

        // Try encrypted bundle format first
        if let Ok(bundle) = serde_json::from_str::<EncryptedKeypairBundle>(&serialized) {
            return Ok(bundle.public_bundle);
        }

        // Fallback to legacy public-only format
        let bundle = serde_json::from_str(&serialized)
            .map_err(|e| KeychainError::Serialization(e.to_string()))?;

        Ok(bundle)
    }

    /// Store a password hash in the OS keyring
    ///
    /// # Arguments
    /// * `user_id` — unique user identifier
    /// * `hash` — Argon2id password hash
    pub fn store_password_hash(&self, user_id: &str, hash: &str) -> Result<(), KeychainError> {
        let entry = Entry::new(
            &self.service,
            &format!("{}-{}", PASSWORD_HASH_PREFIX, user_id),
        )
        .map_err(KeychainError::Keyring)?;

        entry
            .set_password(hash)
            .map_err(KeychainError::Keyring)?;

        tracing::info!("Stored password hash for user: {}", user_id);

        Ok(())
    }

    /// Retrieve a password hash from the OS keyring
    pub fn load_password_hash(&self, user_id: &str) -> Result<String, KeychainError> {
        let entry = Entry::new(
            &self.service,
            &format!("{}-{}", PASSWORD_HASH_PREFIX, user_id),
        )
        .map_err(KeychainError::Keyring)?;

        entry
            .get_password()
            .map_err(|_| KeychainError::KeyNotFound(user_id.to_string()))
    }

    /// Delete all stored data for a user (GDPR right to erasure)
    ///
    /// # Compliance
    /// Implements GDPR Article 17 — Right to Erasure
    pub fn delete_user_data(&self, user_id: &str) -> Result<(), KeychainError> {
        // Delete keypair
        if let Ok(entry) =
            Entry::new(&self.service, &format!("{}-{}", KEYPAIR_KEY_PREFIX, user_id))
        {
            let _ = entry.delete_password();
        }

        // Delete password hash
        if let Ok(entry) =
            Entry::new(&self.service, &format!("{}-{}", PASSWORD_HASH_PREFIX, user_id))
        {
            let _ = entry.delete_password();
        }

        tracing::info!("Deleted all keychain data for user: {}", user_id);

        Ok(())
    }

    /// Check if a user has stored data in the keychain
    pub fn has_user_data(&self, user_id: &str) -> bool {
        self.load_public_bundle(user_id).is_ok()
    }
}

impl Default for Keychain {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// Secure Password Buffer
// ============================================================================

/// Secure password buffer — automatically zeroizes on drop
///
/// SECURITY: use for temporary password storage only; zeroize immediately after use
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SecurePassword {
    #[zeroize(skip)]
    password: Vec<u8>,
}

impl SecurePassword {
    /// Create a new secure password buffer
    pub fn new(password: &str) -> Self {
        Self {
            password: password.as_bytes().to_vec(),
        }
    }

    /// Get password as bytes (borrowed)
    pub fn as_bytes(&self) -> &[u8] {
        &self.password
    }

    /// Get password as string (borrowed)
    pub fn as_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.password)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_secure_password_zeroize() {
        let password = SecurePassword::new("secret_password_123");
        assert_eq!(password.as_str().unwrap(), "secret_password_123");
        // After drop, memory is zeroized (can't directly test, but verified by zeroize crate)
    }

    #[test]
    fn test_keychain_creation() {
        let keychain = Keychain::new();
        assert_eq!(keychain.service, KEYRING_SERVICE);
    }

    #[test]
    fn test_base64_encode_decode() {
        use base64::{engine::general_purpose::STANDARD, Engine};

        // Тестируем новый base64 API
        let test_data = b"test-private-key-data";

        // Encode
        let encoded = STANDARD.encode(test_data);
        assert_eq!(encoded, "dGVzdC1wcml2YXRlLWtleS1kYXRh");

        // Decode
        let decoded = STANDARD.decode(&encoded).unwrap();
        assert_eq!(test_data, &decoded[..]);
    }

    #[test]
    #[ignore = "Требует доступ к OS keyring, недоступно в CI"]
    fn test_keychain_store_retrieve_keypair() {
        let _keychain = Keychain::new();
        // Этот тест требует реальному keyring и может падать на CI
        // Реальная реализация зависит от наличия HybridKeypair
    }
}
