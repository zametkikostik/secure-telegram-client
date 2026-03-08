# 🔑 ИНСТРУКЦИЯ ПО НАСТРОЙКЕ API КЛЮЧЕЙ

## 📝 КРАТКОЕ РУКОВОДСТВО

### 1. Скопируйте пример конфигурации

```bash
cd /home/kostik/secure-telegram-client
cp config.toml.example config.toml
```

### 2. Заполните API ключи

Откройте `config.toml` в редакторе и замените `YOUR_*` на ваши ключи.

---

## 💰 КРИПТОБИРЖИ

### Bitget (Покупка криптовалюты)

**Где получить:**
1. Зайди на https://www.bitget.com
2. Залогинься → Profile (иконка человека)
3. API Management
4. Create API Key
5. Выбери разрешения:
   - ✅ Spot Trading
   - ✅ Read
6. Скопируй ключи (покажиются 1 раз!)

**Что вписать в config.toml:**
```toml
[api_keys.bitget]
api_key = "a1b2c3d4-..."  # Твой API Key
secret_key = "x9y8z7..."  # Твой Secret Key
passphrase = "ТВОЙ_ПАРОЛЬ"  # Придумай сам
testnet = true  # true для тестов, false для production
```

---

### 0x Protocol (DEX обмен)

**Где получить:**
1. Зайди на https://0x.org
2. Dashboard → API Keys
3. Create Key (бесплатно до 1000 запросов/мес)

**Что вписать:**
```toml
[api_keys.zeroex]
api_key = "ТВОЙ_0X_API_KEY"  # Или оставь пустым
chain_id = 1  # 1=Ethereum, 137=Polygon, 56=BSC
```

---

### ABCEX (Платёжный шлюз)

**Где получить:**
1. Зайди на https://abcex.com/merchant
2. Register как мерчант
3. Settings → API
4. Generate API Key

**Что вписать:**
```toml
[api_keys.abcex]
api_key = "ТВОЙ_ABCEX_API_KEY"
merchant_id = "ТВОЙ_MERCHANT_ID"
testnet = true  # true для тестов
```

---

## 🤖 AI СЕРВИСЫ

### Qwen API (Перевод, AI, TTS)

**Где получить:**
1. Зайди на https://dashscope.console.aliyun.com/
2. Register/Login
3. Console → API Keys
4. Create API Key

**Что вписать:**
```toml
[api_keys.qwen]
api_key = "sk-..."  # Твой Qwen API ключ
model = "qwen-max"  # или qwen-plus, qwen-turbo
max_tokens = 2000
```

---

## 🔗 WEB3 ИНФРАСТРУКТУРА

### Infura (Ethereum/Polygon узлы)

**Где получить:**
1. Зайди на https://infura.io
2. Sign Up
3. Create New Project
4. Выбери сеть: Ethereum + Polygon
5. Скопируй Project ID

**Что вписать:**
```toml
[api_keys.infura]
project_id = "a1b2c3d4e5f6..."  # Project ID из dashboard
project_secret = "..."  # Project Secret (опционально)
```

---

## 🔐 ГЕНЕРАЦИЯ КЛЮЧА ШИФРОВАНИЯ БД

Сгенерируй случайный 32-байтный ключ:

```bash
# Linux/Mac
openssl rand -hex 32

# Или в Rust
cargo run --bin generate-key
```

**Что вписать:**
```toml
[database]
encryption_key = "a1b2c3d4e5f6..."  # 64 hex символа = 32 байта
```

---

## ✅ ПРОВЕРКА КОНФИГУРАЦИИ

После заполнения `config.toml`:

```bash
# Проверить что файл валидный
cargo run --bin liberty-reach -- --config config.toml

# Должно вывести:
# ✓ Configuration loaded successfully
# ✓ Database initialized
# ✓ API keys validated
```

---

## 🚀 ЗАПУСК С КОНФИГУРАЦИЕЙ

```bash
# Запуск с config.toml
./target/release/liberty-reach --config config.toml

# Или через cargo
cargo run --release -- --config config.toml
```

---

## 📊 ТАБЛИЦА ВСЕХ API

| Сервис | Для чего | Обязательно | Тестовый режим |
|--------|----------|-------------|----------------|
| Bitget | Покупка крипты | ❌ Нет | ✅ Есть |
| 0x Protocol | DEX обмен | ❌ Нет | ✅ Есть |
| ABCEX | Платежи | ❌ Нет | ✅ Есть |
| Qwen | AI перевод | ❌ Нет | ❌ Нет |
| Infura | Web3 узлы | ❌ Нет | ✅ Есть |

---

## 🔒 БЕЗОПАСНОСТЬ

⚠️ **ВАЖНО:**

1. **Никогда не коммить `config.toml` в git!**
   ```bash
   echo "config.toml" >> .gitignore
   ```

2. **Используй testnet для тестов!**
   ```toml
   testnet = true  # Меньше комиссий, безопасно
   ```

3. **Ограничь права API ключей!**
   - Bitget: Только Spot Trading + Read
   - Не давай права на вывод!

4. **Регулярно меняй ключи!**
   - Раз в 3-6 месяцев
   - При подозрении на утечку

---

## 📞 ПОДДЕРЖКА

Если возникли проблемы:

1. Проверь что ключи скопированы без пробелов
2. Убедись что testnet = true для тестов
3. Проверь права доступа API ключей
4. Посмотри логи: `tail -f liberty-reach.log`

---

*Liberty Reach Messenger v2.0*  
*Настройка API ключей*  
*8 марта 2026*
