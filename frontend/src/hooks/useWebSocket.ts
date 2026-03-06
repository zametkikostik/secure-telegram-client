import { useEffect, useRef, useCallback } from 'react';
import { useAuthStore } from '../store/authStore';
import { useChatStore, Message } from '../store/chatStore';

export function useWebSocket() {
  const wsRef = useRef<WebSocket | null>(null);
  const token = useAuthStore((state) => state.token);
  const addMessage = useChatStore((state) => state.addMessage);

  const connect = useCallback(() => {
    if (!token) return;

    const wsUrl = `ws://localhost:8008/ws?token=${token}`;
    const ws = new WebSocket(wsUrl);

    ws.onopen = () => {
      console.log('WebSocket подключен');
      // Отправка авторизации
      ws.send(JSON.stringify({ type: 'auth', token }));
    };

    ws.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data);
        
        switch (data.type) {
          case 'message':
            // Добавление нового сообщения
            const message: Message = {
              id: data.id,
              chat_id: data.chat_id,
              sender_id: data.sender_id,
              content: data.content,
              translated_content: data.translated_content,
              type: data.type,
              file_url: data.file_url,
              is_edited: false,
              created_at: data.created_at,
            };
            addMessage(message);
            break;
            
          case 'typing':
            // Пользователь печатает
            console.log('Пользователь печатает:', data.chat_id);
            break;
            
          case 'read':
            // Сообщения прочитаны
            console.log('Сообщения прочитаны:', data.message_ids);
            break;
        }
      } catch (error) {
        console.error('Ошибка обработки сообщения:', error);
      }
    };

    ws.onerror = (error) => {
      console.error('WebSocket ошибка:', error);
    };

    ws.onclose = () => {
      console.log('WebSocket отключен. Переподключение через 5 секунд...');
      setTimeout(connect, 5000);
    };

    wsRef.current = ws;
  }, [token, addMessage]);

  const sendMessage = useCallback((chatId: string, content: string, type = 'text') => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'message',
        chat_id: chatId,
        content,
        message_type: type,
      }));
    }
  }, []);

  const sendTyping = useCallback((chatId: string) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'typing',
        chat_id: chatId,
      }));
    }
  }, []);

  const markAsRead = useCallback((chatId: string, messageIds: string[]) => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        type: 'read',
        chat_id: chatId,
        message_ids: messageIds,
      }));
    }
  }, []);

  useEffect(() => {
    connect();
    
    return () => {
      wsRef.current?.close();
    };
  }, [connect]);

  return {
    sendMessage,
    sendTyping,
    markAsRead,
  };
}
