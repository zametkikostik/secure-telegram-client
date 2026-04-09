/**
 * WebSocket Client for Secure Messenger
 * 
 * Provides:
 * - Real-time message notifications
 * - P2P signaling exchange
 * - Connection state management
 * - Automatic reconnection with exponential backoff
 * - Subscribe/unsubscribe to notification channels
 */

// ============================================================================
// Types
// ============================================================================

export type NotificationChannel = 'messages' | 'p2p_signaling' | 'delivery_status' | 'system' | 'message_edited' | 'message_deleted' | 'presence'

export type WsMessageType =
  | 'subscribe'
  | 'unsubscribe'
  | 'p2p_offer'
  | 'p2p_answer'
  | 'ice_candidate'
  | 'p2p_hangup'
  | 'ping'
  | 'notification'
  | 'p2p_connected'
  | 'error'
  | 'pong'
  | 'message_edited'
  | 'message_deleted'
  | 'presence_update'

export interface IceCandidate {
  candidate: string
  sdp_mid?: string
  sdp_m_line_index?: number
}

export interface NewMessagePayload {
  chat_id: string
  message_id: string
  sender_id: string
  encrypted_content: string
  timestamp: string
}

export interface MessageEditedPayload {
  id: string
  chat_id: string
  sender_id: string
  content: string
  edited_at: string | null
}

export interface MessageDeletedPayload {
  id: string
  chat_id: string
}

export interface DeliveryUpdatePayload {
  message_id: string
  status: string
  timestamp: string
}

export interface P2PEventPayload {
  event_type: string
  peer_id: string
  data: Record<string, unknown>
}

export interface SystemNotificationPayload {
  message: string
  code: string
}

export type NotificationPayload = 
  | NewMessagePayload
  | DeliveryUpdatePayload
  | P2PEventPayload
  | SystemNotificationPayload

export interface WsMessage {
  type: WsMessageType
  user_id?: string
  channels?: NotificationChannel[]
  target_user_id?: string
  call_id?: string
  sdp?: string
  candidates?: IceCandidate[]
  candidate?: IceCandidate
  reason?: string
  channel?: NotificationChannel
  payload?: NotificationPayload
  code?: string
  message?: string
  peer_id?: string
  // MessageEdited fields
  id?: string
  chat_id?: string
  sender_id?: string
  content?: string
  edited_at?: string | null
}

export type ConnectionState = 'connecting' | 'connected' | 'disconnected' | 'reconnecting'

export interface WebSocketClientConfig {
  baseUrl: string
  userId?: string
  autoConnect?: boolean
  maxReconnectAttempts?: number
  reconnectDelay?: number
  channels?: NotificationChannel[]
}

// ============================================================================
// Event Handlers
// ============================================================================

export type MessageHandler = (payload: NotificationPayload) => void
export type P2PHandler = (event: P2PEventPayload) => void
export type StateChangeHandler = (state: ConnectionState) => void
export type ErrorHandler = (error: Error) => void

// ============================================================================
// WebSocket Client Class
// ============================================================================

export class WebSocketClient {
  private ws: WebSocket | null = null
  private config: Required<WebSocketClientConfig>
  private connectionState: ConnectionState = 'disconnected'
  private reconnectAttempts = 0
  private reconnectTimer: number | null = null
  private pingTimer: number | null = null
  private messageHandlers: Map<NotificationChannel, Set<MessageHandler>> = new Map()
  private p2pHandlers: Set<P2PHandler> = new Set()
  private stateChangeHandlers: Set<StateChangeHandler> = new Set()
  private errorHandlers: Set<ErrorHandler> = new Set()

  constructor(config: WebSocketClientConfig) {
    this.config = {
      baseUrl: config.baseUrl,
      userId: config.userId || '',
      autoConnect: config.autoConnect ?? false,
      maxReconnectAttempts: config.maxReconnectAttempts ?? 10,
      reconnectDelay: config.reconnectDelay ?? 1000,
      channels: config.channels || ['messages', 'p2p_signaling', 'delivery_status', 'message_edited', 'message_deleted'],
    }
  }

  // ============================================================================
  // Connection Management
  // ============================================================================

