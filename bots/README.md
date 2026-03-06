# 🤖 Secure Telegram Bots Platform

> **BotFather + ManyChat аналог + IPFS интеграция**

Полноценная платформа для создания и управления ботами.

## 🚀 Возможности

### 👨‍💼 BotFather аналог

- ✅ Создание ботов через API
- ✅ Управление токенами ботов
- ✅ Верификация ботов
- ✅ Статистика ботов

### 🏗️ ManyChat конструктор ботов

- ✅ Визуальный конструктор flow (сценариев)
- ✅ Блоки: сообщение, кнопка, условие, задержка
- ✅ Триггеры: keywords, команды, события
- ✅ Webhooks для интеграции

### 🌐 IPFS через Pinata.cloud

- ✅ Загрузка файлов на IPFS
- ✅ Загрузка JSON метаданных
- ✅ Gateway для доступа к файлам
- ✅ Интеграция с ботами

## 📋 Быстрый старт

### 1. Установка зависимостей

```bash
cd bots
cargo build --release
```

### 2. Настройка переменных окружения

```bash
export BOTS_ADDR="0.0.0.0:8081"
export PINATA_API_KEY="your-pinata-api-key"
export PINATA_SECRET_KEY="your-pinata-secret-key"
```

### 3. Запуск

```bash
cargo run --release
```

## 🔗 API Endpoints

### BotFather API

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `GET` | `/api/v1/bots` | Список ботов |
| `POST` | `/api/v1/bots` | Создать бота |
| `GET` | `/api/v1/bots/:bot_id` | Информация о боте |
| `DELETE` | `/api/v1/bots/:bot_id` | Удалить бота |
| `POST` | `/api/v1/bots/:bot_id/token` | Пересоздать токен |

### Bot Builder (Flow)

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `GET` | `/api/v1/bots/:bot_id/flows` | Список flow |
| `POST` | `/api/v1/bots/:bot_id/flows` | Создать flow |
| `GET` | `/api/v1/bots/:bot_id/flows/:flow_id` | Получить flow |
| `PUT` | `/api/v1/bots/:bot_id/flows/:flow_id` | Обновить flow |
| `DELETE` | `/api/v1/bots/:bot_id/flows/:flow_id` | Удалить flow |

### Блоки

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `GET` | `/api/v1/bots/:bot_id/blocks` | Список блоков |
| `POST` | `/api/v1/bots/:bot_id/blocks` | Создать блок |
| `PUT` | `/api/v1/blocks/:block_id` | Обновить блок |
| `DELETE` | `/api/v1/blocks/:block_id` | Удалить блок |

### Webhooks

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `GET` | `/api/v1/bots/:bot_id/webhook` | Получить webhooks |
| `POST` | `/api/v1/bots/:bot_id/webhook` | Установить webhook |
| `DELETE` | `/api/v1/bots/:bot_id/webhook` | Удалить webhook |

### IPFS

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `POST` | `/api/v1/ipfs/upload` | Загрузить файл |
| `GET` | `/api/v1/ipfs/:cid` | Получить информацию |

## 📖 Примеры использования

### Создание бота (BotFather)

```bash
curl -X POST http://localhost:8081/api/v1/bots \
  -H "Content-Type: application/json" \
  -d '{
    "username": "my_awesome_bot",
    "name": "My Awesome Bot",
    "description": "This is my first bot"
  }'
```

**Ответ:**
```json
{
  "id": "bot_123",
  "owner_id": "user_456",
  "username": "my_awesome_bot",
  "name": "My Awesome Bot",
  "description": "This is my first bot",
  "token": "bot_abc123xyz789",
  "is_verified": false,
  "created_at": "2026-03-06T18:00:00Z"
}
```

### Создание flow (сценария)

```bash
curl -X POST http://localhost:8081/api/v1/bots/bot_123/flows \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Welcome Flow",
    "description": "Приветствие новых пользователей",
    "trigger_type": "keyword",
    "trigger_value": "/start"
  }'
```

### Создание блоков

```bash
# Блок приветствия
curl -X POST http://localhost:8081/api/v1/bots/bot_123/blocks \
  -H "Content-Type: application/json" \
  -d '{
    "flow_id": "flow_123",
    "block_type": "message",
    "content": "Привет! Добро пожаловать!",
    "position": 0
  }'

# Блок с кнопками
curl -X POST http://localhost:8081/api/v1/bots/bot_123/blocks \
  -H "Content-Type: application/json" \
  -d '{
    "flow_id": "flow_123",
    "block_type": "buttons",
    "content": "{\"text\": \"Выберите опцию:\", \"buttons\": [\"О нас\", \"Контакты\", \"Помощь\"]}",
    "position": 1
  }'
```

### Загрузка файла на IPFS

```bash
curl -X POST http://localhost:8081/api/v1/ipfs/upload \
  -H "pinata_api_key: your-api-key" \
  -H "pinata_secret_key: your-secret-key" \
  -F "file=@image.png"
```

**Ответ:**
```json
{
  "id": "file_123",
  "cid": "QmHash123...",
  "filename": "image.png",
  "size": 102400,
  "ipfs_url": "ipfs://QmHash123...",
  "gateway_url": "https://gateway.pinata.cloud/ipfs/QmHash123..."
}
```

## 🏗️ Типы блоков

| Тип | Описание | Пример |
|-----|----------|--------|
| `message` | Текстовое сообщение | "Привет!" |
| `image` | Изображение | URL картинки |
| `video` | Видео | URL видео |
| `audio` | Аудио | URL аудио |
| `buttons` | Кнопки | `{"text": "Выбор", "buttons": ["A", "B"]}` |
| `condition` | Условие | `{"if": "text == 'yes'", "then": "block_1"}` |
| `delay` | Задержка | `{"seconds": 5}` |
| `api_call` | HTTP запрос | `{"url": "https://api.example.com"}` |
| `set_variable` | Переменная | `{"name": "user_name", "value": "John"}` |

## 🎯 Триггеры

| Тип | Описание | Пример |
|-----|----------|--------|
| `keyword` | Ключевое слово | `/start`, "привет" |
| `command` | Команда | `/help`, `/settings` |
| `subscribe` | Подписка на бота | - |
| `unsubscribe` | Отписка | - |
| `message` | Любое сообщение | - |
| `payload` | Payload кнопка | `PAYLOAD_MENU` |

## 📊 Статистика бота

```bash
curl http://localhost:8081/api/v1/bots/bot_123/stats
```

**Ответ:**
```json
{
  "bot_id": "bot_123",
  "total_subscribers": 1500,
  "total_messages": 25000,
  "active_flows": 5,
  "total_blocks": 42
}
```

## 🔐 Безопасность

- ✅ Уникальные токены для каждого бота
- ✅ Webhook secrets для верификации
- ✅ Rate limiting (опционально)
- ✅ CORS настройки

## 🌐 Pinata.cloud API

Если нужен API от Pinata.cloud:

```bash
export PINATA_API_KEY="your-api-key"
export PINATA_SECRET_KEY="your-secret-key"
```

Или используйте свой Pinata API из `.env` файла.

## 📱 Интеграция с мессенджером

Боты интегрируются через WebSocket:

```typescript
// Подключение бота
const bot = new SecureBot({
  token: 'bot_abc123',
  webhook: 'https://your-server.com/webhook'
});

bot.onMessage(async (message) => {
  if (message.text === '/start') {
    await bot.sendMessage(message.chat_id, 'Привет!');
  }
});

bot.start();
```

---

**Secure Telegram Team © 2026**
