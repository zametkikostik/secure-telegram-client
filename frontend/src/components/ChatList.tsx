import { useState, useRef } from 'react'
import { useTranslation } from 'react-i18next'
import { useChatStore } from '../services/chatStore'
import { formatDistanceToNow } from 'date-fns'
import { ru } from 'date-fns/locale'
import { FiMessageSquare, FiHash, FiVolume2, FiLock, FiUpload, FiSettings } from 'react-icons/fi'
import clsx from 'clsx'
import { TelegramImportModal } from './TelegramImportModal'
import { AdBanner } from './AdBanner'
import { AdSettingsModal } from './AdSettings'

interface ChatItem {
  id: string
  name: string
  type: 'private' | 'group' | 'channel'
  lastMessage?: string
  lastMessageTime?: Date
  unreadCount: number
  isMuted: boolean
  isOnline?: boolean
  avatar?: string
}

// Демо-данные для UI
const DEMO_CHATS: ChatItem[] = [
  {
    id: '1',
    name: 'Алиса Иванова',
    type: 'private',
    lastMessage: 'Привет! Как дела с проектом?',
    lastMessageTime: new Date(Date.now() - 5 * 60 * 1000),
    unreadCount: 2,
    isMuted: false,
    isOnline: true,
  },
  {
    id: '2',
    name: 'Разработчики',
    type: 'group',
    lastMessage: 'Боб: Запушил фикс в main',
    lastMessageTime: new Date(Date.now() - 15 * 60 * 1000),
    unreadCount: 5,
    isMuted: false,
  },
  {
    id: '3',
    name: 'Канал новостей',
    type: 'channel',
    lastMessage: 'Обновление безопасности v2.1',
    lastMessageTime: new Date(Date.now() - 60 * 60 * 1000),
    unreadCount: 0,
    isMuted: true,
  },
  {
    id: '4',
    name: 'Боб Петров',
    type: 'private',
    lastMessage: 'Ок, договорились!',
    lastMessageTime: new Date(Date.now() - 24 * 60 * 60 * 1000),
    unreadCount: 0,
    isMuted: false,
    isOnline: false,
  },
  {
    id: '5',
    name: 'Дизайн команда',
    type: 'group',
    lastMessage: 'Вы: Отправил макеты в Figma',
    lastMessageTime: new Date(Date.now() - 2 * 24 * 60 * 60 * 1000),
    unreadCount: 0,
    isMuted: false,
  },
]

const CHAT_TYPE_LABELS = {
  private: 'Личный чат',
  group: 'Группа',
  channel: 'Канал',
}

