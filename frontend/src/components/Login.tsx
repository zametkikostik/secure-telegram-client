import { useState } from 'react'

export function Login() {
  const [password, setPassword] = useState('')
  const [isRegistering, setIsRegistering] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    // TODO: реализовать аутентификацию через Tauri
    console.log('Login with password:', password)
  }

  return (
    <div className="flex items-center justify-center min-h-screen bg-dark-950">
      <div className="card w-96 p-8">
        <h1 className="text-2xl font-bold text-center mb-6">
          🔐 Secure Messenger
        </h1>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label htmlFor="password" className="block text-sm font-medium mb-2">
              {isRegistering ? 'Создайте пароль' : 'Введите пароль'}
            </label>
            <input
              id="password"
              type="password"
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              className="input-primary w-full"
              placeholder="Минимум 8 символов"
              required
              minLength={8}
            />
          </div>
          <button type="submit" className="btn-primary w-full">
            {isRegistering ? 'Создать аккаунт' : 'Войти'}
          </button>
          <button
            type="button"
            className="btn-secondary w-full"
            onClick={() => setIsRegistering(!isRegistering)}
          >
            {isRegistering ? 'Уже есть аккаунт?' : 'Создать аккаунт'}
          </button>
        </form>
        <p className="text-xs text-dark-400 mt-4 text-center">
          Приватные ключи генерируются локально и никогда не покидают устройство
        </p>
      </div>
    </div>
  )
}
