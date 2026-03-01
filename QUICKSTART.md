# 🚀 Быстрый старт Secure Telegram Client v2.0

## ✅ Что реализовано в v2.0

### Готовые модули
- ✅ **Постквантовое шифрование** (Kyber-1024 + XChaCha20-Poly1305)
- ✅ **obfs4 транспорт** для обхода DPI
- ✅ **Детектор блокировок** (DNS/TCP RST/TLS/DPI)
- ✅ **Авто-переключение транспортов**
- ✅ **Зашифрованная очередь сообщений** (SQLite + SQLCipher)
- ✅ **Децентрализованные обновления** (IPFS + Ed25519 подпись)
- ✅ **SOCKS5 прокси** поддержка

### В разработке
- 🟡 libp2p P2P fallback
- 🟡 Shadowsocks транспорт
- 🟡 DNS over HTTPS (полная интеграция)

---

## 📦 Установка

### 1. Требования

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install -y cmake clang libssl-dev pkg-config git libsqlite3-dev
```

**macOS:**
```bash
brew install cmake openssl sqlite
```

**Windows:**
```powershell
# Установите vcpkg и выполните:
vcpkg install openssl:x64-windows sqlite3:x64-windows
```

### 2. Сборка из исходников

```bash
# Клонируйте репозиторий
git clone https://github.com/zametkikostik/secure-telegram-client.git
cd secure-telegram-client

# Сборка release версии
cargo build --release

# Проверка
./target/release/secure-tg --version
```

### 3. Docker

```bash
docker build -t secure-tg .
docker run -it secure-tg --help
```

---

## ⚙️ Настройка

### 1. Инициализация конфигурации

```bash
./target/release/secure-tg --init-config
```

Файл будет создан в:
- **Linux**: `~/.config/secure-telegram-client/config.json`
- **macOS**: `~/Library/Application Support/secure-telegram-client/config.json`
- **Windows**: `%APPDATA%\secure-telegram-client\config.json`

### 2. Редактирование конфигурации

Откройте файл и установите:

```json
{
  "api_id": 123456,
  "api_hash": "your_api_hash_from_my.telegram.org",
  "encryption": {
    "kyber_enabled": true,
    "steganography_enabled": true,
    "obfuscation_enabled": true,
    "auto_steganography": true
  },
  "transport": {
    "transports": [
      {"type": "Direct", "priority": 1},
      {"type": "Obfs4", "bridge_addr": "bridge.example.com:443", "public_key": "...", "priority": 2}
    ]
  },
  "stealth_mode": true,
  "auto_update": true
}
```

**Важно**: Получите `api_id` и `api_hash` на https://my.telegram.org/apps

### 3. Настройка obfs4 моста

Добавьте мост в конфигурацию:

```json
{
  "transport": {
    "transports": [
      {
        "type": "Obfs4",
        "bridge_addr": "bridge.example.com:443",
        "public_key": "abcdef1234567890abcdef1234567890abcdef1234567890abcdef1234567890"
      }
    ]
  }
}
```

---

## 🎮 Использование

### Запуск клиента

```bash
./target/release/secure-tg
```

### Основные команды CLI

```
> help                    # Показать справку
> auth +79991234567       # Авторизация по номеру
> code 12345              # Ввод кода из SMS
> password mypass         # Ввод пароля 2FA
> send 12345678 Привет    # Отправить сообщение
> chats 10                # Показать 10 чатов
> history 12345678 20     # История чата (20 сообщений)
> status                  # Статус подключения
> quit                    # Выход
```

### Опции командной строки

```bash
# Проверка обновлений
./target/release/secure-tg --check-update

# Обновление через IPFS
./target/release/secure-tg --update

# Создание конфигурации
./target/release/secure-tg --init-config

# Debug логирование
./target/release/secure-tg -v

# Версия
./target/release/secure-tg --version
```

---

## 🔧 Тестирование

### Запуск тестов

```bash
# Все тесты
cargo test --all

# Тесты криптографии
cargo test crypto

# Тесты obfs4
cargo test obfs4

# Тесты детектора блокировок
cargo test blockage

# Тесты хранилища
cargo test storage
```

### Бенчмарки

```bash
cargo bench
```

Результаты будут в `target/criterion/`.

---

## 🛠️ Решение проблем

### Ошибка: "Неверный формат публичного ключа"

Убедитесь, что публичный ключ obfs4 моста в формате hex (64 символа).

### Ошибка: "TDLib клиент не создан"

Проверьте, что установлен TDLib:
```bash
# Ubuntu
sudo apt-get install libtdjson-dev

# macOS
brew install tdlib
```

### Ошибка: "DNS блокировка"

Включите DNS over HTTPS в конфигурации или используйте obfs4 транспорт.

### Клиент не подключается

1. Проверьте интернет соединение
2. Попробуйте другой транспорт (SOCKS5/obfs4)
3. Проверьте `cargo test blockage` для диагностики

---

## 📚 Документация

- [ARCHITECTURE.md](ARCHITECTURE.md) - Архитектура проекта
- [IMPLEMENTATION_STATUS.md](IMPLEMENTATION_STATUS.md) - Статус реализации
- [DISCLAIMER.md](DISCLAIMER.md) - Предупреждения и ограничения
- [CONTRIBUTING.md](CONTRIBUTING.md) - Как внести вклад
- [CHANGELOG.md](CHANGELOG.md) - История изменений

---

## 🔐 Безопасность

### Включено по умолчанию:
- ✅ Постквантовое шифрование (Kyber-1024)
- ✅ AEAD шифрование (XChaCha20-Poly1305)
- ✅ obfs4 обфускация трафика
- ✅ Шифрование базы данных (SQLCipher)

### Отключить для тестирования:
```json
{
  "stealth_mode": false,
  "obfuscation_enabled": false
}
```

---

## 📞 Поддержка

- **GitHub Issues**: https://github.com/zametkikostik/secure-telegram-client/issues
- **Документация**: https://github.com/zametkikostik/secure-telegram-client/wiki

---

## ⚠️ Предупреждение

**Не используйте для критически важной коммуникации!**

Проект в активной разработке. Возможны уязвимости и ошибки.

См. [DISCLAIMER.md](DISCLAIMER.md) для подробной информации.

---

**Версия**: 0.2.0  
**Дата**: 2024-02-27  
**Статус**: ✅ Готов к тестированию
