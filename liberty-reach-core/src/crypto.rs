//! Cryptography Module
//! 
//! Многослойное шифрование, post-quantum криптография, стеганография.

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Sha512};
use sha3::Sha3_256;
use blake3::Hasher as Blake3Hasher;
use hmac::{Hmac, Mac};
use hkdf::Hkdf;
use argon2::{Argon2, PasswordHash, PasswordHasher, password_hash::SaltString};
use rand::{Rng, RngCore, thread_rng};
use aes_gcm::{Aes256Gcm, KeyInit, Nonce, aead::Aead};
use chacha20poly1305::{ChaCha20Poly1305, KeyInit as ChaChaKeyInit, XNonce};
use ed25519_dalek::{SigningKey as EdSigningKey, VerifyingKey, Signature, Signer, Verifier};
use x25519_dalek::{PublicKey as XPublicKey};
use pqcrypto_kyber::kyber1024;
use pqcrypto_traits::kem::{PublicKey as KemPublicKey, SecretKey as KemSecretKey, SharedSecret, Ciphertext};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use image::{DynamicImage, GenericImageView, GenericImage, Rgba};
use std::path::Path;

// ============================================================================
// АЛГОРИТМЫ ШИФРОВАНИЯ
// ============================================================================

/// Алгоритм симметричного шифрования
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EncryptionAlgorithm {
    /// AES-256-GCM
    Aes256Gcm,
    /// ChaCha20-Poly1305
    ChaCha20Poly1305,
}

impl Default for EncryptionAlgorithm {
    fn default() -> Self {
        Self::Aes256Gcm
    }
}

// ============================================================================
// AES-256-GCM
// ============================================================================

/// AES-256-GCM шифрование
pub struct AesGcmEncrypter {
    cipher: Aes256Gcm,
}

impl AesGcmEncrypter {
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            cipher: Aes256Gcm::new_from_slice(key).expect("key size is 32 bytes"),
        }
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|e| anyhow::anyhow!("AES-GCM encryption failed: {}", e))?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
        if nonce.len() != 12 {
            bail!("Invalid nonce length");
        }

        let nonce = Nonce::from_slice(nonce);
        let plaintext = self.cipher
            .decrypt(nonce, ciphertext.as_ref())
            .map_err(|e| anyhow::anyhow!("AES-GCM decryption failed: {}", e))?;

        Ok(plaintext)
    }
}

// ============================================================================
/// ChaCha20-Poly1305
pub struct ChaChaEncrypter {
    cipher: ChaCha20Poly1305,
}

impl ChaChaEncrypter {
    pub fn new(key: &[u8; 32]) -> Self {
        Self {
            cipher: ChaCha20Poly1305::new_from_slice(key).expect("key size is 32 bytes"),
        }
    }
    
    pub fn encrypt(&self, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut nonce_bytes = [0u8; 12];
        thread_rng().fill_bytes(&mut nonce_bytes);

        let ciphertext = self.cipher
            .encrypt(&nonce_bytes.into(), plaintext)
            .map_err(|e| anyhow::anyhow!("ChaCha20-Poly1305 encryption failed: {}", e))?;

        Ok((ciphertext, nonce_bytes.to_vec()))
    }

    pub fn decrypt(&self, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
        if nonce.len() != 12 {
            bail!("Invalid nonce length");
        }

        let nonce_array: [u8; 12] = nonce.try_into().map_err(|_| anyhow!("Invalid nonce length"))?;
        let plaintext = self.cipher
            .decrypt(&nonce_array.into(), ciphertext)
            .map_err(|e| anyhow::anyhow!("ChaCha20-Poly1305 decryption failed: {}", e))?;

        Ok(plaintext)
    }
}

// ============================================================================
// POST-QUANTUM KEM (Kyber1024)
// ============================================================================

/// Post-quantum Key Encapsulation Mechanism
pub struct Kyber1024Kem {
    public_key: Option<kyber1024::PublicKey>,
    secret_key: Option<kyber1024::SecretKey>,
}

