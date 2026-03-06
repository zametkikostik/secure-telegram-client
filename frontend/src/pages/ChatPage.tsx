import React, { useState, useEffect, useRef } from 'react';
import { useChatStore, Message } from '../store/chatStore';
import { useWebSocket } from '../hooks/useWebSocket';
import { chatApi, messageApi } from '../api';
import { useAuthStore } from '../store/authStore';

// API URL из переменных окружения или default
const API_URL = import.meta.env.VITE_API_URL || 'http://localhost:8008/api/v1';
const WS_URL = import.meta.env.VITE_WS_URL || 'ws://localhost:8008/ws';

export default function ChatPage() {
  const { sendMessage } = useWebSocket();
  const user = useAuthStore((state) => state.user);
  const chats = useChatStore((state) => state.chats);
  const messages = useChatStore((state) => state.messages);
  const activeChat = useChatStore((state) => state.activeChat);
  const setChats = useChatStore((state) => state.setChats);
  const setActiveChat = useChatStore((state) => state.setActiveChat);
  const setMessages = useChatStore((state) => state.setMessages);
  const addMessage = useChatStore((state) => state.addMessage);

  const [inputText, setInputText] = useState('');
  const [loading, setLoading] = useState(true);
  const messagesEndRef = useRef<HTMLDivElement>(null);

  // Загрузка чатов
  useEffect(() => {
    const loadChats = async () => {
      try {
        const response = await chatApi.list();
        setChats(response.data);
      } catch (error) {
        console.error('Ошибка загрузки чатов:', error);
      } finally {
        setLoading(false);
      }
    };

    loadChats();
  }, [setChats]);

  // Загрузка сообщений при выборе чата
  useEffect(() => {
    if (activeChat) {
      const loadMessages = async () => {
        try {
          const response = await messageApi.list(activeChat);
          const messagesWithMine = response.data.map((msg: Message) => ({
            ...msg,
            isMine: msg.sender_id === user?.id,
          }));
          setMessages(activeChat, messagesWithMine);
        } catch (error) {
          console.error('Ошибка загрузки сообщений:', error);
        }
      };

      loadMessages();
    }
  }, [activeChat, user?.id, setMessages]);

  // Автопрокрутка к последнему сообщению
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [messages]);

  const handleSend = async () => {
    if (!inputText.trim() || !activeChat) return;

    const optimisticMessage: Message = {
      id: Date.now().toString(),
      chat_id: activeChat,
      sender_id: user!.id,
      content: inputText,
      type: 'text',
      is_edited: false,
      created_at: new Date().toISOString(),
      isMine: true,
    };

    addMessage(optimisticMessage);
    setInputText('');

    try {
      await messageApi.send(activeChat, inputText);
      sendMessage(activeChat, inputText);
    } catch (error) {
      console.error('Ошибка отправки:', error);
    }
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  };

  if (loading) {
    return (
      <div className="flex items-center justify-center h-screen">
        <div className="text-xl text-gray-600">Загрузка...</div>
      </div>
    );
  }

  return (
    <div className="flex h-screen bg-gray-100">
      {/* Список чатов */}
      <div className="w-80 bg-white border-r border-gray-200 flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <h2 className="text-xl font-bold text-gray-800">Чаты</h2>
        </div>
        <div className="flex-1 overflow-y-auto">
          {chats.map((chat) => (
            <div
              key={chat.id}
              onClick={() => setActiveChat(chat.id)}
              className={`p-4 cursor-pointer hover:bg-gray-50 transition ${
                activeChat === chat.id ? 'bg-primary-50' : ''
              }`}
            >
              <div className="flex items-center">
                <div className="w-12 h-12 rounded-full bg-primary-500 flex items-center justify-center text-white font-bold">
                  {chat.name?.[0]?.toUpperCase() || 'C'}
                </div>
                <div className="ml-3 flex-1 min-w-0">
                  <div className="font-medium text-gray-900 truncate">
                    {chat.name || 'Чат'}
                  </div>
                  <div className="text-sm text-gray-500 truncate">
                    {chat.last_message?.content || 'Нет сообщений'}
                  </div>
                </div>
              </div>
            </div>
          ))}
        </div>
      </div>

      {/* Окно чата */}
      <div className="flex-1 flex flex-col">
        {activeChat ? (
          <>
            {/* Заголовок */}
            <div className="p-4 bg-white border-b border-gray-200">
              <h3 className="text-lg font-bold text-gray-800">
                {chats.find((c) => c.id === activeChat)?.name || 'Чат'}
              </h3>
            </div>

            {/* Сообщения */}
            <div className="flex-1 overflow-y-auto p-4 space-y-4">
              {(messages[activeChat] || []).map((msg) => (
                <div
                  key={msg.id}
                  className={`flex ${msg.isMine ? 'justify-end' : 'justify-start'}`}
                >
                  <div
                    className={`max-w-xs lg:max-w-md px-4 py-2 rounded-2xl ${
                      msg.isMine
                        ? 'bg-primary-600 text-white'
                        : 'bg-gray-200 text-gray-900'
                    }`}
                  >
                    <p className="break-words">{msg.content}</p>
                    <p
                      className={`text-xs mt-1 ${
                        msg.isMine ? 'text-primary-200' : 'text-gray-500'
                      }`}
                    >
                      {new Date(msg.created_at).toLocaleTimeString([], {
                        hour: '2-digit',
                        minute: '2-digit',
                      })}
                    </p>
                    {msg.translated_content && (
                      <p className="text-xs mt-2 pt-2 border-t border-gray-300">
                        🌐 {msg.translated_content}
                      </p>
                    )}
                  </div>
                </div>
              ))}
              <div ref={messagesEndRef} />
            </div>

            {/* Ввод сообщения */}
            <div className="p-4 bg-white border-t border-gray-200">
              <div className="flex items-center space-x-2">
                <input
                  type="text"
                  value={inputText}
                  onChange={(e) => setInputText(e.target.value)}
                  onKeyPress={handleKeyPress}
                  placeholder="Введите сообщение..."
                  className="flex-1 px-4 py-3 border border-gray-300 rounded-full focus:ring-2 focus:ring-primary-500 focus:border-transparent"
                />
                <button
                  onClick={handleSend}
                  disabled={!inputText.trim()}
                  className="px-6 py-3 bg-primary-600 text-white rounded-full font-medium hover:bg-primary-700 transition disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  ➤
                </button>
              </div>
            </div>
          </>
        ) : (
          <div className="flex items-center justify-center h-full">
            <div className="text-center text-gray-500">
              <p className="text-xl">Выберите чат для начала общения</p>
            </div>
          </div>
        )}
      </div>
    </div>
  );
}
