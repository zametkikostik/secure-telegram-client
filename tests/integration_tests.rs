//! Интеграционные тесты для Secure Telegram Client

use secure_telegram_client::crypto::xchacha;
use secure_telegram_client::crypto::dh;
use secure_telegram_client::obfs::obfs4::Obfs4Transformer;
use secure_telegram_client::stego::lsb::{LsbSteganography, generate_test_image};

/// Тест полного цикла шифрования/расшифрования
#[test]
fn test_full_encryption_cycle() {
    // Генерация ключа XChaCha
    let key = xchacha::XChaChaCipher::generate_key();
    let cipher = xchacha::XChaChaCipher::new(key);

    // Исходное сообщение
    let plaintext = b"Secret message for Telegram";

    // Шифрование
    let (nonce, ciphertext) = cipher.encrypt(plaintext).unwrap();

    // Расшифровка
    let decrypted = cipher.decrypt(&nonce, &ciphertext).unwrap();

    assert_eq!(plaintext.to_vec(), decrypted);
}

/// Тест обмена ключами Diffie-Hellman
#[test]
fn test_dh_key_exchange() {
    // Генерация пар ключей для Алисы и Боба
    let alice = dh::X25519KeyPair::generate();
    let bob = dh::X25519KeyPair::generate();

    // Обмен ключами
    let alice_secret = alice.compute_shared_secret(&bob.public_key_bytes());
    let bob_secret = bob.compute_shared_secret(&alice.public_key_bytes());

    // Секреты должны совпасть
    assert_eq!(alice_secret, bob_secret);

    // Вывод ключа шифрования
    let salt = b"secure-telegram-salt";
    let info = b"encryption-key";
    let encryption_key = dh::KeyExchange::derive_key(&alice_secret, salt, info);

    assert_eq!(encryption_key.len(), 32);
}

/// Тест obfs4 обфускации
#[test]
fn test_obfs4_obfuscation() {
    let mut transformer = Obfs4Transformer::new();

    let original = b"Secret traffic data";

    // Обфускация
    let obfuscated = transformer.obfuscate(original);

    // Деобфускация
    let mut recv_transformer = Obfs4Transformer::with_seed(transformer.seed);
    recv_transformer.packet_counter = transformer.packet_counter - 1;
    let deobfuscated = recv_transformer.deobfuscate(&obfuscated).unwrap();

    assert_eq!(original.to_vec(), deobfuscated);

    // Обфусцированные данные должны отличаться от оригинала
    assert_ne!(original.to_vec(), obfuscated);
}

/// Тест стенографии
#[test]
fn test_steganography_embed_extract() {
    let stego = LsbSteganography::new();
    let test_image = generate_test_image(200, 200);

    let secret_data = b"This is a hidden message in the image!";

    // Встраивание данных
    let stego_image = stego.embed(&test_image, secret_data).unwrap();

    // Извлечение данных
    let extracted = stego.extract(&stego_image).unwrap();

    assert_eq!(secret_data.to_vec(), extracted);
}

/// Тест производительности шифрования
#[test]
fn test_encryption_performance() {
    use std::time::Instant;

    let key = xchacha::XChaChaCipher::generate_key();
    let cipher = xchacha::XChaChaCipher::new(key);

    let message_size = 1024 * 1024; // 1 MB
    let plaintext = vec![0u8; message_size];

    let start = Instant::now();
    let (nonce, ciphertext) = cipher.encrypt(&plaintext).unwrap();
    let encrypt_duration = start.elapsed();

    let start = Instant::now();
    let decrypted = cipher.decrypt(&nonce, &ciphertext).unwrap();
    let decrypt_duration = start.elapsed();

    println!("Шифрование 1MB: {:?}", encrypt_duration);
    println!("Расшифровка 1MB: {:?}", decrypt_duration);

    assert_eq!(plaintext, decrypted);

    // Проверка что шифрование/расшифровка достаточно быстрые (< 1 секунды для 1MB)
    assert!(encrypt_duration.as_millis() < 1000);
    assert!(decrypt_duration.as_millis() < 1000);
}