impl Kyber1024Kem {
    /// Создать новую пару ключей
    pub fn keypair() -> Result<(Self, Vec<u8>)> {
        let (pk, sk) = kyber1024::keypair();
        
        let kem = Self {
            public_key: Some(pk),
            secret_key: Some(sk),
        };
        
        let pk_bytes = pk.as_bytes().to_vec();
        Ok((kem, pk_bytes))
    }
    
    /// Создать из публичного ключа (для инкапсуляции)
    pub fn from_public_key(pk_bytes: &[u8]) -> Result<Self> {
        let pk = kyber1024::PublicKey::from_bytes(pk_bytes)
            .context("Invalid Kyber1024 public key")?;
        
        Ok(Self {
            public_key: Some(pk),
            secret_key: None,
        })
    }
    
    /// Создать из секретного ключа (для деинкапсуляции)
    pub fn from_secret_key(sk_bytes: &[u8]) -> Result<Self> {
        let sk = kyber1024::SecretKey::from_bytes(sk_bytes)
            .context("Invalid Kyber1024 secret key")?;
        
        Ok(Self {
            public_key: None,
            secret_key: Some(sk),
        })
    }
    
    /// Инкапсуляция (создание общего секрета)
    pub fn encapsulate(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let pk = self.public_key.as_ref()
            .ok_or_else(|| anyhow!("No public key available"))?;
        
        let (ct, ss) = kyber1024::encapsulate(pk);
        
        Ok((ct.as_bytes().to_vec(), ss.as_bytes().to_vec()))
    }
    
    /// Деинкапсуляция (получение общего секрета)
    pub fn decapsulate(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let sk = self.secret_key.as_ref()
            .ok_or_else(|| anyhow!("No secret key available"))?;

        let ct = pqcrypto_kyber::kyber1024::Ciphertext::from_bytes(ciphertext)
            .map_err(|_| anyhow!("Invalid ciphertext"))?;

        let _ss = pqcrypto_kyber::kyber1024::decapsulate(&ct, sk);

        Ok(_ss.as_bytes().to_vec())
    }
    
    /// Экспорт публичного ключа
    pub fn export_public_key(&self) -> Result<Vec<u8>> {
        let pk = self.public_key.as_ref()
            .ok_or_else(|| anyhow!("No public key available"))?;
        
        Ok(pk.as_bytes().to_vec())
    }
    
    /// Экспорт секретного ключа
    pub fn export_secret_key(&self) -> Result<Vec<u8>> {
        let sk = self.secret_key.as_ref()
            .ok_or_else(|| anyhow!("No secret key available"))?;
        
        Ok(sk.as_bytes().to_vec())
    }
}

// ============================================================================
// ED25519 ПОДПИСИ
// ============================================================================

/// Ed25519 подписи
pub struct Ed25519Signer {
    signing_key: EdSigningKey,
    verifying_key: VerifyingKey,
}

impl Ed25519Signer {
    /// Создать новую пару ключей
    pub fn new() -> Self {
        let mut key_bytes = [0u8; 32];
        thread_rng().fill_bytes(&mut key_bytes);
        let signing_key = EdSigningKey::from_bytes(&key_bytes);
        let verifying_key = signing_key.verifying_key();

        Self {
            signing_key,
            verifying_key,
        }
    }
    
    /// Создать из байтов секретного ключа
    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != 64 {
            bail!("Ed25519 secret key must be 64 bytes");
        }
        
        let signing_key = EdSigningKey::from_bytes(bytes[..32].try_into()?);
        let verifying_key = signing_key.verifying_key();
        
        Ok(Self {
            signing_key,
            verifying_key,
        })
    }
    
    /// Подписать сообщение
    pub fn sign(&self, message: &[u8]) -> Vec<u8> {
        let signature = self.signing_key.sign(message);
        signature.to_bytes().to_vec()
    }
    
    /// Проверить подпись
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<()> {
        let sig = Signature::from_slice(signature)
            .context("Invalid signature format")?;
        
        self.verifying_key
            .verify(message, &sig)
            .context("Signature verification failed")?;
        
        Ok(())
    }
    
    /// Экспорт публичного ключа
    pub fn export_public_key(&self) -> Vec<u8> {
        self.verifying_key.as_bytes().to_vec()
    }
    
    /// Экспорт секретного ключа
    pub fn export_secret_key(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(64);
        bytes.extend_from_slice(self.signing_key.as_bytes());
        bytes.extend_from_slice(self.verifying_key.as_bytes());
        bytes
    }
}

