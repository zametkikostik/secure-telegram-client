// Fallback API Client
// Automatically falls back to P2P (Cloudflare Worker) when main backend is unavailable

import { api } from './apiClient';
import { p2p } from './p2pClient';

// ============================================================================
// Types
// ============================================================================

export interface FallbackMessage {
  id: string;
  chatId: string;
  senderId: string;
  content: string; // Encrypted content
  msgType: string;
  createdAt: string;
  editedAt: string | null;
  replyTo: string | null;
  isP2P: boolean; // True if sent via P2P fallback
}

export type ConnectionMode = 'backend' | 'p2p' | 'offline';

// ============================================================================
// Fallback Client Class
// ============================================================================

class FallbackClient {
  private mode: ConnectionMode = 'offline';
  private backendAvailable = false;
  private p2pAvailable = false;
  private checkInterval: number | null = null;
  private modeChangeListeners: Set<(mode: ConnectionMode) => void> = new Set();

  constructor() {
    // Check initial state
    this.checkConnections();
  }

  // ============================================================================
  // Connection Management
  // ============================================================================

  /**
   * Check if backend and P2P are available
   */
  async checkConnections(): Promise<{ backend: boolean; p2p: boolean }> {
    // Check backend
    try {
      await api.health();
      this.backendAvailable = true;
    } catch {
      this.backendAvailable = false;
    }

    // Check P2P
    try {
      await p2p.healthCheck();
      this.p2pAvailable = true;
    } catch {
      this.p2pAvailable = false;
    }

    // Update mode
    this.updateMode();

    return {
      backend: this.backendAvailable,
      p2p: this.p2pAvailable,
    };
  }

  /**
   * Start connection monitoring
   */
  startMonitoring(intervalMs = 30000): void {
    if (this.checkInterval) return;

    this.checkInterval = window.setInterval(async () => {
      await this.checkConnections();
    }, intervalMs);
  }

  /**
   * Stop connection monitoring
   */
  stopMonitoring(): void {
    if (this.checkInterval) {
      clearInterval(this.checkInterval);
      this.checkInterval = null;
    }
  }

  /**
   * Get current connection mode
   */
  getMode(): ConnectionMode {
    return this.mode;
  }

  /**
   * Check if backend is available
   */
  isBackendAvailable(): boolean {
    return this.backendAvailable;
  }

  /**
   * Check if P2P is available
   */
  isP2PAvailable(): boolean {
    return this.p2pAvailable;
  }

  // ============================================================================
  // Auth (always uses backend)
  // ============================================================================

  async register(username: string, password: string, displayName?: string) {
    if (!this.backendAvailable) {
      throw new Error('Backend unavailable. Cannot register without backend.');
    }
    return api.register(username, password, displayName);
  }

  async login(username: string, password: string) {
    if (!this.backendAvailable) {
      throw new Error('Backend unavailable. Cannot login without backend.');
    }
    return api.login(username, password);
  }

  async logout() {
    api.logout();
    // Also unregister P2P device
    p2p.unregister();
  }

  // ============================================================================
  // Chats (backend only for now)
  // ============================================================================

  async listChats() {
    if (!this.backendAvailable) {
      throw new Error('Chat list unavailable in P2P mode.');
    }
    return api.listChats();
  }

  async createChat(name: string, chatType?: string, participants?: string[]) {
    if (!this.backendAvailable) {
      throw new Error('Chat creation unavailable in P2P mode.');
    }
    return api.createChat(name, chatType, participants);
  }

  async getChat(id: string) {
    if (!this.backendAvailable) {
      throw new Error('Chat details unavailable in P2P mode.');
    }
    return api.getChat(id);
  }

  // ============================================================================
  // Messages (with P2P fallback)
  // ============================================================================