export function ChatList() {
  const { t, i18n } = useTranslation()
  const [chats] = useState<ChatItem[]>(DEMO_CHATS)
  const [searchQuery, setSearchQuery] = useState('')
  const [activeIndex, setActiveIndex] = useState(-1)
  const [isImportModalOpen, setIsImportModalOpen] = useState(false)
  const [isAdSettingsOpen, setIsAdSettingsOpen] = useState(false)
  const [bannerEnabled, setBannerEnabled] = useState(true)
  const { selectedChat, setSelectedChat } = useChatStore()
  const searchRef = useRef<HTMLInputElement>(null)
  const listRef = useRef<HTMLDivElement>(null)

  const filteredChats = chats.filter((chat) =>
    chat.name.toLowerCase().includes(searchQuery.toLowerCase())
  )

  const formatTime = (date?: Date) => {
    if (!date) return ''
    const now = new Date()
    const diff = now.getTime() - date.getTime()

    if (diff < 24 * 60 * 60 * 1000) {
      return date.toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit' })
    }

    const locale = i18n.language.startsWith('ru') ? ru : undefined
    return formatDistanceToNow(date, { locale, addSuffix: true })
  }

  const getChatIcon = (type: string) => {
    switch (type) {
      case 'group':
        return <FiHash className="w-4 h-4" aria-hidden="true" />
      case 'channel':
        return <FiVolume2 className="w-4 h-4" aria-hidden="true" />
      default:
        return <FiMessageSquare className="w-4 h-4" aria-hidden="true" />
    }
  }

  const getAvatarColor = (id: string) => {
    const colors = [
      'bg-blue-500',
      'bg-green-500',
      'bg-purple-500',
      'bg-pink-500',
      'bg-yellow-500',
      'bg-indigo-500',
    ]
    const index = parseInt(id) % colors.length
    return colors[index]
  }

  // Keyboard navigation
  const handleListKeyDown = (e: React.KeyboardEvent) => {
    switch (e.key) {
      case 'ArrowDown':
        e.preventDefault()
        setActiveIndex((prev) => (prev + 1) % filteredChats.length)
        break
      case 'ArrowUp':
        e.preventDefault()
        setActiveIndex((prev) => (prev - 1 + filteredChats.length) % filteredChats.length)
        break
      case 'Enter':
        e.preventDefault()
        if (activeIndex >= 0 && activeIndex < filteredChats.length) {
          setSelectedChat(filteredChats[activeIndex].id)
        }
        break
      case '/':
        e.preventDefault()
        searchRef.current?.focus()
        break
    }
  }

  const handleChatClick = (chatId: string, index: number) => {
    setSelectedChat(chatId)
    setActiveIndex(index)
  }

  // Count total unread
  const totalUnread = chats.reduce((sum, c) => sum + (c.isMuted ? 0 : c.unreadCount), 0)

  return (
    <div
      className="w-80 flex flex-col h-screen border-r"
      style={{
        backgroundColor: 'var(--color-bg-secondary)',
        borderColor: 'var(--color-border)',
      }}
      role="navigation"
      aria-label={t('common.chats')}
    >
      {/* Header */}
      <div
        className="p-4 border-b"
        style={{ borderColor: 'var(--color-border)' }}
      >
        <div className="flex items-center justify-between mb-4">
          <h2
            className="text-lg font-semibold"
            style={{ color: 'var(--color-text-primary)' }}
            id="chat-list-heading"
          >
            {t('common.chats')}
          </h2>
          <div
            className="flex items-center gap-2 text-xs"
            style={{ color: 'var(--color-text-muted)' }}
          >
            <FiLock className="w-3 h-3" aria-hidden="true" />
            <span>{t('encryption.p2p')}</span>
          </div>
        </div>

        {/* Search */}
        <div className="relative">
          <label htmlFor="chat-search" className="sr-only">
            {t('chat.search_chats')}
          </label>
          <input
            ref={searchRef}
            id="chat-search"
            type="text"
            placeholder={t('chat.search_chats')}
            value={searchQuery}
            onChange={(e) => {
              setSearchQuery(e.target.value)
              setActiveIndex(-1)
            }}
            className="w-full px-3 py-2 rounded-lg text-sm transition-colors focus:outline-none focus:ring-2"
            style={{
              backgroundColor: 'var(--color-bg-tertiary)',
              color: 'var(--color-text-primary)',
              border: '1px solid var(--color-border)',
            }}
            aria-describedby="search-hint"
          />
          <span id="search-hint" className="sr-only">
            {t('a11y.press_slash', 'Нажмите / для фокуса на поиск')}
          </span>
        </div>
      </div>

      {/* Chat List */}
      <div
        ref={listRef}
        className="flex-1 overflow-y-auto"
        role="listbox"
        aria-labelledby="chat-list-heading"
        aria-multiselectable="false"
        onKeyDown={handleListKeyDown}
        tabIndex={0}
      >
        {filteredChats.length === 0 ? (
          <div
            className="p-8 text-center"
            style={{ color: 'var(--color-text-muted)' }}
            role="status"
          >
            <FiMessageSquare
              className="w-12 h-12 mx-auto mb-3 opacity-50"
              aria-hidden="true"
            />
            <p className="text-sm">{t('chat.no_chats')}</p>
            <p className="text-xs mt-1">{t('chat.no_chats_hint')}</p>
          </div>
        ) : (
          filteredChats.map((chat, index) => {
            const isSelected = selectedChat === chat.id
            const isActive = index === activeIndex

            return (
              <button
                key={chat.id}
                role="option"
                aria-selected={isSelected}
                aria-label={`${chat.name}${chat.isOnline ? `, ${t('status.online')}` : `, ${t('status.offline')}`}${chat.unreadCount > 0 ? `, ${chat.unreadCount} непрочитанных` : ''}`}
                onClick={() => handleChatClick(chat.id, index)}
                onMouseEnter={() => setActiveIndex(index)}
                onMouseLeave={() => setActiveIndex(-1)}
                className={clsx(
                  'w-full p-4 flex items-start gap-3 transition-colors border-b text-start',
                  isSelected && 'border-l-4',
                  isActive && !isSelected && 'opacity-80'
                )}
                style={{
                  backgroundColor: isSelected
                    ? 'var(--color-bg-tertiary)'
                    : isActive
                    ? 'var(--color-bg-tertiary)'
                    : 'transparent',
                  borderColor: isSelected
                    ? 'var(--color-accent)'
                    : 'var(--color-border)',
                  borderLeftColor: isSelected ? 'var(--color-accent)' : undefined,
                }}
              >
                {/* Avatar */}
                <div className="relative flex-shrink-0">
                  <div
                    className={clsx(
                      'w-12 h-12 rounded-full flex items-center justify-center text-white font-semibold',
                      getAvatarColor(chat.id)
                    )}
                    aria-hidden="true"
                  >
                    {chat.avatar ? (
                      <img
                        src={chat.avatar}
                        alt=""
                        className="w-full h-full rounded-full object-cover"
                        aria-hidden="true"
                      />
                    ) : (
                      chat.name.charAt(0).toUpperCase()
                    )}
                  </div>

                  {/* Online indicator */}
                  {chat.type === 'private' && chat.isOnline && (
                    <div
                      className="absolute bottom-0 right-0 w-3 h-3 bg-green-500 rounded-full border-2"
                      style={{ borderColor: 'var(--color-bg-secondary)' }}
                      aria-label={t('status.online')}
                      title={t('status.online')}
                    />
                  )}
                </div>

                {/* Content */}
                <div className="flex-1 min-w-0">
                  <div className="flex items-center justify-between mb-1">
                    <div className="flex items-center gap-2 min-w-0 flex-1">
                      <span
                        className="font-medium truncate"
                        style={{ color: 'var(--color-text-primary)' }}
                      >
                        {chat.name}
                      </span>
                      {getChatIcon(chat.type)}
                      <span className="sr-only">
                        {CHAT_TYPE_LABELS[chat.type]}
                      </span>
                    </div>
                    <span
                      className="text-xs flex-shrink-0"
                      style={{ color: 'var(--color-text-muted)' }}
                    >
                      {formatTime(chat.lastMessageTime)}
                    </span>
                  </div>

                  <div className="flex items-center justify-between">
                    <p
                      className="text-sm truncate"
                      style={{ color: 'var(--color-text-secondary)' }}
                    >
                      {chat.lastMessage || t('message.no_messages')}
                    </p>

                    {chat.unreadCount > 0 && (
                      <span
                        className={clsx(
                          'ml-2 px-2 py-0.5 text-xs font-semibold rounded-full flex-shrink-0',
                          chat.isMuted ? 'text-gray-400' : 'text-white'
                        )}
                        style={{
                          backgroundColor: chat.isMuted
                            ? 'var(--color-bg-tertiary)'
                            : 'var(--color-accent)',
                        }}
                        aria-label={`${chat.unreadCount} непрочитанных`}
                      >
                        {chat.unreadCount}
                      </span>
                    )}
                  </div>
                </div>
              </button>
            )
          })
        )}
      </div>

      {/* Footer */}
      <div
        className="border-t"
        style={{
          backgroundColor: 'var(--color-bg-secondary)',
          borderColor: 'var(--color-border)',
        }}
      >
        {/* Ad Banner */}
        <AdBanner
          enabled={bannerEnabled}
          onHidden={() => setBannerEnabled(false)}
        />

        <div className="p-3">
          <div className="space-y-2">
            {/* Import from Telegram button */}
            <button
              onClick={() => setIsImportModalOpen(true)}
              className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors"
              style={{
                backgroundColor: 'var(--color-bg-tertiary)',
                color: 'var(--color-accent)',
                border: `1px solid var(--color-border)`,
              }}
              aria-label={t('import.telegram.title')}
            >
              <FiUpload className="w-4 h-4" aria-hidden="true" />
              {t('import.telegram.button_label')}
            </button>

            {/* Ad Settings button */}
            <button
              onClick={() => setIsAdSettingsOpen(true)}
              className="w-full flex items-center justify-center gap-2 px-3 py-2 rounded-lg text-sm font-medium transition-colors"
              style={{
                backgroundColor: 'var(--color-bg-tertiary)',
                color: 'var(--color-text-primary)',
                border: `1px solid var(--color-border)`,
              }}
              aria-label="Настройки рекламы"
            >
              <FiSettings className="w-4 h-4" aria-hidden="true" />
              Настройки рекламы
            </button>

            <div
              className="flex items-center justify-between text-xs"
              style={{ color: 'var(--color-text-muted)' }}
            >
              <span>
                {chats.length} {t('common.chats').toLowerCase()}
              </span>
              {totalUnread > 0 && (
                <span aria-live="polite" aria-atomic="true">
                  {totalUnread} {t('common.messages').toLowerCase()}
                </span>
              )}
              <span className="flex items-center gap-1">
                <FiLock className="w-3 h-3" aria-hidden="true" />
                {t('app.tagline')}
              </span>
            </div>
          </div>
        </div>
      </div>

      {/* Telegram Import Modal */}
      <TelegramImportModal
        isOpen={isImportModalOpen}
        onClose={() => setIsImportModalOpen(false)}
        onImportComplete={(result) => {
          console.log('[ChatList] Import completed:', result)
          // TODO: Обновить список чатов на основе результата импорта
        }}
      />

      {/* Ad Settings Modal */}
      <AdSettingsModal
        isOpen={isAdSettingsOpen}
        onClose={() => setIsAdSettingsOpen(false)}
      />
    </div>
  )
}
