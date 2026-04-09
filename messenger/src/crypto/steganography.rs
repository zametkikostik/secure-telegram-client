//! LSB Steganography for RGB images
//!
//! Hides data in the least significant bits of RGB channels.
//! Simple, fast, but statistically detectable — suitable only
//! as an additional layer of plausible deniability.
//!
//! SECURITY: требует аудита перед production
//! TODO: реализовать более стойкую стеганографию (spread spectrum, DCT-based)
//! TODO: pentest перед release

use image::{ImageBuffer, Rgb};
use thiserror::Error;

// ============================================================================
// Error Types
// ============================================================================

#[derive(Error, Debug)]
pub enum StegoError {
    #[error("Data too large: {data_size} bits > {max_size} bits capacity")]
    DataTooLarge { data_size: usize, max_size: usize },
}

// ============================================================================
// Type aliases
// ============================================================================

/// 8-bit RGB image type
pub type ImageRgb8 = ImageBuffer<Rgb<u8>, Vec<u8>>;

// ============================================================================
// Public API
// ============================================================================

/// Hide data in image using LSB steganography
///
/// # Arguments
/// * `image` — the carrier image (owned, consumed)
/// * `data` — secret data to embed
/// * `bit_count` — number of bits from `data` to embed
///
/// # Returns
/// * `Ok(ImageRgb8)` — image with hidden data
/// * `Err(StegoError)` — if data exceeds image capacity
///
/// SECURITY: modifies image in-place; original data is lost
pub fn hide(
    mut image: ImageRgb8,
    data: &[u8],
    bit_count: usize,
) -> Result<ImageRgb8, StegoError> {
    let (width, height) = image.dimensions();
    let max_bits = (width * height * 3) as usize;

    if bit_count > max_bits {
        return Err(StegoError::DataTooLarge {
            data_size: bit_count,
            max_size: max_bits,
        });
    }

    let mut bit_index: usize = 0;

    'outer: for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);
            let mut new_pixel = *pixel;
            let mut done = false;

            for channel in 0..3 {
                if bit_index >= bit_count {
                    done = true;
                    break;
                }

                let byte_index = bit_index / 8;
                let bit_offset = 7 - (bit_index % 8);
                let bit = (data[byte_index] >> bit_offset) & 1;

                new_pixel[channel] = (new_pixel[channel] & 0xFE) | bit;
                bit_index += 1;
            }

            image.put_pixel(x, y, new_pixel);
            if done {
                break 'outer;
            }
        }
    }

    Ok(image)
}

/// Extract data from image using LSB steganography
///
/// # Arguments
/// * `image` — the steganographic image
/// * `bit_count` — number of bits to extract
///
/// # Returns
/// * `Vec<u8>` — extracted data (padded to full bytes)
pub fn extract(image: &ImageRgb8, bit_count: usize) -> Vec<u8> {
    let (width, height) = image.dimensions();
    let mut data = vec![0u8; (bit_count + 7) / 8];
    let mut bit_index: usize = 0;

    'outer: for y in 0..height {
        for x in 0..width {
            let pixel = image.get_pixel(x, y);

            for channel in 0..3 {
                if bit_index >= bit_count {
                    break 'outer;
                }

                let bit = pixel[channel] & 1;
                let byte_index = bit_index / 8;
                let bit_offset = 7 - (bit_index % 8);

                data[byte_index] |= bit << bit_offset;
                bit_index += 1;
            }
        }
    }

    data
}

/// Calculate the maximum data capacity of an image in bits
pub fn capacity_bits(image: &ImageRgb8) -> usize {
    let (width, height) = image.dimensions();
    (width * height * 3) as usize
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn create_test_image(width: u32, height: u32) -> ImageRgb8 {
        ImageBuffer::from_pixel(width, height, Rgb([128u8, 128u8, 128u8]))
    }

    #[test]
    fn test_hide_extract_roundtrip() {
        let img = create_test_image(100, 100);
        let secret = b"Hello, Steganography!";
        let bit_count = secret.len() * 8;

        let encoded = hide(img, secret, bit_count).unwrap();
        let extracted = extract(&encoded, bit_count);

        assert_eq!(&secret[..], &extracted[..secret.len()]);
    }

    #[test]
    fn test_capacity_calculation() {
        let img = create_test_image(100, 100);
        let cap = capacity_bits(&img);
        // 100 * 100 * 3 = 30000 bits
        assert_eq!(cap, 30_000);
    }

    #[test]
    fn test_data_too_large() {
        let img = create_test_image(10, 10);
        let max_bits = capacity_bits(&img); // 300 bits
        let big_data = vec![0u8; 100];       // 800 bits

        let result = hide(img, &big_data, big_data.len() * 8);
        assert!(result.is_err());
    }

    #[test]
    fn test_binary_data_all_values() {
        let img = create_test_image(50, 50);
        let secret: Vec<u8> = (0..=255).collect();
        let bit_count = secret.len() * 8;

        let encoded = hide(img, &secret, bit_count).unwrap();
        let extracted = extract(&encoded, bit_count);

        assert_eq!(secret, extracted[..secret.len()]);
    }

    #[test]
    fn test_empty_data() {
        let img = create_test_image(10, 10);
        let encoded = hide(img, &[], 0).unwrap();
        let extracted = extract(&encoded, 0);
        assert!(extracted.is_empty());
    }
}
