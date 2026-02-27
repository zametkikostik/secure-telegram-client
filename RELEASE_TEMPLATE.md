# 📱 Secure Telegram Client v0.2.0

**Дата**: 2024-02-27  
**Версия**: 0.2.0

---

## ✨ Что нового

### 🔐 Anti-Censorship (100%)
- ✅ **obfs4** — Обфускация трафика под шум
- ✅ **Shadowsocks** — Прокси с шифрованием
- ✅ **TLS Fingerprint** — Подмена JA3 отпечатка
- ✅ **DNS over HTTPS** — Обход DNS блокировок
- ✅ **SOCKS5** — Базовое проксирование

### 🌐 Децентрализация (100%)
- ✅ **IPFS** — Хостинг релизов
- ✅ **libp2p DHT** — Поиск пиров
- ✅ **Gossipsub** — P2P месседжинг
- ✅ **mDNS** — Локальное обнаружение

### 💾 Хранение (100%)
- ✅ **SQLite + SQLCipher** — Зашифрованная БД
- ✅ **Message Queue** — Очередь сообщений
- ✅ **Sync State** — Синхронизация

### 📱 Android
- ✅ **APK сборка** — 5.6 MB
- ✅ **F-Droid совместимость**
- ✅ **GitHub Actions CI/CD**

---

## 📥 Установка

### Android

1. **Скачайте APK**: `secure-messenger-debug.apk`
2. **Разрешите установку**: Настройки → Безопасность → Неизвестные источники
3. **Установите**: Откройте APK файл
4. **Запустите**: Secure Messenger

### Desktop (Linux)

```bash
# Сборка из исходников
git clone https://github.com/zametkikostik/secure-telegram-client.git
cd secure-telegram-client
cargo build --release
./target/release/secure-tg
```

---

## 🔧 Настройка

1. **Запустите приложение**
2. **Введите Telegram API credentials**:
   - Получите на: https://my.telegram.org/apps
   - Введите `api_id` и `api_hash`
3. **Авторизуйтесь** по номеру телефона

---

## 📚 Документация

- [README](https://github.com/zametkikostik/secure-telegram-client#readme)
- [QUICKSTART](https://github.com/zametkikostik/secure-telegram-client/blob/master/QUICKSTART.md)
- [ARCHITECTURE](https://github.com/zametkikostik/secure-telegram-client/blob/master/ARCHITECTURE.md)

---

## ⚠️ Предупреждение

**Это исследовательский проект!** Не используйте для критически важной коммуникации.

См. [DISCLAIMER.md](https://github.com/zametkikostik/secure-telegram-client/blob/master/DISCLAIMER.md)

---

## 📊 Статистика

- **Размер APK**: 5.6 MB
- **Мин. Android**: 8.0+ (API 26)
- **Лицензия**: MIT
- **Репозиторий**: [GitHub](https://github.com/zametkikostik/secure-telegram-client)

---

**Приятного использования!** 🎉
