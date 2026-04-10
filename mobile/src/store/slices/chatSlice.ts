/**
 * Chat Slice - управление чатами и сообщениями
 */

import {createSlice, PayloadAction} from '@reduxjs/toolkit';

interface Message {
  id: string;
  text: string;
  senderId: string;
  timestamp: string;
  isEncrypted: boolean;
}

interface Chat {
  id: string;
  name: string;
  avatar: string | null;
  lastMessage: Message | null;
  unreadCount: number;
  isGroup: boolean;
  isOnline: boolean;
}

interface ChatState {
  chats: Chat[];
  currentChat: Chat | null;
  messages: Record<string, Message[]>;
  loading: boolean;
  error: string | null;
}

const initialState: ChatState = {
  chats: [],
  currentChat: null,
  messages: {},
  loading: false,
  error: null,
};

const chatSlice = createSlice({
  name: 'chats',
  initialState,
  reducers: {
    fetchChatsStart: (state) => {
      state.loading = true;
      state.error = null;
    },
    fetchChatsSuccess: (state, action: PayloadAction<Chat[]>) => {
      state.chats = action.payload;
      state.loading = false;
    },
    fetchChatsFailure: (state, action: PayloadAction<string>) => {
      state.loading = false;
      state.error = action.payload;
    },
    setCurrentChat: (state, action: PayloadAction<Chat>) => {
      state.currentChat = action.payload;
    },
    addMessage: (state, action: PayloadAction<{chatId: string; message: Message}>) => {
      const {chatId, message} = action.payload;
      if (!state.messages[chatId]) {
        state.messages[chatId] = [];
      }
      state.messages[chatId].push(message);
    },
    clearChat: (state) => {
      state.currentChat = null;
    },
  },
});

export const {
  fetchChatsStart,
  fetchChatsSuccess,
  fetchChatsFailure,
  setCurrentChat,
  addMessage,
  clearChat,
} = chatSlice.actions;

export default chatSlice.reducer;
