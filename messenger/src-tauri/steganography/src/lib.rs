// Steganography module - hiding encrypted messages in images
// SECURITY: требует аудита перед production
// TODO: pentest перед release
//
// Этот модуль позволяет скрывать зашифрованные сообщения
// в изображениях PNG с использованием LSB steganography

use image::{DynamicImage, GenericImageView, GenericImage};
use rand::Rng;

/// Скрытие данных в изображении (LSB steganography)
pub fn hide_data(image: &mut DynamicImage, data: &[u8]) -> Result<(), StegoError> {
    let (width, height) = image.dimensions();
    let max_bytes = ((width * height * 3) / 8) as usize;

    if data.len() > max_bytes {
        return Err(StegoError::DataTooLarge {
            data_size: data.len(),
            max_size: max_bytes,
        });
    }

    // SECURITY: это упрощённая реализация, требует аудита
    // TODO: реализовать более стойкую стеганографию (с рандомным разбросом бит)

    let mut bit_index = 0;

    for y in 0..height {
        for x in 0..width {
            if bit_index >= data.len() * 8 {
                return Ok(());
            }

            let pixel = image.get_pixel(x, y);
            let mut new_pixel = pixel.clone();

            // Модифицируем LSB каждого канала
            for channel in 0..3 {
                if bit_index < data.len() * 8 {
                    let byte_index = bit_index / 8;
                    let bit_offset = 7 - (bit_index % 8);
                    let bit = (data[byte_index] >> bit_offset) & 1;

                    let channel_value = new_pixel[channel] as u8;
                    let new_value = (channel_value & 0xFE) | bit;
                    new_pixel[channel] = new_value;

                    bit_index += 1;
                }
            }

            image.put_pixel(x, y, new_pixel);
        }
    }

    Ok(())
}

/// Извлечение данных из изображения
pub fn extract_data(image: &DynamicImage, bit_count: usize) -> Result<Vec<u8>, StegoError> {
    let (width, height) = image.dimensions();
    let mut data = vec![0u8; (bit_count + 7) / 8];
    let mut bit_index = 0;

    for y in 0..height {
        for x in 0..width {
            if bit_index >= bit_count {
                return Ok(data);
            }

            let pixel = image.get_pixel(x, y);

            for channel in 0..3 {
                if bit_index < bit_count {
                    let bit = pixel[channel] & 1;
                    let byte_index = bit_index / 8;
                    let bit_offset = 7 - (bit_index % 8);

                    data[byte_index] |= bit << bit_offset;
                    bit_index += 1;
                }
            }
        }
    }

    Ok(data)
}

/// Добавление шума для маскировки факта стеганографии
// SECURITY: требует аудита перед production
pub fn add_noise(image: &mut DynamicImage, noise_level: u8) {
    let mut rng = rand::thread_rng();
    let (width, height) = image.dimensions();

    for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let mut new_pixel = pixel.clone();

            for channel in 0..3 {
                let noise = rng.gen_range(0..noise_level) as i16;
                let value = pixel[channel] as i16 + noise - (noise_level as i16 / 2);
                new_pixel[channel] = value.clamp(0, 255) as u8;
            }

            image.put_pixel(x, y, new_pixel);
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum StegoError {
    #[error("Data too large: {data_size} > {max_size}")]
    DataTooLarge { data_size: usize, max_size: usize },
    #[error("Image error: {0}")]
    ImageError(String),
}
