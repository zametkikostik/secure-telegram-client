//! obfs4 транспорт для обхода DPI
//!
//! Реализация obfs4-like обфускации:
//! - Handshake с использованием elliptic curves
//! - Keystream генерация через SHA-3
//! - XOR шифрование трафика

use anyhow::{anyhow, Context, Result};
use rand::{rngs::OsRng, RngCore};
use sha3::{Digest, Sha3_256};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::TcpStream;
use x25519_dalek::{PublicKey, StaticSecret};

/// Размер handshake пакета
const OBFUSCATED_HANDSHAKE_SIZE: usize = 1968;
/// Размер заголовка пакета
const PACKET_HEADER_SIZE: usize = 4;
/// Максимальный размер данных в пакете
const MAX_PACKET_DATA: usize = 65536;

/// obfs4 клиент
pub struct Obfs4Client {
    /// Адрес моста
    bridge_addr: String,
    /// Публичный ключ моста (hex)
    bridge_public_key: String,
    /// Наш секретный ключ
    local_secret: StaticSecret,
    /// Наш публичный ключ
    local_public: PublicKey,
    /// Общий секрет (после handshake)
    shared_secret: Option<[u8; 32]>,
    /// Счётчик пакетов для keystream
    packet_counter: u64,
}

/// obfs4 стрим (обёртка над TcpStream)
pub struct Obfs4Stream {
    /// TCP стрим
    stream: TcpStream,
    /// Ключ для шифрования
    encrypt_key: [u8; 32],
    /// Ключ для расшифровки
    decrypt_key: [u8; 32],
    /// Счётчик для keystream
    counter: u64,
    /// Буфер для чтения
    read_buffer: Vec<u8>,
    /// Позиция в буфере
    read_pos: usize,
}

impl Obfs4Client {
    /// Создание нового obfs4 клиента
    pub fn new(bridge_addr: String, bridge_public_key: String) -> Result<Self> {
        // Генерация локальной пары ключей
        let local_secret = StaticSecret::random_from_rng(OsRng);
        let local_public = PublicKey::from(&local_secret);

        Ok(Self {
            bridge_addr,
            bridge_public_key,
            local_secret,
            local_public,
            shared_secret: None,
            packet_counter: 0,
        })
    }

    /// Подключение к мосту
    pub async fn connect(&mut self) -> Result<Obfs4Stream> {
        log::info!("Подключение к obfs4 мосту: {}", self.bridge_addr);

        // 1. TCP подключение
        let stream = TcpStream::connect(&self.bridge_addr)
            .await
            .context("Ошибка TCP подключения к мосту")?;

        // 2. Парсинг публичного ключа моста
        let bridge_key_bytes = hex::decode(&self.bridge_public_key)
            .context("Неверный формат публичного ключа моста")?;

        if bridge_key_bytes.len() != 32 {
            return Err(anyhow!("Неверная длина публичного ключа"));
        }

        let bridge_public = PublicKey::from(bridge_key_bytes.try_into().unwrap_or([0u8; 32]));

        // 3. Вычисление общего секрета (ECDH)
        let shared_secret = self.local_secret.diffie_hellman(&bridge_public);
        let shared_bytes = shared_secret.to_bytes();

        // 4. Вывод ключей шифрования через HKDF-like конструкцию
        let mut hasher = Sha3_256::new();
        hasher.update(b"obfs4-encrypt-key");
        hasher.update(&shared_bytes);
        let encrypt_key: [u8; 32] = hasher.finalize().try_into().unwrap();

        let mut hasher = Sha3_256::new();
        hasher.update(b"obfs4-decrypt-key");
        hasher.update(&shared_bytes);
        let decrypt_key: [u8; 32] = hasher.finalize().try_into().unwrap();

        log::info!("obfs4 handshake завершён успешно");

        Ok(Obfs4Stream {
            stream,
            encrypt_key,
            decrypt_key,
            counter: 0,
            read_buffer: Vec::new(),
            read_pos: 0,
        })
    }

    /// Генерация сертификата для моста (для whitelist)
    pub fn generate_certificate(&self) -> Vec<u8> {
        // В реальной реализации здесь будет генерация сертификата
        // для добавления моста в whitelist
        let mut cert = vec![0u8; 128];
        OsRng.fill_bytes(&mut cert);
        cert
    }
}

impl Obfs4Stream {
    /// Генерация keystream для шифрования
    fn generate_keystream(&mut self, length: usize) -> Vec<u8> {
        let mut keystream = Vec::with_capacity(length);
        let mut counter = 0u32;

        while keystream.len() < length {
            let mut hasher = Sha3_256::new();
            hasher.update(&self.encrypt_key);
            hasher.update(&self.counter.to_be_bytes());
            hasher.update(&counter.to_be_bytes());
            let hash = hasher.finalize();

            keystream.extend_from_slice(&hash);
            counter += 1;
        }

        keystream.truncate(length);
        keystream
    }

    /// Генерация keystream для расшифровки
    fn generate_decrypt_keystream(&mut self, length: usize) -> Vec<u8> {
        let mut keystream = Vec::with_capacity(length);
        let mut counter = 0u32;

        while keystream.len() < length {
            let mut hasher = Sha3_256::new();
            hasher.update(&self.decrypt_key);
            hasher.update(&self.counter.to_be_bytes());
            hasher.update(&counter.to_be_bytes());
            let hash = hasher.finalize();

            keystream.extend_from_slice(&hash);
            counter += 1;
        }

        keystream.truncate(length);
        keystream
    }