  /**
   * Send message - tries backend first, then P2P
   */
  async sendMessage(
    chatId: string,
    content: string,
    msgType = 'text',
    replyTo?: string,
    toDeviceId?: string // Required for P2P
  ): Promise<{ success: boolean; messageId: string; isP2P: boolean }> {
    // Try backend first
    if (this.backendAvailable) {
      try {
        const message = await api.sendMessage(chatId, content, msgType, replyTo);
        return {
          success: true,
          messageId: message.id,
          isP2P: false,
        };
      } catch (error) {
        console.warn('Backend send failed, trying P2P:', error);
        this.backendAvailable = false;
        this.updateMode();
      }
    }

    // Fallback to P2P
    if (this.p2pAvailable && toDeviceId) {
      try {
        const response = await p2p.sendMessage(toDeviceId, content, msgType as any);
        return {
          success: response.success,
          messageId: response.messageId,
          isP2P: true,
        };
      } catch (error) {
        console.error('P2P send failed:', error);
        throw new Error('Failed to send message via both backend and P2P');
      }
    }

    throw new Error('No available connection to send message');
  }

  /**
   * Get messages - from backend or P2P
   */
  async listMessages(chatId: string, limit = 50, offset = 0) {
    if (this.backendAvailable) {
      return api.listMessages(chatId, limit, offset);
    }

    if (this.p2pAvailable) {
      // Get P2P messages and convert format
      const p2pMessages = await p2p.getMessages();
      return p2pMessages.map(this.convertP2PMessage);
    }

    throw new Error('No available connection to get messages');
  }

  // ============================================================================
  // P2P Specific
  // ============================================================================

  /**
   * Register device for P2P messaging
   */
  async registerP2P(publicKey: string) {
    if (!api.isAuthenticated()) {
      throw new Error('Must be authenticated to register P2P device.');
    }

    // Get user ID from token or API
    const user = await api.getMe();

    return p2p.register(user.id, publicKey);
  }

  /**
   * Start receiving P2P messages
   */
  startP2PPolling(intervalMs = 5000) {
    p2p.startPolling(intervalMs);

    // Add handler to convert P2P messages
    p2p.onMessage((msg) => {
      console.log('P2P message received:', msg);
      // Could emit to a global event bus here
    });
  }

  /**
   * Stop P2P polling
   */
  stopP2PPolling() {
    p2p.stopPolling();
  }

  /**
   * Sync contacts to Cloudflare Worker
   */
  async syncP2PContacts(contacts: string[]) {
    return p2p.syncContacts(contacts);
  }

  /**
   * Get P2P contacts
   */
  async getP2PContacts(): Promise<string[]> {
    return p2p.getContacts();
  }

  // ============================================================================
  // Event Listeners
  // ============================================================================

  /**
   * Listen for connection mode changes
   */
  onModeChange(handler: (mode: ConnectionMode) => void): () => void {
    this.modeChangeListeners.add(handler);

    return () => {
      this.modeChangeListeners.delete(handler);
    };
  }

  // ============================================================================
  // Private Methods
  // ============================================================================

  private updateMode(): void {
    const oldMode = this.mode;

    if (this.backendAvailable) {
      this.mode = 'backend';
    } else if (this.p2pAvailable) {
      this.mode = 'p2p';
    } else {
      this.mode = 'offline';
    }

    // Notify listeners if mode changed
    if (oldMode !== this.mode) {
      this.notifyModeChangeListeners();
    }
  }

  private notifyModeChangeListeners(): void {
    for (const handler of this.modeChangeListeners) {
      try {
        handler(this.mode);
      } catch (error) {
        console.error('Mode change handler error:', error);
      }
    }
  }

  private convertP2PMessage(msg: any): FallbackMessage {
    return {
      id: msg.id,
      chatId: msg.fromDeviceId, // Use deviceId as chatId for P2P
      senderId: msg.fromDeviceId,
      content: msg.encryptedMessage,
      msgType: msg.messageType,
      createdAt: new Date(msg.timestamp).toISOString(),
      editedAt: null,
      replyTo: null,
      isP2P: true,
    };
  }
}

// ============================================================================
// Singleton Export
// ============================================================================

export const fallbackApi = new FallbackClient();
