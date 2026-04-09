/**
 * Bot API Client
 *
 * Client for the Bot Platform API.
 * Handles bot CRUD, commands, webhooks, and bot store.
 */

import axios, { AxiosInstance } from 'axios';
import type {
  Bot,
  BotCommand,
  BotWebhook,
  BotStoreListing,
  CreateBotPayload,
  UpdateBotPayload,
  CreateCommandPayload,
  CreateWebhookPayload,
} from '../types/bot';

const API_BASE = import.meta.env.VITE_BOT_API_URL || '/api/v1/bots';

function getAuthHeaders(): Record<string, string> {
  const token = localStorage.getItem('auth_token');
  return token ? { Authorization: `Bearer ${token}` } : {};
}

const apiClient = axios.create({
  baseURL: API_BASE,
  headers: { 'Content-Type': 'application/json' },
});

// Add auth interceptor
apiClient.interceptors.request.use((config) => {
  const headers = getAuthHeaders();
  Object.assign(config.headers, headers);
  return config;
});

// ============================================================================
// Bot Management
// ============================================================================

export async function createBot(payload: CreateBotPayload): Promise<Bot> {
  const { data } = await apiClient.post<Bot>('', payload);
  return data;
}

export async function listMyBots(): Promise<Bot[]> {
  const { data } = await apiClient.get<Bot[]>('');
  return data;
}

export async function getBot(botId: string): Promise<Bot> {
  const { data } = await apiClient.get<Bot>(`/${botId}`);
  return data;
}

export async function updateBot(botId: string, payload: UpdateBotPayload): Promise<Bot> {
  const { data } = await apiClient.put<Bot>(`/${botId}`, payload);
  return data;
}

export async function deleteBot(botId: string): Promise<void> {
  await apiClient.delete(`/${botId}`);
}

export async function rotateBotToken(botId: string): Promise<{ token: string }> {
  const { data } = await apiClient.post(`/${botId}/token/rotate`);
  return data;
}

// ============================================================================
// Bot Commands
// ============================================================================

export async function listBotCommands(botId: string): Promise<BotCommand[]> {
  const { data } = await apiClient.get<BotCommand[]>(`/${botId}/commands`);
  return data;
}

export async function createBotCommand(
  botId: string,
  payload: CreateCommandPayload,
): Promise<BotCommand> {
  const { data } = await apiClient.post<BotCommand>(`/${botId}/commands`, payload);
  return data;
}

export async function deleteBotCommand(botId: string, commandId: string): Promise<void> {
  await apiClient.delete(`/${botId}/commands/${commandId}`);
}

// ============================================================================
// Bot Webhooks
// ============================================================================

export async function listBotWebhooks(botId: string): Promise<BotWebhook[]> {
  const { data } = await apiClient.get<BotWebhook[]>(`/${botId}/webhooks`);
  return data;
}

export async function createBotWebhook(
  botId: string,
  payload: CreateWebhookPayload,
): Promise<BotWebhook> {
  const { data } = await apiClient.post<BotWebhook>(`/${botId}/webhooks`, payload);
  return data;
}

export async function deleteBotWebhook(botId: string, webhookId: string): Promise<void> {
  await apiClient.delete(`/${botId}/webhooks/${webhookId}`);
}

// ============================================================================
// Bot Store (Marketplace)
// ============================================================================

export async function listStoreBots(category?: string): Promise<BotStoreListing[]> {
  const params = category ? { category } : {};
  const { data } = await apiClient.get<BotStoreListing[]>('/store', { params });
  return data;
}

export async function getStoreBot(botId: string): Promise<BotStoreListing> {
  const { data } = await apiClient.get<BotStoreListing>(`/store/${botId}`);
  return data;
}

export async function installBot(botId: string): Promise<Bot> {
  const { data } = await apiClient.post<Bot>(`/store/${botId}/install`);
  return data;
}

export async function uninstallBot(botId: string): Promise<void> {
  await apiClient.post(`/store/${botId}/uninstall`);
}

// ============================================================================
// Bot Search
// ============================================================================

export async function searchBots(query: string): Promise<BotStoreListing[]> {
  const { data } = await apiClient.get<BotStoreListing[]>('/search', {
    params: { q: query },
  });
  return data;
}

// Export the raw axios instance for custom requests
export { apiClient as botApiClient };
