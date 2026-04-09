/**
 * MessageContextMenu — Context menu for message actions (edit, delete, reply)
 */
import { useState, useEffect, useRef } from 'react'
import { FiEdit2, FiTrash2, FiCornerUpRight, FiX, FiMapPin } from 'react-icons/fi'

interface MessageContextMenuProps {
  messageId: string
  isOwnMessage: boolean
  isPinned?: boolean
  onEdit: (messageId: string) => void
  onDelete: (messageId: string) => void
  onReply: (messageId: string) => void
  onPin: (messageId: string) => void
  onClose: () => void
  x: number
  y: number
}

export const MessageContextMenu: React.FC<MessageContextMenuProps> = ({
  messageId,
  isOwnMessage,
  isPinned = false,
  onEdit,
  onDelete,
  onReply,
  onPin,
  onClose,
  x,
  y,
}) => {
  const menuRef = useRef<HTMLDivElement>(null)

  // Close on outside click
  useEffect(() => {
    const handleClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        onClose()
      }
    }
    document.addEventListener('mousedown', handleClick)
    return () => document.removeEventListener('mousedown', handleClick)
  }, [onClose])

  // Adjust position if menu goes off screen
  const [position, setPosition] = useState({ x, y })

  useEffect(() => {
    const menu = menuRef.current
    if (!menu) return

    const rect = menu.getBoundingClientRect()
    const adjustedX = x + rect.width > window.innerWidth ? x - rect.width : x
    const adjustedY = y + rect.height > window.innerHeight ? y - rect.height : y
    setPosition({ x: adjustedX, y: adjustedY })
  }, [x, y])

  return (
    <div
      ref={menuRef}
      className="fixed z-50 bg-gray-800 border border-gray-700 rounded-lg shadow-xl py-1 min-w-[180px]"
      style={{ left: position.x, top: position.y }}
    >
      <div className="flex items-center justify-between px-3 py-2 border-b border-gray-700">
        <span className="text-xs text-gray-400">Действия</span>
        <button onClick={onClose} className="text-gray-400 hover:text-white">
          <FiX className="w-4 h-4" />
        </button>
      </div>

      <button
        onClick={() => onPin(messageId)}
        className="w-full px-4 py-2 text-sm text-left flex items-center gap-3 text-yellow-400 hover:bg-yellow-900/30 hover:text-yellow-300 transition-colors"
      >
        <FiMapPin className="w-4 h-4" />
        {isPinned ? 'Открепить' : 'Закрепить'}
      </button>

      <button
        onClick={() => onReply(messageId)}
        className="w-full px-4 py-2 text-sm text-left flex items-center gap-3 text-gray-300 hover:bg-gray-700 hover:text-white transition-colors"
      >
        <FiCornerUpRight className="w-4 h-4" />
        Ответить
      </button>

      {isOwnMessage && (
        <>
          <button
            onClick={() => onEdit(messageId)}
            className="w-full px-4 py-2 text-sm text-left flex items-center gap-3 text-gray-300 hover:bg-gray-700 hover:text-white transition-colors"
          >
            <FiEdit2 className="w-4 h-4" />
            Редактировать
          </button>

          <button
            onClick={() => onDelete(messageId)}
            className="w-full px-4 py-2 text-sm text-left flex items-center gap-3 text-red-400 hover:bg-red-900/30 hover:text-red-300 transition-colors"
          >
            <FiTrash2 className="w-4 h-4" />
            Удалить
          </button>
        </>
      )}
    </div>
  )
}
