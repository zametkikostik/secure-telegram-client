# 🔌 WebSocket Real-Time Notifications

## 📋 Overview

Реализована полноценная WebSocket система для real-time уведомлений и P2P signaling в Secure Messenger.

### Возможности

✅ **Real-time уведомления** - Мгновенная доставка зашифрованных сообщений  
✅ **P2P Signaling** - Обмен WebRTC SDP/ICE candidates для прямых соединений  
✅ **Автоматический reconnection** - Exponential backoff при разрыве соединения  
✅ **Система подписок** - Подписка на различные каналы уведомлений  
✅ **Offline синхронизация** - Получение пропущенных сообщений через Cloudflare Worker  
✅ **Connection monitoring** - Отслеживание состояния соединения и online статуса  
✅ **Keep-alive ping/pong** - Автоматическое поддержание соединения  

## 🏗️ Architecture

### Компоненты системы

```
┌─────────────────────────────────────────────────────────────┐
│                         Frontend (React)                     │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────────┐  ┌─────────────────┐  ┌────────────┐ │
│  │  WebSocket       │  │  Offline Sync   │  │  Chat      │ │
│  │  Client          │  │  Service        │  │  Store     │ │
│  │                  │  │  (Cloudflare)   │  │  (Zustand) │ │
│  └────────┬─────────┘  └────────┬────────┘  └─────┬──────┘ │
│           │                     │                  │         │
└───────────┼─────────────────────┼──────────────────┼─────────┘
            │                     │                  │
            │ WebSocket           │ HTTP Polling     │ State
            │                     │                  │
┌───────────┼─────────────────────┼──────────────────┼─────────┐
│           ▼                     ▼                  ▼         │
│  ┌──────────────────────────────────────────────────────┐   │
│  │           Axum Backend Server (Rust)                 │   │
│  ├──────────────────────────────────────────────────────┤   │
│  │  ┌──────────────────┐  ┌──────────────────────────┐  │   │
│  │  │  Connection      │  │  WebSocket Handler       │  │   │
│  │  │  Manager         │  │  - P2P Signaling         │  │   │
│  │  │                  │  │  - Notifications         │  │   │
│  │  └──────────────────┘  └──────────────────────────┘  │   │
│  └──────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────┘
            │
            │ Fallback
            ▼
┌─────────────────────────────────────────────────────────────┐
│               Cloudflare Worker                             │
│          (Offline Message Storage)                          │
└─────────────────────────────────────────────────────────────┘
```

## 📁 Files Created

### Backend (Rust/Axum)

| File | Description |
|------|-------------|
| `server/src/ws/mod.rs` | WebSocket модуль |
| `server/src/ws/manager.rs` | ConnectionManager - управление соединениями и рассылка уведомлений |
| `server/src/ws/handler.rs` | Обработка WebSocket сообщений (P2P signaling, подписки) |
| `server/src/models/ws.rs` | Модели WebSocket сообщений (типы, каналы, payload) |
| `server/src/state.rs` | Обновлен для включения ConnectionManager |
| `server/src/routes/ws.rs` | Роут для WebSocket endpoint |
| `server/Cargo.toml` | Добавлена зависимость `futures` |

### Frontend (TypeScript/React)

| File | Description |
|------|-------------|
| `frontend/src/services/webSocketClient.ts` | WebSocket клиент с автоматическим reconnection |
| `frontend/src/services/offlineSyncService.ts` | Сервис синхронизации offline сообщений через Cloudflare |
| `frontend/src/services/chatStore.ts` | Интеграция WebSocket в Zustand store |
| `frontend/src/components/ConnectionStatusBar.tsx` | Компонент отображения статуса соединения |
| `frontend/src/App.tsx` | Инициализация WebSocket при старте приложения |
| `frontend/.env.example` | Переменные окружения |

### Documentation

| File | Description |
|------|-------------|
| `docs/WEBSOCKET_API.md` | Полная документация WebSocket API |
| `docs/WEBSOCKET_README.md` | Это файл - обзор реализации |

## 🚀 Quick Start

### 1. Запуск сервера

