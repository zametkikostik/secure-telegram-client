# ⏱️ Таймер самоуничтожения сообщений

Реализована функция таймера самоуничтожения сообщений как в Telegram!

## 🚀 Возможности

### 1. Настройка таймера для чата
Установите время жизни для всех сообщений в чате.

**API:** `POST /chats/:chat_id/self-destruct`

```json
// Запрос
{
  "timer_seconds": 3600  // 1 час
}

// Ответ
{
  "chat_id": "chat_123",
  "timer_seconds": 3600,
  "enabled": true
}
```

### 2. Отправка сообщения с таймером
Отправьте сообщение, которое удалится через заданное время.

**API:** `POST /chats/:chat_id/messages/self-destruct`

```json
// Запрос
{
  "content": "Это сообщение исчезнет через 1 минуту",
  "timer_seconds": 60  // 1 минута
}

// Ответ
{
  "id": "msg_123",
  "chat_id": "chat_456",
  "content": "Это сообщение исчезнет через 1 минуту",
  "self_destruct_timer": 60,
  "delete_at": "2026-03-06T19:01:00Z",
  "created_at": "2026-03-06T19:00:00Z"
}
```

### 3. Отключение таймера
Отключите таймер самоуничтожения для чата.

**API:** `POST /chats/:chat_id/self-destruct/disable`

### 4. Получение настроек таймера
Узнайте текущие настройки таймера чата.

**API:** `GET /chats/:chat_id/self-destruct`

```json
// Ответ
{
  "chat_id": "chat_123",
  "timer_seconds": 3600,
  "enabled": true
}
```

## ⏰ Предустановленные значения таймера

| Время | Seconds |
|-------|---------|
| 1 минута | 60 |
| 1 час | 3600 |
| 1 день | 86400 |
| 1 неделя | 604800 |
| 1 месяц | 2592000 |

## 📱 Клиентская реализация

### React Native (Android)

```typescript
// Установить таймер для чата
async function setChatSelfDestruct(chatId: string, seconds: number) {
  const response = await fetch(`/api/v1/chats/${chatId}/self-destruct`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ timer_seconds: seconds }),
  });
  return response.json();
}

// Отправить сообщение с таймером
async function sendSelfDestructMessage(chatId: string, content: string, seconds: number) {
  const response = await fetch(`/api/v1/chats/${chatId}/messages/self-destruct`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      content,
      timer_seconds: seconds,
    }),
  });
  return response.json();
}

// Отключить таймер
async function disableSelfDestruct(chatId: string) {
  const response = await fetch(`/api/v1/chats/${chatId}/self-destruct/disable`, {
    method: 'POST',
  });
  return response.status === 200;
}

// Получить настройки таймера
async function getSelfDestructSettings(chatId: string) {
  const response = await fetch(`/api/v1/chats/${chatId}/self-destruct`);
  return response.json();
}
```

### Tauri (Desktop)

```rust
// Установить таймер
async fn set_self_destruct(chat_id: &str, seconds: i64) -> Result<SelfDestructConfig> {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("http://localhost:8008/api/v1/chats/{}/self-destruct", chat_id))
        .json(&SetSelfDestructRequest { timer_seconds: seconds })
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}

// Отправить сообщение с таймером
async fn send_self_destruct_msg(chat_id: &str, content: &str, seconds: i64) -> Result<Message> {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("http://localhost:8008/api/v1/chats/{}/messages/self-destruct", chat_id))
        .json(&SendSelfDestructRequest {
            content: content.to_string(),
            timer_seconds: seconds,
            ..Default::default()
        })
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}
```

## 🗑️ Автоматическое удаление

Сообщения автоматически удаляются каждые 5 минут:

```rust
// server/src/main.rs
tokio::spawn(async move {
    let mut interval = interval(Duration::from_secs(300)); // Каждые 5 минут
    loop {
        interval.tick().await;
        match sqlx::query("DELETE FROM messages WHERE delete_at IS NOT NULL AND delete_at < CURRENT_TIMESTAMP")
            .execute(&db)
            .await
        {
            Ok(result) => {
                let count = result.rows_affected();
                if count > 0 {
                    tracing::info!("Удалено {} сообщений с таймером", count);
                }
            }
            Err(e) => {
                tracing::error!("Ошибка очистки: {}", e);
            }
        }
    }
});
```

## 📊 База данных

### Таблица chats (обновлено)

```sql
ALTER TABLE chats ADD COLUMN self_destruct_timer INTEGER DEFAULT NULL;
```

### Таблица messages (обновлено)

```sql
ALTER TABLE messages ADD COLUMN self_destruct_timer INTEGER DEFAULT NULL;
ALTER TABLE messages ADD COLUMN delete_at DATETIME DEFAULT NULL;
CREATE INDEX idx_messages_delete_at ON messages(delete_at) WHERE delete_at IS NOT NULL;
```

## 🔗 API Endpoints

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `GET` | `/chats/:chat_id/self-destruct` | Получить настройки таймера |
| `POST` | `/chats/:chat_id/self-destruct` | Установить таймер |
| `POST` | `/chats/:chat_id/self-destruct/disable` | Отключить таймер |
| `POST` | `/chats/:chat_id/messages/self-destruct` | Отправить сообщение с таймером |

## 🎯 Примеры использования

### Секретный чат (1 минута)

```typescript
// Установить таймер 1 минута для секретного чата
await setChatSelfDestruct(secretChatId, 60);

// Отправить сообщение
await sendSelfDestructMessage(secretChatId, "Секретная информация", 60);
```

### Временный чат (1 час)

```typescript
// Установить таймер 1 час
await setChatSelfDestruct(tempChatId, 3600);
```

### Отключение таймера

```typescript
// Отключить таймер
await disableSelfDestruct(chatId);
```

## ✅ Статус

- ✅ Настройка таймера для чата
- ✅ Отправка сообщений с таймером
- ✅ Автоматическое удаление
- ✅ API для получения настроек
- ✅ Отключение таймера

---

**Secure Telegram Team © 2026**
