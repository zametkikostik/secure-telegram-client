# 🔌 WebSocket API Documentation

## Overview

WebSocket provides:
- **Real-time message notifications** - Instant delivery of encrypted messages
- **P2P Signaling** - Exchange WebRTC SDP and ICE candidates for direct connections
- **Connection state management** - Automatic reconnection with exponential backoff
- **Channel subscriptions** - Subscribe to specific notification types

## Connection

### URL Format
```
ws://localhost:3000/api/v1/ws
wss://your-domain.com/api/v1/ws
```

### Connection Lifecycle
1. Client connects to `/api/v1/ws`
2. Server assigns unique `connection_id`
3. Client sends `subscribe` message with `user_id` and channels
4. Server authenticates and subscribes client
5. Ping/pong keeps connection alive (30s interval)
6. On disconnect, client auto-reconnects with exponential backoff

## Message Types

### Client → Server

#### 1. Subscribe to Channels
```json
{
  "type": "subscribe",
  "user_id": "user-123",
  "channels": ["messages", "p2p_signaling", "delivery_status"]
}
```

**Channels:**
- `messages` - New encrypted message notifications
- `p2p_signaling` - P2P WebRTC signaling data
- `delivery_status` - Message delivery status updates
- `system` - System notifications

#### 2. Unsubscribe from Channels
```json
{
  "type": "unsubscribe",
  "user_id": "user-123",
  "channels": ["messages"]  // Optional, omit to unsubscribe from all
}
```

#### 3. P2P Offer (WebRTC SDP Offer)
```json
{
  "type": "p2p_offer",
  "target_user_id": "user-456",
  "sdp": "v=0\r\no=- ...",
  "candidates": [
    {
      "candidate": "candidate:1 1 UDP 2130706431 ...",
      "sdp_mid": "0",
      "sdp_m_line_index": 0
    }
  ]
}
```

#### 4. P2P Answer (WebRTC SDP Answer)
```json
{
  "type": "p2p_answer",
  "target_user_id": "user-123",
  "sdp": "v=0\r\no=- ...",
  "candidates": [...]
}
```

#### 5. ICE Candidate
```json
{
  "type": "ice_candidate",
  "target_user_id": "user-456",
  "candidate": {
    "candidate": "candidate:2 1 UDP ...",
    "sdp_mid": "0",
    "sdp_m_line_index": 0
  }
}
```

#### 6. Ping (Keep-alive)
```json
{
  "type": "ping"
}
```

### Server → Client

#### 1. New Message Notification
```json
{
  "type": "notification",
  "channel": "messages",
  "payload": {
    "chat_id": "chat-789",
    "message_id": "msg-001",
    "sender_id": "user-456",
    "encrypted_content": "base64-encoded-encrypted-data",
    "timestamp": "2026-04-05T12:00:00Z"
  }
}
```

#### 2. Delivery Status Update
```json
{
  "type": "notification",
  "channel": "delivery_status",
  "payload": {
    "message_id": "msg-001",
    "status": "delivered",  // sent | delivered | read
    "timestamp": "2026-04-05T12:00:01Z"
  }
}
```

#### 3. P2P Event
```json
{
  "type": "notification",
  "channel": "p2p_signaling",
  "payload": {
    "event_type": "offer",  // offer | answer | ice_candidate
    "peer_id": "user-456",
    "data": {
      "sdp": "...",
      "candidates": [...]
    }
  }
}
```

#### 4. System Notification
```json
{
  "type": "notification",
  "channel": "system",
  "payload": {
    "message": "Successfully subscribed to channels",
    "code": "subscribed"
  }
}
```

#### 5. Pong
```json
{
  "type": "pong"
}
```

#### 6. Error
```json
{
  "type": "error",
  "code": "invalid_message",
  "message": "Failed to parse message"
}
```

## Frontend Usage

### Basic Example
```typescript
import { getWebSocketClient } from './services/webSocketClient'

const wsClient = getWebSocketClient({
  userId: 'user-123',
  baseUrl: 'http://localhost:3000',
  autoConnect: true,
  channels: ['messages', 'p2p_signaling'],
})

// Listen for new messages
wsClient.onMessage('messages', (payload) => {
  console.log('New message:', payload)
})

// Listen for P2P events
wsClient.onP2PEvent((event) => {
  if (event.event_type === 'offer') {
    // Handle WebRTC offer
  }
})

// Listen for connection state changes
wsClient.onStateChange((state) => {
  console.log('Connection state:', state)
  // 'connecting' | 'connected' | 'disconnected' | 'reconnecting'
})
```

