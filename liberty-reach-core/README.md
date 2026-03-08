# 🗽 Liberty Reach Messenger v2.0
## Universal Resilient Edition

[![Version](https://img.shields.io/badge/version-2.0.0-blue.svg)](https://github.com/liberty-reach/messenger)
[![Rust](https://img.shields.io/badge/rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT-green.svg)](LICENSE)

**Secure • P2P • Resilient • Post-Quantum**

---

## 📖 Описание

Liberty Reach Messenger — это мессенджер нового поколения с полной децентрализацией, 
поддержкой P2P звонков через WebRTC, обфускацией трафика для обхода цензуры и 
post-quantum криптографией для защиты от будущих угроз.

### Ключевые Особенности

- 🔐 **Thick Core Model** — весь функционал в изолированной Rust библиотеке
- 🌐 **P2P Архитектура** — libp2p с TCP/QUIC, Gossipsub, Kademlia DHT
- 📞 **WebRTC Звонки** — аудио/видео с Opus 6-32 kbps для 2G/EDGE сетей
- 🎭 **Обфускация Трафика** — HTTPS, Obfs4, Snowflake, DNS Tunnel
- 🖼️ **Стеганография** — LSB скрытие данных в изображениях
- 🔮 **Post-Quantum Crypto** — Kyber1024 KEM
- 🗄️ **SQLCipher БД** — полное шифрование базы данных
- 📱 **Flutter/Tauri UI** — кроссплатформенный интерфейс через FFI

---

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────────────────┐
│                        UI Layer (Flutter/Tauri)                 │
│                         ┌─────────────┐                         │
│                         │   FFI Bridge │                        │
│                         └──────┬──────┘                         │
└────────────────────────────────┼────────────────────────────────┘
                                 │
┌────────────────────────────────▼────────────────────────────────┐
│                    Liberty Reach Core (Rust)                    │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐ │
│  │   Bridge    │ │   Crypto    │ │   Engine    │ │  Storage  │ │
│  │  (FFI Layer)│ │ (Encryption)│ │  (Reactor)  │ │(SQLCipher)│ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘ │
│  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐ ┌───────────┐ │
│  │     P2P     │ │    Media    │ │     SIP     │ │Obfuscation│ │
│  │  (libp2p)   │ │  (WebRTC)   │ │   (VoIP)    │ │ (Traffic) │ │
│  └─────────────┘ └─────────────┘ └─────────────┘ └───────────┘ │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📦 Структура Проекта

```
liberty-reach/
├── Cargo.toml              # Workspace configuration
├── liberty-reach-core/     # Core Rust library
│   ├── Cargo.toml
│   ├── build.rs            # Build script для FFI
│   └── src/
│       ├── lib.rs          # Main library entry
│       ├── bridge.rs       # FFI layer (Commands/Events)
│       ├── crypto.rs       # Cryptography module
│       ├── storage.rs      # SQLCipher database
│       ├── engine.rs       # Main reactor (tokio::select!)
│       ├── p2p.rs          # libp2p network
│       ├── media.rs        # WebRTC media
│       ├── sip.rs          # SIP VoIP
│       ├── obfuscation.rs  # Traffic obfuscation
│       └── ffi.rs          # Flutter Rust Bridge
└── liberty-reach/          # CLI application
    ├── Cargo.toml
    └── src/
        └── main.rs         # CLI entry point
```

---

## 🚀 Быстрый Старт

### Требования

- Rust 1.75+
- Cargo
- (Опционально) Flutter для UI

### Сборка

```bash
# Клонировать репозиторий
git clone https://github.com/liberty-reach/messenger.git
cd messenger

# Собрать ядро
cargo build -p liberty-reach-core --release

# Собрать CLI
cargo build -p liberty-reach --release

# Запустить CLI
./target/release/liberty-reach --help
```

### Использование CLI

```bash
# Запустить с параметрами
./target/release/liberty-reach \
    --user-id user123 \
    --username alice \
    --bootstrap /ip4/192.168.1.1/tcp/4001 \
    run

# Отправить сообщение
./target/release/liberty-reach send bob "Hello, World!"

# Начать звонок
./target/release/liberty-reach call bob audio

# Получить историю
./target/release/liberty-reach history bob --limit 50
```

---

## 🔐 Криптография

### Поддерживаемые Алгоритмы

| Алгоритм | Тип | Размер Ключа | Назначение |
|----------|-----|--------------|------------|
| AES-256-GCM | Симметричное | 256 бит | Быстрое шифрование |
| ChaCha20-Poly1305 | Симметричное | 256 бит | Альтернатива AES |
| X25519 | Key Exchange | 256 бит | ECDH обмен ключами |
| Ed25519 | Signatures | 256 бит | Цифровые подписи |
| Kyber1024 | Post-Quantum KEM | 1024 бит | Защита от квантовых атак |
| SHA-256/512 | Hash | - | Хэширование |
| BLAKE3 | Hash | - | Быстрое хэширование |
| Argon2 | KDF | - | Хэширование паролей |

### Пример Шифрования

```rust
use liberty_reach_core::crypto::{AesGcmEncrypter, CryptoContainer, EncryptionAlgorithm};

// AES-256-GCM
let key = [42u8; 32];
let encrypter = AesGcmEncrypter::new(&key);
let (ciphertext, nonce) = encrypter.encrypt(b"Hello, World!")?;
let plaintext = encrypter.decrypt(&ciphertext, &nonce)?;

// Crypto Container (универсальный)
let mut crypto = CryptoContainer::new();
crypto.set_aes_key(key);
let (ct, nonce) = crypto.encrypt(EncryptionAlgorithm::Aes256Gcm, b"Hello")?;
```

---

## 🌐 P2P Сеть

### Транспорты

- **TCP** с Noise + Yamux — основной транспорт
- **QUIC** — UDP-based для низкой задержки
- **Комбинированный** — TCP ∨ QUIC для надежности

### Протоколы

- **Gossipsub** — Pub/Sub для чатов
- **Kademlia DHT** — Маршрутизация и обнаружение
- **mDNS** — Локальное обнаружение в LAN
- **Ping** — Проверка доступности пиров

### Пример Подключения

```rust
use liberty_reach_core::p2p::create_p2p_manager;

let bootstrap_nodes = vec![
    "/ip4/192.168.1.1/tcp/4001".to_string(),
    "/ip4/192.168.1.2/tcp/4001/quic-v1".to_string(),
];

let (cmd_tx, event_rx, peer_id) = create_p2p_manager(
    bootstrap_nodes,
    true,  // enable_quic
    false, // enable_obfuscation
).await?;
```

---

## 🎭 Обфускация Трафика

### Режимы

| Режим | Описание |
|-------|----------|
| `Disabled` | Без обфускации |
| `Https` | Маскировка под TLS 1.2 |
| `Obfs4` | Tor-style obfuscation |
| `Snowflake` | WebRTC proxies |
| `DnsTunnel` | DNS туннелирование |
| `Hybrid` | Obfs4 + HTTPS |

### Пример Использования

```rust
use liberty_reach_core::obfuscation::{ObfuscationManager, ObfuscationConfig};
use liberty_reach_core::bridge::ObfuscationMode;

let config = ObfuscationConfig {
    mode: ObfuscationMode::Obfs4,
    enabled: true,
    ..Default::default()
};

let manager = ObfuscationManager::new(config);
manager.initialize().await?;

let obfuscated = manager.obfuscate(b"Secret message").await?;
let deobfuscated = manager.deobfuscate(&obfuscated).await?;
```

---

## 🖼️ Стеганография

### LSB (Least Significant Bit)

```rust
use liberty_reach_core::crypto::LsbSteganography;
use image::GenericImageView;

// Скрыть сообщение
let mut img = image::open("cover.png")?;
LsbSteganography::hide_message(&mut img, &secret_data)?;
img.save("output.png")?;

// Извлечь сообщение
let extracted = LsbSteganography::extract_message_from_file("output.png")?;
```

**Ёмкость:** 4 бита на пиксель (RGBA)  
**Пример:** Изображение 1920x1080 = ~8 MB данных

---

## 🗄️ База Данных

### SQLCipher Конфигурация

- **Шифрование:** AES-256
- **KDF:** PBKDF2-HMAC-SHA512 (256000 итераций)
- **HMAC:** HMAC-SHA512
- **Режим:** WAL для производительности

### Пример Использования

```rust
use liberty_reach_core::storage::{DatabaseManager, DatabaseConfig};
use liberty_reach_core::crypto::generate_random_key;

let config = DatabaseConfig {
    path: "liberty_reach.db".into(),
    encryption_key: generate_random_key(),
    create_if_missing: true,
};

let db = DatabaseManager::new(config).await?;
```

---

## 📞 WebRTC Звонки

### Opus Codec

- **Битрейт:** 6-32 kbps (переменный)
- **Sample Rate:** 8-48 kHz
- **Режимы:** Narrowband → Fullband
- **FEC:** Forward Error Correction

### План Реализации (Phase 2)

1. Signaling Server
2. PeerConnection wrapper
3. ICE/STUN/TURN интеграция
4. Opus codec интеграция
5. Video codec (VP8/VP9)
6. Интеграция с P2P

---

## 🔌 FFI Integration

### Flutter Rust Bridge

```rust
use flutter_rust_bridge::frb;

#[frb(async)]
pub async fn frb_create_core(
    db_path: String,
    encryption_key: Vec<u8>,
    user_id: String,
    username: String,
) -> Result<FrbCoreHandle, String> {
    // ...
}
```

### Генерация Bindings

```bash
# Установить flutter_rust_bridge
cargo install flutter_rust_bridge_codegen

# Сгенерировать bindings
flutter_rust_bridge_codegen generate
```

---

## 🧪 Тестирование

```bash
# Запустить все тесты
cargo test -p liberty-reach-core

# Запустить с output
cargo test -p liberty-reach-core -- --nocapture

# Benchmark (требует criterion)
cargo bench -p liberty-reach-core
```

---

## 📊 Статистика Phase 1

| Метрика | Значение |
|---------|----------|
| Rust модулей | 10 |
| Строк кода | ~4600 |
| Тестов | 20 |
| Файлов | 15 |

---

## 🛣️ Roadmap

### Phase 1 ✅ (Завершена)
- [x] Workspace configuration
- [x] Bridge (FFI layer)
- [x] Crypto module
- [x] Storage (SQLCipher)
- [x] Engine (Reactor)
- [x] P2P (libp2p)
- [x] Obfuscation module
- [x] Steganography

### Phase 2 ⏳ (В работе)
- [ ] WebRTC Signaling Server
- [ ] PeerConnection wrapper
- [ ] Opus codec интеграция
- [ ] Video codec интеграция
- [ ] ICE/STUN/TURN
- [ ] Интеграция с P2P

### Phase 3 📅 (Планируется)
- [ ] Flutter UI
- [ ] Tauri Desktop UI
- [ ] Мобильные приложения (iOS/Android)
- [ ] Групповые звонки (SFU)
- [ ] Screen sharing

---

## 📝 Лицензия

MIT License — см. [LICENSE](LICENSE)

---

## 👥 Команда

**Lead Developer:** Liberty Reach Team  
**Contact:** zametkikostik@gmail.com

---

## 🙏 Благодарности

- **libp2p** — P2P network stack
- **Rust** — Язык программирования
- **Flutter** — UI framework
- **WebRTC** — Real-time communication
- **Tor Project** — Obfs4 inspiration

---

*Liberty Reach Messenger — Свобода. Приватность. Устойчивость.*
