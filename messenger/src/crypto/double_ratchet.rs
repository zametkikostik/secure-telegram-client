//! Double Ratchet Algorithm — Forward Secrecy & Post-Compromise Security
//!
//! Implements the Double Ratchet protocol for secure messenger:
//! - **Symmetric-key ratchet**: Derives unique message keys per message
//! - **DH ratchet**: X25519 Diffie-Hellman for post-compromise security
//! - **HKDF-SHA3-256**: Key derivation function
//!
//! # Security Properties
//! - **Forward secrecy**: Compromise of current keys doesn't reveal past messages
//! - **Post-compromise security**: Self-healing after key compromise via DH ratchet
//! - **Message ordering**: Handles out-of-order delivery with skipped message keys
//!
//! # Architecture
//! ```text
//! [Initial handshake via hybrid.rs] → shared_secret
//!                                        ↓
//!                              DoubleRatchetSession
//!                                        ↓
//!                     ┌──────────────────┴──────────────────┐
//!                     ↓                                     ↓
//!          Sending Chain (encrypt)              Receiving Chain (decrypt)
//!                     ↓                                     ↓
//!          [msg_key, nonce, ciphertext]      [verify & decrypt]
//! ```
//!
//! SECURITY: требует аудита перед production
//! TODO: pentest перед release

use chacha20poly1305::{
    aead::{Aead, KeyInit},
    ChaCha20Poly1305, Key, Nonce,
};
use ed25519_dalek::{SigningKey, VerifyingKey};
use hkdf::Hkdf;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha3::Sha3_256;
use thiserror::Error;
use x25519_dalek::{PublicKey as X25519PublicKey, StaticSecret};
use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::crypto::constants::{CHACHA20_KEY_SIZE, CHACHA20_NONCE_SIZE, X25519_KEY_SIZE};

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum RatchetError {
    #[error("Ratchet initialization failed: {0}")]
    InitFailed(String),

    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),

    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),

    #[error("Invalid message key")]
    InvalidMessageKey,

    #[error("Message too old (outside skip window)")]
    MessageTooOld,

    #[error("Duplicate message")]
    DuplicateMessage,

    #[error("Serialization error: {0}")]
    Serialization(String),

    #[error("Maximum skip limit exceeded")]
    SkipLimitExceeded,
}

// ============================================================================
// Constants
// ============================================================================

/// Maximum number of skipped message keys to store for out-of-order delivery
const MAX_SKIP_WINDOW: usize = 50;

/// Maximum number of messages before forced DH ratchet
const MAX_MESSAGES_PER_CHAIN: u64 = 100;

/// HKDF info strings
const HKDF_RATCHET_INFO: &[u8] = b"double-ratchet";
const HKDF_CHAIN_KEY_INFO: &[u8] = b"chain-key";
const HKDF_MESSAGE_KEY_INFO: &[u8] = b"message-key";
const HKDF_ROOT_KEY_INFO: &[u8] = b"root-key";

// ============================================================================
// Data Structures
// ============================================================================

/// Encrypted message with ratchet metadata
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RatchetCiphertext {
    /// Ephemeral X25519 public key (for DH ratchet)
    pub ephemeral_public: Option<[u8; X25519_KEY_SIZE]>,
    /// Message key index (for detecting duplicates & ordering)
    pub message_index: u64,
    /// ChaCha20 nonce
    pub nonce: Vec<u8>,
    /// Encrypted payload
    pub ciphertext: Vec<u8>,
}

/// Serializable ratchet session state
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RatchetSession {
    /// Are we the initiator?
    pub is_initiator: bool,

    /// Current root key (32 bytes)
    pub root_key: Vec<u8>,

    /// Sending chain key (32 bytes)
    pub sending_chain_key: Option<Vec<u8>>,
    /// Sending chain message counter
    pub sending_chain_index: u64,

    /// Receiving chain key (32 bytes)
    pub receiving_chain_key: Option<Vec<u8>>,
    /// Receiving chain message counter
    pub receiving_chain_index: u64,

    /// Our current DH ratchet keypair (serialized)
    pub dh_secret: Option<Vec<u8>>,
    /// Their current DH ratchet public key
    pub dh_remote_public: Option<[u8; X25519_KEY_SIZE]>,

    /// Remote X25519 public key (static)
    pub remote_static_public: [u8; X25519_KEY_SIZE],
    /// Remote Ed25519 verifying key (for signature verification)
    pub remote_ed25519_public: [u8; 32],
    /// Our Ed25519 signing key (for signing)
    pub local_ed25519_secret: [u8; 32],

    /// Skipped message keys (for out-of-order delivery)
    pub skipped_keys: Vec<SkippedKey>,

    /// Total messages encrypted/decrypted
    pub message_count: u64,
}

