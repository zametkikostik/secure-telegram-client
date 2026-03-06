# 🆕 Новые функции Secure Telegram

Все функции из Telegram реализованы в Secure Telegram!

## 📋 Реализованные функции

### ✅ 24-часовые сообщения (Исчезающие)

**API:** `POST /chats/:chat_id/messages/auto-delete`

Сообщения автоматически удаляются через 24 часа после отправки.

```json
// Запрос
{
  "content": "Это сообщение исчезнет через 24 часа",
  "type": "text"
}

// Ответ
{
  "id": "msg_123",
  "chat_id": "chat_456",
  "sender_id": "user_789",
  "content": "Это сообщение исчезнет через 24 часа",
  "auto_delete_hours": 24,
  "delete_at": "2026-03-07T18:00:00Z",
  "created_at": "2026-03-06T18:00:00Z"
}
```

**Настройка автоудаления:**

```bash
# Установить автоудаление для чата
POST /chats/:chat_id/auto-delete
{
  "hours": 24
}
```

### 💕 Семейные статусы

**API:** `GET/POST /users/:user_id/family-status`

Поддерживаемые статусы:
- `single` — Не женат/Не замужем
- `in_relationship` — Встречаюсь
- `engaged` — Помолвлен(а)
- `married` — Женат/Замужем
- `in_civil_partnership` — В гражданском браке
- `divorced` — Разведён(а)
- `widowed` — Вдовец/Вдова

```json
// Запрос
{
  "status": "married",
  "partner_id": "user_partner_123"
}

// Ответ
{
  "status": "married",
  "partner_id": "user_partner_123",
  "partner_name": "Иван Иванов"
}
```

**Семейные связи:**

При установке статуса `married` или `in_relationship`:
- Создаётся связь в таблице `family_relations`
- Статус партнёра автоматически обновляется
- Появляется возможность общих чатов

### 🖼️ Синхронизированные обои чата

**API:** `GET/POST /chats/:chat_id/wallpaper`

Установите обои чата, которые будут отображаться у обоих собеседников!

```json
// Запрос
{
  "wallpaper_url": "/wallpapers/roses.jpg",
  "wallpaper_type": "nature",
  "sync_to_chat": true
}

// Ответ
{
  "chat_id": "chat_456",
  "wallpaper_url": "/wallpapers/roses.jpg",
  "wallpaper_type": "nature",
  "synced": true
}
```

**Предустановленные обои:**

```bash
# Получить список всех обоев
GET /wallpapers
```

#### Категории обоев:

**🌸 Природа:**
- `roses` — Розы
- `nature` — Природа
- `mountains` — Горы
- `ocean` — Океан
- `sunset` — Закат

**🎨 Абстракции:**
- `abstract` — Абстракция
- `geometric` — Геометрия

**⚫ Сплошные:**
- `dark` — Тёмная
- `light` — Светлая
- `blue` — Синяя
- `green` — Зелёная

### 📡 WebSocket уведомления

При изменении обоев или семейного статуса, все участники чата получают уведомление через WebSocket:

```json
{
  "type": "wallpaper_updated",
  "chat_id": "chat_456",
  "wallpaper_url": "/wallpapers/roses.jpg",
  "updated_by": "user_123",
  "timestamp": "2026-03-06T18:00:00Z"
}
```

```json
{
  "type": "family_status_updated",
  "user_id": "user_123",
  "status": "married",
  "partner_id": "user_456",
  "timestamp": "2026-03-06T18:00:00Z"
}
```

## 🗑️ Очистка просроченных сообщений

Задача по расписанию автоматически удаляет сообщения с истёкшим сроком:

```rust
// Запуск каждые 5 минут
tokio::spawn(async {
    let mut interval = tokio::time::interval(Duration::from_secs(300));
    loop {
        interval.tick().await;
        match features::cleanup_expired_messages(&db).await {
            Ok(count) => tracing::info!("Удалено {} просроченных сообщений", count),
            Err(e) => tracing::error!("Ошибка очистки: {}", e),
        }
    }
});
```

