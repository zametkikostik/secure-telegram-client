//! Diffie-Hellman обмен ключами на основе X25519 (Curve25519)
//!
//! X25519 обеспечивает безопасный обмен ключами по незащищённому каналу.

use anyhow::Result;
use rand::rngs::OsRng;
use x25519_dalek::{PublicKey, StaticSecret};

/// Размер открытого ключа X25519
pub const X25519_PUBLIC_KEY_SIZE: usize = 32;
/// Размер секретного ключа X25519
pub const X25519_SECRET_KEY_SIZE: usize = 32;
/// Размер общего секрета
pub const X25519_SHARED_SECRET_SIZE: usize = 32;

/// Пара ключей X25519
pub struct X25519KeyPair {
    secret_key: StaticSecret,
    public_key: PublicKey,
}

impl X25519KeyPair {
    /// Генерация новой пары ключей
    pub fn generate() -> Self {
        log::debug!("Генерация пары ключей X25519...");

        let secret_key = StaticSecret::random_from_rng(OsRng);
        let public_key = PublicKey::from(&secret_key);

        log::debug!("Пара ключей X25519 сгенерирована");

        Self {
            secret_key,
            public_key,
        }
    }

    /// Получение открытого ключа в виде байтов
    pub fn public_key_bytes(&self) -> [u8; X25519_PUBLIC_KEY_SIZE] {
        self.public_key.to_bytes()
    }

    /// Вычисление общего секрета по открытому ключу другой стороны
    pub fn compute_shared_secret(
        &self,
        peer_public_key: &[u8; X25519_PUBLIC_KEY_SIZE],
    ) -> [u8; X25519_SHARED_SECRET_SIZE] {
        log::debug!("Вычисление общего секрета X25519...");

        let peer_public = PublicKey::from(*peer_public_key);
        let shared_secret = self.secret_key.diffie_hellman(&peer_public);

        shared_secret.to_bytes()
    }

    /// Создание пары ключей из сырых байтов
    pub fn from_bytes(secret_bytes: [u8; X25519_SECRET_KEY_SIZE]) -> Self {
        let secret_key = StaticSecret::from(secret_bytes);
        let public_key = PublicKey::from(&secret_key);

        Self {
            secret_key,
            public_key,
        }
    }
}

/// Протокол обмена ключами
pub struct KeyExchange {
    keypair: X25519KeyPair,
}

impl KeyExchange {
    /// Создание нового экземпляра для обмена ключами
    pub fn new() -> Self {
        Self {
            keypair: X25519KeyPair::generate(),
        }
    }

    /// Получение открытого ключа для отправки другой стороне
    pub fn get_public_key(&self) -> [u8; X25519_PUBLIC_KEY_SIZE] {
        self.keypair.public_key_bytes()
    }

    /// Вычисление общего секрета после получения открытого ключа другой стороны
    pub fn compute_shared_secret(
        &self,
        peer_public_key: &[u8; X25519_PUBLIC_KEY_SIZE],
    ) -> [u8; X25519_SHARED_SECRET_SIZE] {
        self.keypair.compute_shared_secret(peer_public_key)
    }

    /// HKDF для вывода ключа из общего секрета
    pub fn derive_key(
        shared_secret: &[u8; X25519_SHARED_SECRET_SIZE],
        salt: &[u8],
        info: &[u8],
    ) -> [u8; 32] {
        use hkdf::Hkdf;
        use sha2::Sha256;

        let hkdf = Hkdf::<Sha256>::new(Some(salt), shared_secret);
        let mut okm = [0u8; 32];
        hkdf.expand(info, &mut okm)
            .expect("32 байт - валидный размер для вывода");

        okm
    }
}

impl Default for KeyExchange {
    fn default() -> Self {
        Self::new()
    }
}

/// Тестирование модуля
pub fn test() -> Result<()> {
    log::debug!("Тестирование X25519 модуля...");

    // Генерация двух пар ключей (Алиса и Боб)
    let alice = X25519KeyPair::generate();
    let bob = X25519KeyPair::generate();

    // Обмен ключами
    let alice_secret = alice.compute_shared_secret(&bob.public_key_bytes());
    let bob_secret = bob.compute_shared_secret(&alice.public_key_bytes());

    // Секреты должны совпадать
    assert_eq!(alice_secret, bob_secret);

    // Тест HKDF
    let salt = b"secure-telegram-salt";
    let info = b"encryption-key";
    let key = KeyExchange::derive_key(&alice_secret, salt, info);
    assert_eq!(key.len(), 32);

    log::debug!("X25519 тест пройден");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_key_exchange() {
        let alice = X25519KeyPair::generate();
        let bob = X25519KeyPair::generate();

        let alice_secret = alice.compute_shared_secret(&bob.public_key_bytes());
        let bob_secret = bob.compute_shared_secret(&alice.public_key_bytes());

        assert_eq!(alice_secret, bob_secret);
    }

    #[test]
    fn test_key_derivation() {
        let keypair = X25519KeyPair::generate();
        let peer = X25519KeyPair::generate();

        let shared = keypair.compute_shared_secret(&peer.public_key_bytes());
        let derived = KeyExchange::derive_key(&shared, b"salt", b"info");

        assert_eq!(derived.len(), 32);
    }

    #[test]
    fn test_public_key_size() {
        let keypair = X25519KeyPair::generate();
        assert_eq!(keypair.public_key_bytes().len(), X25519_PUBLIC_KEY_SIZE);
    }
}