```bash
cd server
cargo run
# Server listening on 0.0.0.0:3000
```

### 2. Настройка frontend

```bash
cd frontend
cp .env.example .env.local
# Отредактируйте .env.local с вашими URL
```

### 3. Запуск frontend

```bash
npm run dev
```

### 4. Проверка WebSocket

Откройте браузер и проверьте консоль. Вы должны увидеть:
```
[WebSocket] Connecting to: ws://localhost:3000/api/v1/ws
[WebSocket] Connected
[WebSocket] Subscribed to channels: messages, p2p_signaling, delivery_status
[ChatStore] WebSocket state: connected
```

## 🔧 Usage Examples

### Подписка на уведомления

```typescript
import { getWebSocketClient } from './services/webSocketClient'

const wsClient = getWebSocketClient({
  userId: 'user-123',
  baseUrl: 'http://localhost:3000',
  autoConnect: true,
  channels: ['messages', 'p2p_signaling'],
})

// Получение новых сообщений
wsClient.onMessage('messages', (payload) => {
  console.log('Новое сообщение:', payload)
  // payload: { chat_id, message_id, sender_id, encrypted_content, timestamp }
})

// P2P signaling
wsClient.onP2PEvent((event) => {
  if (event.event_type === 'offer') {
    console.log('Получен WebRTC offer от:', event.peer_id)
  }
})

// Статус соединения
wsClient.onStateChange((state) => {
  console.log('Состояние:', state)
  // 'connecting' | 'connected' | 'disconnected' | 'reconnecting'
})
```

### P2P Signaling

```typescript
// Отправка WebRTC offer
wsClient.sendP2POffer(
  'user-456',
  sdpOffer,
  iceCandidates
)

// Отправка WebRTC answer
wsClient.sendP2PAnswer(
  'user-123',
  sdpAnswer,
  iceCandidates
)

// Отправка ICE candidate
wsClient.sendIceCandidate(
  'user-456',
  { candidate: 'candidate:...', sdp_mid: '0', sdp_m_line_index: 0 }
)
```

### Offline Sync

```typescript
import { createOfflineSyncService } from './services/offlineSyncService'

const syncService = createOfflineSyncService(
  { cloudflareUrl: 'https://secure-messenger-push.kostik.workers.dev' },
  wsClient,
  'user-123'
)

// Автоматически синхронизирует сообщения при отключении
```

## 📡 WebSocket Protocol

### Формат URL
```
ws://localhost:3000/api/v1/ws
wss://your-domain.com/api/v1/ws
```

### Типы сообщений

**Client → Server:**
- `subscribe` - Подписка на каналы
- `unsubscribe` - Отписка от каналов
- `p2p_offer` - WebRTC SDP offer
- `p2p_answer` - WebRTC SDP answer
- `ice_candidate` - ICE candidate
- `ping` - Keep-alive

**Server → Client:**
- `notification` - Уведомление (сообщение, статус, P2P событие)
- `pong` - Ответ на ping
- `error` - Ошибка сервера

### Каналы уведомлений

| Канал | Описание |
|-------|----------|
| `messages` | Новые зашифрованные сообщения |
| `p2p_signaling` | WebRTC signaling для P2P |
| `delivery_status` | Статус доставки сообщений |
| `system` | Системные уведомления |

Полная спецификация в [WEBSOCKET_API.md](./WEBSOCKET_API.md)

## 🔄 Connection Resilience

### Автоматический Reconnection

При разрыве соединения клиент автоматически переподключается:

- **Максимум попыток:** 10
- **Начальная задержка:** 1 секунда
- **Backoff:** Экспоненциальный (1s → 2s → 4s → 8s → ...)
- **Сброс счетчика:** При успешном подключении

### Offline Sync

Когда WebSocket недоступен:
1. Запускается периодический polling Cloudflare Worker
2. Пропущенные сообщения сохраняются в Cloudflare KV/D1
3. При восстановлении соединения происходит синхронизация
4. Все сообщения доставляются без потерь

## 🧪 Testing

### Тестирование с wscat