  /**
   * Connect to WebSocket server
   */
  connect(): Promise<void> {
    return new Promise((resolve, reject) => {
      if (this.ws && this.ws.readyState === WebSocket.OPEN) {
        resolve()
        return
      }

      this.updateState('connecting')

      const wsUrl = this.buildWebSocketUrl()
      console.log('[WebSocket] Connecting to:', wsUrl)

      this.ws = new WebSocket(wsUrl)

      this.ws.onopen = () => {
        console.log('[WebSocket] Connected')
        this.updateState('connected')
        this.reconnectAttempts = 0
        this.startPingInterval()
        this.subscribeToChannels(this.config.channels)
        resolve()
      }

      this.ws.onmessage = (event: MessageEvent) => {
        this.handleMessage(event.data)
      }

      this.ws.onclose = (event: CloseEvent) => {
        console.log('[WebSocket] Disconnected:', event.code, event.reason)
        this.updateState('disconnected')
        this.stopPingInterval()
        this.scheduleReconnect()
      }

      this.ws.onerror = (error: Event) => {
        console.error('[WebSocket] Error:', error)
        this.emitError(new Error('WebSocket connection error'))
        reject(new Error('WebSocket connection error'))
      }
    })
  }

  /**
   * Disconnect from WebSocket server
   */
  disconnect(): void {
    console.log('[WebSocket] Manual disconnect')
    this.stopPingInterval()
    this.clearReconnectTimer()
    this.updateState('disconnected')
    
    if (this.ws) {
      this.ws.close(1000, 'Client disconnected')
      this.ws = null
    }
  }

  /**
   * Reconnect manually
   */
  async reconnect(): Promise<void> {
    console.log('[WebSocket] Manual reconnect')
    this.disconnect()
    await this.connect()
  }

  // ============================================================================
  // Subscription Management
  // ============================================================================

  /**
   * Subscribe to notification channels
   */
  subscribeToChannels(channels: NotificationChannel[]): void {
    if (this.connectionState !== 'connected') {
      console.warn('[WebSocket] Cannot subscribe: not connected')
      return
    }

    const message: WsMessage = {
      type: 'subscribe',
      user_id: this.config.userId,
      channels,
    }

    this.send(message)
    console.log('[WebSocket] Subscribed to channels:', channels)
  }

  /**
   * Unsubscribe from notification channels
   */
  unsubscribeFromChannels(channels?: NotificationChannel[]): void {
    if (this.connectionState !== 'connected') {
      console.warn('[WebSocket] Cannot unsubscribe: not connected')
      return
    }

    const message: WsMessage = {
      type: 'unsubscribe',
      user_id: this.config.userId,
      channels,
    }

    this.send(message)
    console.log('[WebSocket] Unsubscribed from channels:', channels || 'all')
  }

  // ============================================================================
  // P2P Signaling
  // ============================================================================

  /**
   * Send P2P offer to another user
   */
  sendP2POffer(targetUserId: string, sdp: string, candidates: IceCandidate[]): void {
    if (this.connectionState !== 'connected') {
      console.warn('[WebSocket] Cannot send P2P offer: not connected')
      return
    }

    const message: WsMessage = {
      type: 'p2p_offer',
      target_user_id: targetUserId,
      sdp,
      candidates,
    }

    this.send(message)
    console.log('[WebSocket] P2P Offer sent to:', targetUserId)
  }

  /**
   * Send P2P answer to another user
   */
  sendP2PAnswer(targetUserId: string, sdp: string, candidates: IceCandidate[]): void {
    if (this.connectionState !== 'connected') {
      console.warn('[WebSocket] Cannot send P2P answer: not connected')
      return
    }

    const message: WsMessage = {
      type: 'p2p_answer',
      target_user_id: targetUserId,
      sdp,
      candidates,
    }

    this.send(message)
    console.log('[WebSocket] P2P Answer sent to:', targetUserId)
  }

  /**
   * Send ICE candidate to another user
   */
  sendIceCandidate(targetUserId: string, candidate: IceCandidate): void {
    if (this.connectionState !== 'connected') {
      console.warn('[WebSocket] Cannot send ICE candidate: not connected')
      return
    }

    const message: WsMessage = {
      type: 'ice_candidate',
      target_user_id: targetUserId,
      candidate,
    }

    this.send(message)
    console.log('[WebSocket] ICE Candidate sent to:', targetUserId)
  }