## 📱 Клиентская реализация

### React Native (Android)

```typescript
// Установка семейного статуса
async function setFamilyStatus(status: string, partnerId?: string) {
  const response = await fetch(`/api/v1/users/${userId}/family-status`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ status, partner_id: partnerId }),
  });
  return response.json();
}

// Установка обоев чата
async function setChatWallpaper(chatId: string, url: string, sync: boolean) {
  const response = await fetch(`/api/v1/chats/${chatId}/wallpaper`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      wallpaper_url: url,
      wallpaper_type: 'custom',
      sync_to_chat: sync,
    }),
  });
  return response.json();
}

// Отправка 24h сообщения
async function sendAutoDeleteMessage(chatId: string, content: string) {
  const response = await fetch(`/api/v1/chats/${chatId}/messages/auto-delete`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ content }),
  });
  return response.json();
}
```

### Tauri (Desktop)

```rust
// Получение списка обоев
async fn list_wallpapers() -> Result<Vec<WallpaperPreset>> {
    let response = reqwest::get("http://localhost:8008/api/v1/wallpapers")
        .await?
        .json()
        .await?;
    Ok(response)
}

// Установка семейного статуса
async fn set_family_status(status: &str, partner_id: Option<&str>) -> Result<FamilyStatus> {
    let client = reqwest::Client::new();
    let response = client
        .post(&format!("http://localhost:8008/api/v1/users/{}/family-status", user_id))
        .json(&SetFamilyStatusRequest {
            status: status.to_string(),
            partner_id: partner_id.map(String::from),
        })
        .send()
        .await?
        .json()
        .await?;
    Ok(response)
}
```

## 📊 База данных

### Таблица `users`

```sql
ALTER TABLE users ADD COLUMN family_status TEXT DEFAULT 'single';
```

### Таблица `chats`

```sql
ALTER TABLE chats ADD COLUMN wallpaper_url TEXT;
ALTER TABLE chats ADD COLUMN wallpaper_sync BOOLEAN DEFAULT FALSE;
ALTER TABLE chats ADD COLUMN auto_delete_hours INTEGER DEFAULT NULL;
```

### Таблица `messages`

```sql
ALTER TABLE messages ADD COLUMN auto_delete_hours INTEGER DEFAULT NULL;
ALTER TABLE messages ADD COLUMN delete_at DATETIME DEFAULT NULL;
CREATE INDEX idx_messages_delete_at ON messages(delete_at) WHERE delete_at IS NOT NULL;
```

### Таблица `family_relations`

```sql
CREATE TABLE family_relations (
    id TEXT PRIMARY KEY,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    relative_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    relation_type TEXT NOT NULL, -- 'partner', 'parent', 'child', 'sibling'
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(user_id, relative_id)
);
```

### Таблица `chat_wallpapers`

```sql
CREATE TABLE chat_wallpapers (
    chat_id TEXT REFERENCES chats(id) ON DELETE CASCADE,
    user_id TEXT REFERENCES users(id) ON DELETE CASCADE,
    wallpaper_url TEXT NOT NULL,
    wallpaper_type TEXT DEFAULT 'custom',
    synced BOOLEAN DEFAULT FALSE,
    updated_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (chat_id, user_id)
);
```

## 🔗 API Endpoints

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `GET` | `/users/:user_id/family-status` | Получить семейный статус |
| `POST` | `/users/:user_id/family-status` | Установить семейный статус |
| `GET` | `/chats/:chat_id/wallpaper` | Получить обои чата |
| `POST` | `/chats/:chat_id/wallpaper` | Установить обои чата |
| `GET` | `/wallpapers` | Список всех обоев |
| `POST` | `/chats/:chat_id/auto-delete` | Настроить автоудаление |
| `POST` | `/chats/:chat_id/messages/auto-delete` | Отправить сообщение с автоудалением |

---

**Secure Telegram Team © 2026**
