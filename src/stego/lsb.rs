//! LSB (Least Significant Bit) стенография
//!
//! Встраивание данных в младшие биты пикселей изображения.
//! Изменения незаметны для человеческого глаза.

use anyhow::{anyhow, Result};
use image::{DynamicImage, ImageBuffer, Rgb, Rgba};
use std::path::Path;

/// Максимальное количество бит на канал
const BITS_PER_CHANNEL: usize = 1;
/// Количество каналов в RGB
const CHANNELS: usize = 3;
/// Бит на байт
const BITS_PER_BYTE: usize = 8;

/// LSB стенография
pub struct LsbSteganography {
    /// Количество бит для встраивания на канал
    bits_per_channel: usize,
}

impl LsbSteganography {
    /// Создание нового экземпляра
    pub fn new() -> Self {
        Self {
            bits_per_channel: BITS_PER_CHANNEL,
        }
    }

    /// Встраивание данных в изображение
    pub fn embed(&self, image: &DynamicImage, data: &[u8]) -> Result<DynamicImage> {
        log::debug!("Встраивание {} байт в изображение", data.len());

        // Конвертация в RGB
        let rgb_image = image.to_rgb8();
        let (width, height) = rgb_image.dimensions();

        // Проверка вместимости
        let capacity = self.get_capacity(width, height);
        if data.len() > capacity {
            return Err(anyhow!(
                "Данные ({} байт) не помещаются в изображение (макс. {} байт)",
                data.len(),
                capacity
            ));
        }

        // Создание копии изображения для модификации
        let mut result = rgb_image.clone();

        // Добавление заголовка с размером данных
        let mut full_data = Vec::with_capacity(4 + data.len());
        full_data.extend_from_slice(&(data.len() as u32).to_be_bytes());
        full_data.extend_from_slice(data);

        // Встраивание битов
        let mut bit_index = 0;
        let total_bits = full_data.len() * BITS_PER_BYTE;

        'outer: for y in 0..height {
            for x in 0..width {
                let pixel = result.get_pixel(x, y);
                let mut new_pixel = *pixel;

                for channel in 0..CHANNELS {
                    if bit_index >= total_bits {
                        break 'outer;
                    }

                    let bit = self.get_bit(&full_data, bit_index);
                    new_pixel[channel] = self.set_lsb(new_pixel[channel], bit);

                    bit_index += 1;
                }

                result.put_pixel(x, y, new_pixel);
            }
        }

        log::debug!(
            "Данные встроены (использовано {} из {} байт)",
            data.len(),
            capacity
        );

        Ok(DynamicImage::ImageRgb8(result))
    }

    /// Извлечение данных из изображения
    pub fn extract(&self, image: &DynamicImage) -> Result<Vec<u8>> {
        log::debug!("Извлечение данных из изображения");

        let rgb_image = image.to_rgb8();
        let (width, height) = rgb_image.dimensions();

        // Извлечение заголовка (размер данных)
        let mut header_bits = Vec::new();
        let mut bit_index = 0;

        'header: for y in 0..height {
            for x in 0..width {
                let pixel = rgb_image.get_pixel(x, y);

                for channel in 0..CHANNELS {
                    if bit_index >= 32 {
                        break 'header;
                    }

                    header_bits.push(self.get_lsb(pixel[channel]));
                    bit_index += 1;
                }
            }
        }

        let data_size = self.bits_to_u32(&header_bits)?;
        log::debug!("Размер данных: {} байт", data_size);

        if data_size == 0 {
            return Ok(Vec::new());
        }

        // Проверка вместимости
        let total_bits_needed = (4 + data_size as usize) * BITS_PER_BYTE;
        let total_bits_available = (width * height * CHANNELS as u32) as usize;

        if total_bits_needed > total_bits_available {
            return Err(anyhow!("Недостаточно данных в изображении"));
        }

        // Извлечение всех данных
        let mut all_bits = Vec::new();
        bit_index = 0;
        let total_bits_to_extract = total_bits_needed;

        'extract: for y in 0..height {
            for x in 0..width {
                let pixel = rgb_image.get_pixel(x, y);

                for channel in 0..CHANNELS {
                    if bit_index >= total_bits_to_extract {
                        break 'extract;
                    }

                    all_bits.push(self.get_lsb(pixel[channel]));
                    bit_index += 1;
                }
            }
        }

        // Пропуск заголовка и конвертация в байты
        let data_bits = &all_bits[32..];
        let data = self.bits_to_bytes(data_bits, data_size as usize);

        log::debug!("Данные извлечены ({} байт)", data.len());

        Ok(data)
    }

    /// Встраивание данных из файла в изображение и сохранение
    pub fn embed_to_file<P: AsRef<Path>>(
        &self,
        image_path: P,
        data: &[u8],
        output_path: P,
    ) -> Result<()> {
        let image = image::open(image_path.as_ref())
            .map_err(|e| anyhow!("Ошибка чтения изображения: {}", e))?;

        let stego_image = self.embed(&image, data)?;

        stego_image
            .save(output_path.as_ref())
            .map_err(|e| anyhow!("Ошибка сохранения изображения: {}", e))?;

        Ok(())
    }

    /// Извлечение данных из файла изображения
    pub fn extract_from_file<P: AsRef<Path>>(&self, image_path: P) -> Result<Vec<u8>> {
        let image = image::open(image_path.as_ref())
            .map_err(|e| anyhow!("Ошибка чтения изображения: {}", e))?;

        self.extract(&image)
    }

    /// Получение максимальной вместимости изображения в байтах
    fn get_capacity(&self, width: u32, height: u32) -> usize {
        let total_bits = (width * height * CHANNELS as u32) as usize;
        (total_bits / BITS_PER_BYTE) - 4 // Минус заголовок
    }

    /// Получение бита из байта по индексу
    fn get_bit(&self, data: &[u8], bit_index: usize) -> u8 {
        let byte_index = bit_index / BITS_PER_BYTE;
        let bit_position = 7 - (bit_index % BITS_PER_BYTE);
        (data[byte_index] >> bit_position) & 1
    }

    /// Получение LSB бита
    fn get_lsb(&self, byte: u8) -> u8 {
        byte & 1
    }

    /// Установка LSB бита
    fn set_lsb(&self, byte: u8, bit: u8) -> u8 {
        (byte & 0xFE) | bit
    }

    /// Конвертация битов в u32
    fn bits_to_u32(&self, bits: &[u8]) -> Result<u32> {
        if bits.len() < 32 {
            return Err(anyhow!("Недостаточно бит для конвертации в u32"));
        }

        let mut value: u32 = 0;
        for (i, &bit) in bits.iter().take(32).enumerate() {
            value |= (bit as u32) << (31 - i);
        }

        Ok(value)
    }

    /// Конвертация битов в байты
    fn bits_to_bytes(&self, bits: &[u8], byte_count: usize) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(byte_count);

        for i in 0..byte_count {
            let mut byte: u8 = 0;
            for j in 0..8 {
                if i * 8 + j < bits.len() {
                    byte = (byte << 1) | bits[i * 8 + j];
                } else {
                    byte <<= 1;
                }
            }
            bytes.push(byte);
        }

        bytes
    }
}