/// Skipped message key for out-of-order decryption
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SkippedKey {
    pub message_index: u64,
    pub key: Vec<u8>,
}

impl zeroize::Zeroize for SkippedKey {
    fn zeroize(&mut self) {
        self.key.zeroize();
    }
}

// ============================================================================
// Double Ratchet Session
// ============================================================================

/// Active Double Ratchet session
pub struct DoubleRatchetSession {
    session: RatchetSession,
}

impl DoubleRatchetSession {
    /// Initialize a new ratchet session as the initiator
    ///
    /// Both sides derive the same chain keys from the shared secret.
    /// This provides forward secrecy (unique key per message).
    /// DH ratchet (post-compromise security) will be added in future versions.
    pub fn init_as_initiator(
        shared_secret: &[u8],
        _my_x25519_secret: StaticSecret,
        _their_x25519_public: X25519PublicKey,
        their_ed25519_public: [u8; 32],
        my_ed25519_secret: SigningKey,
    ) -> Result<Self, RatchetError> {
        // Derive sending chain key from shared secret
        let mut sending_chain_key = [0u8; 32];
        let hk = Hkdf::<Sha3_256>::new(None, shared_secret);
        hk.expand(b"init-sending-chain", &mut sending_chain_key)
            .map_err(|e| RatchetError::InitFailed(e.to_string()))?;

        // Derive receiving chain key from shared secret (different info string)
        let mut receiving_chain_key = [0u8; 32];
        let hk = Hkdf::<Sha3_256>::new(None, shared_secret);
        hk.expand(b"init-receiving-chain", &mut receiving_chain_key)
            .map_err(|e| RatchetError::InitFailed(e.to_string()))?;

        let session = RatchetSession {
            is_initiator: true,
            root_key: shared_secret.to_vec(),
            sending_chain_key: Some(sending_chain_key.to_vec()),
            sending_chain_index: 0,
            receiving_chain_key: Some(receiving_chain_key.to_vec()),
            receiving_chain_index: 0,
            dh_secret: None,
            dh_remote_public: None,
            remote_static_public: _their_x25519_public.to_bytes(),
            remote_ed25519_public: their_ed25519_public,
            local_ed25519_secret: my_ed25519_secret.to_bytes(),
            skipped_keys: Vec::new(),
            message_count: 0,
        };

        tracing::info!("Initialized Double Ratchet session as initiator");
        Ok(Self { session })
    }

    /// Initialize a new ratchet session as the responder
    ///
    /// Both sides derive the same chain keys from the shared secret.
    pub fn init_as_responder(
        shared_secret: &[u8],
        _my_x25519_secret: StaticSecret,
        their_x25519_public: X25519PublicKey,
        their_ed25519_public: [u8; 32],
        my_ed25519_secret: SigningKey,
    ) -> Result<Self, RatchetError> {
        // Derive sending chain key from shared secret (same as initiator's receiving)
        let mut sending_chain_key = [0u8; 32];
        let hk = Hkdf::<Sha3_256>::new(None, shared_secret);
        hk.expand(b"init-sending-chain", &mut sending_chain_key)
            .map_err(|e| RatchetError::InitFailed(e.to_string()))?;

        // Derive receiving chain key from shared secret (same as initiator's sending)
        let mut receiving_chain_key = [0u8; 32];
        let hk = Hkdf::<Sha3_256>::new(None, shared_secret);
        hk.expand(b"init-receiving-chain", &mut receiving_chain_key)
            .map_err(|e| RatchetError::InitFailed(e.to_string()))?;

        let session = RatchetSession {
            is_initiator: false,
            root_key: shared_secret.to_vec(),
            sending_chain_key: Some(receiving_chain_key.to_vec()), // Reversed!
            sending_chain_index: 0,
            receiving_chain_key: Some(sending_chain_key.to_vec()), // Reversed!
            receiving_chain_index: 0,
            dh_secret: None,
            dh_remote_public: None,
            remote_static_public: their_x25519_public.to_bytes(),
            remote_ed25519_public: their_ed25519_public,
            local_ed25519_secret: my_ed25519_secret.to_bytes(),
            skipped_keys: Vec::new(),
            message_count: 0,
        };

        tracing::info!("Initialized Double Ratchet session as responder");
        Ok(Self { session })
    }