  /**
   * Send P2P hangup notification
   */
  sendP2PHangup(targetUserId: string, callId: string, reason: string): void {
    if (this.connectionState !== 'connected') {
      console.warn('[WebSocket] Cannot send P2P hangup: not connected')
      return
    }

    const message: WsMessage = {
      type: 'p2p_hangup',
      target_user_id: targetUserId,
      call_id: callId,
      reason,
    }

    this.send(message)
    console.log('[WebSocket] P2P Hangup sent to:', targetUserId, 'reason:', reason)
  }

  // ============================================================================
  // Event Handlers Registration
  // ============================================================================

  /**
   * Register message handler for specific channel
   */
  onMessage(channel: NotificationChannel, handler: MessageHandler): void {
    if (!this.messageHandlers.has(channel)) {
      this.messageHandlers.set(channel, new Set())
    }
    this.messageHandlers.get(channel)!.add(handler)
  }

  /**
   * Remove message handler
   */
  offMessage(channel: NotificationChannel, handler: MessageHandler): void {
    this.messageHandlers.get(channel)?.delete(handler)
  }

  /**
   * Register P2P event handler
   */
  onP2PEvent(handler: P2PHandler): void {
    this.p2pHandlers.add(handler)
  }

  /**
   * Remove P2P event handler
   */
  offP2PEvent(handler: P2PHandler): void {
    this.p2pHandlers.delete(handler)
  }

  /**
   * Register connection state change handler
   */
  onStateChange(handler: StateChangeHandler): void {
    this.stateChangeHandlers.add(handler)
  }

  /**
   * Remove connection state change handler
   */
  offStateChange(handler: StateChangeHandler): void {
    this.stateChangeHandlers.delete(handler)
  }

  /**
   * Register error handler
   */
  onError(handler: ErrorHandler): void {
    this.errorHandlers.add(handler)
  }

  /**
   * Remove error handler
   */
  offError(handler: ErrorHandler): void {
    this.errorHandlers.delete(handler)
  }

  // ============================================================================
  // Getters
  // ============================================================================

  getConnectionState(): ConnectionState {
    return this.connectionState
  }

  isConnected(): boolean {
    return this.connectionState === 'connected'
  }

  // ============================================================================
  // Private Methods
  // ============================================================================

  /**
   * Build WebSocket URL from config
   */
  private buildWebSocketUrl(): string {
    const baseUrl = this.config.baseUrl
      .replace('http://', 'ws://')
      .replace('https://', 'wss://')
    
    return `${baseUrl}/api/v1/ws`
  }

  /**
   * Send message through WebSocket
   */
  private send(message: WsMessage): void {
    if (!this.ws || this.ws.readyState !== WebSocket.OPEN) {
      console.warn('[WebSocket] Cannot send: connection not open')
      return
    }

    try {
      this.ws.send(JSON.stringify(message))
    } catch (error) {
      console.error('[WebSocket] Failed to send message:', error)
      this.emitError(new Error('Failed to send message'))
    }
  }

  /**
   * Handle incoming message
   */
  private handleMessage(data: string): void {
    try {
      const message: WsMessage = JSON.parse(data)
      console.log('[WebSocket] Received:', message.type)

      switch (message.type) {
        case 'notification':
          this.handleNotification(message)
          break
        case 'p2p_offer':
        case 'p2p_answer':
        case 'ice_candidate':
        case 'p2p_hangup':
          this.handleP2PEvent(message)
          break
        case 'message_edited':
        case 'message_deleted':
          this.handleMessageEdit(message)
          break
        case 'presence_update':
          this.handlePresenceUpdate(message)
          break
        case 'pong':
          // Pong received, connection is alive
          break
        case 'error':
          this.handleServerError(message)
          break
        default:
          console.log('[WebSocket] Unknown message type:', message.type)
      }
    } catch (error) {
      console.error('[WebSocket] Failed to parse message:', error)
      this.emitError(new Error('Failed to parse incoming message'))
    }
  }

  /**
   * Handle notification message
   */
  private handleNotification(message: WsMessage): void {
    if (!message.channel || !message.payload) {
      console.warn('[WebSocket] Invalid notification message')
      return
    }

    const channel = message.channel as NotificationChannel
    const payload = message.payload as NotificationPayload

    const handlers = this.messageHandlers.get(channel)
    if (handlers) {
      handlers.forEach(handler => handler(payload))
    }
  }

