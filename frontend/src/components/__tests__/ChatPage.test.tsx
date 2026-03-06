import React from 'react';
import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import ChatPage from '../../pages/ChatPage';
import { useAuthStore } from '../../store/authStore';
import { useChatStore } from '../../store/chatStore';

jest.mock('../../store/authStore');
jest.mock('../../store/chatStore');

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
    },
  });
  
  return ({ children }: { children: React.ReactNode }) => (
    <MemoryRouter>
      <QueryClientProvider client={queryClient}>
        {children}
      </QueryClientProvider>
    </MemoryRouter>
  );
};

describe('ChatPage', () => {
  beforeEach(() => {
    (useAuthStore as jest.Mock).mockReturnValue({
      user: { id: 'user-1', username: 'testuser', public_key: 'key' },
      token: 'token',
      isAuthenticated: true,
    });

    (useChatStore as jest.Mock).mockReturnValue({
      chats: [
        { id: '1', name: 'Test Chat', last_message: { content: 'Hello' }, unread_count: 0 },
      ],
      messages: { '1': [] },
      activeChat: null,
      setChats: jest.fn(),
      setActiveChat: jest.fn(),
      setMessages: jest.fn(),
      addMessage: jest.fn(),
    });
  });

  it('должен рендерить список чатов', () => {
    render(<ChatPage />, { wrapper: createWrapper() });
    
    expect(screen.getByText('Test Chat')).toBeInTheDocument();
  });

  it('должен позволять отправлять сообщения', async () => {
    render(<ChatPage />, { wrapper: createWrapper() });
    
    // Клик на чат
    fireEvent.click(screen.getByText('Test Chat'));
    
    // Ввод сообщения
    const input = screen.getByPlaceholderText(/введите сообщение/i);
    fireEvent.change(input, { target: { value: 'Hello!' } });
    
    // Отправка
    const sendButton = screen.getByText('➤');
    fireEvent.click(sendButton);
    
    await waitFor(() => {
      expect(input).toHaveValue('');
    });
  });
});