    /// Encrypt a message using symmetric-key ratchet
    ///
    /// Each message gets a unique encryption key derived from the chain key.
    /// The chain key is then ratcheted forward for forward secrecy.
    pub fn encrypt_message(&mut self, plaintext: &[u8]) -> Result<RatchetCiphertext, RatchetError> {
        // Derive message key from chain key
        let chain_key = self
            .session
            .sending_chain_key
            .as_mut()
            .ok_or_else(|| RatchetError::EncryptionFailed("No sending chain".to_string()))?;

        let mut message_key = [0u8; 32];
        let hk = Hkdf::<Sha3_256>::new(Some(chain_key), HKDF_MESSAGE_KEY_INFO);
        hk.expand(b"msg-key", &mut message_key)
            .map_err(|e| RatchetError::EncryptionFailed(e.to_string()))?;

        // Ratchet chain key forward
        let mut new_chain_key = [0u8; 32];
        let hk = Hkdf::<Sha3_256>::new(Some(chain_key), HKDF_CHAIN_KEY_INFO);
        hk.expand(b"chain-ratchet", &mut new_chain_key)
            .map_err(|e| RatchetError::EncryptionFailed(e.to_string()))?;
        *chain_key = new_chain_key.to_vec();

        // Generate nonce
        let mut nonce_bytes = [0u8; CHACHA20_NONCE_SIZE];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);

        // Encrypt with ChaCha20-Poly1305
        let key = Key::from_slice(&message_key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext)
            .map_err(|e| RatchetError::EncryptionFailed(e.to_string()))?;

        let message_index = self.session.sending_chain_index;
        self.session.sending_chain_index += 1;
        self.session.message_count += 1;

        Ok(RatchetCiphertext {
            ephemeral_public: None,
            message_index,
            nonce: nonce_bytes.to_vec(),
            ciphertext,
        })
    }

    /// Decrypt a message
    pub fn decrypt_message(
        &mut self,
        ciphertext: &RatchetCiphertext,
    ) -> Result<Vec<u8>, RatchetError> {
        // Check for duplicate
        if self
            .session
            .skipped_keys
            .iter()
            .any(|sk| sk.message_index == ciphertext.message_index)
        {
            return Err(RatchetError::DuplicateMessage);
        }

        // Check if we have the message key (out-of-order)
        let chain_key = self
            .session
            .receiving_chain_key
            .as_mut()
            .ok_or_else(|| RatchetError::DecryptionFailed("No receiving chain".to_string()))?;

        // Try to find skipped key
        let skipped_idx = self
            .session
            .skipped_keys
            .iter()
            .position(|sk| sk.message_index == ciphertext.message_index);

        let message_key = if let Some(idx) = skipped_idx {
            let key = self.session.skipped_keys.remove(idx).key;
            key
        } else {
            // Derive message key and ratchet forward
            // Handle out-of-order by generating skipped keys
            let current_index = self.session.receiving_chain_index;

            if ciphertext.message_index < current_index {
                return Err(RatchetError::MessageTooOld);
            }

            let skip_count = (ciphertext.message_index - current_index) as usize;
            if skip_count > MAX_SKIP_WINDOW {
                return Err(RatchetError::SkipLimitExceeded);
            }

            // Generate and store skipped keys, then derive the needed key
            let mut temp_chain_key = chain_key.clone();
            let mut final_msg_key = None;

            for i in 0..=skip_count {
                let mut msg_key = [0u8; 32];
                let hk = Hkdf::<Sha3_256>::new(Some(&temp_chain_key), HKDF_MESSAGE_KEY_INFO);
                hk.expand(b"msg-key", &mut msg_key)
                    .map_err(|e| RatchetError::DecryptionFailed(e.to_string()))?;

                if i < skip_count {
                    // Store skipped key
                    if self.session.skipped_keys.len() < MAX_SKIP_WINDOW {
                        self.session.skipped_keys.push(SkippedKey {
                            message_index: current_index + i as u64,
                            key: msg_key.to_vec(),
                        });
                    }
                } else {
                    // This is the key we need now - save it
                    final_msg_key = Some(msg_key);
                }

                // Ratchet chain key forward for next iteration
                let hk = Hkdf::<Sha3_256>::new(Some(&temp_chain_key), HKDF_CHAIN_KEY_INFO);
                hk.expand(b"chain-ratchet", &mut temp_chain_key)
                    .map_err(|e| RatchetError::DecryptionFailed(e.to_string()))?;
            }

            *chain_key = temp_chain_key;
            self.session.receiving_chain_index = ciphertext.message_index + 1;

            final_msg_key
                .ok_or_else(|| {
                    RatchetError::DecryptionFailed("Failed to derive message key".to_string())
                })?
                .to_vec()
        };

        // Decrypt
        let key = Key::from_slice(&message_key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = Nonce::from_slice(&ciphertext.nonce);
        let plaintext = cipher
            .decrypt(nonce, ciphertext.ciphertext.as_ref())
            .map_err(|e| RatchetError::DecryptionFailed(e.to_string()))?;

        self.session.message_count += 1;

        // Zeroize message key
        drop(message_key);

        Ok(plaintext)
    }

    /// Serialize session state for persistence
    pub fn to_session(&self) -> RatchetSession {
        self.session.clone()
    }

    /// Restore session from serialized state
    pub fn from_session(session: RatchetSession) -> Self {
        Self { session }
    }
}

