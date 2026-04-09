/**
 * Bot Store — Zustand state management for bot platform
 */

import { create } from 'zustand';
import type {
  Bot,
  BotCommand,
  BotStoreListing,
  BotCategory,
  CreateBotPayload,
} from '../types/bot';
import * as botApi from './botApiClient';

interface BotPlatformState {
  // State
  myBots: Bot[];
  storeBots: BotStoreListing[];
  selectedBot: Bot | null;
  botCommands: Record<string, BotCommand[]>;  // botId -> commands
  loading: boolean;
  error: string | null;

  // Bot CRUD
  loadMyBots: () => Promise<void>;
  createBot: (payload: CreateBotPayload) => Promise<Bot>;
  deleteBot: (botId: string) => Promise<void>;
  selectBot: (bot: Bot | null) => void;

  // Commands
  loadBotCommands: (botId: string) => Promise<void>;

  // Store
  loadStore: (category?: BotCategory) => Promise<void>;
  installFromStore: (botId: string) => Promise<void>;

  // Search
  searchBots: (query: string) => Promise<BotStoreListing[]>;

  // Utility
  reset: () => void;
}

export const useBotPlatformStore = create<BotPlatformState>((set, get) => ({
  myBots: [],
  storeBots: [],
  selectedBot: null,
  botCommands: {},
  loading: false,
  error: null,

  loadMyBots: async () => {
    set({ loading: true, error: null });
    try {
      const bots = await botApi.listMyBots();
      set({ myBots: bots, loading: false });
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  createBot: async (payload: CreateBotPayload) => {
    set({ loading: true, error: null });
    try {
      const bot = await botApi.createBot(payload);
      set((state) => ({
        myBots: [...state.myBots, bot],
        loading: false,
      }));
      return bot;
    } catch (e: any) {
      set({ error: e.message, loading: false });
      throw e;
    }
  },

  deleteBot: async (botId: string) => {
    set({ loading: true, error: null });
    try {
      await botApi.deleteBot(botId);
      set((state) => ({
        myBots: state.myBots.filter((b) => b.id !== botId),
        selectedBot: state.selectedBot?.id === botId ? null : state.selectedBot,
        loading: false,
      }));
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  selectBot: (bot: Bot | null) => {
    set({ selectedBot: bot });
  },

  loadBotCommands: async (botId: string) => {
    try {
      const commands = await botApi.listBotCommands(botId);
      set((state) => ({
        botCommands: { ...state.botCommands, [botId]: commands },
      }));
    } catch (e: any) {
      set({ error: e.message });
    }
  },

  loadStore: async (category?: BotCategory) => {
    set({ loading: true, error: null });
    try {
      const bots = await botApi.listStoreBots(category);
      set({ storeBots: bots, loading: false });
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  installFromStore: async (botId: string) => {
    set({ loading: true, error: null });
    try {
      const bot = await botApi.installBot(botId);
      set((state) => ({
        myBots: [...state.myBots, bot],
        loading: false,
      }));
    } catch (e: any) {
      set({ error: e.message, loading: false });
    }
  },

  searchBots: async (query: string) => {
    try {
      return await botApi.searchBots(query);
    } catch (e: any) {
      set({ error: e.message });
      return [];
    }
  },

  reset: () => {
    set({
      myBots: [],
      storeBots: [],
      selectedBot: null,
      botCommands: {},
      loading: false,
      error: null,
    });
  },
}));
