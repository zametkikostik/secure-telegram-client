# 🔐 Secure Telegram Client

> Децентрализованный мессенджер нового поколения с постквантовым шифрованием

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75+-blue.svg)](https://www.rust-lang.org)
[![Tauri](https://img.shields.io/badge/Tauri-2.0-blue.svg)](https://tauri.app)

---

## 🎯 Цели

| Цель | Описание |
|------|----------|
| **Приватность** | Никаких центральных серверов, трекеров и логов |
| **Безопасность** | Post-quantum E2EE (X25519 + Kyber1024), стеганография |
| **Независимость** | P2P + Cloudflare fallback |
| **Миграция** | Импорт из Telegram/WhatsApp |
| **Монетизация** | Privacy-first реклама + Web3 |

---

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────┐
│              CLIENT LAYER                    │
│  Tauri Desktop  │  React Native  │  Web PWA  │
└────────┬────────┴────────┬────────┴──────────┘
         │                 │
    ┌────▼─────────────────▼────┐
    │     CRYPTO LAYER          │
    │  X25519 + Kyber1024       │
    │  ChaCha20-Poly1305 + HMAC │
    │  Ed25519 + Argon2id       │
    │  LSB Steganography        │
    └───────────────────────────┘
```

---

## 📦 Структура

```
secure-telegram-client/
├── messenger/                    # Tauri desktop app
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs               # Tauri binary entry point
│   │   ├── lib.rs                # Library re-exports
│   │   ├── crypto/
│   │   │   ├── mod.rs
│   │   │   ├── hybrid.rs         # X25519 + Kyber1024 + ChaCha20
│   │   │   ├── steganography.rs  # LSB image steganography
│   │   │   └── constants.rs
│   │   └── auth/
│   │       ├── mod.rs
│   │       └── keychain.rs       # OS keyring integration
│   ├── build.rs
│   ├── tauri.conf.json
│   └── capabilities/
├── server/                       # Axum backend
├── frontend/                     # React + Vite + TypeScript
├── bots/                         # Bot platform
├── cloudflare/                   # Workers
├── mobile/                       # React Native
├── migration-tool/               # Telegram/WhatsApp import
├── smart-contracts/              # Solidity
├── self-hosting/                 # Docker
├── docs/                         # Documentation
└── scripts/                      # Build helpers
```

---

## 🔐 Криптография

| Алгоритм | Назначение | Статус |
|----------|------------|--------|
| **X25519** | ECDH key exchange | ✅ Реализовано |
| **Kyber1024** | Post-quantum KEM | ✅ Реализовано (oqs crate) |
| **ChaCha20-Poly1305** | AEAD encryption | ✅ Реализовано |
| **Argon2id** | Password hashing | ✅ Реализовано |
| **Ed25519** | Digital signatures | ✅ Реализовано |
| **HMAC-SHA3-256** | Message auth | ✅ Реализовано |
| **HKDF-SHA3-256** | Key derivation | ✅ Реализовано |
| **LSB Steganography** | Plausible deniability | ✅ Реализовано |

### Гибридное шифрование

```
E2EE = X25519 ECDH + Kyber1024 KEM → HKDF-SHA3-256 → ChaCha20-Poly1305 + Ed25519
```

---

## 🚀 Быстрый старт

### Установка

#### 1. Установи зависимости

```bash
sudo apt update
sudo apt install -y \
    build-essential \
    cmake \
    git \
    libssl-dev \
    pkg-config \
    clang \
    llvm
```

#### 2. Установи Rust (если ещё нет)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source $HOME/.cargo/env
```

#### 3. Установи liboqs через cargo (автоматически через зависимости)

```bash
cd /home/kostik/secure-messenger/secure-telegram-client
cargo build
```

**Альтернативно: ручная сборка liboqs**

```bash
cd /tmp
git clone https://github.com/open-quantum-safe/liboqs.git
cd liboqs
mkdir build && cd build
cmake -G"Ninja" ..
ninja
sudo ninja install
sudo ldconfig
```

#### 4. Проверь установку

```bash
cargo test --package secure-messenger crypto::hybrid::tests::test_hybrid_encrypt_decrypt
```

### Требования

```bash
# Системные зависимости (Linux Mint/Ubuntu)
sudo apt install -y build-essential cmake pkg-config libssl-dev \
  libgtk-3-dev libwebkit2gtk-4.1-dev libappindicator3-dev \
  librsvg2-dev libclang-dev protobuf-compiler

# Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Node.js
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
nvm install 18
```

### Сборка

```bash
# Все Rust пакеты
cargo build --workspace

# Тесты
cargo test --workspace

# Frontend
cd frontend && npm install && npm run build

# Tauri desktop
cargo tauri build
```

---

## 🧪 Тесты

```bash
# Все тесты (12 проходят ✅)
cargo test -p secure-messenger --lib

# Включая:
# - test_hybrid_encrypt_decrypt  (X25519 + Kyber1024 E2EE)
# - test_sign_verify             (Ed25519 signatures)
# - test_ciphertext_serialization
# - test_tampered_ciphertext_fails
# - test_hide_and_extract_small_data (Steganography)
# - test_binary_data             (All 256 byte values)
```

---

## 📄 Лицензия

MIT License — см. [LICENSE](LICENSE)

---

> **Помни**: Приватность — это не фича, это право человека. 🔐