impl Default for Ed25519Signer {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// X25519 KEY EXCHANGE
// ============================================================================

/// X25519 Key Exchange
pub struct X25519KeyExchange {
    secret_key: [u8; 32],
    public_key: XPublicKey,
}

impl X25519KeyExchange {
    /// Создать новую пару ключей
    pub fn new() -> Self {
        let mut csprng = thread_rng();
        let mut secret_key = [0u8; 32];
        csprng.fill_bytes(&mut secret_key);
        let public_key = XPublicKey::from(secret_key);

        Self {
            secret_key,
            public_key,
        }
    }

    /// Получить публичный ключ
    pub fn public_key(&self) -> XPublicKey {
        self.public_key
    }

    /// Вычислить общий секрет
    pub fn derive_shared(&self, peer_public: &XPublicKey) -> [u8; 32] {
        let shared = x25519_dalek::x25519(self.secret_key, peer_public.to_bytes());
        shared
    }

    /// Экспорт секретного ключа
    pub fn export_secret_key(&self) -> [u8; 32] {
        self.secret_key
    }

    /// Импорт из публичного ключа
    pub fn from_public_key(bytes: [u8; 32]) -> Self {
        Self {
            secret_key: [0u8; 32],
            public_key: XPublicKey::from(bytes),
        }
    }
}

impl Default for X25519KeyExchange {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ХЭШИРОВАНИЕ
// ============================================================================

/// Хэш-функции
pub struct HashFunctions;

impl HashFunctions {
    /// SHA-256
    pub fn sha256(data: &[u8]) -> [u8; 32] {
        use sha2::Digest;
        let mut hasher = Sha256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
    
    /// SHA-512
    pub fn sha512(data: &[u8]) -> [u8; 64] {
        use sha2::Digest;
        let mut hasher = Sha512::new();
        hasher.update(data);
        hasher.finalize().into()
    }
    
    /// SHA3-256
    pub fn sha3_256(data: &[u8]) -> [u8; 32] {
        use sha3::Digest;
        let mut hasher = Sha3_256::new();
        hasher.update(data);
        hasher.finalize().into()
    }
    
    /// BLAKE3
    pub fn blake3(data: &[u8]) -> [u8; 32] {
        blake3::hash(data).into()
    }
    
    /// HMAC-SHA256
    pub fn hmac_sha256(key: &[u8], data: &[u8]) -> Result<Vec<u8>> {
        type HmacSha256 = Hmac<Sha256>;

        let mut mac = <HmacSha256 as KeyInit>::new_from_slice(key)
            .map_err(|_| anyhow!("Invalid HMAC key"))?;
        mac.update(data);
        Ok(mac.finalize().into_bytes().to_vec())
    }
}

// ============================================================================
// KEY DERIVATION
// ============================================================================

/// Key Derivation Functions
pub struct Kdf;

impl Kdf {
    /// Argon2id для хэширования паролей
    pub fn argon2_hash(password: &str) -> Result<String> {
        let salt = SaltString::generate(&mut thread_rng());
        let argon2 = Argon2::default();
        
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Argon2 hashing failed: {}", e))?;
        
        Ok(password_hash.to_string())
    }
    
