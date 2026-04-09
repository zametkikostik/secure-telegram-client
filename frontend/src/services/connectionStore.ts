// Zustand Store for Fallback/Connection State
import { create } from 'zustand';
import { fallbackApi, ConnectionMode } from './fallbackClient';
import { p2p } from './p2pClient';

// ============================================================================
// Types
// ============================================================================

interface ConnectionState {
  // Connection mode
  mode: ConnectionMode;
  backendAvailable: boolean;
  p2pAvailable: boolean;

  // P2P device info
  isP2PRegistered: boolean;
  deviceId: string | null;
  userId: string | null;

  // Message queue (for offline messages)
  queuedMessages: Array<{
    id: string;
    chatId: string;
    content: string;
    msgType: string;
    toDeviceId?: string;
    timestamp: number;
    sent: boolean;
  }>;

  // Actions
  checkConnections: () => Promise<void>;
  startMonitoring: () => void;
  stopMonitoring: () => void;
  registerP2P: (publicKey: string) => Promise<void>;
  unregisterP2P: () => void;
  startP2PPolling: () => void;
  stopP2PPolling: () => void;
  queueMessage: (msg: {
    chatId: string;
    content: string;
    msgType: string;
    toDeviceId?: string;
  }) => string;
  flushQueue: () => Promise<void>;
}

// ============================================================================
// Store
// ============================================================================

export const useConnectionStore = create<ConnectionState>((set, get) => ({
  // Initial state
  mode: 'offline',
  backendAvailable: false,
  p2pAvailable: false,
  isP2PRegistered: false,
  deviceId: null,
  userId: null,
  queuedMessages: [],

  // Check connections
  checkConnections: async () => {
    const status = await fallbackApi.checkConnections();
    const mode = fallbackApi.getMode();

    const device = p2p.getDevice();

    set({
      backendAvailable: status.backend,
      p2pAvailable: status.p2p,
      mode,
      isP2PRegistered: !!device,
      deviceId: device?.deviceId || null,
      userId: device?.userId || null,
    });
  },

  // Start monitoring
  startMonitoring: () => {
    fallbackApi.startMonitoring();

    // Listen for mode changes
    fallbackApi.onModeChange((mode) => {
      set({ mode });
    });
  },

  // Stop monitoring
  stopMonitoring: () => {
    fallbackApi.stopMonitoring();
  },

  // Register P2P device
  registerP2P: async (publicKey: string) => {
    const device = await fallbackApi.registerP2P(publicKey);
    set({
      isP2PRegistered: true,
      deviceId: device.deviceId,
      userId: device.userId,
    });
  },

  // Unregister P2P device
  unregisterP2P: () => {
    fallbackApi.stopP2PPolling();
    set({
      isP2PRegistered: false,
      deviceId: null,
      userId: null,
      queuedMessages: [],
    });
  },

  // Start P2P polling
  startP2PPolling: () => {
    fallbackApi.startP2PPolling();
  },

  // Stop P2P polling
  stopP2PPolling: () => {
    fallbackApi.stopP2PPolling();
  },

  // Queue message for later sending
  queueMessage: (msg: {
    chatId: string;
    content: string;
    msgType: string;
    toDeviceId?: string;
  }) => {
    const id = `queued-${Date.now()}-${Math.random().toString(36).slice(2, 9)}`;

    set((state) => ({
      queuedMessages: [
        ...state.queuedMessages,
        {
          id,
          chatId: msg.chatId,
          content: msg.content,
          msgType: msg.msgType,
          toDeviceId: msg.toDeviceId,
          timestamp: Date.now(),
          sent: false,
        },
      ],
    }));

    return id;
  },

  // Flush queued messages
  flushQueue: async () => {
    const { queuedMessages } = get();

    if (queuedMessages.length === 0) return;

    for (const msg of queuedMessages) {
      if (msg.sent) continue;

      try {
        await fallbackApi.sendMessage(
          msg.chatId,
          msg.content,
          msg.msgType,
          undefined,
          msg.toDeviceId
        );

        // Mark as sent
        set((state) => ({
          queuedMessages: state.queuedMessages.map((m) =>
            m.id === msg.id ? { ...m, sent: true } : m
          ),
        }));
      } catch (error) {
        console.error('Failed to flush queued message:', error);
      }
    }

    // Remove sent messages
    set((state) => ({
      queuedMessages: state.queuedMessages.filter((m) => !m.sent),
    }));
  },
}));

// ============================================================================
// Selectors
// ============================================================================

export const selectConnectionMode = (state: ConnectionState) => state.mode;
export const selectBackendAvailable = (state: ConnectionState) => state.backendAvailable;
export const selectP2PAvailable = (state: ConnectionState) => state.p2pAvailable;
export const selectIsP2PRegistered = (state: ConnectionState) => state.isP2PRegistered;
export const selectQueuedMessages = (state: ConnectionState) => state.queuedMessages;
export const selectQueuedCount = (state: ConnectionState) => state.queuedMessages.length;
