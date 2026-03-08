//! Generate Encryption Key Utility
//! 
//! Утилита для генерации случайного 32-байтного ключа шифрования.

use rand::{RngCore, thread_rng};

fn main() {
    println!("🔑 Liberty Reach - Генератор ключа шифрования\n");
    
    // Генерируем 32 случайных байта
    let mut key = [0u8; 32];
    thread_rng().fill_bytes(&mut key);
    
    // Выводим в hex формате
    let hex_key = hex::encode(&key);
    
    println!("Сгенерирован ключ шифрования (32 байта / 256 бит):");
    println!("{}\n", hex_key);
    
    println!("Добавьте в config.toml:");
    println!("encryption_key = \"{}\"\n", hex_key);
    
    println!("⚠️ ВАЖНО:");
    println!("- Сохраните этот ключ в надёжном месте");
    println!("- Никогда не передавайте его никому");
    println!("- Без этого ключа нельзя будет расшифровать базу данных");
}