  /**
   * Handle P2P event
   */
  private handleP2PEvent(message: WsMessage): void {
    const event: P2PEventPayload = {
      event_type: message.type,
      peer_id: message.peer_id || message.target_user_id || '',
      data: {
        sdp: message.sdp,
        candidates: message.candidates,
        candidate: message.candidate,
        call_id: message.call_id,
        reason: message.reason,
      },
    }

    this.p2pHandlers.forEach(handler => handler(event))
  }

  /**
   * Handle message edit/delete event
   */
  private handleMessageEdit(message: WsMessage): void {
    const channel = message.type === 'message_edited' ? 'message_edited' : 'message_deleted'
    const handlers = this.messageHandlers.get(channel as NotificationChannel)
    if (handlers) {
      const payload = message.type === 'message_edited'
        ? {
            id: message.id || '',
            chat_id: message.chat_id || '',
            sender_id: message.sender_id || '',
            content: message.content || '',
            edited_at: message.edited_at || null,
          } as MessageEditedPayload
        : {
            id: message.id || '',
            chat_id: message.chat_id || '',
          } as MessageDeletedPayload
      handlers.forEach(handler => handler(payload as any))
    }
  }

  /**
   * Handle presence update (online/offline status)
   */
  private handlePresenceUpdate(message: WsMessage): void {
    const handlers = this.messageHandlers.get('presence' as NotificationChannel)
    if (handlers) {
      const payload = { user_id: message.user_id || '', online: message.reason === 'online' }
      handlers.forEach(handler => handler(payload as any))
    }
  }

  /**
   * Handle server error message
   */
  private handleServerError(message: WsMessage): void {
    const error = new Error(message.message || 'Server error')
    console.error('[WebSocket] Server error:', message.code, error.message)
    this.emitError(error)
  }

  /**
   * Update connection state and notify handlers
   */
  private updateState(state: ConnectionState): void {
    this.connectionState = state
    this.stateChangeHandlers.forEach(handler => handler(state))
  }

  /**
   * Schedule automatic reconnection
   */
  private scheduleReconnect(): void {
    if (this.reconnectAttempts >= this.config.maxReconnectAttempts) {
      console.warn('[WebSocket] Max reconnect attempts reached')
      this.emitError(new Error('Max reconnect attempts reached'))
      return
    }

    if (this.reconnectTimer !== null) {
      return // Already scheduled
    }

    this.reconnectAttempts++
    const delay = this.config.reconnectDelay * Math.pow(2, this.reconnectAttempts - 1)
    
    console.log(`[WebSocket] Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts})`)
    this.updateState('reconnecting')

    this.reconnectTimer = window.setTimeout(async () => {
      this.reconnectTimer = null
      try {
        await this.connect()
      } catch {
        console.error('[WebSocket] Reconnection failed')
      }
    }, delay)
  }

  /**
   * Clear reconnect timer
   */
  private clearReconnectTimer(): void {
    if (this.reconnectTimer !== null) {
      window.clearTimeout(this.reconnectTimer)
      this.reconnectTimer = null
    }
  }

  /**
   * Start ping interval to keep connection alive
   */
  private startPingInterval(): void {
    this.pingTimer = window.setInterval(() => {
      this.send({ type: 'ping' })
    }, 30000) // Ping every 30 seconds
  }

  /**
   * Stop ping interval
   */
  private stopPingInterval(): void {
    if (this.pingTimer !== null) {
      window.clearInterval(this.pingTimer)
      this.pingTimer = null
    }
  }

  /**
   * Emit error to handlers
   */
  private emitError(error: Error): void {
    this.errorHandlers.forEach(handler => handler(error))
  }
}

// ============================================================================
// Singleton Instance
// ============================================================================

let wsInstance: WebSocketClient | null = null

/**
 * Get or create WebSocket singleton
 */
export function getWebSocketClient(config: WebSocketClientConfig): WebSocketClient {
  if (!wsInstance) {
    wsInstance = new WebSocketClient(config)
  }
  return wsInstance
}

/**
 * Disconnect and reset singleton
 */
export function resetWebSocketClient(): void {
  if (wsInstance) {
    wsInstance.disconnect()
    wsInstance = null
  }
}
