# 🔐 Secure Telegram Client

Telegram клиент с **постквантовым шифрованием**, **DPI обходом** и **автоматической стенографией**.

## ⚡ Особенности

| Функция | Описание |
|---------|----------|
| 🛡️ **Постквантовое шифрование** | Kyber-1024 + XChaCha20-Poly1305 |
| 🔄 **Key Exchange** | X25519 Diffie-Hellman |
| 👻 **DPI Obfuscation** | obfs4-подобная обфускация трафика |
| 🖼️ **Стенография** | LSB в изображения (автоматически) |
| 🚀 **Автообновление** | Проверка и установка обновлений |
| 📦 **TDLib** | Официальная библиотека Telegram |

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────────┐
│                    Ваш клиент                           │
├─────────────────────────────────────────────────────────┤
│  Сообщение → Kyber + XChaCha20 → obfs4 → TDLib → TLS   │
└─────────────────────────────────────────────────────────┘
```

## 🚀 Быстрый старт

### Требования

- Rust 1.75+
- CMake 3.10+
- TDLib 2.0+

### Сборка

```bash
# Клонируем репозиторий
git clone https://github.com/YOUR_USERNAME/secure-telegram-client.git
cd secure-telegram-client

# Собираем релизную версию
cargo build --release

# Запуск
./target/release/secure-tg
```

### Docker сборка

```bash
docker build -t secure-tg .
docker run -it secure-tg
```

## 🔐 Шифрование

### Используемые алгоритмы

1. **Постквантовое**: Kyber-1024 (NIST стандартизировано)
2. **Симметричное**: XChaCha20-Poly1305 (256-bit ключ)
3. **Key Exchange**: X25519 (Curve25519)

### Схема работы

```
1. Генерация пары ключей Kyber
2. Обмен открытыми ключами через X25519
3. Вывод общего секрета (HKDF)
4. Шифрование сообщения XChaCha20
5. Обфускация obfs4
6. Отправка через TDLib
```

## 👻 DPI Обход

Используется obfs4 для маскировки трафика под случайный шум:

- Нет узнаваемых паттернов
- Устойчив к активным зондам
- Работает через прокси

## 🖼️ Стенография

Автоматическое встраивание данных в изображения:

- **LSB (Least Significant Bit)** - незаметно для глаза
- **Адаптивный выбор** - анализ изображения перед встраиванием
- **Извлечение** - автоматическое при получении

```rust
// Пример использования
let image = image::open("photo.jpg")?;
let stego = Steganography::embed(&image, &encrypted_data)?;
stego.save("photo_stego.png")?;
```

## 🔄 Автообновление

При запуске клиент проверяет GitHub Releases:

```bash
# Проверка обновлений
secure-tg --check-update

# Принудительное обновление
secure-tg --update
```

## 📁 Структура проекта

```
secure-telegram-client/
├── src/
│   ├── main.rs           # Точка входа
│   ├── crypto/           # Криптография
│   │   ├── kyber.rs      # Постквантовое шифрование
│   │   ├── xchacha.rs    # Симметричное шифрование
│   │   └── dh.rs         # Diffie-Hellman
│   ├── obfs/             # Обфускация
│   │   └── obfs4.rs      # obfs4 реализация
│   ├── stego/            # Стенография
│   │   └── lsb.rs        # LSB метод
│   ├── tdlib/            # TDLib обёртка
│   │   └── client.rs     # TDLib клиент
│   ├── updater/          # Автообновление
│   │   └── github.rs     # GitHub Releases API
│   └── config/           # Конфигурация
│       └── settings.rs   # Настройки
├── tests/                # Тесты
├── scripts/              # Скрипты сборки
├── .github/workflows/    # GitHub Actions
└── Cargo.toml
```

## 🛠️ Конфигурация

Создайте `config.json`:

```json
{
  "api_id": 123456,
  "api_hash": "your_api_hash",
  "encryption": {
    "kyber_enabled": true,
    "steganography_enabled": true,
    "obfuscation_enabled": true
  },
  "proxy": {
    "enabled": false,
    "host": "127.0.0.1",
    "port": 1080
  },
  "auto_update": true
}
```

## 🧪 Тестирование

```bash
# Запустить все тесты
cargo test

# Тесты криптографии
cargo test crypto

# Тесты стенографии
cargo test stego
```

## 📊 Бенчмарки

```bash
cargo bench
```

## ⚠️ Предупреждения

1. **Не используйте для критически важных данных** - это экспериментальный проект
2. **Telegram может заблокировать** - модифицированный протокол
3. **Производительность** - шифрование добавляет задержку

## 📜 Лицензия

MIT License

## 🤝 Contributing

1. Fork репозиторий
2. Создай ветку (`git checkout -b feature/amazing-feature`)
3. Commit изменений (`git commit -m 'Add amazing feature'`)
4. Push (`git push origin feature/amazing-feature`)
5. Открой Pull Request

## 📞 Контакты

- GitHub Issues
- Telegram: @your_contact

---

**⚠️ Disclaimer**: Этот проект создан в образовательных целях. Используйте на свой страх и риск.
