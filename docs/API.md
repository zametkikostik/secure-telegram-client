# Liberty Reach — API Документация

## 📖 Содержание

1. [Обзор](#обзор)
2. [Аутентификация](#аутентификация)
3. [Чаты](#чаты)
4. [Звонки](#звонки)
5. [Web3](#web3)
6. [AI](#ai)

---

## 📡 Обзор

### Base URL

```
Production: https://api.libertyreach.io
Self-hosted: http://localhost:8008
```

### Формат запросов

```json
Content-Type: application/json
Authorization: Bearer <token>
```

### Коды ответов

| Код | Описание |
|-----|----------|
| 200 | Успех |
| 400 | Неверный запрос |
| 401 | Неавторизован |
| 404 | Не найдено |
| 500 | Ошибка сервера |

---

## 🔐 Аутентификация

### Регистрация

```http
POST /api/v1/auth/register
```

**Request:**
```json
{
  "username": "user123",
  "password": "secure_password"
}
```

**Response:**
```json
{
  "user_id": "uuid",
  "token": "jwt_token",
  "mnemonic": "word1 word2 ... word12"
}
```

### Вход

```http
POST /api/v1/auth/login
```

**Request:**
```json
{
  "username": "user123",
  "password": "secure_password"
}
```

**Response:**
```json
{
  "user_id": "uuid",
  "token": "jwt_token"
}
```

---

## 💬 Чаты

### Получить список чатов

```http
GET /api/v1/chats
```

**Response:**
```json
{
  "chats": [
    {
      "id": "uuid",
      "type": "private|group|channel",
      "name": "Chat Name",
      "last_message": "Hello!",
      "last_message_at": "2024-01-01T00:00:00Z",
      "unread_count": 5
    }
  ]
}
```

### Отправить сообщение

```http
POST /api/v1/chats/{chat_id}/messages
```

**Request:**
```json
{
  "text": "Hello, World!",
  "reply_to": "message_id"
}
```

**Response:**
```json
{
  "message_id": "uuid",
  "timestamp": "2024-01-01T00:00:00Z"
}
```

### Получить сообщения

```http
GET /api/v1/chats/{chat_id}/messages?limit=50&offset=0
```

**Response:**
```json
{
  "messages": [
    {
      "id": "uuid",
      "from": "user_id",
      "text": "Hello!",
      "timestamp": "2024-01-01T00:00:00Z",
      "translated_text": "Здравей!",
      "is_read": true
    }
  ]
}
```

---

## 📞 Звонки

### Начать звонок

```http
POST /api/v1/calls
```

**Request:**
```json
{
  "callee": "user_id",
  "type": "audio|video"
}
```

**Response:**
```json
{
  "call_id": "uuid",
  "status": "ringing"
}
```

### Отправить SDP Offer

```http
POST /api/v1/calls/{call_id}/offer
```

**Request:**
```json
{
  "sdp": "v=0..."
}
```

### Получить ICE кандидаты

```http
GET /api/v1/calls/{call_id}/ice-candidates
```

---

## 💰 Web3

### Получить баланс

```http
GET /api/v1/wallet/balance
```

**Response:**
```json
{
  "ETH": "1.5",
  "USDT": "1000.0",
  "USDC": "500.0"
}
```

### Обменять токены (0x)

```http
POST /api/v1/web3/swap
```

**Request:**
```json
{
  "from_token": "ETH",
  "to_token": "USDT",
  "amount": "0.1"
}
```

**Response:**
```json
{
  "quote_id": "uuid",
  "rate": "2000.0",
  "fee": "0.002",
  "total": "199.6"
}
```

### Создать P2P сделку

```http
POST /api/v1/web3/p2p/deals
```

**Request:**
```json
{
  "seller": "0x...",
  "amount": "100",
  "token": "USDT"
}
```

**Response:**
```json
{
  "deal_id": "123",
  "contract_address": "0x...",
  "status": "pending"
}
```

---

## 🤖 AI

### Перевод текста

```http
POST /api/v1/ai/translate
```

**Request:**
```json
{
  "text": "Hello, World!",
  "from": "en",
  "to": "bg"
}
```

**Response:**
```json
{
  "translated_text": "Здравей, свят!"
}
```

### Саммаризация

```http
POST /api/v1/ai/summarize
```

**Request:**
```json
{
  "text": "Long text..."
}
```

**Response:**
```json
{
  "summary": "Short summary..."
}
```

### Генерация кода

```http
POST /api/v1/ai/generate-code
```

**Request:**
```json
{
  "description": "Function to add two numbers",
  "language": "rust"
}
```

**Response:**
```json
{
  "code": "fn add(a: i32, b: i32) -> i32 { a + b }"
}
```

### Speech-to-Text

```http
POST /api/v1/ai/speech-to-text
Content-Type: multipart/form-data
```

**Request:**
```
file: audio.wav
```

**Response:**
```json
{
  "text": "Привет, как дела?"
}
```

---

## 📊 WebSocket

### Подключение

```
wss://api.libertyreach.io/ws
```

### Сообщения

**Подписка на чат:**
```json
{
  "type": "subscribe",
  "chat_id": "uuid"
}
```

**Получение сообщения:**
```json
{
  "type": "message",
  "chat_id": "uuid",
  "message": {
    "id": "uuid",
    "text": "Hello!"
  }
}
```

---

## 📬 Поддержка

Email: api@libertyreach.io  
Docs: https://docs.libertyreach.io
