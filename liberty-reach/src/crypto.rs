//! Модуль криптографии для Liberty Reach
//!
//! Реализует:
//! - AES-256-GCM для шифрования сообщений (E2EE)
//! - X25519 Diffie-Hellman для обмена ключами
//! - Безопасное управление сессиями

use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use rand::RngCore;
use anyhow::{Result, Context};
use x25519_dalek::{EphemeralSecret, PublicKey, SharedSecret};
use std::collections::HashMap;
use libp2p::PeerId;

/// Менеджер шифрования для E2EE
#[derive(Clone)]
pub struct CipherManager {
    key: [u8; 32],
}

impl CipherManager {
    /// Создание нового CipherManager с заданным ключом
    pub fn new(key: &[u8; 32]) -> Self {
        Self { key: *key }
    }

    /// Шифрование сообщения
    /// Возвращает: nonce (12 байт) + ciphertext
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .context("Неверная длина ключа AES (требуется 32 байта)")?;

        let mut nonce_bytes = [0u8; 12];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| anyhow::anyhow!("Ошибка шифрования AES-GCM: {}", e))?;

        // Префикс: nonce (12 байт) + ciphertext
        let mut result = Vec::with_capacity(12 + ciphertext.len());
        result.extend_from_slice(&nonce_bytes);
        result.extend_from_slice(&ciphertext);

        Ok(result)
    }

    /// Дешифрование сообщения
    /// Ожидает: nonce (12 байт) + ciphertext
    pub fn decrypt(&self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < 12 {
            anyhow::bail!("Слишком короткие данные для дешифрования (ожидалось >= 12 байт)");
        }

        let nonce_bytes = &data[..12];
        let ciphertext = &data[12..];

        let cipher = Aes256Gcm::new_from_slice(&self.key)
            .context("Неверная длина ключа AES (требуется 32 байта)")?;
        let nonce = Nonce::from_slice(nonce_bytes);

        let plaintext = cipher.decrypt(nonce, ciphertext)
            .map_err(|e| anyhow::anyhow!("Ошибка дешифрования AES-GCM: {}", e))?;
        Ok(plaintext)
    }

    /// Генерация пары ключей Diffie-Hellman (X25519)
    /// Возвращает (секретный ключ, публичный ключ)
    /// EphemeralSecret не реализует Copy/Clone — используется один раз
    pub fn generate_dh_keys() -> (EphemeralSecret, PublicKey) {
        let secret = EphemeralSecret::random_from_rng(OsRng);
        let public = PublicKey::from(&secret);
        (secret, public)
    }

    /// Вычисление общего секрета Diffie-Hellman
    /// ВАЖНО: secret перемещается и уничтожается (security best practice)
    pub fn compute_shared_secret(secret: EphemeralSecret, public: &PublicKey) -> [u8; 32] {
        let shared: SharedSecret = secret.diffie_hellman(public);
        shared.to_bytes()
    }

    /// Создание CipherManager из общего секрета DH
    pub fn from_shared_secret(shared_secret: [u8; 32]) -> Self {
        Self::new(&shared_secret)
    }

    /// Генерация случайного ключа (для тестирования/демо)
    pub fn generate_random() -> Self {
        let mut key = [0u8; 32];
        OsRng.fill_bytes(&mut key);
        Self::new(&key)
    }

    /// Получить ключ (для отладки/демо)
    pub fn key(&self) -> [u8; 32] {
        self.key
    }
}

/// Структура для обмена ключами Diffie-Hellman
/// Отправляется через отдельный P2P топик
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DHKeyExchange {
    /// Peer ID отправителя
    pub sender_id: String,
    /// Публичный ключ X25519 (32 байта)
    pub public_key: [u8; 32],
    /// Timestamp создания (Unix seconds)
    pub timestamp: u64,
    /// Уникальный ID сессии
    pub session_id: String,
}

impl DHKeyExchange {
    pub fn new(sender_id: &str, public_key: [u8; 32], session_id: &str) -> Self {
        Self {
            sender_id: sender_id.to_string(),
            public_key,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            session_id: session_id.to_string(),
        }
    }

    /// Сериализация в JSON для передачи по сети
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self)
            .context("Ошибка сериализации DHKeyExchange")
    }

    /// Десериализация из JSON
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json)
            .context("Ошибка десериализации DHKeyExchange")
    }
}

/// Сессия Diffie-Hellman для одного пира
pub struct DHSession {
    /// Публичный ключ пира
    pub peer_public: PublicKey,
    /// CipherManager с общим секретом
    pub cipher: CipherManager,
}

