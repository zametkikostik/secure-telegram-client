//! Симметричное шифрование XChaCha20-Poly1305
//! 
//! XChaCha20-Poly1305 обеспечивает конфиденциальность и аутентификацию данных.
//! Использует 256-битный ключ и 192-битный nonce.

use anyhow::{Result, anyhow};
use chacha20poly1305::{XChaCha20Poly1305, KeyInit, XNonce};
use chacha20poly1305::aead::Aead;
use rand::rngs::OsRng;
use rand::RngCore;

/// Размер ключа XChaCha20-Poly1305
pub const XCHACHA_KEY_SIZE: usize = 32;
/// Размер nonce
pub const XCHACHA_NONCE_SIZE: usize = 24;
/// Размер тега аутентификации
pub const XCHACHA_TAG_SIZE: usize = 16;

/// Шифр XChaCha20-Poly1305
pub struct XChaChaCipher {
    key: [u8; XCHACHA_KEY_SIZE],
}

impl XChaChaCipher {
    /// Создание нового шифра с ключом
    pub fn new(key: [u8; XCHACHA_KEY_SIZE]) -> Self {
        Self { key }
    }

    /// Генерация случайного ключа
    pub fn generate_key() -> [u8; XCHACHA_KEY_SIZE] {
        let mut key = [0u8; XCHACHA_KEY_SIZE];
        OsRng.fill_bytes(&mut key);
        key
    }

    /// Шифрование данных
    /// Возвращает (nonce, ciphertext)
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        log::debug!("Шифрование XChaCha20-Poly1305...");

        let cipher = XChaCha20Poly1305::new_from_slice(&self.key)
            .map_err(|e| anyhow!("Ошибка инициализации шифра: {}", e))?;

        // Генерация случайного nonce (24 байта для XChaCha)
        let mut nonce_bytes = [0u8; XCHACHA_NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);
        let nonce = XNonce::from_slice(&nonce_bytes);

        // Шифрование
        let ciphertext = cipher.encrypt(nonce, plaintext)
            .map_err(|e| anyhow!("Ошибка шифрования: {}", e))?;

        log::debug!("Данные зашифрованы ({} байт)", ciphertext.len());

        Ok((nonce_bytes.to_vec(), ciphertext))
    }

    /// Расшифровка данных
    pub fn decrypt(&self, nonce: &[u8], ciphertext: &[u8]) -> Result<Vec<u8>> {
        if nonce.len() != XCHACHA_NONCE_SIZE {
            return Err(anyhow!("Неверный размер nonce"));
        }

        log::debug!("Расшифровка XChaCha20-Poly1305...");

        let cipher = XChaCha20Poly1305::new_from_slice(&self.key)
            .map_err(|e| anyhow!("Ошибка инициализации шифра: {}", e))?;

        let nonce = XNonce::from_slice(nonce);
        let plaintext = cipher.decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| anyhow!("Ошибка расшифровки: {}", e))?;

        log::debug!("Данные расшифрованы ({} байт)", plaintext.len());

        Ok(plaintext)
    }
}

/// Тестирование модуля
pub fn test() -> Result<()> {
    log::debug!("Тестирование XChaCha20-Poly1305 модуля...");
    
    // Генерация ключа
    let key = XChaChaCipher::generate_key();
    let cipher = XChaChaCipher::new(key);
    
    // Тестовые данные
    let plaintext = b"Hello, Secure Telegram!";
    
    // Шифрование
    let (nonce, ciphertext) = cipher.encrypt(plaintext)?;
    
    // Расшифровка
    let decrypted = cipher.decrypt(&nonce, &ciphertext)?;
    
    // Проверка
    assert_eq!(plaintext.to_vec(), decrypted);
    
    log::debug!("XChaCha20-Poly1305 тест пройден");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_encrypt_decrypt() {
        let key = XChaChaCipher::generate_key();
        let cipher = XChaChaCipher::new(key);
        
        let plaintext = b"Test message for encryption";
        let (nonce, ciphertext) = cipher.encrypt(plaintext).unwrap();
        let decrypted = cipher.decrypt(&nonce, &ciphertext).unwrap();
        
        assert_eq!(plaintext.to_vec(), decrypted);
    }
    
    #[test]
    fn test_different_nonces() {
        let key = XChaChaCipher::generate_key();
        let cipher = XChaChaCipher::new(key);
        
        let plaintext = b"Same message";
        let (nonce1, ct1) = cipher.encrypt(plaintext).unwrap();
        let (nonce2, ct2) = cipher.encrypt(plaintext).unwrap();
        
        // Nonce и ciphertext должны быть разными
        assert_ne!(nonce1, nonce2);
        assert_ne!(ct1, ct2);
        
        // Оба должны расшифровываться одинаково
        let dec1 = cipher.decrypt(&nonce1, &ct1).unwrap();
        let dec2 = cipher.decrypt(&nonce2, &ct2).unwrap();
        assert_eq!(dec1, dec2);
    }
}
