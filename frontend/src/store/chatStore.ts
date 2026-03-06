import { create } from 'zustand';

export interface Message {
  id: string;
  chat_id: string;
  sender_id: string;
  content: string;
  translated_content?: string;
  type: string;
  file_url?: string;
  reply_to_id?: string;
  is_edited: boolean;
  created_at: string;
  isMine?: boolean;
}

export interface Chat {
  id: string;
  type: string;
  name?: string;
  description?: string;
  owner_id?: string;
  last_message?: {
    id: string;
    content: string;
    sender_id: string;
    created_at: string;
  };
  unread_count: number;
}

interface ChatState {
  chats: Chat[];
  messages: Record<string, Message[]>;
  activeChat: string | null;
  setChats: (chats: Chat[]) => void;
  addChat: (chat: Chat) => void;
  setActiveChat: (chatId: string | null) => void;
  addMessage: (message: Message) => void;
  setMessages: (chatId: string, messages: Message[]) => void;
}

export const useChatStore = create<ChatState>((set) => ({
  chats: [],
  messages: {},
  activeChat: null,
  
  setChats: (chats) => set({ chats }),
  
  addChat: (chat) => set((state) => ({ 
    chats: [...state.chats, chat] 
  })),
  
  setActiveChat: (chatId) => set({ activeChat: chatId }),
  
  addMessage: (message) => set((state) => {
    const chatMessages = state.messages[message.chat_id] || [];
    return {
      messages: {
        ...state.messages,
        [message.chat_id]: [...chatMessages, message],
      },
    };
  }),
  
  setMessages: (chatId, messages) => set((state) => ({
    messages: {
      ...state.messages,
      [chatId]: messages,
    },
  })),
}));
