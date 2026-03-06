import React from 'react';
import { render, screen } from '@testing-library/react';
import { MemoryRouter } from 'react-router-dom';
import ProtectedRoute from '../ProtectedRoute';
import { useAuthStore } from '../../store/authStore';

// Mock store
jest.mock('../../store/authStore');

describe('ProtectedRoute', () => {
  const mockUseAuthStore = useAuthStore as jest.MockedFunction<typeof useAuthStore>;

  it('должен рендерить children если пользователь авторизован', () => {
    mockUseAuthStore.mockReturnValue({
      user: { id: '1', username: 'test', public_key: 'key' },
      token: 'token',
      isAuthenticated: true,
      login: jest.fn(),
      logout: jest.fn(),
    });

    render(
      <MemoryRouter>
        <ProtectedRoute>
          <div data-testid="protected-content">Protected Content</div>
        </ProtectedRoute>
      </MemoryRouter>
    );

    expect(screen.getByTestId('protected-content')).toBeInTheDocument();
  });

  it('должен перенаправлять на /login если пользователь не авторизован', () => {
    mockUseAuthStore.mockReturnValue({
      user: null,
      token: null,
      isAuthenticated: false,
      login: jest.fn(),
      logout: jest.fn(),
    });

    render(
      <MemoryRouter initialEntries={['/protected']}>
        <ProtectedRoute>
          <div>Protected Content</div>
        </ProtectedRoute>
      </MemoryRouter>
    );

    // Проверяем что контент не рендерится
    expect(screen.queryByText('Protected Content')).not.toBeInTheDocument();
  });
});
