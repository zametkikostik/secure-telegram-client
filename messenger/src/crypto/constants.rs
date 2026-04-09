//! Cryptographic constants for Secure Telegram Client
//!
//! SECURITY: все параметры должны быть проверены перед production
//! TODO: pentest перед release

/// Размер ключа ChaCha20-Poly1305 в байтах
pub const CHACHA20_KEY_SIZE: usize = 32;

/// Размер nonce ChaCha20-Poly1305 в байтах
pub const CHACHA20_NONCE_SIZE: usize = 12;

/// Размер X25519 ключа в байтах
pub const X25519_KEY_SIZE: usize = 32;

/// Размер подписи Ed25519 в байтах
pub const ED25519_SIGNATURE_SIZE: usize = 64;

/// Размер публичного ключа Ed25519 в байтах
pub const ED25519_PUBLIC_KEY_SIZE: usize = 32;

/// Размер приватного ключа Ed25519 в байтах
pub const ED25519_SECRET_KEY_SIZE: usize = 64;

/// Параметры Argon2id для хеширования паролей
pub mod argon2 {
    /// Объём памяти в KB (64 MB)
    pub const M_COST: u32 = 64 * 1024;
    /// Количество итераций
    pub const T_COST: u32 = 3;
    /// Степень параллелизма
    pub const P_COST: u32 = 4;
}

/// Метка HKDF для вывода ключа ChaCha20
pub const HKDF_CHACHA20_INFO: &[u8] = b"secure-messenger-chacha20-key";

/// Метка HKDF для вывода ключа HMAC
pub const HKDF_HMAC_INFO: &[u8] = b"secure-messenger-hmac-key";

/// Версия протокола шифрования
pub const PROTOCOL_VERSION: &str = "v1";