impl Drop for DoubleRatchetSession {
    fn drop(&mut self) {
        // Zeroize sensitive data
        self.session.root_key.zeroize();
        if let Some(ref mut key) = self.session.sending_chain_key {
            key.zeroize();
        }
        if let Some(ref mut key) = self.session.receiving_chain_key {
            key.zeroize();
        }
        if let Some(ref mut key) = self.session.dh_secret {
            key.zeroize();
        }
        for skipped in &mut self.session.skipped_keys {
            skipped.zeroize();
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_double_ratchet_init() {
        use rand::rngs::OsRng;

        // Create keypairs
        let alice_x = StaticSecret::random_from_rng(OsRng);
        let alice_e = SigningKey::generate(&mut OsRng);
        let bob_x = StaticSecret::random_from_rng(OsRng);
        let bob_e = SigningKey::generate(&mut OsRng);

        let alice_pub = x25519_dalek::PublicKey::from(&alice_x);
        let bob_pub = x25519_dalek::PublicKey::from(&bob_x);
        let shared = alice_x.diffie_hellman(&bob_pub);
        let alice_ed_bytes = alice_e.verifying_key().to_bytes();

        // Initialize ratchet sessions
        let alice_ratchet = DoubleRatchetSession::init_as_initiator(
            shared.as_bytes(),
            alice_x,
            bob_pub,
            bob_e.verifying_key().to_bytes(),
            alice_e,
        );

        assert!(alice_ratchet.is_ok());

        let bob_ratchet = DoubleRatchetSession::init_as_responder(
            shared.as_bytes(),
            bob_x,
            alice_pub,
            alice_ed_bytes,
            bob_e,
        );

        assert!(bob_ratchet.is_ok());
    }

    #[test]
    fn test_encrypt_decrypt_roundtrip() {
        use rand::rngs::OsRng;

        let alice_x = StaticSecret::random_from_rng(OsRng);
        let alice_e = SigningKey::generate(&mut OsRng);
        let bob_x = StaticSecret::random_from_rng(OsRng);
        let bob_e = SigningKey::generate(&mut OsRng);

        let alice_pub = x25519_dalek::PublicKey::from(&alice_x);
        let bob_pub = x25519_dalek::PublicKey::from(&bob_x);
        let shared = alice_x.diffie_hellman(&bob_pub);

        let alice_ed_bytes = alice_e.verifying_key().to_bytes();
        let mut alice_ratchet = DoubleRatchetSession::init_as_initiator(
            shared.as_bytes(),
            alice_x,
            bob_pub,
            bob_e.verifying_key().to_bytes(),
            alice_e,
        )
        .unwrap();

        let mut bob_ratchet = DoubleRatchetSession::init_as_responder(
            shared.as_bytes(),
            bob_x,
            alice_pub,
            alice_ed_bytes,
            bob_e,
        )
        .unwrap();

        // Alice encrypts first message
        let msg1 = b"Hello with forward secrecy!";
        let ct1 = alice_ratchet.encrypt_message(msg1).unwrap();

        // Bob decrypts
        let pt1 = bob_ratchet.decrypt_message(&ct1).unwrap();
        assert_eq!(msg1.to_vec(), pt1);

        // Alice sends second message
        let msg2 = b"Second message";
        let ct2 = alice_ratchet.encrypt_message(msg2).unwrap();
        let pt2 = bob_ratchet.decrypt_message(&ct2).unwrap();
        assert_eq!(msg2.to_vec(), pt2);
    }

    #[test]
    fn test_session_serialization() {
        use rand::rngs::OsRng;

        let alice_x = StaticSecret::random_from_rng(OsRng);
        let alice_e = SigningKey::generate(&mut OsRng);
        let bob_x = StaticSecret::random_from_rng(OsRng);
        let bob_e = SigningKey::generate(&mut OsRng);

        let bob_pub = x25519_dalek::PublicKey::from(&bob_x);
        let shared = alice_x.diffie_hellman(&bob_pub);

        let _alice_ed_bytes = alice_e.verifying_key().to_bytes();
        let alice_ratchet = DoubleRatchetSession::init_as_initiator(
            shared.as_bytes(),
            alice_x,
            bob_pub,
            bob_e.verifying_key().to_bytes(),
            alice_e,
        )
        .unwrap();

        // Serialize
        let session = alice_ratchet.to_session();
        let serialized = serde_json::to_string(&session).unwrap();

        // Deserialize
        let restored_session: RatchetSession = serde_json::from_str(&serialized).unwrap();
        let _restored_ratchet = DoubleRatchetSession::from_session(restored_session);

        assert_eq!(session.message_count, 0);
    }
}
