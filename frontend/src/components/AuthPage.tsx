import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { api } from '../services/apiClient'

interface AuthPageProps { onAuth: () => void }

export function AuthPage({ onAuth }: AuthPageProps) {
  const { t } = useTranslation()
  const [isLogin, setIsLogin] = useState(true)
  const [username, setUsername] = useState('')
  const [password, setPassword] = useState('')
  const [displayName, setDisplayName] = useState('')
  const [error, setError] = useState('')
  const [loading, setLoading] = useState(false)

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault()
    setError('')
    setLoading(true)
    try {
      if (isLogin) {
        await api.login(username, password)
      } else {
        await api.register(username, password, displayName || undefined)
      }
      onAuth()
    } catch (err: any) {
      setError(err.message || 'Authentication failed')
    } finally {
      setLoading(false)
    }
  }

  return (
    <div className="min-h-screen flex items-center justify-center bg-gray-900">
      <div className="bg-gray-800 p-8 rounded-xl shadow-2xl w-full max-w-md">
        <h1 className="text-2xl font-bold text-white mb-2 text-center">🔒 Secure Messenger</h1>
        <p className="text-gray-400 text-center mb-6 text-sm">Post-quantum E2EE messaging</p>

        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <label className="block text-sm text-gray-300 mb-1">Username</label>
            <input type="text" value={username} onChange={e => setUsername(e.target.value)} required
              className="w-full px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none"
              placeholder="Enter username" minLength={3} maxLength={50} />
          </div>

          {!isLogin && (
            <div>
              <label className="block text-sm text-gray-300 mb-1">Display Name</label>
              <input type="text" value={displayName} onChange={e => setDisplayName(e.target.value)}
                className="w-full px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none"
                placeholder="Your display name" />
            </div>
          )}

          <div>
            <label className="block text-sm text-gray-300 mb-1">Password</label>
            <input type="password" value={password} onChange={e => setPassword(e.target.value)} required
              className="w-full px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none"
              placeholder="Enter password" minLength={6} />
          </div>

          {error && <div className="text-red-400 text-sm bg-red-900/30 p-2 rounded">{error}</div>}

          <button type="submit" disabled={loading}
            className="w-full py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg font-medium transition">
            {loading ? '...' : isLogin ? 'Login' : 'Register'}
          </button>
        </form>

        <button onClick={() => { setIsLogin(!isLogin); setError('') }}
          className="w-full mt-4 text-sm text-blue-400 hover:text-blue-300">
          {isLogin ? "Don't have an account? Register" : 'Already have an account? Login'}
        </button>
      </div>
    </div>
  )
}