    /// Argon2 verify
    pub fn argon2_verify(password: &str, hash: &str) -> Result<bool> {
        use argon2::password_hash::PasswordVerifier;
        
        let parsed_hash = PasswordHash::new(hash)
            .map_err(|_| anyhow!("Invalid password hash format"))?;
        
        Ok(Argon2::default()
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }
    
    /// HKDF для вывода ключей
    pub fn hkdf_derive(ikm: &[u8], salt: &[u8], info: &[u8], length: usize) -> Result<Vec<u8>> {
        let hk = Hkdf::<Sha256>::new(Some(salt), ikm);
        let mut okm = vec![0u8; length];
        
        hk.expand(info, &mut okm)
            .map_err(|_| anyhow!("HKDF expand failed"))?;
        
        Ok(okm)
    }
    
    /// PBKDF2 для совместимости
    pub fn pbkdf2_derive(password: &[u8], salt: &[u8], iterations: u32, length: usize) -> Result<Vec<u8>> {
        use pbkdf2::pbkdf2_hmac;
        use sha2::Sha256;
        
        let mut derived_key = vec![0u8; length];
        pbkdf2_hmac::<Sha256>(password, salt, iterations, &mut derived_key);
        
        Ok(derived_key)
    }
}

// ============================================================================
// STEGANOGRAPHY (LSB)
// ============================================================================

/// LSB Steganography
pub struct LsbSteganography;

impl LsbSteganography {
    /// Скрыть сообщение в изображении
    pub fn hide_message(image: &mut DynamicImage, message: &[u8]) -> Result<()> {
        let (width, height) = image.dimensions();
        let max_capacity = (width * height * 4) as usize / 8;
        
        if message.len() > max_capacity {
            bail!("Message too large for this image");
        }
        
        // Добавляем длину сообщения (4 байта)
        let mut data = Vec::with_capacity(4 + message.len());
        data.extend_from_slice(&(message.len() as u32).to_le_bytes());
        data.extend_from_slice(message);
        
        let mut bit_index = 0;
        let total_bits = data.len() * 8;
        
        // Получаем мутабельный доступ к буферу
        let buffer = image.as_mut_rgba8().ok_or_else(|| anyhow!("Image format not RGBA"))?;
        
        for y in 0..height {
            for x in 0..width {
                if bit_index >= total_bits {
                    return Ok(());
                }

                for channel in 0..4 {
                    if bit_index >= total_bits {
                        return Ok(());
                    }

                    let byte_index = bit_index / 8;
                    let bit_position = 7 - (bit_index % 8);
                    let bit = (data[byte_index] >> bit_position) & 1;

                    let pixel = buffer.get_pixel_mut(x, y);
                    let channel_value = match channel {
                        0 => pixel[0],
                        1 => pixel[1],
                        2 => pixel[2],
                        3 => pixel[3],
                        _ => unreachable!(),
                    };

                    let new_value = (channel_value & 0xFE) | bit;
                    pixel[channel as usize] = new_value;

                    bit_index += 1;
                }
            }
        }

        Ok(())
    }
    
    /// Извлечь сообщение из изображения
    pub fn extract_message(image: &DynamicImage) -> Result<Vec<u8>> {
        let (width, height) = image.dimensions();
        
        // Извлекаем первые 32 бита для длины
        let mut length_bits = Vec::with_capacity(32);
        let mut bit_index = 0;
        
        'length_loop: for y in 0..height {
            for x in 0..width {
                if bit_index >= 32 {
                    break 'length_loop;
                }
                
                let pixel = image.get_pixel(x, y);
                
                for channel in 0..4 {
                    if bit_index >= 32 {
                        break 'length_loop;
                    }
                    
                    let channel_value = match channel {
                        0 => pixel[0],
                        1 => pixel[1],
                        2 => pixel[2],
                        3 => pixel[3],
                        _ => unreachable!(),
                    };
                    
                    length_bits.push(channel_value & 1);
                    bit_index += 1;
                }
            }
        }
        
        // Конвертируем биты в длину
        let mut length_bytes = [0u8; 4];
        for (i, bit) in length_bits.iter().enumerate() {
            let byte_index = i / 8;
            let bit_position = 7 - (i % 8);
            length_bytes[byte_index] |= (*bit as u8) << bit_position;
        }
        
        let message_length = u32::from_le_bytes(length_bytes) as usize;
        
        // Извлекаем сообщение
        let mut message = Vec::with_capacity(message_length);
        let mut current_byte = 0u8;
        let mut bits_in_byte = 0;
        bit_index = 0;
        let total_bits = 32 + message_length * 8;
        
