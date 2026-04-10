/**
 * API Service
 */

import axios from 'axios';
import {API_BASE_URL} from '../utils/constants';

const api = axios.create({
  baseURL: API_BASE_URL,
  timeout: 10000,
  headers: {
    'Content-Type': 'application/json',
  },
});

// Interceptor для добавления токена
api.interceptors.request.use(
  async config => {
    // TODO: Получить токен из encrypted storage
    // const token = await AsyncStorage.getItem('authToken');
    // if (token) {
    //   config.headers.Authorization = `Bearer ${token}`;
    // }
    return config;
  },
  error => {
    return Promise.reject(error);
  },
);

// Interceptor для обработки ошибок
api.interceptors.response.use(
  response => response,
  async error => {
    if (error.response?.status === 401) {
      // TODO: Выход из системы
      console.error('Unauthorized, logging out...');
    }
    return Promise.reject(error);
  },
);

// Auth API
export const authAPI = {
  login: async (phone: string, password: string) => {
    const response = await api.post('/auth/login', {phone, password});
    return response.data;
  },
  register: async (phone: string, password: string, username: string) => {
    const response = await api.post('/auth/register', {phone, password, username});
    return response.data;
  },
  verifyCode: async (code: string) => {
    const response = await api.post('/auth/verify', {code});
    return response.data;
  },
};

// Chats API
export const chatsAPI = {
  getChats: async () => {
    const response = await api.get('/chats');
    return response.data;
  },
  getChatMessages: async (chatId: string) => {
    const response = await api.get(`/chats/${chatId}/messages`);
    return response.data;
  },
  sendMessage: async (chatId: string, text: string, encryptedPayload: string) => {
    const response = await api.post(`/chats/${chatId}/messages`, {
      text,
      encryptedPayload,
      timestamp: new Date().toISOString(),
    });
    return response.data;
  },
  createChat: async (userId: string, isGroup: boolean = false) => {
    const response = await api.post('/chats', {userId, isGroup});
    return response.data;
  },
};

// Users API
export const usersAPI = {
  getProfile: async () => {
    const response = await api.get('/users/me');
    return response.data;
  },
  getContacts: async () => {
    const response = await api.get('/users/contacts');
    return response.data;
  },
  searchUsers: async (query: string) => {
    const response = await api.get(`/users/search?q=${query}`);
    return response.data;
  },
  updateProfile: async (data: any) => {
    const response = await api.put('/users/me', data);
    return response.data;
  },
};

export default api;
