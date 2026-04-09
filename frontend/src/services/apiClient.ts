// @ts-nocheck
// API Client for Secure Messenger Backend
const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000';

interface AuthResponse { token: string; user_id: string; username: string; display_name: string | null }
interface User { id: string; username: string; display_name: string | null; public_key: string | null; avatar_url: string | null; is_online: number | null; last_seen: string | null; created_at: string }
interface Chat { id: string; name: string; chat_type: string; created_by: string; created_at: string; last_message_at: string | null; participant_count: number }
interface ChatListItem { id: string; name: string; chat_type: string; last_message: string | null; last_message_at: string | null; unread_count: number; is_online?: boolean }
interface Message { id: string; chat_id: string; sender_id: string; sender_name: string | null; content: string; msg_type: string; created_at: string; edited_at: string | null; reply_to: string | null }

class ApiClient {
  private token: string | null = null;

  constructor() {
    this.token = localStorage.getItem('auth_token');
  }

  private async request<T>(path: string, options: RequestInit = {}): Promise<T> {
    const headers: Record<string, string> = { 'Content-Type': 'application/json', ...(options.headers as Record<string, string>) };
    if (this.token) headers['Authorization'] = `Bearer ${this.token}`;

    const res = await fetch(`${API_BASE}${path}`, { ...options, headers });
    if (!res.ok) {
      const err = await res.json().catch(() => ({ error: res.statusText }));
      if (res.status === 401) { this.token = null; localStorage.removeItem('auth_token'); }
      throw new Error(err.error || `HTTP ${res.status}`);
    }
    if (res.status === 204) return {} as T;
    return res.json();
  }

  // Auth
  async register(username: string, password: string, displayName?: string): Promise<AuthResponse> {
    const res = await this.request<AuthResponse>('/api/v1/auth/register', { method: 'POST', body: JSON.stringify({ username, password, display_name: displayName }) });
    this.token = res.token; localStorage.setItem('auth_token', res.token); return res;
  }

  async login(username: string, password: string): Promise<AuthResponse> {
    const res = await this.request<AuthResponse>('/api/v1/auth/login', { method: 'POST', body: JSON.stringify({ username, password }) });
    this.token = res.token; localStorage.setItem('auth_token', res.token); return res;
  }

  async getMe(): Promise<User> { return this.request<User>('/api/v1/users/me'); }

  async getUser(id: string): Promise<User> { return this.request<User>(`/api/v1/users/${id}`); }

  // Chats
  async listChats(): Promise<ChatListItem[]> { return this.request<ChatListItem[]>('/api/v1/chats'); }

  async createChat(name: string, chatType?: string, participants?: string[]): Promise<Chat> {
    return this.request<Chat>('/api/v1/chats', { method: 'POST', body: JSON.stringify({ name, chat_type: chatType, participants }) });
  }

  async getChat(id: string): Promise<Chat> { return this.request<Chat>(`/api/v1/chats/${id}`); }

  // Messages
  async listMessages(chatId: string, limit = 50, offset = 0): Promise<Message[]> {
    return this.request<Message[]>(`/api/v1/chats/${chatId}/messages?limit=${limit}&offset=${offset}`);
  }

  async sendMessage(chatId: string, content: string, msgType?: string, replyTo?: string, destroyAfterSeconds?: number, scheduledFor?: string): Promise<Message> {
    return this.request<Message>(`/api/v1/chats/${chatId}/messages`, { method: 'POST', body: JSON.stringify({ content, msg_type: msgType, reply_to: replyTo, destroy_after_seconds: destroyAfterSeconds, scheduled_for: scheduledFor }) });
  }

  async getMessage(id: string): Promise<Message> { return this.request<Message>(`/api/v1/messages/${id}`); }

  // Edit and delete messages
  async editMessage(id: string, content: string): Promise<{ edited: boolean; edited_at: string }> {
    return this.request(`/api/v1/messages/${id}`, { method: 'PUT', body: JSON.stringify({ content }) });
  }

  async deleteMessage(id: string): Promise<{ deleted: boolean }> {
    return this.request(`/api/v1/messages/${id}`, { method: 'DELETE' });
  }

  // Profile
  async updateProfile(displayName?: string, x25519?: string, ed25519?: string, avatarUrl?: string, familyStatus?: string): Promise<User> {
    return this.request<User>('/api/v1/users/me', { method: 'PUT', body: JSON.stringify({ display_name: displayName, public_key_x25519: x25519, public_key_ed25519: ed25519, avatar_url: avatarUrl, family_status: familyStatus }) });
  }

  // Chat wallpapers
  async getWallpaper(chatId: string): Promise<{ color: string; pattern: string; custom_url: string | null }> {
    return this.request(`/api/v1/chats/${chatId}/wallpaper`);
  }
  async setWallpaper(chatId: string, color?: string, pattern?: string, customUrl?: string): Promise<any> {
    return this.request(`/api/v1/chats/${chatId}/wallpaper`, { method: 'PUT', body: JSON.stringify({ color, pattern, custom_url: customUrl }) });
  }

  // Pinned messages
  async pinMessage(messageId: string): Promise<any> {
    return this.request(`/api/v1/messages/${messageId}/pin`, { method: 'POST' });
  }
  async unpinMessage(messageId: string): Promise<any> {
    return this.request(`/api/v1/messages/${messageId}/unpin`, { method: 'POST' });
  }
  async getPinnedMessages(chatId: string): Promise<Message[]> {
    return this.request(`/api/v1/chats/${chatId}/pinned`);
  }

  // File upload
  async uploadFile(file: File, chatId: string): Promise<{ id: string; url: string; name: string; size: number; mime_type: string }> {
    const formData = new FormData();
    formData.append('file', file);
    formData.append('chat_id', chatId);
    const token = this.token;
    const response = await fetch(`${this.baseUrl}/api/v1/files`, {
      method: 'POST',
      headers: { 'Authorization': `Bearer ${token}` },
      body: formData,
    });
    if (!response.ok) throw new Error('Upload failed');
    return response.json();
  }

  // Search users
  async searchUsers(query: string, limit = 20): Promise<User[]> {
    return this.request<User[]>(`/api/v1/users/search?q=${encodeURIComponent(query)}&limit=${limit}`);
  }

  // Health
  async health(): Promise<{ status: string; version: string; ts: number }> {
    return this.request('/health');
  }

  logout() { this.token = null; localStorage.removeItem('auth_token'); }
  isAuthenticated(): boolean { return !!this.token; }
}

export const api = new ApiClient();
export type { AuthResponse, User, Chat, ChatListItem, Message };
