# 🔐 ENV LOCAL - НАСТРОЙКА СЕКРЕТНЫХ КЛЮЧЕЙ

## 📍 ГДЕ НАХОДИТСЯ

```
/home/kostik/secure-telegram-client/.env.local
```

---

## ⚠️ ВАЖНО!

**`.env.local` содержит СЕКРЕТНЫЕ КЛЮЧИ!**

- ✅ **НИКОГДА не коммить** в git
- ✅ **НЕ передавай** этот файл никому
- ✅ **Храни** в секрете
- ✅ Файл уже добавлен в `.gitignore`

---

## 🚀 БЫСТРЫЙ СТАРТ

### 1. Скопируй шаблон

```bash
cd /home/kostik/secure-telegram-client
cp .env.example .env.local
```

### 2. Открой для редактирования

```bash
nano .env.local
```

### 3. Заполни ключи

#### 🔑 Bitget API (Криптобиржа)

```bash
# Получить: https://www.bitget.com → Profile → API Management
BITGET_API_KEY=a1b2c3d4-5678-90ab-cdef-1234567890ab
BITGET_SECRET_KEY=x9y8z7w6v5u4t3s2r1q0p9o8n7m6l5k4
BITGET_PASSPHRASE=MySecurePass123!
BITGET_TESTNET=true
```

#### 🤖 Qwen AI (Перевод)

```bash
# Получить: https://dashscope.console.aliyun.com/
QWEN_API_KEY=sk-qwen1234567890abcdef
QWEN_MODEL=qwen-max
QWEN_MAX_TOKENS=2000
```

#### 🔗 Infura (Web3)

```bash
# Получить: https://infura.io → Create Project
INFURA_PROJECT_ID=a1b2c3d4e5f6g7h8i9j0
INFURA_PROJECT_SECRET=your-project-secret
```

#### 🔐 База данных

```bash
# Сгенерировать: cargo run --bin generate_key
DB_ENCRYPTION_KEY=cce76cf11f43b4cb4c4ad5a9ed5026bea6f6bc00d0f7cf305360e74cfb454a47
```

### 4. Сохрани

В nano:
- **Ctrl + O** → **Enter** (сохранить)
- **Ctrl + X** (выйти)

---

## 📋 ВСЕ ПЕРЕМЕННЫЕ

### Cryptocurrency

| Переменная | Описание | Где получить |
|------------|----------|--------------|
| `BITGET_API_KEY` | Bitget API ключ | bitget.com |
| `BITGET_SECRET_KEY` | Bitget секретный ключ | bitget.com |
| `BITGET_PASSPHRASE` | Bitget пароль | Придумай сам |
| `ZEROEX_API_KEY` | 0x Protocol API | 0x.org |
| `ABCEX_API_KEY` | ABCEX API | abcex.com |

### AI Services

| Переменная | Описание | Где получить |
|------------|----------|--------------|
| `QWEN_API_KEY` | Qwen API ключ | dashscope.aliyun.com |
| `QWEN_MODEL` | Модель AI | qwen-max / qwen-plus |

### Web3

| Переменная | Описание | Где получить |
|------------|----------|--------------|
| `INFURA_PROJECT_ID` | Infura Project ID | infura.io |
| `INFURA_PROJECT_SECRET` | Infura Secret | infura.io |

### Database

| Переменная | Описание | Где получить |
|------------|----------|--------------|
| `DB_ENCRYPTION_KEY` | Ключ шифрования БД | `cargo run --bin generate_key` |

---

## ✅ ПРОВЕРКА

После заполнения проверь:

```bash
# Проверить что файл существует
ls -la .env.local

# Посмотреть переменные
grep BITGET .env.local

# Запустить с .env.local
cargo run --release
```

В логах должно быть:
```
✓ Environment loaded from .env.local
✓ Liberty Reach Core v2.0.0 initialized
```

---

## 🔒 БЕЗОПАСНОСТЬ

### ✅ Что уже сделано:

- ✅ `.env.local` в `.gitignore`
- ✅ Шаблон `.env.example` без секретов
- ✅ Автоматическая загрузка при старте

### ⚠️ Что делать тебе:

1. **НЕ коммить `.env.local`!**
   ```bash
   git status  # Убедись что .env.local не в списке
   ```

2. **Используй testnet для тестов!**
   ```bash
   BITGET_TESTNET=true
   ABCEX_TESTNET=true
   ```

3. **Регулярно меняй ключи!**
   - Раз в 3-6 месяцев
   - При подозрении на утечку

4. **Ограничь права API!**
   - Bitget: Только Spot Trading + Read
   - Не давай права на вывод!

---

## 📁 СТРУКТУРА ФАЙЛОВ

```
/home/kostik/secure-telegram-client/
├── .env.local          # 🔐 СЕКРЕТНО! Не коммить!
├── .env.example        # ✅ Шаблон (можно коммить)
├── .gitignore          # ✅ .env.local уже добавлен
├── config.toml         # Конфигурация
└── config.toml.example # Шаблон конфига
```

---

## 🛠️ ТРАБЛШУТИНГ

### Ошибка: ".env.local not found"

**Решение:**
```bash
cp .env.example .env.local
nano .env.local  # Заполни ключи
```

### Ошибка: "Invalid API key"

**Решение:**
1. Проверь что ключи скопированы без пробелов
2. Убедись что `BITGET_TESTNET=true` для тестов
3. Проверь права доступа API ключей

### Ошибка: "Invalid encryption key"

**Решение:**
```bash
# Сгенерируй новый ключ
cargo run --bin generate_key

# Вставь в .env.local
DB_ENCRYPTION_KEY=новый_ключ
```

---

## 📞 ПОДДЕРЖКА

Если возникли проблемы:

1. Проверь `.env.local` существует
2. Проверь что ключи без пробелов
3. Посмотри логи: `tail -f liberty-reach.log`
4. Прочитай `API_KEYS_GUIDE.md`

---

*Liberty Reach Messenger v2.0*  
*ENV Local Configuration*  
*8 марта 2026*

**🔐 ХРАНИ КЛЮЧИ В СЕКРЕТЕ! 🔐**
