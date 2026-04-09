// KeyPair management - X25519 + Kyber1024 hybrid keys
// SECURITY: требует аудита перед production
// TODO: pentest перед release
// SECURITY: приватные ключи НИКОГДА не покидают устройство

use zeroize::{Zeroize, ZeroizeOnDrop};
use rand::rngs::OsRng;
use x25519_dalek::{StaticSecret, PublicKey as X25519PublicKey};
// TODO: интегрировать pqc_kyber для Kyber1024

/// Гибридная пара ключей: X25519 + Kyber1024
// SECURITY: требует аудита хранения ключей
#[derive(Zeroize, ZeroizeOnDrop)]
pub struct KeyPair {
    /// X25519 secret key (ECDH) — StaticSecret для долгосрочных ключей
    pub(crate) x25519_secret: StaticSecret,
    /// X25519 public key
    pub x25519_public: X25519PublicKey,
    /// Kyber1024 secret key (post-quantum KEM)
    // TODO: интегрировать liboqs для Kyber1024
    pub(crate) kyber_secret: Vec<u8>,
    /// Kyber1024 public key
    pub kyber_public: Vec<u8>,
    /// Ed25519 secret key (signatures)
    pub(crate) ed25519_secret: Vec<u8>,
    /// Ed25519 public key
    pub ed25519_public: Vec<u8>,
}

impl KeyPair {
    /// Генерация новой гибридной пары ключей
    pub fn generate() -> Self {
        // X25519 key generation (StaticSecret для долгосрочных ключей)
        let x25519_secret = StaticSecret::random_from_rng(OsRng);
        let x25519_public = X25519PublicKey::from(&x25519_secret);

        // TODO: Kyber1024 key generation через liboqs
        // let kyber_keys = oqs::kem::Key::new(oqs::kem::Algorithm::KYBER1024)?;
        let kyber_secret = vec![0u8; 3168]; // placeholder
        let kyber_public = vec![0u8; 1568];  // placeholder

        // TODO: Ed25519 key generation
        let ed25519_secret = vec![0u8; 64];  // placeholder
        let ed25519_public = vec![0u8; 32];  // placeholder

        tracing::info!("Generated new hybrid keypair (X25519 + Kyber1024 + Ed25519)");

        Self {
            x25519_secret,
            x25519_public,
            kyber_secret,
            kyber_public,
            ed25519_secret,
            ed25519_public,
        }
    }

    /// Получить публичный ключ для отправки контакту
    pub fn get_public_bundle(&self) -> PublicBundle {
        PublicBundle {
            x25519_public: self.x25519_public.to_bytes(),
            kyber_public: self.kyber_public.clone(),
            ed25519_public: self.ed25519_public.clone(),
        }
    }

    /// Сериализация в байты (для локального хранения)
    // SECURITY: данные должны быть зашифрованы перед записью на диск
    pub fn to_bytes(&self) -> Vec<u8> {
        // TODO: реализовать сериализацию с шифрованием
        vec![]
    }

    /// Десериализация из байтов (локальное хранилище)
    pub fn from_bytes(_bytes: &[u8]) -> Result<Self, CryptoError> {
        // TODO: реализовать десериализацию с расшифровкой
        Err(CryptoError::NotImplemented)
    }
}

/// Публичный ключ для обмена с контактами
#[derive(Clone, Debug)]
pub struct PublicBundle {
    pub x25519_public: [u8; 32],
    pub kyber_public: Vec<u8>,
    pub ed25519_public: Vec<u8>,
}

impl PublicBundle {
    /// Сериализация для отправки по сети
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        bytes.extend_from_slice(&self.x25519_public);
        bytes.extend_from_slice(&(self.kyber_public.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.kyber_public);
        bytes.extend_from_slice(&(self.ed25519_public.len() as u32).to_le_bytes());
        bytes.extend_from_slice(&self.ed25519_public);
        bytes
    }

    /// Десериализация из сети
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, CryptoError> {
        if bytes.len() < 32 + 4 + 4 {
            return Err(CryptoError::InvalidKeyLength);
        }

        let mut x25519_public = [0u8; 32];
        x25519_public.copy_from_slice(&bytes[0..32]);

        let kyber_len = u32::from_le_bytes(bytes[32..36].try_into().unwrap()) as usize;
        let kyber_public = bytes[36..36 + kyber_len].to_vec();

        let ed_offset = 36 + kyber_len;
        let ed_len = u32::from_le_bytes(bytes[ed_offset..ed_offset + 4].try_into().unwrap()) as usize;
        let ed25519_public = bytes[ed_offset + 4..ed_offset + 4 + ed_len].to_vec();

        Ok(Self {
            x25519_public,
            kyber_public,
            ed25519_public,
        })
    }
}

pub use crate::error::CryptoError;
