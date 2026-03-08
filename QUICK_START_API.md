# 🎉 LIBERTY REACH MESSENGER v2.0
## БЫСТРЫЙ СТАРТ - НАСТРОЙКА API КЛЮЧЕЙ

---

## ✅ ЧТО УЖЕ ГОТОВО

- ✅ Все модули реализованы (14 модулей, ~10,000 строк)
- ✅ Компиляция без ошибок
- ✅ База данных SQLCipher
- ✅ P2P сеть
- ✅ AI перевод
- ✅ WebRTC звонки
- ✅ Боты платформа
- ✅ Web3 интеграции
- ✅ Конфигурация через toml

---

## 🚀 БЫСТРЫЙ СТАРТ (5 МИНУТ)

### Шаг 1: Сгенерировать ключ шифрования

```bash
cd /home/kostik/secure-telegram-client
cargo run --bin generate_key
```

**Скопируй ключ** и сохрани для следующего шага!

---

### Шаг 2: Создать config.toml

```bash
cp config.toml.example config.toml
nano config.toml  # Или любой редактор
```

---

### Шаг 3: Заполнить API ключи

#### 🔐 Обязательно (для тестов):

```toml
[database]
encryption_key = "ВСТАВЬ_КЛЮЧ_ИЗ_ШАГА_1"

[api_keys.bitget]
api_key = "test"  # Для тестов
secret_key = "test"
passphrase = "test"
testnet = true

[api_keys.infura]
project_id = "test"  # Для тестов
```

#### 💰 Опционально (для production):

Получи реальные ключи:

1. **Bitget** (покупка криптовалюты):
   - https://www.bitget.com → Profile → API Management
   - Вставь в `api_keys.bitget`

2. **Qwen** (AI перевод):
   - https://dashscope.console.aliyun.com/
   - Вставь в `api_keys.qwen`

3. **Infura** (Web3):
   - https://infura.io → Create Project
   - Вставь в `api_keys.infura`

---

### Шаг 4: Проверить конфигурацию

```bash
# Запустить проверку
cargo check -p liberty-reach-core

# Должно вывести:
# ✅ Finished dev profile [unoptimized + debuginfo] target(s)
```

---

### Шаг 5: Запустить мессенджер

```bash
# Запуск с конфигурацией
cargo run --release -- --config config.toml

# Или через CLI
./target/release/liberty-reach --config config.toml
```

---

## 📋 ТАБЛИЦА API КЛЮЧЕЙ

| Сервис | Обязательно | Тестовый режим | Production |
|--------|-------------|----------------|------------|
| **Database** | ✅ Обязательно | Генерация через `generate_key` | Тот же ключ |
| **Bitget** | ❌ Нет | `testnet = true` | Реальные ключи |
| **0x Protocol** | ❌ Нет | Бесплатно до 1000 запросов | API key |
| **ABCEX** | ❌ Нет | `testnet = true` | Реальные ключи |
| **Qwen** | ❌ Нет | Платный | API key |
| **Infura** | ❌ Нет | Бесплатно до 100k запросов | Project ID |

---

## 🔐 БЕЗОПАСНОСТЬ

### ⚠️ ВАЖНО!

1. **Никогда не коммить `config.toml` в git!**
   ```bash
   echo "config.toml" >> .gitignore
   git add .gitignore
   ```

2. **Используй testnet для тестов!**
   ```toml
   testnet = true  # Меньше комиссий, безопасно
   ```

3. **Ограничь права API ключей!**
   - Bitget: Только Spot Trading + Read
   - Не давай права на вывод!

4. **Храни ключи в секрете!**
   - Используй менеджеры паролей (1Password, Bitwarden)
   - Никогда не передавай ключи

---

## 📊 ПРОВЕРКА РАБОТОСПОСОБНОСТИ

### Тест 1: База данных

```bash
cargo run --release
# ✓ Database initialized
# ✓ Liberty Reach Core v2.0.0 initialized
```

### Тест 2: API ключи

```bash
cargo run --release -- --validate-config
# ✓ Configuration loaded
# ✓ API keys validated
```

### Тест 3: Web3

```bash
cargo run --release -- --test-web3
# ✓ Web3 provider connected
# ✓ Chain ID: 1 (Ethereum)
```

### Тест 4: AI

```bash
cargo run --release -- --test-ai "Hello"
# ✓ AI response: Hello! How can I help you?
```

---

## 🛠️ ТРАБЛШУТИНГ

### Ошибка: "Failed to read config file"

**Решение:**
```bash
# Проверь что файл существует
ls -la config.toml

# Если нет, скопируй пример
cp config.toml.example config.toml
```

### Ошибка: "Invalid encryption key"

**Решение:**
```bash
# Сгенерируй новый ключ
cargo run --bin generate_key

# Вставь в config.toml
encryption_key = "новый_ключ"
```

### Ошибка: "API key invalid"

**Решение:**
1. Проверь что ключи скопированы без пробелов
2. Убедись что `testnet = true` для тестов
3. Проверь права доступа API ключей

---

## 📞 ПОДДЕРЖКА

Если возникли проблемы:

1. Посмотри логи: `tail -f liberty-reach.log`
2. Проверь конфигурацию: `cargo run -- --validate-config`
3. Прочитай `API_KEYS_GUIDE.md`
4. Создай issue на GitHub

---

## 🎯 СЛЕДУЮЩИЕ ШАГИ

1. ✅ Настроить API ключи (5 мин)
2. ✅ Протестировать локально (10 мин)
3. ⏳ Собрать Flutter UI (2-4 недели)
4. ⏳ Бета тестирование (1-2 недели)
5. ⏳ Production релиз 🚀

---

*Liberty Reach Messenger v2.0*  
*Быстрый старт*  
*8 марта 2026*

**🎉 ГОТОВО! МОЖНО ЗАПУСКАТЬ! 🎉**