### P2P Signaling Example
```typescript
import { getWebSocketClient } from './services/webSocketClient'

const wsClient = getWebSocketClient({
  userId: 'user-123',
  baseUrl: 'http://localhost:3000',
})

// Create WebRTC peer connection
const peerConnection = new RTCPeerConnection()

// When creating offer
peerConnection.onnegotiationneeded = async () => {
  const offer = await peerConnection.createOffer()
  await peerConnection.setLocalDescription(offer)
  
  // Collect ICE candidates
  const candidates: IceCandidate[] = []
  peerConnection.onicecandidate = (event) => {
    if (event.candidate) {
      candidates.push({
        candidate: event.candidate.candidate,
        sdp_mid: event.candidate.sdpMid || undefined,
        sdp_m_line_index: event.candidate.sdpMLineIndex || undefined,
      })
    }
  }
  
  // Send offer via WebSocket
  wsClient.sendP2POffer(
    'user-456',
    offer.sdp!,
    candidates
  )
}

// Receive answer via WebSocket
wsClient.onP2PEvent(async (event) => {
  if (event.event_type === 'answer') {
    await peerConnection.setRemoteDescription(
      new RTCSessionDescription({ sdp: event.data.sdp })
    )
  }
  
  if (event.event_type === 'ice_candidate' && event.data.candidate) {
    await peerConnection.addIceCandidate(
      new RTCIceCandidate(event.data.candidate)
    )
  }
})
```

## Backend Architecture

### Components

1. **ConnectionManager** (`server/src/ws/manager.rs`)
   - Manages active WebSocket connections
   - Handles subscriptions per channel
   - Broadcasts notifications to subscribers
   - Tracks online users

2. **WebSocket Handler** (`server/src/ws/handler.rs`)
   - Processes incoming messages
   - Routes P2P signaling between users
   - Sends notifications to clients

3. **AppState** (`server/src/state.rs`)
   - Holds singleton ConnectionManager
   - Shared across all request handlers

### Server-Side Notification

To send notification to user:

```rust
use crate::models::ws::{NotificationChannel, NotificationPayload};

// When new message arrives
let payload = NotificationPayload::NewMessage {
    chat_id: chat_id.to_string(),
    message_id: msg_id.to_string(),
    sender_id: sender_id.to_string(),
    encrypted_content: encrypted_content,
    timestamp: chrono::Utc::now().to_rfc3339(),
};

state.ws_manager.notify_user(
    &user_id,
    NotificationChannel::Messages,
    payload,
).await;
```

## Connection Resilience

### Automatic Reconnection
- **Max attempts:** 10
- **Initial delay:** 1 second
- **Backoff:** Exponential (1s, 2s, 4s, 8s, ...)
- **Reset:** On successful connection

### Keep-Alive
- **Ping interval:** 30 seconds
- **Timeout:** Detected by server
- **Auto-cleanup:** On connection close

## Security Considerations

1. **Authentication:** Required before subscribing
2. **Authorization:** Users only receive their own notifications
3. **Encryption:** Messages are E2EE (X25519 + Kyber1024)
4. **Rate Limiting:** Server should implement rate limits
5. **CORS:** Restrict to allowed origins

## Testing

### Manual Test with wscat
```bash
# Install wscat
npm install -g wscat

# Connect to WebSocket
wscat -c ws://localhost:3000/api/v1/ws

# Subscribe
{"type": "subscribe", "user_id": "test-user", "channels": ["messages"]}

# Server will respond with
{"type": "notification", "channel": "system", "payload": {"message": "Successfully subscribed to channels", "code": "subscribed"}}
```

## Cloudflare Fallback

When WebSocket connection fails:
1. Client falls back to HTTP polling
2. Cloudflare Worker stores pending notifications
3. Client retrieves missed messages on reconnect
4. Sync logic ensures no message loss

See `cloudflare/worker/README.md` for Cloudflare API details.
