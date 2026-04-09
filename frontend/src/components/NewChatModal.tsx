import { useState, useEffect } from 'react'
import { FiX, FiUsers, FiHash, FiUser, FiSearch, FiPlus, FiCheck } from 'react-icons/fi'
import { api, User } from '../services/apiClient'

// ============================================================================
// Types
// ============================================================================

type ChatType = 'direct' | 'group' | 'channel'

interface NewChatModalProps {
  onClose: () => void
  onChatCreated: (chatId: string) => void
}

// ============================================================================
// Component
// ============================================================================

export function NewChatModal({ onClose, onChatCreated }: NewChatModalProps) {
  const [chatType, setChatType] = useState<ChatType>('direct')
  const [chatName, setChatName] = useState('')
  const [chatDescription, setChatDescription] = useState('')
  const [searchQuery, setSearchQuery] = useState('')
  const [searchResults, setSearchResults] = useState<User[]>([])
  const [selectedParticipants, setSelectedParticipants] = useState<string[]>([])
  const [loading, setLoading] = useState(false)
  const [error, setError] = useState<string | null>(null)
  const [searching, setSearching] = useState(false)
  const [currentUserId, setCurrentUserId] = useState<string | null>(null)

  // Load current user ID
  useEffect(() => {
    api.getMe().then(user => setCurrentUserId(user.id)).catch(() => {})
  }, [])

  // ============================================================================
  // User Search
  // ============================================================================

  useEffect(() => {
    if (searchQuery.length < 2) {
      setSearchResults([])
      return
    }

    const timer = setTimeout(async () => {
      setSearching(true)
      try {
        const users = await api.searchUsers(searchQuery, 10)
        setSearchResults(users.filter(u => u.id !== currentUserId))
      } catch {
        setSearchResults([])
      } finally {
        setSearching(false)
      }
    }, 300)

    return () => clearTimeout(timer)
  }, [searchQuery])

  // ============================================================================
  // Participant Management
  // ============================================================================

  const toggleParticipant = (userId: string) => {
    setSelectedParticipants(prev =>
      prev.includes(userId)
        ? prev.filter(id => id !== userId)
        : [...prev, userId]
    )
  }

  // ============================================================================
  // Chat Creation
  // ============================================================================

  const handleCreateChat = async () => {
    if (!chatName.trim()) {
      setError('Введите название чата')
      return
    }

    if (chatType === 'direct' && selectedParticipants.length === 0) {
      setError('Выберите пользователя для личного чата')
      return
    }

    setLoading(true)
    setError(null)

    try {
      const chat = await api.createChat(
        chatName.trim(),
        chatType,
        selectedParticipants.length > 0 ? selectedParticipants : undefined
      )
      onChatCreated(chat.id)
    } catch (e: any) {
      setError(e.message || 'Не удалось создать чат')
    } finally {
      setLoading(false)
    }
  }

  // ============================================================================
  // Render
  // ============================================================================

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-sm" onClick={onClose}>
      <div
        className="bg-gray-800 rounded-2xl w-full max-w-md mx-4 shadow-2xl border border-gray-700"
        onClick={e => e.stopPropagation()}
      >
        {/* Header */}
        <div className="flex items-center justify-between p-4 border-b border-gray-700">
          <h2 className="text-xl font-semibold text-white">Новый чат</h2>
          <button onClick={onClose} className="p-2 rounded-lg hover:bg-gray-700 text-gray-400 transition-colors">
            <FiX className="w-5 h-5" />
          </button>
        </div>

        {/* Content */}
        <div className="p-4 space-y-4 max-h-[70vh] overflow-y-auto">
          {/* Chat Type Selector */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-2">Тип чата</label>
            <div className="grid grid-cols-3 gap-2">
              <button
                onClick={() => setChatType('direct')}
                className={`flex flex-col items-center gap-1 p-3 rounded-xl border-2 transition-all ${
                  chatType === 'direct'
                    ? 'border-blue-500 bg-blue-500/10 text-blue-400'
                    : 'border-gray-600 hover:border-gray-500 text-gray-400'
                }`}
              >
                <FiUser className="w-5 h-5" />
                <span className="text-xs">Личный</span>
              </button>

              <button
                onClick={() => setChatType('group')}
                className={`flex flex-col items-center gap-1 p-3 rounded-xl border-2 transition-all ${
                  chatType === 'group'
                    ? 'border-blue-500 bg-blue-500/10 text-blue-400'
                    : 'border-gray-600 hover:border-gray-500 text-gray-400'
                }`}
              >
                <FiUsers className="w-5 h-5" />
                <span className="text-xs">Группа</span>
              </button>

              <button
                onClick={() => setChatType('channel')}
                className={`flex flex-col items-center gap-1 p-3 rounded-xl border-2 transition-all ${
                  chatType === 'channel'
                    ? 'border-blue-500 bg-blue-500/10 text-blue-400'
                    : 'border-gray-600 hover:border-gray-500 text-gray-400'
                }`}
              >
                <FiHash className="w-5 h-5" />
                <span className="text-xs">Канал</span>
              </button>
            </div>
          </div>

          {/* Chat Name */}
          <div>
            <label className="block text-sm font-medium text-gray-300 mb-1">
              {chatType === 'direct' ? 'Имя контакта' : chatType === 'group' ? 'Название группы' : 'Название канала'}
            </label>
            <input
              type="text"
              value={chatName}
              onChange={e => setChatName(e.target.value)}
              placeholder={
                chatType === 'direct' ? 'Введите имя...' :
                chatType === 'group' ? 'Моя группа...' :
                'Мой канал...'
              }
              className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
              autoFocus
            />
          </div>

          {/* Description (for groups and channels) */}
          {(chatType === 'group' || chatType === 'channel') && (
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Описание (необязательно)</label>
              <textarea
                value={chatDescription}
                onChange={e => setChatDescription(e.target.value)}
                placeholder="Описание чата..."
                rows={2}
                className="w-full px-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent resize-none"
              />
            </div>
          )}

          {/* Participant Search (for groups) */}
          {chatType === 'group' && (
            <div>
              <label className="block text-sm font-medium text-gray-300 mb-1">Добавить участников</label>
              <div className="relative">
                <FiSearch className="absolute left-3 top-1/2 -translate-y-1/2 w-4 h-4 text-gray-500" />
                <input
                  type="text"
                  value={searchQuery}
                  onChange={e => setSearchQuery(e.target.value)}
                  placeholder="Поиск по username..."
                  className="w-full pl-10 pr-3 py-2 bg-gray-700 border border-gray-600 rounded-lg text-white placeholder-gray-500 focus:outline-none focus:ring-2 focus:ring-blue-500 focus:border-transparent"
                />
              </div>

              {/* Search Results */}
              {searching && (
                <div className="mt-2 text-sm text-gray-500 text-center py-2">Поиск...</div>
              )}

              {searchResults.length > 0 && (
                <div className="mt-2 space-y-1 max-h-32 overflow-y-auto">
                  {searchResults.map(user => (
                    <button
                      key={user.id}
                      onClick={() => toggleParticipant(user.id)}
                      className={`w-full flex items-center gap-3 p-2 rounded-lg transition-colors ${
                        selectedParticipants.includes(user.id)
                          ? 'bg-blue-500/20 border border-blue-500'
                          : 'bg-gray-700 hover:bg-gray-600'
                      }`}
                    >
                      <div className="w-8 h-8 rounded-full bg-gray-600 flex items-center justify-center text-sm font-medium">
                        {(user.display_name || user.username).charAt(0).toUpperCase()}
                      </div>
                      <div className="flex-1 text-left">
                        <div className="text-sm font-medium">{user.display_name || user.username}</div>
                        <div className="text-xs text-gray-500">@{user.username}</div>
                      </div>
                      {selectedParticipants.includes(user.id) && (
                        <FiCheck className="w-4 h-4 text-blue-400" />
                      )}
                    </button>
                  ))}
                </div>
              )}

              {/* Selected Participants */}
              {selectedParticipants.length > 0 && (
                <div className="mt-2">
                  <div className="text-xs text-gray-500 mb-1">Выбрано: {selectedParticipants.length}</div>
                  <div className="flex flex-wrap gap-1">
                    {selectedParticipants.map(id => (
                      <span key={id} className="px-2 py-1 bg-blue-500/20 text-blue-400 text-xs rounded-full">
                        {id.slice(0, 8)}...
                      </span>
                    ))}
                  </div>
                </div>
              )}
            </div>
          )}

          {/* Error Message */}
          {error && (
            <div className="p-3 bg-red-500/10 border border-red-500/30 rounded-lg text-red-400 text-sm">
              {error}
            </div>
          )}
        </div>

        {/* Footer */}
        <div className="p-4 border-t border-gray-700 flex gap-2">
          <button
            onClick={onClose}
            className="flex-1 px-4 py-2 bg-gray-700 hover:bg-gray-600 text-gray-300 rounded-lg transition-colors"
          >
            Отмена
          </button>
          <button
            onClick={handleCreateChat}
            disabled={loading || !chatName.trim()}
            className="flex-1 px-4 py-2 bg-blue-600 hover:bg-blue-500 disabled:bg-gray-600 disabled:text-gray-500 text-white rounded-lg transition-colors flex items-center justify-center gap-2"
          >
            <FiPlus className="w-4 h-4" />
            {loading ? 'Создание...' : 'Создать'}
          </button>
        </div>
      </div>
    </div>
  )
}
