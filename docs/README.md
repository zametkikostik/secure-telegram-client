# 📚 Documentation Index

> Полная документация по Secure Telegram Client

---

## 📖 Руководства

| Документ | Описание |
|----------|----------|
| [README](../README.md) | Обзор проекта и быстрый старт |
| [SECURITY.md](SECURITY.md) | Требования к безопасности и аудит |
| [API.md](API.md) | API документация (TODO) |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Архитектура системы (TODO) |
| [DEPLOY.md](DEPLOY.md) | Деплой и self-hosting (TODO) |
| [CONTRIBUTING.md](CONTRIBUTING.md) | Как контрибьютить (TODO) |

---

## 🏗️ Архитектура

### Компоненты

1. **messenger/** — Tauri desktop приложение
   - `src-tauri/` — Rust backend
   - `src/` — React frontend
   - `crypto/` — Криптографические примитивы
   - `steganography/` — Стеганография

2. **server/** — Axum backend
   - REST API
   - WebSocket signaling
   - SQLite/PostgreSQL

3. **frontend/** — React web UI
   - Vite + TypeScript
   - Tailwind CSS
   - PWA support

4. **mobile/** — React Native
   - Android (готово)
   - iOS (в планах)

5. **cloudflare/** — Workers
   - Push notifications
   - Signaling
   - File storage (R2)

6. **bots/** — Bot platform
   - Команды ботов
   - Интеграции

7. **migration-tool/** — Миграция
   - Telegram import
   - WhatsApp import

8. **smart-contracts/** — Web3
   - P2PEscrow
   - FeeSplitter

---

## 🔐 Криптография

### Алгоритмы

| Алгоритм | Назначение | Реализация |
|----------|------------|------------|
| X25519 | ECDH key exchange | ✅ x25519-dalek |
| Kyber1024 | Post-quantum KEM | ⬜ liboqs (TODO) |
| ChaCha20-Poly1305 | AEAD encryption | ✅ chacha20poly1305 |
| Argon2id | Password hashing | ✅ argon2 |
| Ed25519 | Digital signatures | ✅ ed25519-dalek |
| ML-DSA | Post-quantum signatures | ⬜ liboqs (TODO) |
| HMAC-SHA3-256 | Message authentication | ✅ hmac + sha3 |

### Key Management

См. [SECURITY.md](SECURITY.md#-key-management)

---

## 🚀 Быстрый старт

См. [README.md](../README.md#-быстрый-старт)

---

## 📋 Фазы разработки

См. [README.md](../README.md#-фазы-разработки)
