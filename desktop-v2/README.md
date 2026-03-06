# Secure Telegram Desktop v2.0

> **Оптимизировано для Linux Mint**

Приватный мессенджер с post-quantum шифрованием, P2P-сетью и полной независимостью от центральных серверов.

## 🚀 Быстрый старт

### Установка из .deb пакета

```bash
# Скачать пакет
wget https://github.com/zametkikostik/secure-telegram-client/releases/latest/download/secure-telegram-desktop_2.0.0_amd64.deb

# Установить
sudo dpkg -i secure-telegram-desktop_2.0.0_amd64.deb

# Исправить зависимости (если нужно)
sudo apt-get install -f -y
```

### Сборка из исходников

```bash
# Установка зависимостей
sudo apt-get update
sudo apt-get install -y \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libappindicator3-dev \
    librsvg2-dev \
    libnotify-dev \
    libsecret-1-dev \
    libssl-dev \
    pkg-config \
    build-essential \
    cargo \
    npm

# Сборка
cd desktop-v2
chmod +x scripts/build.sh
./scripts/build.sh
```

## 📋 Требования

### Минимальные

- **ОС:** Linux Mint 20+ (Ubuntu 20.04+)
- **Процессор:** 2 ядра, 64-bit
- **ОЗУ:** 512 MB
- **Место на диске:** 200 MB

### Рекомендуемые

- **ОС:** Linux Mint 21+ (Ubuntu 22.04+)
- **Процессор:** 4 ядра, 64-bit
- **ОЗУ:** 2 GB
- **Место на диске:** 500 MB

## 🔧 Конфигурация

### Автозапуск

```bash
# Включить автозапуск
systemctl --user enable secure-telegram-desktop.service

# Отключить автозапуск
systemctl --user disable secure-telegram-desktop.service
```

### Конфигурационный файл

`~/.config/secure-telegram/config.json`

```json
{
  "theme": "dark",
  "notifications": true,
  "autostart": false,
  "minimize_to_tray": true,
  "p2p_enabled": true,
  "encryption": {
    "kyber_enabled": true,
    "steganography_enabled": false
  }
}
```

## 🎯 Особенности Linux Mint

### Интеграция с Cinnamon

- ✅ Иконка в системном трее
- ✅ Уведомления через libnotify
- ✅ Автозапуск через systemd
- ✅ MIME типы для ссылок
- ✅ Глобальные горячие клавиши

### Оптимизации

- Использование системных библиотек GTK3
- Нативные уведомления Linux Mint
- Интеграция с меню приложений
- Поддержка HiDPI дисплеев

## 🔐 Безопасность

### Криптография

| Алгоритм | Назначение | Стандарт |
|----------|------------|----------|
| Kyber1024 | Post-quantum KEM | NIST |
| X25519 | Key Exchange | RFC 7748 |
| Ed25519 | Подписи | RFC 8032 |
| AES-256-GCM | Шифрование | NIST |
| ChaCha20-Poly1305 | Шифрование | RFC 8439 |

### Хранение данных

- База данных шифруется SQLCipher
- Ключи хранятся в GNOME Keyring
- Поддержка TPM (опционально)

## 📦 Форматы пакетов

| Формат | Команда | Размер |
|--------|---------|--------|
| **.deb** | `dpkg -i` | ~50 MB |
| **.rpm** | `rpm -i` | ~52 MB |
| **AppImage** | `./AppImage` | ~80 MB |

## 🐛 Известные проблемы

### Linux Mint 20.x

- Может потребоваться ручная установка libwebkit2gtk-4.0-37
- Уведомления могут не работать без libnotify4

### Linux Mint 21.x

- Полная совместимость
- Все функции работают корректно

## 📞 Поддержка

- **GitHub Issues:** https://github.com/zametkikostik/secure-telegram-client/issues
- **Документация:** https://docs.secure-telegram.io
- **Telegram:** @secure_telegram_support

## 📄 Лицензия

MIT License - см. LICENSE

---

**Secure Telegram Team © 2026**
