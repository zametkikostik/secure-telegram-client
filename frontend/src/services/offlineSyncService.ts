/**
 * Offline Notification Sync Service
 * 
 * Syncs missed messages when WebSocket reconnects
 * Uses Cloudflare Worker as fallback when main server is unavailable
 */

import { WebSocketClient } from './webSocketClient'

export interface OfflineMessage {
  id: string
  chat_id: string
  sender_id: string
  encrypted_content: string
  timestamp: string
}

export interface SyncConfig {
  cloudflareUrl: string
  userId: string
  syncInterval?: number  // milliseconds, 0 to disable
  maxRetries?: number
}

export class OfflineSyncService {
  private config: SyncConfig
  private wsClient: WebSocketClient
  private syncTimer: number | null = null
  private lastSyncTimestamp: string | null = null
  private isSyncing = false
  private retryCount = 0

  constructor(config: SyncConfig, wsClient: WebSocketClient) {
    this.config = {
      cloudflareUrl: config.cloudflareUrl,
      userId: config.userId,
      syncInterval: config.syncInterval ?? 30000, // 30 seconds
      maxRetries: config.maxRetries ?? 3,
    }
    this.wsClient = wsClient
  }

  /**
   * Start periodic sync
   */
  start(): void {
    console.log('[OfflineSync] Starting periodic sync')
    
    // Initial sync
    this.sync()

    // Periodic sync
    if (this.config.syncInterval && this.config.syncInterval > 0) {
      this.syncTimer = window.setInterval(() => {
        this.sync()
      }, this.config.syncInterval)
    }
  }

  /**
   * Stop periodic sync
   */
  stop(): void {
    console.log('[OfflineSync] Stopping periodic sync')
    
    if (this.syncTimer !== null) {
      window.clearInterval(this.syncTimer)
      this.syncTimer = null
    }
  }

  /**
   * Sync missed messages from Cloudflare
   */
  async sync(): Promise<void> {
    if (this.isSyncing) {
      console.log('[OfflineSync] Sync already in progress, skipping')
      return
    }

    // Don't sync if WebSocket is connected and receiving messages
    if (this.wsClient.isConnected()) {
      console.log('[OfflineSync] WebSocket connected, skipping sync')
      this.retryCount = 0
      return
    }

    this.isSyncing = true
    console.log('[OfflineSync] Starting sync with Cloudflare Worker')

    try {
      let url = `${this.config.cloudflareUrl}/api/v1/msg/${this.config.userId}`
      
      if (this.lastSyncTimestamp) {
        url += `?since=${encodeURIComponent(this.lastSyncTimestamp)}`
      }

      const response = await fetch(url, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      })

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      const data = await response.json()
      
      if (data.messages && data.messages.length > 0) {
        console.log(`[OfflineSync] Received ${data.messages.length} offline messages`)
        
        // Emit each message as notification
        for (const msg of data.messages) {
          this.emitMessageNotification(msg)
        }

        // Update last sync timestamp
        this.lastSyncTimestamp = new Date().toISOString()
        this.retryCount = 0
      } else {
        console.log('[OfflineSync] No new messages')
      }
    } catch (error) {
      console.error('[OfflineSync] Sync failed:', error)
      
      this.retryCount++
      if (this.retryCount >= this.config.maxRetries!) {
        console.warn('[OfflineSync] Max retries reached, stopping sync')
        this.stop()
      }
    } finally {
      this.isSyncing = false
    }
  }

  /**
   * Register push token with Cloudflare Worker
   */
  async registerPushToken(token: string, platform: 'android' | 'ios' | 'web'): Promise<boolean> {
    try {
      const response = await fetch(`${this.config.cloudflareUrl}/api/v1/register`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          user_id: this.config.userId,
          push_token: token,
          platform,
          timestamp: new Date().toISOString(),
        }),
      })

      if (!response.ok) {
        throw new Error(`HTTP ${response.status}: ${response.statusText}`)
      }

      console.log('[OfflineSync] Push token registered successfully')
      return true
    } catch (error) {
      console.error('[OfflineSync] Failed to register push token:', error)
      return false
    }
  }

  /**
   * Get pending message count
   */
  async getPendingCount(): Promise<number> {
    try {
      const response = await fetch(
        `${this.config.cloudflareUrl}/api/v1/msg/${this.config.userId}/count`,
        { method: 'GET' }
      )

      if (!response.ok) {
        return 0
      }

      const data = await response.json()
      return data.count || 0
    } catch {
      return 0
    }
  }

  /**
   * Emit message notification to WebSocket handlers
   */
  private emitMessageNotification(_message: OfflineMessage): void {
    // TODO: Интегрировать с WebSocket клиентом для отправки уведомлений
    console.log('[OfflineSync] Would emit offline message notification:', _message.id)
  }
}

/**
 * Create and configure offline sync service
 */
export function createOfflineSyncService(
  config: Omit<SyncConfig, 'userId'>,
  wsClient: WebSocketClient,
  userId: string
): OfflineSyncService {
  const syncService = new OfflineSyncService(
    { ...config, userId },
    wsClient
  )
  
  // Auto-start when WebSocket reconnects
  wsClient.onStateChange((state) => {
    if (state === 'disconnected') {
      console.log('[OfflineSync] WebSocket disconnected, starting sync')
      syncService.start()
    } else if (state === 'connected') {
      console.log('[OfflineSync] WebSocket connected, stopping sync')
      syncService.stop()
    }
  })

  return syncService
}