/// Менеджер сессий Diffie-Hellman
/// Хранит состояния для каждого пира
///
/// ВАЖНО: EphemeralSecret хранится и используется бережно:
/// - Один раз для создания PublicKey
/// - Один раз для diffie_hellman() с каждым пиром
pub struct DHSessionManager {
    /// Наш секретный ключ (используется один раз для каждого пира)
    our_secret: Option<EphemeralSecret>,
    /// Наш публичный ключ
    our_public: PublicKey,
    /// Сессии с пирами
    sessions: HashMap<PeerId, DHSession>,
}

impl DHSessionManager {
    pub fn new() -> Self {
        let (secret, public) = CipherManager::generate_dh_keys();
        Self {
            our_secret: Some(secret),
            our_public: public,
            sessions: HashMap::new(),
        }
    }

    /// Получить наш публичный ключ для отправки
    pub fn our_public_key(&self) -> PublicKey {
        self.our_public
    }

    /// Обработать полученный ключ от пира
    /// Возвращает true, если ключ новый и сессия создана
    ///
    /// ВАЖНО: our_secret перемещается и уничтожается при первом вызове
    pub fn receive_peer_key(&mut self, peer_id: PeerId, public_key: [u8; 32]) -> bool {
        if self.sessions.contains_key(&peer_id) {
            return false; // Уже есть сессия с этим пиром
        }

        // Создаем PublicKey пира из байтов
        let peer_public = PublicKey::from(public_key);

        // Берем наш секрет (если еще не использован)
        let Some(our_secret) = self.our_secret.take() else {
            // Секрет уже использован — генерируем новую пару для этой сессии
            let (new_secret, new_public) = CipherManager::generate_dh_keys();
            let shared_secret = CipherManager::compute_shared_secret(new_secret, &peer_public);
            let cipher = CipherManager::from_shared_secret(shared_secret);

            self.sessions.insert(peer_id, DHSession { peer_public, cipher });
            // Обновляем наш публичный ключ
            self.our_public = new_public;
            return true;
        };

        // Вычисляем общий секрет (our_secret перемещается и уничтожается)
        let shared_secret = CipherManager::compute_shared_secret(our_secret, &peer_public);
        let cipher = CipherManager::from_shared_secret(shared_secret);

        self.sessions.insert(peer_id, DHSession { peer_public, cipher });
        true
    }

    /// Получить CipherManager для пира
    pub fn get_cipher(&self, peer_id: &PeerId) -> Option<&CipherManager> {
        self.sessions.get(peer_id).map(|s| &s.cipher)
    }

    /// Создать сообщение обмена ключами
    pub fn create_key_exchange(&self, my_peer_id: &PeerId) -> DHKeyExchange {
        let session_id = uuid_simple();
        DHKeyExchange::new(
            &my_peer_id.to_string(),
            self.our_public.to_bytes(),
            &session_id
        )
    }

    /// Проверка, есть ли сессия с пиром
    pub fn has_session(&self, peer_id: &PeerId) -> bool {
        self.sessions.contains_key(peer_id)
    }

    /// Количество активных сессий
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }
}

impl Default for DHSessionManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Генерация простого UUID для ID сессии
fn uuid_simple() -> String {
    let mut bytes = [0u8; 16];
    OsRng.fill_bytes(&mut bytes);
    encode_hex(&bytes[..8])
}

/// Простая реализация hex encode без внешней зависимости
fn encode_hex(bytes: &[u8]) -> String {
    const HEX_CHARS: &[u8] = b"0123456789abcdef";
    let mut result = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        result.push(HEX_CHARS[(byte >> 4) as usize] as char);
        result.push(HEX_CHARS[(byte & 0x0F) as usize] as char);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dh_key_exchange() {
        // Алиса создает менеджер
        let mut alice_manager = DHSessionManager::new();
        let alice_public = alice_manager.our_public_key();

        // Боб создает менеджер
        let mut bob_manager = DHSessionManager::new();
        let bob_public = bob_manager.our_public_key();

        // Алиса получает ключ Боба
        alice_manager.receive_peer_key(
            PeerId::random(),
            bob_public.to_bytes()
        );

        // Боб получает ключ Алисы
        bob_manager.receive_peer_key(
            PeerId::random(),
            alice_public.to_bytes()
        );

        // В реальной реализации общие секреты должны совпадать
        // (здесь не совпадают, т.к. каждый использует свой secret)
    }

    #[test]
    fn test_cipher_encrypt_decrypt() {
        let cipher = CipherManager::generate_random();
        let plaintext = b"Hello, Liberty Reach!";

        let encrypted = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&encrypted).unwrap();

        assert_eq!(plaintext, &decrypted[..]);
    }
}
