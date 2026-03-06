// messenger/src/crypto/steganography.rs
//! Стеганография LSB (Least Significant Bit)

use image::{DynamicImage, GenericImageView, GenericImage, Rgba};

pub struct LSBSteganography;

impl LSBSteganography {
    pub fn hide_message(image: &mut DynamicImage, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        let (width, height) = image.dimensions();
        let mut data = image.as_mut_rgba8().unwrap();
        
        let message_bytes = message.as_bytes();
        let mut bit_index = 0;
        
        // Скрываем длину сообщения (первые 32 бита)
        let len = message_bytes.len() as u32;
        for i in 0..32 {
            let bit = ((len >> i) & 1) as u8;
            let pixel_idx = bit_index / 8;
            let channel = bit_index % 8;
            
            let x = (pixel_idx % width) as u32;
            let y = (pixel_idx / width) as u32;
            
            let mut pixel = data.get_pixel(x, y);
            let channel_idx = channel % 4;
            pixel[channel_idx] = (pixel[channel_idx] & 0xFE) | bit;
            data.put_pixel(x, y, pixel);
            
            bit_index += 1;
        }
        
        // Скрываем само сообщение
        for byte in message_bytes {
            for i in 0..8 {
                let bit = ((byte >> i) & 1) as u8;
                let pixel_idx = bit_index / 8;
                let channel = bit_index % 8;
                
                let x = (pixel_idx % width) as u32;
                let y = (pixel_idx / width) as u32;
                
                let mut pixel = data.get_pixel(x, y);
                let channel_idx = channel % 4;
                pixel[channel_idx] = (pixel[channel_idx] & 0xFE) | bit;
                data.put_pixel(x, y, pixel);
                
                bit_index += 1;
            }
        }
        
        Ok(())
    }
    
    pub fn extract_message(image: &DynamicImage) -> Result<String, Box<dyn std::error::Error>> {
        let (width, height) = image.dimensions();
        let data = image.as_rgba8().unwrap();
        
        // Читаем длину сообщения (первые 32 бита)
        let mut len: u32 = 0;
        for i in 0..32 {
            let pixel_idx = i / 8;
            let channel = i % 8;
            
            let x = (pixel_idx % width) as u32;
            let y = (pixel_idx / width) as u32;
            
            let pixel = data.get_pixel(x, y);
            let channel_idx = channel % 4;
            let bit = (pixel[channel_idx] & 1) as u32;
            
            len |= bit << i;
        }
        
        // Читаем сообщение
        let mut message_bytes = Vec::with_capacity(len as usize);
        let mut bit_index = 32;
        
        for _ in 0..len {
            let mut byte: u8 = 0;
            for i in 0..8 {
                let pixel_idx = bit_index / 8;
                let channel = bit_index % 8;
                
                let x = (pixel_idx % width) as u32;
                let y = (pixel_idx / width) as u32;
                
                let pixel = data.get_pixel(x, y);
                let channel_idx = channel % 4;
                let bit = (pixel[channel_idx] & 1) as u8;
                
                byte |= bit << i;
                bit_index += 1;
            }
            message_bytes.push(byte);
        }
        
        Ok(String::from_utf8(message_bytes)?)
    }
}
