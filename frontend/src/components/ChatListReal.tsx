import { useState, useEffect, useCallback } from 'react'
import { formatDistanceToNow } from 'date-fns'
import { FiMessageSquare, FiHash, FiVolume2, FiPlus, FiCpu } from 'react-icons/fi'
import { api, type ChatListItem } from '../services/apiClient'
import { getWebSocketClient } from '../services/webSocketClient'
import clsx from 'clsx'
import { StoryBar } from './StoryBar'
import { StoryViewer } from './StoryViewer'
import { NewChatModal } from './NewChatModal'
import { useStoryStore } from '../services/storyStore'
import { BotManager } from './BotManager'
import { BotStore } from './BotStore'
import { Web3Panel } from './web3/Web3Panel'

interface ChatListProps {
  selectedChat: string | null
  onSelectChat: (id: string) => void
  onLogout: () => void
}

const TYPE_ICONS: Record<string, React.ReactNode> = {
  direct: <FiMessageSquare className="w-4 h-4" />,
  group: <FiHash className="w-4 h-4" />,
  channel: <FiVolume2 className="w-4 h-4" />,
}

const COLORS = ['bg-blue-500','bg-green-500','bg-purple-500','bg-pink-500','bg-yellow-500','bg-indigo-500']

export function ChatList({ selectedChat, onSelectChat, onLogout }: ChatListProps) {
  const [chats, setChats] = useState<ChatListItem[]>([])
  const [loading, setLoading] = useState(true)
  const [error, setError] = useState('')
  const [showNewChatModal, setShowNewChatModal] = useState(false)
  const [onlineUsers, setOnlineUsers] = useState<Set<string>>(new Set())
  const [showBotsPanel, setShowBotsPanel] = useState(false)
  const [showBotStore, setShowBotStore] = useState(false)
  const [showWeb3Panel, setShowWeb3Panel] = useState(false)
  const { openStoryViewer } = useStoryStore()

  const loadChats = useCallback(async () => {
    try {
      const list = await api.listChats()
      setChats(list)
      // Fetch online status for all participants
      const online = new Set<string>()
      for (const chat of list) {
        if (chat.chat_type === 'direct') {
          try {
            const participants = await fetch(`/api/v1/chats/${chat.id}/participants`, {
              headers: { 'Authorization': `Bearer ${localStorage.getItem('auth_token')}` }
            }).then(r => r.json())
            for (const p of participants) {
              if (p.is_online) online.add(p.user_id)
            }
          } catch {}
        }
      }
      setOnlineUsers(online)
    } catch (e: any) {
      if (e.message !== 'Missing auth') setError(e.message)
    } finally {
      setLoading(false)
    }
  }, [])

  useEffect(() => { loadChats() }, [loadChats])

  // Subscribe to presence updates
  useEffect(() => {
    const wsClient = getWebSocketClient({
      baseUrl: import.meta.env.VITE_API_URL || 'http://localhost:3000',
      userId: '',
      autoConnect: false,
      channels: ['presence'],
    })
    wsClient.connect().catch(() => {})
    wsClient.onMessage('presence', (payload: any) => {
      setOnlineUsers(prev => {
        const next = new Set(prev)
        if (payload.online) next.add(payload.user_id)
        else next.delete(payload.user_id)
        return next
      })
    })
    return () => wsClient.disconnect()
  }, [])

  const handleChatCreated = (chatId: string) => {
    setShowNewChatModal(false)
    loadChats()
    onSelectChat(chatId)
  }

  const fmtTime = (d?: string) => {
    if (!d) return ''
    try { return formatDistanceToNow(new Date(d), { addSuffix: true }) } catch { return '' }
  }

  return (
    <div className="w-80 flex flex-col h-screen border-r" style={{ backgroundColor: 'var(--color-bg-secondary, #1e293b)', borderColor: 'var(--color-border, #334155)' }}>
      {/* Header */}
      <div className="p-4 border-b" style={{ borderColor: 'var(--color-border, #334155)' }}>
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-lg font-semibold" style={{ color: 'var(--color-text-primary, #f1f5f9)' }}>Chats</h2>
          <div className="flex gap-2">
            <button onClick={() => setShowWeb3Panel(!showWeb3Panel)} className="p-2 rounded hover:bg-gray-700 text-gray-400 transition-colors" title="Web3">
              <span className="text-lg">💎</span>
            </button>
            <button onClick={() => setShowBotsPanel(!showBotsPanel)} className="p-2 rounded hover:bg-gray-700 text-gray-400 transition-colors" title="Боты">
              <FiCpu className="w-5 h-5" />
            </button>
            <button onClick={() => setShowNewChatModal(true)} className="p-2 rounded hover:bg-gray-700 text-gray-400 transition-colors" title="Новый чат">
              <FiPlus className="w-5 h-5" />
            </button>
            <button onClick={onLogout} className="px-3 py-1 text-xs rounded bg-red-600 hover:bg-red-700 text-white">Logout</button>
          </div>
        </div>
      </div>

      {/* Story Bar */}
      <StoryBar
        onOpenStory={(index) => openStoryViewer(index)}
        onCreateStory={() => console.log('Create story')}
      />

      {/* Chat List */}
      <div className="flex-1 overflow-y-auto">
        {loading && <div className="p-8 text-center text-gray-500">Loading...</div>}
        {error && <div className="p-4 text-center text-red-400 text-sm">{error}</div>}
        {!loading && !error && chats.length === 0 && (
          <div className="p-8 text-center text-gray-500">
            <FiMessageSquare className="w-12 h-12 mx-auto mb-3 opacity-50" />
            <p className="text-sm">No chats yet</p>
            <p className="text-xs mt-1">Click + to create a chat</p>
          </div>
        )}
        {chats.map((chat, i) => {
          const isOnline = chat.chat_type === 'direct' && onlineUsers.has(chat.id)
          const initials = chat.name.split(' ').map(w => w[0]).join('').slice(0, 2).toUpperCase()
          const avatarClass = `avatar-gradient-${(i % 8) + 1}`
          return (
          <button key={chat.id} onClick={() => onSelectChat(chat.id)}
            className={clsx('w-full p-3 flex items-start gap-3 border-b transition-all duration-200 text-left hover:bg-white/5', selectedChat === chat.id && 'bg-white/10 border-l-[3px]')}
            style={{ borderColor: selectedChat === chat.id ? 'var(--color-accent, #3b82f6)' : 'var(--color-border, #334155)' }}>
            <div className="relative">
              <div className={clsx('w-12 h-12 rounded-full flex items-center justify-center text-white font-semibold flex-shrink-0 text-sm', avatarClass)}>
                {initials}
              </div>
              {isOnline && (
                <div className="absolute -bottom-0.5 -right-0.5 w-3.5 h-3.5 bg-green-500 border-2 border-[#1e293b] rounded-full animate-online-pulse" />
              )}
            </div>
            <div className="flex-1 min-w-0">
              <div className="flex justify-between items-center mb-1">
                <div className="flex items-center gap-2">
                  <span className="font-medium truncate" style={{ color: 'var(--color-text-primary, #f1f5f9)' }}>{chat.name}</span>
                  {isOnline && <span className="text-xs text-green-400">online</span>}
                </div>
                <span className="text-xs text-gray-500 flex-shrink-0 ml-2">{fmtTime(chat.last_message_at)}</span>
              </div>
              <p className="text-sm truncate text-gray-400">{chat.last_message || 'No messages yet'}</p>
            </div>
          </button>
          )
        })}
      </div>
      {/* Story Viewer */}
      <StoryViewer />

      {/* Bots Panel */}
      {showBotsPanel && !showBotStore && (
        <div className="absolute inset-0 bg-gray-900 z-10 overflow-y-auto">
          <div className="p-4 border-b border-gray-700 flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white">Мои боты</h3>
            <div className="flex gap-2">
              <button onClick={() => setShowBotStore(true)} className="px-3 py-1 text-sm rounded bg-blue-600 hover:bg-blue-700 text-white">
                Bot Store
              </button>
              <button onClick={() => setShowBotsPanel(false)} className="px-3 py-1 text-sm rounded bg-gray-700 hover:bg-gray-600 text-white">
                Закрыть
              </button>
            </div>
          </div>
          <BotManager />
        </div>
      )}

      {/* Bot Store */}
      {showBotStore && (
        <div className="absolute inset-0 bg-gray-900 z-10 overflow-y-auto">
          <div className="p-4 border-b border-gray-700 flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white">Bot Store</h3>
            <button onClick={() => setShowBotStore(false)} className="px-3 py-1 text-sm rounded bg-gray-700 hover:bg-gray-600 text-white">
              Закрыть
            </button>
          </div>
          <BotStore />
        </div>
      )}

      {/* Web3 Panel */}
      {showWeb3Panel && (
        <div className="absolute inset-0 bg-gray-900 z-10 overflow-y-auto">
          <div className="p-4 border-b border-gray-700 flex items-center justify-between">
            <h3 className="text-lg font-semibold text-white">Web3 / Crypto</h3>
            <button onClick={() => setShowWeb3Panel(false)} className="px-3 py-1 text-sm rounded bg-gray-700 hover:bg-gray-600 text-white">
              Закрыть
            </button>
          </div>
          <Web3Panel />
        </div>
      )}

      {/* New Chat Modal */}
      {showNewChatModal && (
        <NewChatModal
          onClose={() => setShowNewChatModal(false)}
          onChatCreated={handleChatCreated}
        />
      )}
    </div>
  )
}
