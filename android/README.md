# 🔐 Secure Messenger Android

Бессерверный Android-клиент для безопасного общения с децентрализованными обновлениями через IPFS.

[![License](https://img.shields.io/badge/License-Apache--2.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)
[![F-Droid](https://img.shields.io/f-droid/v/com.example.securemessenger.fdroid)](https://f-droid.org/packages/com.example.securemessenger.fdroid/)
[![Android](https://img.shields.io/badge/Android-8.0+-green.svg)](https://www.android.com/)

---

## 📋 Особенности

### 🔐 Безопасность
- **Постквантовое шифрование** (Kyber-1024)
- **Симметричное шифрование** (XChaCha20-Poly1305)
- **Key Exchange** (X25519 Diffie-Hellman)
- **Подпись релизов** (Ed25519)

### 🌐 Децентрализация
- **IPFS обновления** — без центрального сервера
- **P2P fallback** — общение при недоступности Telegram
- **Зеркала** — Codeberg (ЕС), GitFlic (РФ)

### 👻 Anti-Censorship
- **obfs4** — обфускация трафика под шум
- **Детектор блокировок** — авто-переключение транспортов
- **Pluggable Transports** — расширяемый API

---

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────┐
│                  Android App (Kotlin)               │
├─────────────────────────────────────────────────────┤
│  UI  │  Service  │  Repository  │  ViewModel       │
└─────────────────────────────────────────────────────┘
                        ↓ JNI
┌─────────────────────────────────────────────────────┐
│                  Rust Core Library                  │
├─────────────────────────────────────────────────────┤
│ Telegram │ Crypto │ Transport │ Updater │ P2P      │
└─────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────┐
│  TDLib  │  libp2p  │  IPFS  │  SQLCipher          │
└─────────────────────────────────────────────────────┘
```

---

## 🚀 Быстрый старт

### Требования
- Android Studio Arctic Fox+
- NDK 25+
- Rust 1.75+
- cargo-ndk: `cargo install cargo-ndk`

### Сборка

```bash
# Клонирование
git clone https://github.com/example/secure-messenger-android.git
cd secure-messenger-android

# Сборка debug версии
./gradlew assembleFdroidDebug

# Сборка release версии
./gradlew assembleFdroidRelease

# Установка на устройство
adb install app/build/outputs/apk/fdroid/debug/app-fdroid-debug.apk
```

### Flavor'ы

| Flavor | Описание | Obfuscation |
|--------|----------|-------------|
| `fdroid` | Для F-Droid | ❌ Отключен |
| `full` | Полная версия | ✅ Включен |

---

## 📦 Распространение

### Через IPFS

1. **Подпись APK:**
   ```bash
   ./scripts/sign-release.sh app/build/outputs/apk/fdroid/release/app-fdroid-release-unsigned.apk
   ```

2. **Публикация в IPFS:**
   ```bash
   ./scripts/ipfs-publish.sh app-fdroid-release-signed.apk 0.2.0
   ```

3. **Обновление manifest:**
   ```json
   {
     "latest_version": "0.2.0",
     "apk_cid": "QmYourAPKCID",
     "public_key": "YourPublicKey"
   }
   ```

### Зеркала

- **Codeberg**: https://codeberg.org/example/secure-messenger-android
- **GitFlic**: https://gitflic.ru/project/example/secure-messenger-android

---

## 🔧 Конфигурация

### Переменные окружения

```bash
# Для подписи релизов
export KEYSTORE_PATH=/path/to/keystore
export KEYSTORE_PASSWORD=your_password
export KEY_ALIAS=secure-messenger
export KEY_PASSWORD=your_password

# Для пиннинга в IPFS
export PINATA_API_KEY=your_key
export PINATA_SECRET_KEY=your_secret
```

### Сборочные флаги

```toml
# В Cargo.toml
[features]
default = ["obfuscation"]
no-obfuscation = []  # Для F-Droid
p2p = []  # P2P fallback
```

---

## 🧪 Тестирование

### Unit тесты

```bash
# Rust тесты
cd core && cargo test

# Android тесты
./gradlew testFdroidDebugUnitTest
```

### Ручное тестирование

- [ ] Сборка на чистой Ubuntu (Docker)
- [ ] Обновление через IPFS на Android 10/12/14
- [ ] Обход DPI через obfs4
- [ ] P2P чат между двумя устройствами

---

## 📄 Лицензия

Apache-2.0 — см. [LICENSE](LICENSE)

---

## ⚠️ Предупреждение

**Это исследовательский проект.** Не используйте для критически важной коммуникации.

Использование средств обхода блокировок может регулироваться законодательством вашей страны.

---

## 📞 Контакты

- GitHub: https://github.com/example/secure-messenger-android
- Issues: https://github.com/example/secure-messenger-android/issues
- F-Droid: https://f-droid.org/packages/com.example.securemessenger.fdroid/

---

**Версия**: 0.2.0  
**Дата**: 2024-02-27
