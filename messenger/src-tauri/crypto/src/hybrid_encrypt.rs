// Hybrid encryption: X25519 + Kyber1024 + ChaCha20-Poly1305 + HMAC
// SECURITY: требует аудита перед production
// TODO: pentest перед release

use chacha20poly1305::{ChaCha20Poly1305, Key, Nonce, aead::{Aead, KeyInit}};
use rand::rngs::OsRng;
use rand::RngCore;
use x25519_dalek::{EphemeralSecret, PublicKey};
use sha3::Sha3_256;
use hmac::{Hmac, Mac};
use crate::keypair::{KeyPair, PublicBundle};
use crate::error::CryptoError;

type HmacSha3 = Hmac<Sha3_256>;

/// Гибридный шифратор: X25519 + Kyber1024 → ChaCha20-Poly1305 + HMAC
pub struct HybridEncryptor;

impl HybridEncryptor {
    /// Шифрование сообщения с использованием гибридного подхода
    ///
    /// Алгоритм:
    /// 1. Генерируем ephemeral X25519 ключ
    /// 2. ECDH с публичным ключом получателя
    /// 3. Kyber1024 encapsulation
    /// 4. HKDF для получения общего ключа
    /// 5. ChaCha20-Poly1305 для шифрования
    /// 6. HMAC-SHA3 для аутентификации
    pub fn encrypt(
        message: &[u8],
        recipient_public: &PublicBundle,
    ) -> Result<EncryptedMessage, CryptoError> {
        // 1. Ephemeral X25519 key
        let ephemeral_secret = EphemeralSecret::random_from_rng(OsRng);
        let ephemeral_public = PublicKey::from(&ephemeral_secret);

        // 2. ECDH shared secret
        let recipient_x25519 = PublicKey::from(recipient_public.x25519_public);
        let ecdh_shared = ephemeral_secret.diffie_hellman(&recipient_x25519);

        // TODO: Kyber1024 encapsulation
        // let (kyber_ciphertext, kyber_shared) = kyber_encapsulate(&recipient_public.kyber_public)?;

        // 3. Combine shared secrets (placeholder for now)
        let combined_secret = ecdh_shared.as_bytes().to_vec();

        // 4. Derive encryption key via HKDF
        // TODO: использовать proper HKDF
        let mut key_bytes = [0u8; 32];
        for (i, &b) in combined_secret.iter().take(32).enumerate() {
            key_bytes[i] = b;
        }

        // 5. ChaCha20-Poly1305 encryption
        let key = Key::from_slice(&key_bytes);
        let cipher = ChaCha20Poly1305::new(key);
        // SECURITY: случайная nonce для каждого сообщения
        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = cipher.encrypt(nonce, message.as_ref())
            .map_err(|_| CryptoError::EncryptionFailed)?;

        // 6. HMAC for authentication
        let mut mac = <HmacSha3 as Mac>::new_from_slice(&key_bytes)
            .expect("HMAC can take key of any size");
        mac.update(&ciphertext);
        let hmac_result = mac.finalize().into_bytes();

        tracing::debug!("Message encrypted with hybrid scheme");

        Ok(EncryptedMessage {
            ephemeral_public: ephemeral_public.to_bytes(),
            // kyber_ciphertext: Vec::new(), // TODO
            nonce: nonce.to_vec(),
            ciphertext,
            hmac: hmac_result.to_vec(),
        })
    }

    /// Расшифровка сообщения
    pub fn decrypt(
        encrypted: &EncryptedMessage,
        local_keypair: &KeyPair,
    ) -> Result<Vec<u8>, CryptoError> {
        // 1. ECDH shared secret
        let ephemeral_public = PublicKey::from(encrypted.ephemeral_public);
        let ecdh_shared = local_keypair.x25519_secret.diffie_hellman(&ephemeral_public);

        // TODO: Kyber1024 decapsulation
        // let kyber_shared = kyber_decapsulate(&encrypted.kyber_ciphertext, &local_keypair.kyber_secret)?;

        // 2. Combine shared secrets
        let combined_secret = ecdh_shared.as_bytes().to_vec();

        // 3. Derive encryption key
        let mut key_bytes = [0u8; 32];
        for (i, &b) in combined_secret.iter().take(32).enumerate() {
            key_bytes[i] = b;
        }

        // 4. Verify HMAC
        let mut mac = <HmacSha3 as Mac>::new_from_slice(&key_bytes)
            .expect("HMAC can take key of any size");
        mac.update(&encrypted.ciphertext);
        mac.verify_slice(&encrypted.hmac)
            .map_err(|_| CryptoError::DecryptionFailed)?;

        // 5. ChaCha20-Poly1305 decryption
        let key = Key::from_slice(&key_bytes);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce = Nonce::from_slice(&encrypted.nonce);
        let plaintext = cipher.decrypt(nonce, encrypted.ciphertext.as_ref())
            .map_err(|_| CryptoError::DecryptionFailed)?;

        tracing::debug!("Message decrypted successfully");

        Ok(plaintext)
    }
}

/// Зашифрованное сообщение
#[derive(Debug, Clone)]
pub struct EncryptedMessage {
    /// Ephemeral X25519 public key
    pub ephemeral_public: [u8; 32],
    /// Kyber1024 ciphertext (TODO)
    // pub kyber_ciphertext: Vec<u8>,
    /// Nonce для ChaCha20-Poly1305
    pub nonce: Vec<u8>,
    /// Зашифрованные данные
    pub ciphertext: Vec<u8>,
    /// HMAC для аутентификации
    pub hmac: Vec<u8>,
}

impl EncryptedMessage {
    /// Сериализация для отправки по сети
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.ephemeral_public);
        bytes.extend_from_slice(&(self.nonce.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.nonce);
        bytes.extend_from_slice(&(self.ciphertext.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.ciphertext);
        bytes.extend_from_slice(&(self.hmac.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.hmac);
        bytes
    }

    /// Десериализация из сети
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() < 32 + 4 + 4 + 4 {
            return Err(CryptoError::InvalidKeyLength);
        }

        let mut ephemeral_public = [0u8; 32];
        ephemeral_public.copy_from_slice(&bytes[0..32]);

        let nonce_len = u32::from_le_bytes(bytes[32..36].try_into().unwrap()) as usize;
        let nonce = bytes[36..36 + nonce_len].to_vec();

        let ct_offset = 36 + nonce_len;
        let ct_len = u32::from_le_bytes(bytes[ct_offset..ct_offset + 4].try_into().unwrap()) as usize;
        let ciphertext = bytes[ct_offset + 4..ct_offset + 4 + ct_len].to_vec();

        let hmac_offset = ct_offset + 4 + ct_len;
        let hmac_len = u32::from_le_bytes(bytes[hmac_offset..hmac_offset + 4].try_into().unwrap()) as usize;
        let hmac = bytes[hmac_offset + 4..hmac_offset + 4 + hmac_len].to_vec();

        Ok(Self {
            ephemeral_public,
            nonce,
            ciphertext,
            hmac,
        })
    }
}