        for y in 0..height {
            for x in 0..width {
                if bit_index >= total_bits {
                    break;
                }
                
                let pixel = image.get_pixel(x, y);
                
                for channel in 0..4 {
                    if bit_index >= total_bits {
                        break;
                    }
                    
                    // Пропускаем биты длины
                    if bit_index >= 32 {
                        let bit = match channel {
                            0 => pixel[0] & 1,
                            1 => pixel[1] & 1,
                            2 => pixel[2] & 1,
                            3 => pixel[3] & 1,
                            _ => unreachable!(),
                        };
                        
                        current_byte = (current_byte << 1) | bit;
                        bits_in_byte += 1;
                        
                        if bits_in_byte == 8 {
                            message.push(current_byte);
                            current_byte = 0;
                            bits_in_byte = 0;
                        }
                    }
                    
                    bit_index += 1;
                }
            }
        }
        
        Ok(message)
    }
    
    /// Скрыть сообщение в файле изображения
    pub fn hide_message_to_file<P: AsRef<Path>>(
        image_path: P,
        output_path: P,
        message: &[u8],
    ) -> Result<()> {
        let mut image = image::open(image_path)
            .context("Failed to open image")?;
        
        Self::hide_message(&mut image, message)?;
        
        image.save(output_path)
            .context("Failed to save image")?;
        
        Ok(())
    }
    
    /// Извлечь сообщение из файла изображения
    pub fn extract_message_from_file<P: AsRef<Path>>(image_path: P) -> Result<Vec<u8>> {
        let image = image::open(image_path)
            .context("Failed to open image")?;
        
        Self::extract_message(&image)
    }
}

// ============================================================================
// КРИПТО-КОНТЕЙНЕР
// ============================================================================

/// Универсальный крипто-контейнер для операций
pub struct CryptoContainer {
    aes_key: Option<[u8; 32]>,
    chacha_key: Option<[u8; 32]>,
    kyber: Option<Kyber1024Kem>,
    ed25519: Option<Ed25519Signer>,
    x25519: Option<X25519KeyExchange>,
}

impl CryptoContainer {
    pub fn new() -> Self {
        Self {
            aes_key: None,
            chacha_key: None,
            kyber: None,
            ed25519: None,
            x25519: None,
        }
    }
    
    /// Установить AES ключ
    pub fn set_aes_key(&mut self, key: [u8; 32]) {
        self.aes_key = Some(key);
    }
    
    /// Установить ChaCha ключ
    pub fn set_chacha_key(&mut self, key: [u8; 32]) {
        self.chacha_key = Some(key);
    }
    
    /// Инициализировать Ed25519
    pub fn init_ed25519(&mut self) {
        self.ed25519 = Some(Ed25519Signer::new());
    }
    
    /// Инициализировать X25519
    pub fn init_x25519(&mut self) {
        self.x25519 = Some(X25519KeyExchange::new());
    }
    
    /// Инициализировать Kyber1024
    pub fn init_kyber(&mut self) -> Result<Vec<u8>> {
        let (kem, pk) = Kyber1024Kem::keypair()?;
        self.kyber = Some(kem);
        Ok(pk)
    }
    
    /// Шифровать сообщение
    pub fn encrypt(&self, algorithm: EncryptionAlgorithm, plaintext: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        match algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let key = self.aes_key.ok_or_else(|| anyhow!("AES key not set"))?;
                let encrypter = AesGcmEncrypter::new(&key);
                encrypter.encrypt(plaintext)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let key = self.chacha_key.ok_or_else(|| anyhow!("ChaCha key not set"))?;
                let encrypter = ChaChaEncrypter::new(&key);
                encrypter.encrypt(plaintext)
            }
        }
    }
    
    /// Дешифровать сообщение
    pub fn decrypt(&self, algorithm: EncryptionAlgorithm, ciphertext: &[u8], nonce: &[u8]) -> Result<Vec<u8>> {
        match algorithm {
            EncryptionAlgorithm::Aes256Gcm => {
                let key = self.aes_key.ok_or_else(|| anyhow!("AES key not set"))?;
                let encrypter = AesGcmEncrypter::new(&key);
                encrypter.decrypt(ciphertext, nonce)
            }
            EncryptionAlgorithm::ChaCha20Poly1305 => {
                let key = self.chacha_key.ok_or_else(|| anyhow!("ChaCha key not set"))?;
                let encrypter = ChaChaEncrypter::new(&key);
                encrypter.decrypt(ciphertext, nonce)
            }
        }
    }
    
