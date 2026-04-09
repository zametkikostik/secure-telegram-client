// P2P Client for Cloudflare Worker Integration
// Provides device registration, message send/receive, and contacts sync

const P2P_BASE = import.meta.env.VITE_CLOUDFLARE_WORKER_URL || 'https://secure-messenger-push.zametkikostik.workers.dev';

// ============================================================================
// Types
// ============================================================================

interface P2PDevice {
  deviceId: string;
  userId: string;
  publicKey: string;
  registeredAt: string;
  lastSeen: string;
}

interface P2PMessage {
  id: string;
  fromDeviceId: string;
  toDeviceId: string;
  encryptedMessage: string;
  messageType: 'text' | 'file' | 'voice' | 'video' | 'system';
  timestamp: number;
  delivered: boolean;
}

interface P2PRegisterResponse {
  success: boolean;
  deviceId: string;
}

interface P2PSendResponse {
  success: boolean;
  messageId: string;
  queued: boolean;
}

interface P2PMessagesResponse {
  messages: P2PMessage[];
  count: number;
}

interface P2PContactsResponse {
  contacts: string[];
  count: number;
}

// ============================================================================
// P2P Client Class
// ============================================================================

class P2PClient {
  private deviceId: string | null = null;
  private userId: string | null = null;
  private publicKey: string | null = null;
  private pollInterval: number | null = null;
  private messageHandlers: Set<(msg: P2PMessage) => void> = new Set();
  private isRunning = false;

  constructor() {
    // Load stored device info
    this.deviceId = localStorage.getItem('p2p_device_id');
    this.userId = localStorage.getItem('p2p_user_id');
    this.publicKey = localStorage.getItem('p2p_public_key');
  }

  // ============================================================================
  // Core API
  // ============================================================================

  /**
   * Register device with Cloudflare Worker
   */
  async register(userId: string, publicKey: string): Promise<P2PDevice> {
    const deviceId = this.deviceId || this.generateDeviceId();

    const response = await this.post<P2PRegisterResponse>('/p2p/register', {
      deviceId,
      userId,
      publicKey,
    });

    if (!response.success) {
      throw new Error('Failed to register device');
    }

    // Save device info
    this.deviceId = deviceId;
    this.userId = userId;
    this.publicKey = publicKey;
    localStorage.setItem('p2p_device_id', deviceId);
    localStorage.setItem('p2p_user_id', userId);
    localStorage.setItem('p2p_public_key', publicKey);

    return {
      deviceId,
      userId,
      publicKey,
      registeredAt: new Date().toISOString(),
      lastSeen: new Date().toISOString(),
    };
  }

  /**
   * Send encrypted message to another device
   */
  async sendMessage(
    toDeviceId: string,
    encryptedMessage: string,
    messageType: P2PMessage['messageType'] = 'text'
  ): Promise<P2PSendResponse> {
    if (!this.deviceId) {
      throw new Error('Device not registered. Call register() first.');
    }

    return this.post<P2PSendResponse>('/p2p/send', {
      fromDeviceId: this.deviceId,
      toDeviceId,
      encryptedMessage,
      messageType,
    });
  }

  /**
   * Get pending messages for this device
   */
  async getMessages(): Promise<P2PMessage[]> {
    if (!this.deviceId) {
      throw new Error('Device not registered. Call register() first.');
    }

    const response = await this.get<P2PMessagesResponse>(
      `/p2p/messages?deviceId=${this.deviceId}`
    );

    return response.messages || [];
  }

  /**
   * Sync contacts with Cloudflare Worker
   */
  async syncContacts(contacts: string[]): Promise<{ success: boolean; count: number }> {
    if (!this.userId) {
      throw new Error('User not set. Call register() first.');
    }

    return this.post('/contacts/sync', {
      userId: this.userId,
      contacts,
    });
  }

  /**
   * Get synced contacts
   */
  async getContacts(): Promise<string[]> {
    if (!this.userId) {
      throw new Error('User not set. Call register() first.');
    }

    const response = await this.get<P2PContactsResponse>(
      `/contacts?userId=${this.userId}`
    );

    return response.contacts || [];
  }

  // ============================================================================
  // Message Polling
  // ============================================================================

  /**
   * Start polling for new messages
   */
  startPolling(intervalMs = 5000): void {
    if (this.isRunning) return;
    if (!this.deviceId) {
      throw new Error('Device not registered. Call register() first.');
    }

    this.isRunning = true;

    const poll = async () => {
      try {
        const messages = await this.getMessages();
        for (const msg of messages) {
          this.notifyHandlers(msg);
        }
      } catch (error) {
        console.error('P2P poll error:', error);
      }
    };

    // Poll immediately
    poll();

    // Then at intervals
    this.pollInterval = window.setInterval(poll, intervalMs);
  }

  /**
   * Stop polling
   */
  stopPolling(): void {
    if (this.pollInterval) {
      clearInterval(this.pollInterval);
      this.pollInterval = null;
    }
    this.isRunning = false;
  }

  /**
   * Add message handler
   */
  onMessage(handler: (msg: P2PMessage) => void): () => void {
    this.messageHandlers.add(handler);

    // Return unsubscribe function
    return () => {
      this.messageHandlers.delete(handler);
    };
  }

  // ============================================================================
  // Health Check
  // ============================================================================

  /**
   * Check if Cloudflare Worker is available
   */
  async healthCheck(): Promise<{ status: string; version: string }> {
    return this.get('/health');
  }

  // ============================================================================
  // State Management
  // ============================================================================

  /**
   * Check if device is registered
   */
  isRegistered(): boolean {
    return !!this.deviceId && !!this.userId;
  }

  /**
   * Get current device info
   */
  getDevice(): P2PDevice | null {
    if (!this.deviceId || !this.userId || !this.publicKey) {
      return null;
    }

    return {
      deviceId: this.deviceId,
      userId: this.userId,
      publicKey: this.publicKey,
      registeredAt: new Date().toISOString(),
      lastSeen: new Date().toISOString(),
    };
  }

  /**
   * Unregister device
   */
  unregister(): void {
    this.stopPolling();
    this.deviceId = null;
    this.userId = null;
    this.publicKey = null;
    this.messageHandlers.clear();
    localStorage.removeItem('p2p_device_id');
    localStorage.removeItem('p2p_user_id');
    localStorage.removeItem('p2p_public_key');
  }

  // ============================================================================
  // Private Methods
  // ============================================================================

  private async get<T>(path: string): Promise<T> {
    const url = `${P2P_BASE}${path}`;
    const res = await fetch(url, {
      method: 'GET',
      headers: { 'Content-Type': 'application/json' },
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: res.statusText }));
      throw new Error(err.error || `HTTP ${res.status}`);
    }

    return res.json();
  }

  private async post<T>(path: string, body: any): Promise<T> {
    const url = `${P2P_BASE}${path}`;
    const res = await fetch(url, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify(body),
    });

    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: res.statusText }));
      throw new Error(err.error || `HTTP ${res.status}`);
    }

    return res.json();
  }

  private generateDeviceId(): string {
    return `device-${crypto.randomUUID()}`;
  }

  private notifyHandlers(msg: P2PMessage): void {
    for (const handler of this.messageHandlers) {
      try {
        handler(msg);
      } catch (error) {
        console.error('P2P message handler error:', error);
      }
    }
  }
}

// ============================================================================
// Singleton Export
// ============================================================================

export const p2p = new P2PClient();
export type { P2PDevice, P2PMessage };
export type P2PMessageType = P2PMessage['messageType'];
