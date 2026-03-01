//! Obfs4 обфускация
//!
//! Obfs4 маскирует трафик под случайный шум, что делает его неотличимым
//! от обычного зашифрованного соединения.

use anyhow::{anyhow, Result};
use rand::rngs::OsRng;
use rand::RngCore;

/// Размер заголовка obfs4 пакета
const OBFUSCATED_HEADER_SIZE: usize = 4;

/// Obfs4 трансформер
pub struct Obfs4Transformer {
    /// Seed для инициализации
    seed: [u8; 32],
    /// Счётчик пакетов (для предотвращения повторений)
    packet_counter: u64,
}

impl Obfs4Transformer {
    /// Создание нового трансформера со случайным seed
    pub fn new() -> Self {
        let mut seed = [0u8; 32];
        OsRng.fill_bytes(&mut seed);

        log::debug!("Obfs4 трансформер инициализирован");

        Self {
            seed,
            packet_counter: 0,
        }
    }

    /// Создание трансформера с заданным seed
    pub fn with_seed(seed: [u8; 32]) -> Self {
        Self {
            seed,
            packet_counter: 0,
        }
    }

    /// Обфускация данных
    pub fn obfuscate(&mut self, data: &[u8]) -> Vec<u8> {
        log::trace!("Обфускация {} байт", data.len());

        // Генерация случайного префикса (для маскировки паттернов)
        let prefix_size = 16 + (self.packet_counter % 32) as usize;
        let mut prefix = vec![0u8; prefix_size];
        OsRng.fill_bytes(&mut prefix);

        // XOR шифрование с псевдослучайной последовательностью
        let keystream = self.generate_keystream(data.len());
        let obfuscated: Vec<u8> = data
            .iter()
            .zip(keystream.iter())
            .map(|(&d, &k)| d ^ k)
            .collect();

        // Добавление заголовка с размером
        let mut result =
            Vec::with_capacity(OBFUSCATED_HEADER_SIZE + prefix_size + obfuscated.len());
        result.extend_from_slice(&(data.len() as u32).to_be_bytes());
        result.extend(prefix);
        result.extend(obfuscated);

        self.packet_counter += 1;

        result
    }

    /// Деобфускация данных
    pub fn deobfuscate(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        if data.len() < OBFUSCATED_HEADER_SIZE {
            return Err(anyhow!("Слишком короткие данные для деобфускации"));
        }

        // Чтение размера из заголовка
        let size = u32::from_be_bytes([data[0], data[1], data[2], data[3]]) as usize;

        if data.len() < OBFUSCATED_HEADER_SIZE + size {
            return Err(anyhow!("Недостаточно данных"));
        }

        // Пропуск префикса и чтение зашифрованных данных
        let prefix_size = 16 + (self.packet_counter % 32) as usize;
        let encrypted_start = OBFUSCATED_HEADER_SIZE + prefix_size;
        let encrypted = &data[encrypted_start..encrypted_start + size];

        // Генерация того же keystream для расшифровки
        let keystream = self.generate_keystream(size);
        let decrypted: Vec<u8> = encrypted
            .iter()
            .zip(keystream.iter())
            .map(|(&e, &k)| e ^ k)
            .collect();

        self.packet_counter += 1;

        Ok(decrypted)
    }

    /// Генерация псевдослучайной ключевой последовательности
    fn generate_keystream(&self, length: usize) -> Vec<u8> {
        use sha3::{Digest, Sha3_256};

        let mut keystream = Vec::with_capacity(length);
        let mut counter = 0u32;

        while keystream.len() < length {
            let mut hasher = Sha3_256::new();
            hasher.update(&self.seed);
            hasher.update(&self.packet_counter.to_be_bytes());
            hasher.update(&counter.to_be_bytes());
            let hash = hasher.finalize();

            keystream.extend_from_slice(&hash);
            counter += 1;
        }

        keystream.truncate(length);
        keystream
    }
}

impl Default for Obfs4Transformer {
    fn default() -> Self {
        Self::new()
    }
}

/// Обёртка для асинхронного потока данных
pub struct Obfs4Stream {
    transformer: Obfs4Transformer,
}

impl Obfs4Stream {
    pub fn new() -> Self {
        Self {
            transformer: Obfs4Transformer::new(),
        }
    }

    /// Обфускация перед отправкой
    pub fn send(&mut self, data: &[u8]) -> Vec<u8> {
        self.transformer.obfuscate(data)
    }

    /// Деобфускация после получения
    pub fn recv(&mut self, data: &[u8]) -> Result<Vec<u8>> {
        self.transformer.deobfuscate(data)
    }
}

impl Default for Obfs4Stream {
    fn default() -> Self {
        Self::new()
    }
}

/// Тестирование модуля
pub fn test() -> Result<()> {
    log::debug!("Тестирование obfs4 модуля...");

    let mut transformer = Obfs4Transformer::new();

    // Тестовые данные
    let original = b"Hello, this is a secret message!";

    // Обфускация
    let obfuscated = transformer.obfuscate(original);

    // Деобфускация
    let mut recv_transformer = Obfs4Transformer::with_seed(transformer.seed);
    // Синхронизация счётчика
    recv_transformer.packet_counter = transformer.packet_counter - 1;

    let deobfuscated = recv_transformer.deobfuscate(&obfuscated)?;

    // Проверка
    assert_eq!(original.to_vec(), deobfuscated);

    // Проверка что обфусцированные данные отличаются от оригинала
    assert_ne!(original.to_vec(), obfuscated);

    log::debug!("Obfs4 тест пройден");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obfuscate_deobfuscate() {
        let mut transformer = Obfs4Transformer::new();
        let original = b"Test message";

        let obfuscated = transformer.obfuscate(original);
        let mut recv = Obfs4Transformer::with_seed(transformer.seed);
        recv.packet_counter = transformer.packet_counter - 1;

        let deobfuscated = recv.deobfuscate(&obfuscated).unwrap();
        assert_eq!(original.to_vec(), deobfuscated);
    }

    #[test]
    fn test_different_outputs() {
        let mut transformer = Obfs4Transformer::new();
        let data = b"Same data";

        let obs1 = transformer.obfuscate(data);
        let obs2 = transformer.obfuscate(data);

        // Вывод должен быть разным из-за разных префиксов
        assert_ne!(obs1, obs2);

        // Деобфускация первого сообщения
        let mut recv1 = Obfs4Transformer::with_seed(transformer.seed);
        recv1.packet_counter = 0; // Первое сообщение
        let dec1 = recv1.deobfuscate(&obs1).unwrap();

        // Деобфускация второго сообщения
        let mut recv2 = Obfs4Transformer::with_seed(transformer.seed);
        recv2.packet_counter = 1; // Второе сообщение
        let dec2 = recv2.deobfuscate(&obs2).unwrap();

        assert_eq!(dec1, dec2);
        assert_eq!(data.to_vec(), dec1);
    }

    #[test]
    fn test_empty_data() {
        let mut transformer = Obfs4Transformer::new();
        let original = b"";

        let obfuscated = transformer.obfuscate(original);
        let mut recv = Obfs4Transformer::with_seed(transformer.seed);
        recv.packet_counter = transformer.packet_counter - 1;

        let deobfuscated = recv.deobfuscate(&obfuscated).unwrap();
        assert_eq!(original.to_vec(), deobfuscated);
    }
}
