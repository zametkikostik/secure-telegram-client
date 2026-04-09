import { useState, useRef, useEffect } from 'react'
import { useParams } from 'react-router-dom'
import { useTranslation } from 'react-i18next'
import { useChatStore } from '../services/chatStore'
import { FiMoreVertical, FiLock } from 'react-icons/fi'
import { MessageInput } from './MessageInput'
import {
  EncryptionBadge,
  RouteIndicator,
  DeliveryStatusIndicator,
  ConnectionStatusBar,
  EncryptionNotice,
  type DeliveryStatus,
  type TransportRoute,
} from './EncryptionBadge'
import clsx from 'clsx'

type MessageStatus = DeliveryStatus
type MessageRoute = TransportRoute

interface Message {
  id: string
  senderId: string
  content: string
  timestamp: Date
  status: MessageStatus
  isEncrypted: boolean
  route?: MessageRoute
  encryptionMethod?: string
}

// Демо-сообщения
const DEMO_MESSAGES: Message[] = [
  {
    id: '1',
    senderId: 'peer',
    content: 'Привет! Как продвигается работа над мессенджером?',
    timestamp: new Date(Date.now() - 30 * 60 * 1000),
    status: 'read',
    isEncrypted: true,
    route: 'p2p',
    encryptionMethod: 'X25519+Kyber1024',
  },
  {
    id: '2',
    senderId: 'me',
    content: 'Привет! Только что закончил P2P модуль с libp2p. Теперь работает через Noise + Yamux.',
    timestamp: new Date(Date.now() - 25 * 60 * 1000),
    status: 'read',
    isEncrypted: true,
    route: 'p2p',
    encryptionMethod: 'X25519+Kyber1024',
  },
  {
    id: '3',
    senderId: 'peer',
    content: 'Отлично! А как насчёт постквантового шифрования?',
    timestamp: new Date(Date.now() - 20 * 60 * 1000),
    status: 'read',
    isEncrypted: true,
    route: 'p2p',
    encryptionMethod: 'X25519+Kyber1024',
  },
  {
    id: '4',
    senderId: 'me',
    content: 'Использую X25519 + Kyber1024 для гибридного E2EE. Даже если квантовые компьютеры станут реальностью, наши сообщения защищены.',
    timestamp: new Date(Date.now() - 15 * 60 * 1000),
    status: 'delivered',
    isEncrypted: true,
    route: 'cloudflare',
    encryptionMethod: 'X25519+Kyber1024',
  },
  {
    id: '5',
    senderId: 'peer',
    content: 'Звучит впечатляюще! 🔐',
    timestamp: new Date(Date.now() - 10 * 60 * 1000),
    status: 'read',
    isEncrypted: true,
    route: 'p2p',
    encryptionMethod: 'X25519+Kyber1024',
  },
  {
    id: '6',
    senderId: 'me',
    content: 'Да, и ещё добавил Cloudflare fallback на случай если P2P недоступен. Сообщения шифруются перед отправкой в любом случае.',
    timestamp: new Date(Date.now() - 5 * 60 * 1000),
    status: 'sent',
    isEncrypted: true,
    route: 'cloudflare',
    encryptionMethod: 'X25519+Kyber1024',
  },
]

const STATUS_LABELS: Record<MessageStatus, string> = {
  sending: 'Отправка...',
  sent: 'Отправлено',
  delivered: 'Доставлено',
  read: 'Прочитано',
  failed: 'Не доставлено',
}