```bash
# Установка
npm install -g wscat

# Подключение
wscat -c ws://localhost:3000/api/v1/ws

# Подписка
{"type": "subscribe", "user_id": "test-user", "channels": ["messages"]}

# Ожидание уведомлений
# (отправьте сообщение через API)
```

### Тестирование P2P Signaling

```bash
# Terminal 1 - User A
wscat -c ws://localhost:3000/api/v1/ws
{"type": "subscribe", "user_id": "user-a", "channels": ["p2p_signaling"]}

# Terminal 2 - User B
wscat -c ws://localhost:3000/api/v1/ws
{"type": "subscribe", "user_id": "user-b", "channels": ["p2p_signaling"]}

# Terminal 1 - Send offer to User B
{"type": "p2p_offer", "target_user_id": "user-b", "sdp": "v=0...", "candidates": []}

# User B получит offer через WebSocket
```

### Тестирование Reconnection

```bash
# 1. Подключитесь к серверу
# 2. Остановите сервер (Ctrl+C)
# 3. Наблюдайте reconnection логи:
[WebSocket] Reconnecting in 1000ms (attempt 1)
[WebSocket] Reconnecting in 2000ms (attempt 2)
[WebSocket] Reconnecting in 4000ms (attempt 3)

# 4. Запустите сервер снова
# 5. Клиент автоматически подключится
```

## 🔒 Security

### Меры безопасности

1. **Аутентификация** - Требуется при подписке (user_id)
2. **Авторизация** - Пользователи получают только свои уведомления
3. **E2EE** - Сообщения зашифрованы (X25519 + Kyber1024)
4. **CORS** - Ограничить разрешенными origins
5. **Rate Limiting** - Необходимо реализовать на сервере

### TODO для Production

- [ ] Добавить JWT аутентификацию для WebSocket
- [ ] Реализовать rate limiting на сервере
- [ ] Валидация входящих сообщений
- [ ] Логирование и мониторинг
- [ ] Load testing

## 📊 Monitoring

### Логи сервера

```
WebSocket connection registered: 550e8400-e29b-41d4-a716-446655440000
Connection 550e8400 authenticated as user user-123
Connection 550e8400 subscribed to channels
P2P Offer from 550e8400 to user-456
Notifying user user-456 via connection 660f9511
```

### Логи клиента

```
[WebSocket] Connecting to: ws://localhost:3000/api/v1/ws
[WebSocket] Connected
[WebSocket] Subscribed to channels: messages, p2p_signaling
[ChatStore] New message notification: { chat_id: '...', ... }
[OfflineSync] WebSocket connected, skipping sync
```

## 🚧 Future Enhancements

- [ ] Групповые WebSocket соединения для чатов
- [ ] Пагинация сообщений через WebSocket
- [ ] Typing indicators (пользователь печатает)
- [ ] Online статус пользователей
- [ ] Message read receipts
- [ ] File transfer notifications
- [ ] Voice/video call signaling
- [ ] Message reactions
- [ ] Thread/reply notifications

## 📚 Related Documentation

- [WebSocket API Spec](./WEBSOCKET_API.md) - Полная спецификация API
- [Cloudflare Worker](../cloudflare/worker/README.md) - Offline message storage
- [P2P Architecture](./P2P_ARCHITECTURE.md) - Libp2p + WebRTC integration

## 🐛 Troubleshooting

### WebSocket не подключается

1. Проверьте, что сервер запущен на порту 3000
2. Проверьте CORS настройки сервера
3. Откройте консоль браузера для проверки ошибок
4. Попробуйте подключиться через wscat

### Сообщения не приходят

1. Убедитесь, что подписка успешна (смотрите логи)
2. Проверьте, что каналы подписки правильные
3. Проверьте серверные логи на наличие ошибок отправки

### Reconnection не работает

1. Проверьте доступность сервера
2. Увеличьте `maxReconnectAttempts` в конфиге
3. Проверьте сетевое соединение

### P2P signaling не работает

1. Оба пользователя должны быть online
2. Проверьте `target_user_id` (должен существовать)
3. Убедитесь, что оба подписаны на `p2p_signaling`

## 📝 License

Часть проекта Secure Telegram Client
