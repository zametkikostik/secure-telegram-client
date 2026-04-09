import { useState, useEffect, useRef } from 'react'
import { FiSend, FiArrowLeft, FiSmile, FiLock, FiUnlock, FiPaperclip, FiX, FiImage, FiFile, FiCornerUpLeft, FiSearch } from 'react-icons/fi'
import { api, type Message, type Chat } from '../services/apiClient'
import { getWebSocketClient, WebSocketClient, NotificationPayload } from '../services/webSocketClient'
import { e2ee } from '../crypto/messageCrypto'
import { CallButton } from './CallButton'
import { ReactionBar } from './ReactionBar'
import { StickerPicker } from './StickerPicker'
import { MessageContextMenu } from './MessageContextMenu'
import { GifPicker } from './GifPicker'
import { useStickerStore } from '../services/stickerStore'
import { useTranslation } from 'react-i18next'

interface ChatWindowProps {
  chatId: string
  onBack: () => void
}

export function ChatWindow({ chatId, onBack }: ChatWindowProps) {
  const { t } = useTranslation()
  const { isPickerOpen, closePicker } = useStickerStore()
  const [messages, setMessages] = useState<Message[]>([])
  const [content, setContent] = useState('')
  const [loading, setLoading] = useState(true)
  const [sending, setSending] = useState(false)
  const [chat, setChat] = useState<Chat | null>(null)
  const [error, setError] = useState('')
  const [e2eeReady, setE2eeReady] = useState(false)
  const [attachedFiles, setAttachedFiles] = useState<Array<{ name: string; size: number; type: string; preview?: string }>>([])
  const [replyTo, setReplyTo] = useState<Message | null>(null)
  const [typingUsers, setTypingUsers] = useState<Set<string>>(new Set())
  const [selfDestruct, setSelfDestruct] = useState<number | null>(null) // seconds
  const [scheduledTime, setScheduledTime] = useState<string | null>(null) // ISO timestamp
  const [contextMenu, setContextMenu] = useState<{ messageId: string; x: number; y: number } | null>(null)
  const [editingMessage, setEditingMessage] = useState<{ id: string; content: string } | null>(null)
  const [wallpaper, setWallpaper] = useState<{ color: string; pattern: string }>({ color: '#0f172a', pattern: 'solid' })
  const [pinnedMessages, setPinnedMessages] = useState<Message[]>([])
  const [showGifPicker, setShowGifPicker] = useState(false)
  const [searchQuery, setSearchQuery] = useState('')
  const [showSearch, setShowSearch] = useState(false)
  const fileInputRef = useRef<HTMLInputElement>(null)
  const bottomRef = useRef<HTMLDivElement>(null)
  const wsClientRef = useRef<WebSocketClient | null>(null)

  // Initialize E2EE
  useEffect(() => {
    e2ee.initialize()
      .then(() => setE2eeReady(true))
      .catch(err => console.warn('E2EE init failed:', err))
  }, [])

  // Request notification permission
  useEffect(() => {
    if ('Notification' in window && Notification.permission === 'default') {
      Notification.requestPermission()
    }
  }, [])

  // Play notification sound
  const playNotificationSound = () => {
    try {
      const ctx = new AudioContext()
      const osc = ctx.createOscillator()
      const gain = ctx.createGain()
      osc.connect(gain)
      gain.connect(ctx.destination)
      osc.frequency.value = 880
      osc.type = 'sine'
      gain.gain.setValueAtTime(0.1, ctx.currentTime)
      gain.gain.exponentialRampToValueAtTime(0.001, ctx.currentTime + 0.3)
      osc.start(ctx.currentTime)
      osc.stop(ctx.currentTime + 0.3)
    } catch {}
  }

  // WebSocket connection for real-time messages
  useEffect(() => {
    const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000'
    const token = localStorage.getItem('auth_token')
    
    if (!token) return

    // Decode JWT to get user_id
    const payload = JSON.parse(atob(token.split('.')[1]))
    const userId = payload.sub?.replace('user:', '')

    if (!userId) return

    const wsClient = getWebSocketClient({
      baseUrl: API_BASE,
      userId,
      autoConnect: true,
      channels: ['messages'],
    })

    wsClientRef.current = wsClient

    // Handle incoming messages
    wsClient.onMessage('messages', (payload: NotificationPayload) => {
      const msgPayload = payload as any
      if (msgPayload.chat_id === chatId) {
        let displayContent = msgPayload.encrypted_content || 'Encrypted message'
        let isEncrypted = msgPayload.msg_type === 'e2ee'

        // Try to decrypt if E2EE
        if (isEncrypted) {
          try {
            const encrypted = JSON.parse(msgPayload.encrypted_content)
            if (encrypted.v === 1 && encrypted.k && encrypted.c && encrypted.n) {
              // Import AES key and decrypt
              const keyData = Uint8Array.from(atob(encrypted.k), c => c.charCodeAt(0))
              const aesKey = crypto.subtle.importKey('raw', keyData, 'AES-GCM', false, ['decrypt'])

              const ciphertext = Uint8Array.from(atob(encrypted.c), c => c.charCodeAt(0))
              const nonce = Uint8Array.from(atob(encrypted.n), c => c.charCodeAt(0))

              aesKey.then(key => {
                return crypto.subtle.decrypt({ name: 'AES-GCM', iv: nonce }, key, ciphertext)
              }).then(decrypted => {
                displayContent = new TextDecoder().decode(decrypted)
                // Update message in state
                setMessages(prev => prev.map(m =>
                  m.id === msgPayload.message_id ? { ...m, content: displayContent } : m
                ))
              }).catch(() => {
                displayContent = '🔒 [Unable to decrypt]'
              })
            }
          } catch {
            displayContent = '🔒 [Encrypted]'
          }
        }

        const newMsg: Message = {
          id: msgPayload.message_id,
          chat_id: msgPayload.chat_id,
          sender_id: msgPayload.sender_id,
          sender_name: msgPayload.sender_name,
          content: displayContent,
          msg_type: msgPayload.msg_type || 'text',
          created_at: msgPayload.timestamp,
          edited_at: null,
          reply_to: null,
        }
        setMessages(prev => [...prev, newMsg])

        // Play notification sound
        playNotificationSound()

        // Browser notification when window not focused
        if (document.visibilityState === 'hidden' && Notification.permission === 'granted') {
          new Notification(`Новое сообщение от ${msgPayload.sender_name || 'Contact'}`, {
            body: msgPayload.msg_type === 'e2ee' ? '🔒 Зашифрованное сообщение' : (msgPayload.content || '').slice(0, 100),
            icon: '/icon.png',
            tag: msgPayload.message_id,
          })
        }
      }
    })

    // Handle message edits
    wsClient.onMessage('message_edited', (payload: any) => {
      if (payload.chat_id === chatId) {
        setMessages(prev => prev.map(m =>
          m.id === payload.id ? { ...m, content: payload.content, edited_at: payload.edited_at } : m
        ))
      }
    })

    // Handle message deletes
    wsClient.onMessage('message_deleted', (payload: any) => {
      if (payload.chat_id === chatId) {
        setMessages(prev => prev.filter(m => m.id !== payload.id))
      }
    })

    return () => {
      wsClient.disconnect()
    }
  }, [chatId])

  useEffect(() => {
    let cancelled = false
    const load = async () => {
      try {
        const [msgs, c] = await Promise.all([api.listMessages(chatId), api.getChat(chatId)])
        if (!cancelled) { setMessages(msgs); setChat(c); }
      } catch (e: any) { if (!cancelled) setError(e.message) }
      finally { if (!cancelled) setLoading(false) }
    }
    load()
    return () => { cancelled = true }
  }, [chatId])

  useEffect(() => { bottomRef.current?.scrollIntoView({ behavior: 'smooth' }) }, [messages, attachedFiles])

  // Load wallpaper
  useEffect(() => {
    api.getWallpaper(chatId).then(wp => {
      setWallpaper({ color: wp.color, pattern: wp.pattern })
    }).catch(() => {})
  }, [chatId])

  // Load pinned messages
  useEffect(() => {
    api.getPinnedMessages(chatId).then(msgs => {
      setPinnedMessages(msgs)
    }).catch(() => {})
  }, [chatId])

  // File attachment handlers
  const handleFileSelect = (e: React.ChangeEvent<HTMLInputElement>) => {
    const files = Array.from(e.target.files || [])
    const newFiles = files.map(file => ({
      name: file.name,
      size: file.size,
      type: file.type,
      preview: file.type.startsWith('image/') ? URL.createObjectURL(file) : undefined,
    }))
    setAttachedFiles(prev => [...prev, ...newFiles])
  }

  const removeFile = (index: number) => {
    setAttachedFiles(prev => prev.filter((_, i) => i !== index))
  }

  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`
  }

  const handleSend = async () => {
    if ((!content.trim() && attachedFiles.length === 0) || sending) return
    setSending(true)
    try {
      // Stop typing indicator
      const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000'
      const token = localStorage.getItem('auth_token')
      if (token && chatId) {
        fetch(`${API_BASE}/api/v1/chats/${chatId}/typing`, {
          method: 'POST',
          headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
          body: JSON.stringify({ is_typing: false }),
        }).catch(() => {})
      }

      // Upload attached files
      for (const file of attachedFiles) {
        try {
          const fileObj = new File([], file.name, { type: file.type })
          const uploaded = await api.uploadFile(fileObj, chatId)
          await api.sendMessage(chatId, uploaded.url, 'file', replyTo?.id, selfDestruct || undefined, scheduledTime || undefined)
        } catch (e) {
          console.warn('File upload failed:', e)
        }
      }

      let msgContent = content.trim()
      let isEncrypted = false

      if (e2eeReady && msgContent) {
        try {
          const aesKey = await crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, true, ['encrypt', 'decrypt'])
          const nonce = crypto.getRandomValues(new Uint8Array(12))
          const plaintextBuffer = new TextEncoder().encode(msgContent)
          const ciphertext = await crypto.subtle.encrypt({ name: 'AES-GCM', iv: nonce }, aesKey, plaintextBuffer)
          const rawKey = await crypto.subtle.exportKey('raw', aesKey)
          msgContent = JSON.stringify({ v: 1, k: btoa(String.fromCharCode(...new Uint8Array(rawKey))), c: btoa(String.fromCharCode(...new Uint8Array(ciphertext))), n: btoa(String.fromCharCode(...nonce)) })
          isEncrypted = true
        } catch (err) { console.warn('E2EE failed:', err) }
      }

      const msg = await api.sendMessage(chatId, msgContent, isEncrypted ? 'e2ee' : 'text', replyTo?.id, selfDestruct || undefined, scheduledTime || undefined)
      setMessages(prev => [...prev, { ...msg }])
      setContent('')
      setAttachedFiles([])
      setReplyTo(null)
      setScheduledTime(null)
    } catch (e: any) { setError(e.message) }
    finally { setSending(false) }
  }

  // Typing indicator on input change
  const handleInputChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    setContent(e.target.value)
    const API_BASE = import.meta.env.VITE_API_URL || 'http://localhost:3000'
    const token = localStorage.getItem('auth_token')
    if (token && chatId && e.target.value.length > 0) {
      fetch(`${API_BASE}/api/v1/chats/${chatId}/typing`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', 'Authorization': `Bearer ${token}` },
        body: JSON.stringify({ is_typing: true }),
      }).catch(() => {})
    }
  }

  const fmtTime = (d?: string) => {
    if (!d) return ''
    try { return new Date(d).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' }) } catch { return '' }
  }

  // Edit/Delete handlers
  const handleEditMessage = (messageId: string) => {
    const msg = messages.find(m => m.id === messageId)
    if (msg) {
      setEditingMessage({ id: messageId, content: msg.content })
      setContextMenu(null)
    }
  }

  const handleDeleteMessage = async (messageId: string) => {
    try {
      await api.deleteMessage(messageId)
      setMessages(prev => prev.filter(m => m.id !== messageId))
      setContextMenu(null)
    } catch (e: any) {
      console.error('Failed to delete message:', e)
    }
  }

  const handleReplyFromMenu = (messageId: string) => {
    const msg = messages.find(m => m.id === messageId)
    if (msg) {
      setReplyTo(msg)
      setContextMenu(null)
    }
  }

  const handleSaveEdit = async () => {
    if (!editingMessage) return
    try {
      await api.editMessage(editingMessage.id, editingMessage.content)
      setMessages(prev => prev.map(m =>
        m.id === editingMessage.id ? { ...m, content: editingMessage.content } : m
      ))
      setEditingMessage(null)
    } catch (e: any) {
      console.error('Failed to edit message:', e)
    }
  }

  const handleSendGif = async (gifUrl: string) => {
    try {
      const sentMsg = await api.sendMessage(chatId, gifUrl, 'gif')
      setMessages(prev => [...prev, sentMsg])
    } catch (e) { console.error('Failed to send GIF:', e) }
  }

  const handleContextMenu = (e: React.MouseEvent, messageId: string) => {
    e.preventDefault()
    setContextMenu({ messageId, x: e.clientX, y: e.clientY })
  }

  const handlePinMessage = async (messageId: string) => {
    try {
      const isPinned = pinnedMessages.some(m => m.id === messageId)
      if (isPinned) {
        await api.unpinMessage(messageId)
        setPinnedMessages(prev => prev.filter(m => m.id !== messageId))
      } else {
        await api.pinMessage(messageId)
        const msg = messages.find(m => m.id === messageId)
        if (msg) setPinnedMessages(prev => [msg, ...prev])
      }
      setContextMenu(null)
    } catch (e) { console.error('Pin failed:', e) }
  }

  return (
    <div className="flex-1 flex flex-col h-screen" style={{ backgroundColor: wallpaper.color }}>
      {/* Header */}
      <div className="p-4 border-b flex items-center justify-between gap-3" style={{ backgroundColor: 'var(--color-bg-secondary, #1e293b)', borderColor: 'var(--color-border, #334155)' }}>
        <div className="flex items-center gap-3">
          <button onClick={onBack} className="p-2 rounded hover:bg-gray-700 text-gray-400"><FiArrowLeft /></button>
          <div>
            <div className="flex items-center gap-2">
              <h3 className="font-semibold" style={{ color: 'var(--color-text-primary, #f1f5f9)' }}>{chat?.name || 'Loading...'}</h3>
              {e2eeReady ? (
                <FiLock className="w-3 h-3 text-green-400" title="E2EE активировано" />
              ) : (
                <FiUnlock className="w-3 h-3 text-gray-500" title="E2EE не активировано" />
              )}
            </div>
            <p className="text-xs text-gray-500">{chat?.chat_type} • {chat?.participant_count || 1} participants</p>
            {typingUsers.size > 0 && (
              <p className="text-xs text-blue-400 animate-pulse">
                {Array.from(typingUsers).join(', ')} печатает...
              </p>
            )}
          </div>
        </div>
        {/* Call Buttons */}
        {chat && chat.chat_type === 'private' && (
          <CallButton
            contactId={chat.id}
            contactName={chat.name || 'Contact'}
          />
        )}
        {/* Search Button */}
        <button onClick={() => setShowSearch(!showSearch)} className="p-2 rounded hover:bg-gray-700 text-gray-400 transition-colors" title="Поиск">
          <FiSearch className="w-5 h-5" />
        </button>
      </div>

      {/* Pinned Messages Banner */}
      {pinnedMessages.length > 0 && (
        <div className="px-4 py-2 bg-yellow-900/30 border-b border-yellow-800 flex items-center gap-2">
          <span className="text-yellow-400">📌</span>
          <div className="flex-1 min-w-0">
            {pinnedMessages.map(pm => (
              <p key={pm.id} className="text-sm text-yellow-200 truncate">{pm.content}</p>
            ))}
          </div>
        </div>
      )}

      {/* Search Bar */}
      {showSearch && (
        <div className="px-4 py-2 bg-gray-800 border-b border-gray-700 flex items-center gap-2">
          <FiSearch className="text-gray-400" />
          <input
            type="text"
            value={searchQuery}
            onChange={e => setSearchQuery(e.target.value)}
            placeholder="Поиск по сообщениям..."
            className="flex-1 bg-gray-700 text-white rounded px-3 py-1 text-sm focus:outline-none focus:ring-1 focus:ring-blue-500"
            autoFocus
          />
          <button onClick={() => { setShowSearch(false); setSearchQuery('') }} className="text-gray-400 hover:text-white">
            <FiX className="w-4 h-4" />
          </button>
          {searchQuery && (
            <span className="text-xs text-gray-500">
              {messages.filter(m => m.content.toLowerCase().includes(searchQuery.toLowerCase())).length} найдено
            </span>
          )}
        </div>
      )}

      {/* Messages */}
      <div className="flex-1 overflow-y-auto p-4 space-y-3 chat-scrollbar relative" style={{
        backgroundImage: 'radial-gradient(rgba(255, 255, 255, 0.03) 1px, transparent 1px)',
        backgroundSize: '24px 24px',
      }}>
        {loading && <div className="text-center text-gray-500 py-8">Loading messages...</div>}
        {error && <div className="text-center text-red-400 py-4 text-sm">{error}</div>}
        {messages
          .filter(m => !searchQuery || m.content.toLowerCase().includes(searchQuery.toLowerCase()))
          .map(msg => {
          const isOwn = msg.sender_id === (localStorage.getItem('user_id') || '')
          // Find the message this is replying to
          const repliedMsg = msg.reply_to ? messages.find(m => m.id === msg.reply_to) : null

          return (
            <div key={msg.id} className={`flex flex-col ${isOwn ? 'items-end' : 'items-start'} animate-[messageSlide_0.3s_ease-out]`}>
              <div
                onContextMenu={(e) => handleContextMenu(e, msg.id)}
                className={`max-w-[70%] px-3 py-2 relative group cursor-context-menu shadow-sm ${
                  isOwn 
                    ? 'bg-[#2b5278] text-white rounded-2xl rounded-br-md' 
                    : 'bg-[#3a3a3a] text-gray-100 rounded-2xl rounded-bl-md'
                }`}
                style={{
                  backdropFilter: 'blur(10px)',
                }}
              >
                {/* Reply preview */}
                {repliedMsg && (
                  <div className={`mb-1.5 px-2.5 py-1 rounded-lg border-l-[3px] text-xs ${
                    isOwn ? 'border-white/50 bg-black/20' : 'border-blue-400 bg-black/10'
                  }`}>
                    <div className="flex items-center gap-1 mb-0.5">
                      <FiCornerUpLeft className="w-3 h-3 text-blue-400" />
                      <span className="font-medium text-blue-300">{repliedMsg.sender_name || 'You'}</span>
                    </div>
                    <p className="truncate opacity-70">{repliedMsg.content.slice(0, 50)}{repliedMsg.content.length > 50 ? '...' : ''}</p>
                  </div>
                )}

                {msg.sender_name && !isOwn && <div className="text-xs text-blue-300 mb-1 font-medium">{msg.sender_name}</div>}
                <p className="text-[14px] leading-relaxed whitespace-pre-wrap break-words">{msg.content}</p>
                <div className={`text-[11px] mt-0.5 text-right flex items-center justify-end gap-1 ${isOwn ? 'text-blue-200/70' : 'text-gray-500'}`}>
                  {msg.edited_at && <span className="opacity-50">(ред.)</span>}
                  {fmtTime(msg.created_at)}
                  {msg.msg_type === 'e2ee' && <span className="ml-0.5">🔒</span>}
                  {msg.msg_type === 'gif' && <span className="ml-0.5">🎞️</span>}
                  {msg.msg_type === 'file' && <span className="ml-0.5">📎</span>}
                </div>

                {/* Reply button (on hover) */}
                <button
                  onClick={() => setReplyTo(msg)}
                  className="absolute -top-2 -right-2 p-1.5 bg-gray-800 hover:bg-gray-700 rounded-full opacity-0 group-hover:opacity-100 transition-opacity shadow-lg"
                  title="Reply"
                >
                  <FiCornerUpLeft className="w-3 h-3 text-gray-400" />
                </button>
              </div>
              {/* Reactions */}
              <ReactionBar messageId={msg.id} />
            </div>
          )
        })}
        <div ref={bottomRef} />
      </div>

      {/* Reply Preview Bar */}
      {replyTo && (
        <div className="p-3 bg-gray-700 border-t border-gray-600 flex items-center justify-between">
          <div className="flex items-center gap-2 flex-1 min-w-0">
            <div className="p-2 bg-blue-500/20 rounded">
              <FiCornerUpLeft className="w-4 h-4 text-blue-400" />
            </div>
            <div className="flex-1 min-w-0">
              <div className="text-sm font-medium text-blue-400">
                Reply to {replyTo.sender_name || 'yourself'}
              </div>
              <div className="text-xs text-gray-500 truncate">
                {replyTo.content.slice(0, 80)}{replyTo.content.length > 80 ? '...' : ''}
              </div>
            </div>
          </div>
          <button
            onClick={() => setReplyTo(null)}
            className="p-2 hover:bg-gray-600 rounded-full transition-colors"
          >
            <FiX className="w-4 h-4 text-gray-400" />
          </button>
        </div>
      )}

      {/* Input */}
      <div className="relative border-t" style={{ backgroundColor: 'var(--color-bg-secondary, #1e293b)', borderColor: 'var(--color-border, #334155)' }}>
        {/* Attached Files Preview */}
        {attachedFiles.length > 0 && (
          <div className="p-2 border-b border-gray-700">
            <div className="flex gap-2 overflow-x-auto pb-2">
              {attachedFiles.map((file, index) => (
                <div key={index} className="relative flex-shrink-0">
                  {file.preview ? (
                    <div className="relative">
                      <img src={file.preview} alt={file.name} className="w-20 h-20 object-cover rounded-lg" />
                      <button
                        onClick={() => removeFile(index)}
                        className="absolute -top-1 -right-1 p-1 bg-red-500 rounded-full text-white hover:bg-red-600"
                      >
                        <FiX className="w-3 h-3" />
                      </button>
                    </div>
                  ) : (
                    <div className="relative w-20 h-20 bg-gray-700 rounded-lg flex flex-col items-center justify-center p-1">
                      <button
                        onClick={() => removeFile(index)}
                        className="absolute -top-1 -right-1 p-1 bg-red-500 rounded-full text-white hover:bg-red-600"
                      >
                        <FiX className="w-3 h-3" />
                      </button>
                      {file.type.startsWith('image/') ? (
                        <FiImage className="w-6 h-6 text-gray-400 mb-1" />
                      ) : (
                        <FiFile className="w-6 h-6 text-gray-400 mb-1" />
                      )}
                      <span className="text-xs text-gray-500 truncate text-center w-full">{file.name.slice(0, 8)}</span>
                      <span className="text-xs text-gray-600">{formatFileSize(file.size)}</span>
                    </div>
                  )}
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Sticker Picker */}
        {isPickerOpen && (
          <StickerPicker
            onSelectSticker={(stickerId) => {
              // TODO: Send sticker message via API
              console.log('Sending sticker:', stickerId);
            }}
            onClose={closePicker}
          />
        )}

        <div className="p-4 flex gap-2">
          {/* Attach File Button */}
          <button
            onClick={() => fileInputRef.current?.click()}
            className="p-2 rounded-lg bg-gray-700 text-gray-400 hover:bg-gray-600 transition-colors"
            aria-label="Attach file"
          >
            <FiPaperclip className="w-5 h-5" />
          </button>
          <input
            ref={fileInputRef}
            type="file"
            multiple
            accept="image/*,video/*,audio/*,.pdf,.doc,.docx,.txt,.zip"
            onChange={handleFileSelect}
            className="hidden"
          />
          {/* Sticker Button */}
          <button
            onClick={() => isPickerOpen ? closePicker() : useStickerStore.getState().togglePicker()}
            className={`p-2 rounded-lg transition-colors ${
              isPickerOpen ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
            }`}
            aria-label="Open sticker picker"
          >
            <FiSmile className="w-5 h-5" />
          </button>

          {/* GIF Button */}
          <button
            onClick={() => setShowGifPicker(!showGifPicker)}
            className={`p-2 rounded-lg transition-colors text-xs font-bold ${
              showGifPicker ? 'bg-blue-600 text-white' : 'bg-gray-700 text-gray-400 hover:bg-gray-600'
            }`}
            aria-label="Open GIF picker"
          >
            GIF
          </button>

          {/* Self-destruct Timer */}
          <select
            value={selfDestruct || ''}
            onChange={e => setSelfDestruct(e.target.value ? Number(e.target.value) : null)}
            className="px-2 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 text-sm focus:outline-none"
            title="Self-destruct timer"
          >
            <option value="">⏱️</option>
            <option value="60">1m</option>
            <option value="300">5m</option>
            <option value="3600">1h</option>
            <option value="86400">24h</option>
          </select>

          {/* Scheduled Message */}
          <input
            type="datetime-local"
            value={scheduledTime || ''}
            onChange={e => setScheduledTime(e.target.value || null)}
            className="px-2 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 text-sm focus:outline-none w-40"
            title="Schedule message"
          />

          <input value={content} onChange={handleInputChange} onKeyDown={e => e.key === 'Enter' && handleSend()}
            placeholder={t('message.type_message', 'Type a message...')} disabled={sending}
            className="flex-1 px-4 py-2 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none disabled:opacity-50" />
          <button onClick={handleSend} disabled={sending || !content.trim()}
            className="px-4 py-2 bg-blue-600 hover:bg-blue-700 disabled:opacity-50 text-white rounded-lg transition">
            <FiSend />
          </button>
        </div>
      </div>

      {/* Context Menu */}
      {contextMenu && (
        <MessageContextMenu
          messageId={contextMenu.messageId}
          isOwnMessage={messages.find(m => m.id === contextMenu.messageId)?.sender_id === (localStorage.getItem('user_id') || '')}
          isPinned={pinnedMessages.some(m => m.id === contextMenu.messageId)}
          onEdit={handleEditMessage}
          onDelete={handleDeleteMessage}
          onReply={handleReplyFromMenu}
          onPin={handlePinMessage}
          onClose={() => setContextMenu(null)}
          x={contextMenu.x}
          y={contextMenu.y}
        />
      )}

      {/* Edit Message Modal */}
      {editingMessage && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50 p-4">
          <div className="bg-gray-800 rounded-lg p-4 w-full max-w-lg">
            <h3 className="text-lg font-semibold text-white mb-3">Редактировать сообщение</h3>
            <textarea
              value={editingMessage.content}
              onChange={e => setEditingMessage({ ...editingMessage, content: e.target.value })}
              className="w-full p-3 bg-gray-700 text-white rounded-lg border border-gray-600 focus:border-blue-500 focus:outline-none resize-none"
              rows={4}
            />
            <div className="flex gap-2 mt-3">
              <button onClick={handleSaveEdit} className="flex-1 py-2 bg-blue-600 hover:bg-blue-700 text-white rounded-lg">
                Сохранить
              </button>
              <button onClick={() => setEditingMessage(null)} className="flex-1 py-2 bg-gray-700 hover:bg-gray-600 text-white rounded-lg">
                Отмена
              </button>
            </div>
          </div>
        </div>
      )}

      {/* GIF Picker */}
      {showGifPicker && (
        <div className="absolute bottom-20 right-4 z-40">
          <GifPicker chatId={chatId} onSend={handleSendGif} onClose={() => setShowGifPicker(false)} />
        </div>
      )}
    </div>
  )
}