export function ChatWindow() {
  const { t } = useTranslation()
  const { id } = useParams<{ id: string }>()
  const [messages, setMessages] = useState<Message[]>(DEMO_MESSAGES)
  const [isTyping, setIsTyping] = useState(false)
  const messagesEndRef = useRef<HTMLDivElement>(null)
  const inputRef = useRef<HTMLDivElement>(null)
  const { selectedChat } = useChatStore()

  // Демо-чат
  const chat = {
    id: id || '1',
    name: 'Алиса Иванова',
    isOnline: true,
    avatar: null,
  }

  // Auto-scroll to bottom
  useEffect(() => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' })
  }, [messages])

  const formatTime = (date: Date) => {
    return date.toLocaleTimeString('ru-RU', { hour: '2-digit', minute: '2-digit' })
  }

  const getAvatarColor = () => {
    return 'bg-blue-500'
  }

  const handleSend = (text: string) => {
    const newMessage: Message = {
      id: Date.now().toString(),
      senderId: 'me',
      content: text,
      timestamp: new Date(),
      status: 'sending',
      isEncrypted: true,
      route: 'unknown',
      encryptionMethod: 'X25519+Kyber1024',
    }

    setMessages((prev) => [...prev, newMessage])

    // Имитация отправки
    setTimeout(() => {
      setMessages((prev) =>
        prev.map((msg) =>
          msg.id === newMessage.id ? { ...msg, status: 'sent' } : msg
        )
      )
    }, 500)

    // Имитация доставки
    setTimeout(() => {
      setMessages((prev) =>
        prev.map((msg) =>
          msg.id === newMessage.id ? { ...msg, status: 'delivered' } : msg
        )
      )
    }, 1500)

    // Имитация ответа
    setTimeout(() => {
      setIsTyping(true)

      setTimeout(() => {
        setIsTyping(false)

        const replyMessage: Message = {
          id: (Date.now() + 1).toString(),
          senderId: 'peer',
          content: 'Понял! Это действительно надёжная защита. 👍',
          timestamp: new Date(),
          status: 'read',
          isEncrypted: true,
          route: 'p2p',
          encryptionMethod: 'X25519+Kyber1024',
        }

        setMessages((prev) => [...prev, replyMessage])
      }, 2000)
    }, 2000)
  }

  if (!selectedChat && !id) {
    return (
      <div
        className="flex-1 flex items-center justify-center"
        style={{ backgroundColor: 'var(--color-bg-primary)' }}
      >
        <div className="text-center">
          <FiLock
            className="w-16 h-16 mx-auto mb-4"
            style={{ color: 'var(--color-text-muted)' }}
            aria-hidden="true"
          />
          <h3
            className="text-xl font-semibold mb-2"
            style={{ color: 'var(--color-text-primary)' }}
          >
            {t('chat.no_chats')}
          </h3>
          <p style={{ color: 'var(--color-text-secondary)' }}>
            {t('chat.no_chats_hint')}
          </p>
        </div>
      </div>
    )
  }

  const statusText = chat.isOnline ? t('status.online') : t('status.offline')

  return (
    <div
      className="flex-1 flex flex-col h-screen chat-window"
      style={{ backgroundColor: 'var(--color-bg-primary)' }}
      role="main"
      aria-label={`${t('common.chats')}: ${chat.name}`}
    >
      {/* Header */}
      <header
        className="px-6 py-4 border-b chat-header"
        style={{
          backgroundColor: 'var(--color-bg-secondary)',
          borderColor: 'var(--color-border)',
        }}
      >
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            {/* Avatar */}
            <div className="relative">
              <div
                className={clsx(
                  'w-10 h-10 rounded-full flex items-center justify-center text-white font-semibold',
                  getAvatarColor()
                )}
                aria-hidden="true"
              >
                {chat.name.charAt(0)}
              </div>
              {chat.isOnline && (
                <div
                  className="absolute bottom-0 right-0 w-3 h-3 bg-green-500 rounded-full border-2"
                  style={{ borderColor: 'var(--color-bg-secondary)' }}
                  aria-label={statusText}
                  title={statusText}
                />
              )}
            </div>

            {/* Info */}
            <div>
              <h3
                className="font-semibold"
                style={{ color: 'var(--color-text-primary)' }}
                id="chat-header-title"
              >
                {chat.name}
              </h3>
              <p
                className="text-xs"
                style={{
                  color: chat.isOnline
                    ? 'var(--color-success)'
                    : 'var(--color-text-muted)',
                }}
                aria-live="polite"
              >
                {statusText}
              </p>
            </div>
          </div>

          {/* Actions */}
          <div className="flex items-center gap-2">
            <ConnectionStatusBar route="p2p" encryptionStatus="encrypted" />
            <button
              className="p-2 rounded-lg transition-colors"
              style={{ color: 'var(--color-text-secondary)' }}
              onMouseEnter={(e) => {
                e.currentTarget.style.backgroundColor = 'var(--color-bg-tertiary)'
              }}
              onMouseLeave={(e) => {
                e.currentTarget.style.backgroundColor = 'transparent'
              }}
              aria-label={t('common.settings')}
              title={t('common.settings')}
            >
              <FiMoreVertical className="w-5 h-5" aria-hidden="true" />
            </button>
          </div>
        </div>
      </header>

      {/* Messages */}
      <div
        className="flex-1 overflow-y-auto px-6 py-4 space-y-4 chat-messages"
        role="log"
        aria-label={t('common.messages')}
        aria-live="polite"
        aria-relevant="additions"
      >
        {/* Encryption notice */}
        <EncryptionNotice method="X25519+Kyber1024" />

        {messages.map((message, index) => {
          const isMe = message.senderId === 'me'
          const showAvatar =
            index === messages.length - 1 ||
            messages[index + 1]?.senderId !== message.senderId

          return (
            <div
              key={message.id}
              className={clsx('flex', isMe ? 'justify-end' : 'justify-start')}
            >
              <div
                className={clsx(
                  'max-w-[70%] flex gap-2',
                  isMe ? 'flex-row-reverse' : 'flex-row'
                )}
              >
                {/* Avatar */}
                {showAvatar && !isMe && (
                  <div
                    className="w-8 h-8 rounded-full bg-purple-500 flex items-center justify-center text-white text-sm font-semibold flex-shrink-0"
                    aria-hidden="true"
                  >
                    {chat.name.charAt(0)}
                  </div>
                )}
                {!showAvatar && !isMe && <div className="w-8 flex-shrink-0" />}

                {/* Message bubble */}
                <div
                  className={clsx(
                    'px-4 py-2 rounded-2xl message-bubble',
                    isMe
                      ? 'rounded-br-md'
                      : 'rounded-bl-md'
                  )}
                  style={{
                    backgroundColor: isMe
                      ? 'var(--color-accent)'
                      : 'var(--color-bg-tertiary)',
                    color: isMe
                      ? 'white'
                      : 'var(--color-text-primary)',
                    borderColor: 'var(--color-border)',
                  }}
                  role="article"
                  aria-label={`${isMe ? t('message.sent') : chat.name}: ${message.content.substring(0, 50)}`}
                >
                  <p className="text-sm whitespace-pre-wrap break-words">
                    {message.content}
                  </p>

                  {/* Footer with indicators */}
                  <div
                    className={clsx(
                      'flex items-center justify-end gap-2 mt-1.5 flex-wrap',
                      isMe ? 'text-blue-200' : ''
                    )}
                    style={{
                      color: isMe
                        ? undefined
                        : 'var(--color-text-muted)',
                    }}
                  >
                    {/* Time */}
                    <time
                      className="text-xs"
                      dateTime={message.timestamp.toISOString()}
                    >
                      {formatTime(message.timestamp)}
                    </time>

                    {/* Encryption badge */}
                    <EncryptionBadge
                      status={message.isEncrypted ? 'encrypted' : 'error'}
                      method={message.encryptionMethod}
                      showLabel={false}
                    />

                    {/* Route indicator */}
                    {isMe && (
                      <RouteIndicator
                        route={message.route || 'unknown'}
                        showLabel={false}
                      />
                    )}

                    {/* Status icon */}
                    {isMe && (
                      <span
                        className="sr-only"
                        aria-label={STATUS_LABELS[message.status]}
                      >
                        {STATUS_LABELS[message.status]}
                      </span>
                    )}
                    {isMe && <DeliveryStatusIndicator status={message.status} />}
                  </div>
                </div>
              </div>
            </div>
          )
        })}

        {/* Typing indicator */}
        {isTyping && (
          <div
            className="flex justify-start"
            role="status"
            aria-live="polite"
            aria-label={t('chat.typing')}
          >
            <div
              className="px-4 py-3 rounded-2xl"
              style={{
                backgroundColor: 'var(--color-bg-tertiary)',
                borderRadius: '1rem 1rem 0.25rem 1rem',
              }}
            >
              <div className="flex gap-1" aria-hidden="true">
                <div
                  className="w-2 h-2 bg-gray-500 rounded-full animate-bounce"
                  style={{
                    animationDelay: '0ms',
                    backgroundColor: 'var(--color-text-muted)',
                  }}
                />
                <div
                  className="w-2 h-2 bg-gray-500 rounded-full animate-bounce"
                  style={{
                    animationDelay: '150ms',
                    backgroundColor: 'var(--color-text-muted)',
                  }}
                />
                <div
                  className="w-2 h-2 bg-gray-500 rounded-full animate-bounce"
                  style={{
                    animationDelay: '300ms',
                    backgroundColor: 'var(--color-text-muted)',
                  }}
                />
              </div>
              <span className="sr-only">{t('chat.typing')}</span>
            </div>
          </div>
        )}

        <div ref={messagesEndRef} />
      </div>

      {/* Input */}
      <div
        className="px-4 py-3 border-t chat-input"
        style={{
          backgroundColor: 'var(--color-bg-secondary)',
          borderColor: 'var(--color-border)',
        }}
      >
        <MessageInput onSend={handleSend} ref={inputRef} />
      </div>
    </div>
  )
}
