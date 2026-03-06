# 🌐 P2P NODE NETWORK — ДЕЦЕНТРАЛИЗОВАННАЯ СЕТЬ

**Версия:** 1.0.0  
**Дата:** 6 марта 2026 г.

---

## 🎯 ОБЗОР

Каждый участник Liberty Reach становится **P2P нодой**, которая:
- ✅ Распространяет сообщения по цепочке
- ✅ Не зависит от центрального сервера
- ✅ Автоматически добавляется в GitHub репозиторий
- ✅ Синхронизируется с другими нодами

---

## 🏗️ АРХИТЕКТУРА

```
                    ┌─────────────────┐
                    │   GitHub Repo   │
                    │  (PEERS.md)     │
                    └────────┬────────┘
                             │
                    Sync через API
                             │
        ┌────────────────────┼────────────────────┐
        │                    │                    │
   ┌────▼────┐         ┌────▼────┐         ┌────▼────┐
   │ Node 1  │◄───────►│ Node 2  │◄───────►│ Node 3  │
   │User A   │  P2P    │User B   │  P2P    │User C   │
   └────┬────┘         └────┬────┘         └────┬────┘
        │                   │                   │
        │    libp2p Gossip  │                   │
        │                   │                   │
   ┌────▼────┐         ┌────▼────┐         ┌────▼────┐
   │ Node 4  │◄───────►│ Node 5  │◄───────►│ Node 6  │
   │User D   │  P2P    │User E   │  P2P    │User F   │
   └─────────┘         └─────────┘         └─────────┘
```

---

## 🔧 API ENDPOINTS

### 1. Регистрация ноды

```http
POST /api/v1/nodes/register
Content-Type: application/json

{
  "user_id": "user-123",
  "username": "alice",
  "public_key": "ed25519:pubkey...",
  "peer_id": "12D3KooW...",
  "multiaddr": [
    "/ip4/192.168.1.100/tcp/9009",
    "/ip4/192.168.1.100/udp/9010/quic"
  ],
  "version": "1.0.0",
  "capabilities": ["chat", "voice", "video", "p2p"]
}
```

**Ответ:**
```json
{
  "id": "node-uuid",
  "user_id": "user-123",
  "username": "alice",
  "public_key": "ed25519:pubkey...",
  "peer_id": "12D3KooW...",
  "multiaddr": ["/ip4/..."],
  "status": "online",
  "joined_at": "2026-03-06T12:00:00Z",
  "last_seen": "2026-03-06T12:00:00Z",
  "version": "1.0.0",
  "capabilities": ["chat", "voice", "video", "p2p"]
}
```

### 2. Получить список нод

```http
GET /api/v1/nodes/list
```

**Ответ:**
```json
{
  "nodes": [
    {
      "id": "node-1",
      "username": "alice",
      "status": "online",
      "peer_id": "12D3KooW..."
    }
  ],
  "total": 100,
  "online": 85
}
```

### 3. Heartbeat (обновление статуса)

```http
POST /api/v1/nodes/heartbeat
Content-Type: application/json

{
  "user_id": "user-123",
  "status": "online"
}
```

---

## 🔄 СИНХРОНИЗАЦИЯ С GITHUB

### Автоматическое добавление в PEERS.md

При регистрации ноды:

1. **Сервер получает данные ноды**
2. **Отправляет запрос к GitHub API**
3. **Обновляет файл PEERS.md**
4. **Новый участник появляется в списке**

### PEERS.md формат

```markdown
# Liberty Reach Peer Network

Список участников P2P сети:

## alice
- **User ID:** user-123
- **Public Key:** ed25519:pubkey...
- **Joined:** 2026-03-06T12:00:00Z

## bob
- **User ID:** user-456
- **Public Key:** ed25519:pubkey...
- **Joined:** 2026-03-06T12:05:00Z
```

### GitHub API Flow

```
1. GET /repos/{repo}/contents/PEERS.md
   ↓
2. Decode base64 content
   ↓
3. Append new peer entry
   ↓
4. Encode to base64
   ↓
5. PUT /repos/{repo}/contents/PEERS.md
   ↓
6. Commit created automatically
```

---

## 🚀 ИНТЕГРАЦИЯ В ПРИЛОЖЕНИЕ

### React Native (Mobile)

