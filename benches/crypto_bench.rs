//! Бенчмарки для Secure Telegram Client

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use secure_telegram_client::crypto::xchacha::XChaChaCipher;
use secure_telegram_client::obfs::obfs4::Obfs4Transformer;
use secure_telegram_client::stego::lsb::{generate_test_image, LsbSteganography};

/// Бенчмарк шифрования XChaCha20-Poly1305
fn bench_xchacha_encryption(c: &mut Criterion) {
    let key = XChaChaCipher::generate_key();
    let cipher = XChaChaCipher::new(key);
    let plaintext = b"Hello, Secure Telegram! This is a test message.";

    c.bench_function("xchacha_encrypt_50b", |b| {
        b.iter(|| cipher.encrypt(black_box(plaintext)))
    });

    // Бенчмарк для разных размеров данных
    let mut group = c.benchmark_group("xchacha_encrypt_size");

    for size in [100, 1000, 10000, 100000].iter() {
        let data = vec![0u8; *size];
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            b.iter(|| cipher.encrypt(black_box(data)))
        });
    }
    group.finish();
}

/// Бенчмарк расшифрования XChaCha20-Poly1305
fn bench_xchacha_decryption(c: &mut Criterion) {
    let key = XChaChaCipher::generate_key();
    let cipher = XChaChaCipher::new(key);
    let plaintext = b"Hello, Secure Telegram! This is a test message.";
    let (nonce, ciphertext) = cipher.encrypt(plaintext).unwrap();

    c.bench_function("xchacha_decrypt_50b", |b| {
        b.iter(|| cipher.decrypt(black_box(&nonce), black_box(&ciphertext)))
    });
}

/// Бенчмарк обфускации obfs4
fn bench_obfs4_obfuscation(c: &mut Criterion) {
    let mut transformer = Obfs4Transformer::new();
    let data = b"Secret traffic data for obfuscation";

    c.bench_function("obfs4_obfuscate_40b", |b| {
        b.iter(|| transformer.obfuscate(black_box(data)))
    });

    // Бенчмарк для разных размеров
    let mut group = c.benchmark_group("obfs4_obfuscate_size");

    for size in [100, 1000, 10000].iter() {
        let data = vec![0u8; *size];
        group.bench_with_input(BenchmarkId::from_parameter(size), &data, |b, data| {
            let mut tf = Obfs4Transformer::new();
            b.iter(|| tf.obfuscate(black_box(data)))
        });
    }
    group.finish();
}

/// Бенчмарк стенографии LSB
fn bench_steganography(c: &mut Criterion) {
    let stego = LsbSteganography::new();
    let test_image = generate_test_image(500, 500);
    let secret_data = b"Hidden message in image";

    c.bench_function("stego_embed_500x500", |b| {
        b.iter(|| stego.embed(black_box(&test_image), black_box(secret_data)))
    });

    c.bench_function("stego_extract_500x500", |b| {
        let stego_image = stego.embed(&test_image, secret_data).unwrap();
        b.iter(|| stego.extract(black_box(&stego_image)))
    });

    // Бенчмарк для разных размеров изображений
    let mut group = c.benchmark_group("stego_embed_size");

    for size in [100, 200, 300].iter() {
        let img = generate_test_image(*size, *size);
        group.bench_with_input(BenchmarkId::from_parameter(size), &img, |b, img| {
            let st = LsbSteganography::new();
            b.iter(|| st.embed(black_box(img), black_box(secret_data)))
        });
    }
    group.finish();
}

/// Бенчмарк полного цикла (шифрование + обфускация + стенография)
fn bench_full_pipeline(c: &mut Criterion) {
    let mut group = c.benchmark_group("full_pipeline");

    let key = XChaChaCipher::generate_key();
    let cipher = XChaChaCipher::new(key);
    let plaintext = b"Secret message";

    for size in [100, 200].iter() {
        let test_image = generate_test_image(*size, *size);

        group.bench_with_input(BenchmarkId::from_parameter(size), &test_image, |b, img| {
            b.iter(|| {
                // Шифрование
                let (nonce, ct) = cipher.encrypt(black_box(plaintext)).unwrap();

                // Обфускация
                let mut obfs = Obfs4Transformer::new();
                let obfuscated = obfs.obfuscate(&ct);

                // Встраивание
                let mut data = nonce;
                data.extend_from_slice(&obfuscated);
                let stego = LsbSteganography::new();
                let _ = stego.embed(black_box(img), black_box(&data));
            })
        });
    }
    group.finish();
}

criterion_group!(
    benches,
    bench_xchacha_encryption,
    bench_xchacha_decryption,
    bench_obfs4_obfuscation,
    bench_steganography,
    bench_full_pipeline,
);

criterion_main!(benches);