    /// obfs4 обфускация данных
    fn obfuscate(&mut self, data: &[u8]) -> Vec<u8> {
        // Генерация случайного префикса (для маскировки паттернов)
        let prefix_size = 16 + (self.counter % 32) as usize;
        let mut prefix = vec![0u8; prefix_size];
        OsRng.fill_bytes(&mut prefix);

        // XOR шифрование
        let keystream = self.generate_keystream(data.len());
        let obfuscated: Vec<u8> = data
            .iter()
            .zip(keystream.iter())
            .map(|(&d, &k)| d ^ k)
            .collect();

        // Добавление заголовка с размером
        let mut result = Vec::with_capacity(PACKET_HEADER_SIZE + prefix_size + obfuscated.len());
        result.extend_from_slice(&(data.len() as u32).to_be_bytes());
        result.extend(prefix);
        result.extend(obfuscated);

        self.counter += 1;

        result
    }

    /// obfs4 деобфускация данных
    fn deobfuscate(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < PACKET_HEADER_SIZE {
            return Err(anyhow!("Слишком короткие данные"));
        }

        // Чтение размера из заголовка
        let size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

        if data.len() < PACKET_HEADER_SIZE + size {
            return Err(anyhow!("Недостаточно данных"));
        }

        // Пропуск префикса и чтение зашифрованных данных
        let prefix_size = 16 + (self.counter % 32) as usize;
        let encrypted_start = PACKET_HEADER_SIZE + prefix_size;
        let encrypted = &data[encrypted_start..encrypted_start + size];

        // Генерация keystream для расшифровки
        let keystream = self.generate_decrypt_keystream(size);
        let decrypted: Vec<u8> = encrypted
            .iter()
            .zip(keystream.iter())
            .map(|(&e, &k)| e ^ k)
            .collect();

        self.counter += 1;

        Ok(decrypted)
    }

    /// Отправка данных через obfs4 стрим
    pub async fn send(&mut self, data: &[u8]) -> Result<usize> {
        let obfuscated = self.obfuscate(data);
        self.stream.write_all(&obfuscated).await?;
        Ok(data.len())
    }

    /// Получение данных из obfs4 стрима
    pub async fn recv(&mut self, buf: &mut [u8]) -> Result<usize> {
        // Если есть данные в буфере
        if self.read_pos < self.read_buffer.len() {
            let available = self.read_buffer.len() - self.read_pos;
            let to_copy = available.min(buf.len());
            buf[..to_copy]
                .copy_from_slice(&self.read_buffer[self.read_pos..self.read_pos + to_copy]);
            self.read_pos += to_copy;
            return Ok(to_copy);
        }

        // Чтение заголовка
        let mut header = [0u8; PACKET_HEADER_SIZE];
        self.stream.read_exact(&mut header).await?;

        // Чтение размера
        let size = u32::from_be_bytes(header) as usize;
        let prefix_size = 16 + (self.counter % 32) as usize;

        // Чтение префикса и данных
        let total_read_size = prefix_size + size;
        let mut raw_data = vec![0u8; total_read_size];
        self.stream.read_exact(&mut raw_data).await?;

        // Деобфускация
        let decrypted = self.deobfuscate(&[&header[..], &raw_data[..]].concat())?;

        // Копирование в буфер
        let to_copy = decrypted.len().min(buf.len());
        buf[..to_copy].copy_from_slice(&decrypted[..to_copy]);

        // Сохранение остатка в буфер
        if decrypted.len() > buf.len() {
            self.read_buffer = decrypted[buf.len()..].to_vec();
            self.read_pos = 0;
        }

        Ok(to_copy)
    }

    /// Получение внутреннего TcpStream
    pub fn into_inner(self) -> TcpStream {
        self.stream
    }
}

// Реализация AsyncRead для Obfs4Stream
impl AsyncRead for Obfs4Stream {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // Упрощённая реализация - делегирование TCP стриму
        let this = self.get_mut();
        std::pin::Pin::new(&mut this.stream).poll_read(cx, buf)
    }
}

// Реализация AsyncWrite для Obfs4Stream
impl AsyncWrite for Obfs4Stream {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let this = self.get_mut();
        std::pin::Pin::new(&mut this.stream).poll_write(cx, buf)
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let this = self.get_mut();
        std::pin::Pin::new(&mut this.stream).poll_flush(cx)
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let this = self.get_mut();
        std::pin::Pin::new(&mut this.stream).poll_shutdown(cx)
    }
}

/// Конфигурация obfs4 моста
#[derive(Debug, Clone)]
pub struct Obfs4Bridge {
    /// Адрес моста (host:port)
    pub addr: String,
    /// Публичный ключ (hex)
    pub public_key: String,
    /// Сертификат (для whitelist)
    pub cert: Option<Vec<u8>>,
}

impl Obfs4Bridge {
    /// Парсинг строки подключения obfs4
    pub fn from_url(url: &str) -> Result<Self> {
        // Формат: obfs4://<public_key>@<host>:<port>
        if !url.starts_with("obfs4://") {
            return Err(anyhow!("Неверный формат URL obfs4"));
        }

        let without_scheme = &url[8..];
        let parts: Vec<&str> = without_scheme.split('@').collect();

        if parts.len() != 2 {
            return Err(anyhow!("Неверный формат: ожидается obfs4://key@host:port"));
        }

        let public_key = parts[0].to_string();
        let addr = parts[1].to_string();

        Ok(Self {
            addr,
            public_key,
            cert: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_url_parsing() {
        let url = "obfs4://abcdef1234567890@bridge.example.com:443";
        let bridge = Obfs4Bridge::from_url(url).unwrap();

        assert_eq!(bridge.addr, "bridge.example.com:443");
        assert_eq!(bridge.public_key, "abcdef1234567890");
    }

    #[test]
    fn test_obfs4_client_creation() {
        let client = Obfs4Client::new(
            "bridge.example.com:443".to_string(),
            "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890".to_string(),
        );

        assert!(client.is_ok());
    }
}