    /// Подписать сообщение
    pub fn sign(&self, message: &[u8]) -> Result<Vec<u8>> {
        let signer = self.ed25519.as_ref()
            .ok_or_else(|| anyhow!("Ed25519 not initialized"))?;
        
        Ok(signer.sign(message))
    }
    
    /// Проверить подпись
    pub fn verify(&self, message: &[u8], signature: &[u8]) -> Result<()> {
        let signer = self.ed25519.as_ref()
            .ok_or_else(|| anyhow!("Ed25519 not initialized"))?;
        
        signer.verify(message, signature)
    }
    
    /// Получить X25519 публичный ключ
    pub fn x25519_public_key(&self) -> Result<XPublicKey> {
        let kex = self.x25519.as_ref()
            .ok_or_else(|| anyhow!("X25519 not initialized"))?;
        
        Ok(kex.public_key())
    }
    
    /// Вычислить общий секрет X25519
    pub fn x25519_derive_shared(&self, peer_public: &XPublicKey) -> Result<[u8; 32]> {
        let kex = self.x25519.as_ref()
            .ok_or_else(|| anyhow!("X25519 not initialized"))?;
        
        Ok(kex.derive_shared(peer_public))
    }
    
    /// Kyber инкапсуляция
    pub fn kyber_encapsulate(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let kem = self.kyber.as_ref()
            .ok_or_else(|| anyhow!("Kyber not initialized"))?;
        
        kem.encapsulate()
    }
    
    /// Kyber деинкапсуляция
    pub fn kyber_decapsulate(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        let kem = self.kyber.as_ref()
            .ok_or_else(|| anyhow!("Kyber not initialized"))?;
        
        kem.decapsulate(ciphertext)
    }
}

impl Default for CryptoContainer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_aes_gcm_encrypt_decrypt() {
        let key = [42u8; 32];
        let encrypter = AesGcmEncrypter::new(&key);
        
        let plaintext = b"Hello, Liberty Reach!";
        let (ciphertext, nonce) = encrypter.encrypt(plaintext).unwrap();
        
        let decrypter = AesGcmEncrypter::new(&key);
        let decrypted = decrypter.decrypt(&ciphertext, &nonce).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
    
    #[test]
    fn test_chacha_encrypt_decrypt() {
        let key = [42u8; 32];
        let encrypter = ChaChaEncrypter::new(&key);
        
        let plaintext = b"Hello, Liberty Reach!";
        let (ciphertext, nonce) = encrypter.encrypt(plaintext).unwrap();
        
        let decrypter = ChaChaEncrypter::new(&key);
        let decrypted = decrypter.decrypt(&ciphertext, &nonce).unwrap();
        
        assert_eq!(plaintext, decrypted.as_slice());
    }
    
    #[test]
    fn test_ed25519_sign_verify() {
        let signer = Ed25519Signer::new();
        let message = b"Hello, Liberty Reach!";
        
        let signature = signer.sign(message);
        assert!(signer.verify(message, &signature).is_ok());
        
        let bad_message = b"Bad message";
        assert!(signer.verify(bad_message, &signature).is_err());
    }
    
    #[test]
    fn test_sha256_hash() {
        let data = b"Hello, Liberty Reach!";
        let hash = HashFunctions::sha256(data);
        
        assert_eq!(hash.len(), 32);
        
        // Проверка детерминированности
        let hash2 = HashFunctions::sha256(data);
        assert_eq!(hash, hash2);
    }
    
    #[test]
    fn test_argon2_hash_verify() {
        let password = "super_secret_password_123";
        let hash = Kdf::argon2_hash(password).unwrap();
        
        assert!(Kdf::argon2_verify(password, &hash).unwrap());
        assert!(!Kdf::argon2_verify("wrong_password", &hash).unwrap());
    }
}