impl Default for LsbSteganography {
    fn default() -> Self {
        Self::new()
    }
}

/// Генерация тестового изображения
pub fn generate_test_image(width: u32, height: u32) -> DynamicImage {
    let mut img = ImageBuffer::new(width, height);

    for (x, y, pixel) in img.enumerate_pixels_mut() {
        *pixel = Rgb([
            ((x + y) % 256) as u8,
            ((x * 2) % 256) as u8,
            ((y * 2) % 256) as u8,
        ]);
    }

    DynamicImage::ImageRgb8(img)
}

/// Тестирование модуля
pub fn test() -> Result<()> {
    log::debug!("Тестирование LSB стенографии...");

    let stego = LsbSteganography::new();

    // Создание тестового изображения
    let test_image = generate_test_image(100, 100);

    // Тестовые данные
    let original_data = b"Secret message hidden in image!";

    // Встраивание
    let stego_image = stego.embed(&test_image, original_data)?;

    // Извлечение
    let extracted_data = stego.extract(&stego_image)?;

    // Проверка
    assert_eq!(original_data.to_vec(), extracted_data);

    log::debug!("LSB стенография тест пройден");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_embed_extract() {
        let stego = LsbSteganography::new();
        let test_image = generate_test_image(100, 100);
        let data = b"Test data for steganography";

        let stego_image = stego.embed(&test_image, data).unwrap();
        let extracted = stego.extract(&stego_image).unwrap();

        assert_eq!(data.to_vec(), extracted);
    }

    #[test]
    fn test_empty_data() {
        let stego = LsbSteganography::new();
        let test_image = generate_test_image(50, 50);
        let data = b"";

        let stego_image = stego.embed(&test_image, data).unwrap();
        let extracted = stego.extract(&stego_image).unwrap();

        assert_eq!(data.to_vec(), extracted);
    }

    #[test]
    fn test_capacity() {
        let stego = LsbSteganography::new();
        let test_image = generate_test_image(10, 10);

        // Максимальный размер данных
        let capacity = stego.get_capacity(10, 10);

        // Попытка встроить больше чем возможно
        let large_data = vec![0u8; capacity + 100];
        let result = stego.embed(&test_image, &large_data);

        assert!(result.is_err());
    }

    #[test]
    fn test_large_data() {
        let stego = LsbSteganography::new();
        let test_image = generate_test_image(200, 200);

        // Большие данные
        let data: Vec<u8> = (0..1000).map(|i| (i % 256) as u8).collect();

        let stego_image = stego.embed(&test_image, &data).unwrap();
        let extracted = stego.extract(&stego_image).unwrap();

        assert_eq!(data, extracted);
    }
}