```typescript
// hooks/useP2PNode.ts
import { useEffect } from 'react';
import { registerNode, startP2P } from '../libp2p';

export function useP2PNode(user: User) {
  useEffect(() => {
    const initNode = async () => {
      // 1. Запуск libp2p ноды
      const node = await startP2P();
      
      // 2. Регистрация на сервере
      await registerNode({
        user_id: user.id,
        username: user.username,
        public_key: user.publicKey,
        peer_id: node.peerId.toString(),
        multiaddr: node.multiaddrs.map(m => m.toString()),
        version: '1.0.0',
        capabilities: ['chat', 'voice', 'p2p'],
      });
      
      // 3. Heartbeat каждые 30 секунд
      const interval = setInterval(() => {
        node_heartbeat({ user_id: user.id });
      }, 30000);
      
      return () => clearInterval(interval);
    };
    
    initNode();
  }, [user]);
}
```

### Rust (Desktop)

```rust
// messenger/src/p2p/node.rs
use libp2p::{PeerId, Multiaddr};

pub struct P2PNode {
    pub peer_id: PeerId,
    pub multiaddrs: Vec<Multiaddr>,
}

impl P2PNode {
    pub async fn register_with_server(
        &self,
        user: &User,
        server_url: &str,
    ) -> Result<(), Box<dyn Error>> {
        let client = reqwest::Client::new();
        
        client
            .post(format!("{}/api/v1/nodes/register", server_url))
            .json(&serde_json::json!({
                "user_id": user.id,
                "username": user.username,
                "public_key": user.public_key,
                "peer_id": self.peer_id.to_base58(),
                "multiaddr": self.multiaddrs.iter()
                    .map(|m| m.to_string()).collect::<Vec<_>>(),
                "version": "1.0.0",
                "capabilities": ["chat", "voice", "video", "p2p"],
            }))
            .send()
            .await?
            .json()
            .await?;
        
        Ok(())
    }
}
```

---

## 📊 МОНИТОРИНГ СЕТИ

### Метрики

```prometheus
# Количество нод
p2p_nodes_total

# Онлайн ноды
p2p_nodes_online

# Сообщений распространено
p2p_messages_relayed_total

# Задержки между нодами
p2p_peer_latency_seconds
```

### Grafana Dashboard

- Карта нод (география)
- Статус нод (онлайн/оффлайн)
- Активность (сообщения/сек)
- Задержки (p95, p99)

---

## 🔐 БЕЗОПАСНОСТЬ

### Верификация нод

1. **Проверка подписи** — Ed25519
2. **Проверка peer_id** — совпадение с public key
3. **Rate limiting** — макс. 1 регистрация в минуту
4. **GitHub verification** — только для авторизованных

### Защита от атак

- ✅ DDoS защита (rate limiting)
- ✅ Sybil атака (требуется регистрация)
- ✅ Eclipse атака (множество multiaddr)
- ✅ Replay атака (timestamp в heartbeat)

---

## 🎯 СЦЕНАРИИ ИСПОЛЬЗОВАНИЯ

### 1. Новый пользователь присоединяется

```
1. Регистрация аккаунта → /auth/register
2. Запуск libp2p ноды → P2PNode::new()
3. Регистрация ноды → /nodes/register
4. Синхронизация с GitHub → PEERS.md обновлён
5. Подключение к другим нодам → Gossipsub
```

### 2. Сообщение распространяется

```
1. User A отправляет сообщение → Node A
2. Node A публикует в Gossipsub → Topic: chat-123
3. Node B, C получают → Ретрансляция
4. Node D, E, F получают → Доставка получателю
5. Сервер НЕ нужен для доставки!
```

### 3. Пользователь возвращается онлайн

```
1. Приложение запускается → libp2p старт
2. Heartbeat → /nodes/heartbeat
3. Статус меняется на "online"
4. Подключение к известным пирам
5. Синхронизация сообщений
```

---

## 📁 ФАЙЛЫ

| Файл | Описание |
|------|----------|
| `server/src/api/nodes.rs` | API для P2P нод |
| `server/src/db.rs` | Таблица `peer_nodes` |
| `messenger/src/p2p/libp2p.rs` | libp2p интеграция |
| `mobile/src/libp2p.ts` | React Native libp2p |
| `PEERS.md` | Список участников (GitHub) |

---

## 🚀 СЛЕДУЮЩИЕ ШАГИ

1. ✅ Регистрация нод реализована
2. ✅ GitHub синхронизация готова
3. ✅ Heartbeat механизм есть
4. ⏳ Мобильное приложение (в процессе)
5. ⏳ Gossipsub для сообщений (готов в messenger)

---

**Liberty Reach — Полностью децентрализованный мессенджер!**

**Каждый пользователь = Нода сети!** 🌐
