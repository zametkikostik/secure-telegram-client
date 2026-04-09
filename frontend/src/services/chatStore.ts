// @ts-nocheck
// @ts-nocheck
import { create } from 'zustand'
import { getWebSocketClient, WebSocketClient, NotificationPayload } from './webSocketClient'
import { createOfflineSyncService, OfflineSyncService } from './offlineSyncService'

interface Message {
  id: string
  chatId: string
  senderId: string
  content: string
  timestamp: string
  encrypted?: boolean
}

interface ChatState {
  selectedChat: string | null
  setSelectedChat: (id: string | null) => void

  unreadCount: number
  setUnreadCount: (count: number) => void

  isOnline: boolean
  setIsOnline: (online: boolean) => void

  messages: Message[]
  addMessage: (message: Message) => void

  wsInitialized: boolean
  wsClient: WebSocketClient | null
  offlineSync: OfflineSyncService | null
  initializeWebSocket: (userId: string, baseUrl: string, cloudflareUrl?: string) => void
}

export const useChatStore = create<ChatState>((set, get) => ({
  selectedChat: null,
  setSelectedChat: (id) => set({ selectedChat: id }),

  unreadCount: 0,
  setUnreadCount: (count) => set({ unreadCount: count }),

  isOnline: false,
  setIsOnline: (online) => set({ isOnline: online }),

  messages: [],
  addMessage: (message) => set((state) => ({ 
    messages: [...state.messages, message] 
  })),

  wsInitialized: false,
  wsClient: null,
  offlineSync: null,
  initializeWebSocket: (userId: string, baseUrl: string, cloudflareUrl?: string) => {
    if (get().wsInitialized) return

    const wsClient = getWebSocketClient({
      userId,
      baseUrl,
      autoConnect: true,
      channels: ['messages', 'p2p_signaling', 'delivery_status'],
    })

    // Handle new messages
    wsClient.onMessage('messages', (payload: NotificationPayload) => {
      const msgPayload = payload as any
      console.log('[ChatStore] New message notification:', msgPayload)
      
      get().addMessage({
        id: msgPayload.message_id,
        chatId: msgPayload.chat_id,
        senderId: msgPayload.sender_id,
        content: msgPayload.encrypted_content,
        timestamp: msgPayload.timestamp,
        encrypted: true,
      })

      // Увеличить счетчик непрочитанных если чат не выбран
      if (msgPayload.chat_id !== get().selectedChat) {
        set((state) => ({ unreadCount: state.unreadCount + 1 }))
      }
    })

    // Handle delivery status updates
    wsClient.onMessage('delivery_status', (payload: NotificationPayload) => {
      const deliveryPayload = payload as any
      console.log('[ChatStore] Delivery status update:', deliveryPayload)
      // TODO: Обновить статус доставки сообщения
    })

    // Handle connection state changes
    wsClient.onStateChange((state) => {
      console.log('[ChatStore] WebSocket state:', state)
      set({ isOnline: state === 'connected' })
    })

    // Setup offline sync if Cloudflare URL provided
    let offlineSync = null
    if (cloudflareUrl) {
      offlineSync = createOfflineSyncService(
        { cloudflareUrl },
        wsClient,
        userId
      )
      console.log('[ChatStore] Offline sync service initialized')
    }

    set({ 
      wsInitialized: true,
      wsClient,
      offlineSync,
    })
  },
}))
