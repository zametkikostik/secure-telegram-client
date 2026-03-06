import axios from 'axios';
import { useAuthStore } from '../store/authStore';

const API_BASE_URL = import.meta.env.VITE_API_URL || 'http://localhost:8008/api/v1';

const api = axios.create({
  baseURL: API_BASE_URL,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Interceptor для добавления токена
api.interceptors.request.use((config) => {
  const token = useAuthStore.getState().token;
  if (token) {
    config.headers.Authorization = `Bearer ${token}`;
  }
  return config;
});

// Auth API
export const authApi = {
  register: (username: string, password: string, email?: string) =>
    api.post('/auth/register', { username, password, email }),
  
  login: (username: string, password: string) =>
    api.post('/auth/login', { username, password }),
  
  verify: () => api.post('/auth/verify'),
};

// Chat API
export const chatApi = {
  list: () => api.get('/chats'),
  create: (type: string, name?: string, memberIds?: string[]) =>
    api.post('/chats', { type, name, member_ids: memberIds }),
  get: (chatId: string) => api.get(`/chats/${chatId}`),
};

// Message API
export const messageApi = {
  list: (chatId: string, limit = 50, offset = 0) =>
    api.get(`/chats/${chatId}/messages`, { params: { limit, offset } }),
  send: (chatId: string, content: string, type = 'text', fileUrl?: string) =>
    api.post(`/chats/${chatId}/messages`, { content, type, file_url: fileUrl }),
};

// File API
export const fileApi = {
  upload: (file: File) => {
    const formData = new FormData();
    formData.append('file', file);
    return api.post('/files/upload', formData, {
      headers: { 'Content-Type': 'multipart/form-data' },
    });
  },
  get: (fileId: string) => api.get(`/files/${fileId}`),
};

// AI API
export const aiApi = {
  translate: (text: string, from: string, to: string) =>
    api.post('/ai/translate', { text, from, to }),
  summarize: (text: string, maxLength?: number) =>
    api.post('/ai/summarize', { text, max_length: maxLength }),
  chat: (message: string, context?: Array<{ role: string; content: string }>) =>
    api.post('/ai/chat', { message, context }),
};

// Web3 API
export const web3Api = {
  getBalance: () => api.get('/web3/balance'),
  swap: (fromToken: string, toToken: string, amount: string) =>
    api.post('/web3/swap', { from_token: fromToken, to_token: toToken, amount }),
};

// User API
export const userApi = {
  me: () => api.get('/users/me'),
  get: (userId: string) => api.get(`/users/${userId}`),
};

export default api;
