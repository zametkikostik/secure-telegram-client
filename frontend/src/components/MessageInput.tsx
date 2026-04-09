import { FiSend, FiPaperclip, FiSmile } from 'react-icons/fi'
import { FiShield } from 'react-icons/fi'
import clsx from 'clsx'
import { useState, useRef, useEffect, forwardRef } from 'react'
import { useTranslation } from 'react-i18next'

interface MessageInputProps {
  onSend: (message: string) => void
  disabled?: boolean
  placeholder?: string
  encryptionMethod?: string
}

export const MessageInput = forwardRef<HTMLDivElement, MessageInputProps>(
  function MessageInput(
    {
      onSend,
      disabled = false,
      encryptionMethod = 'X25519+Kyber1024',
    }: MessageInputProps,
    ref
  ) {
    const { t } = useTranslation()
    const [inputValue, setInputValue] = useState('')
    const inputRef = useRef<HTMLInputElement>(null)

    useEffect(() => {
      inputRef.current?.focus()
    }, [])

    const handleSend = () => {
      if (!inputValue.trim() || disabled) return
      onSend(inputValue.trim())
      setInputValue('')
      inputRef.current?.focus()
    }

    const handleKeyPress = (e: React.KeyboardEvent) => {
      if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault()
        handleSend()
      }
    }

    const canSend = inputValue.trim().length > 0 && !disabled

    return (
      <div ref={ref}>
        <div className="flex items-center gap-3">
          {/* Attach file */}
          <button
            className="p-2 rounded-lg transition-colors"
            style={{
              color: 'var(--color-text-secondary)',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--color-bg-tertiary)'
              e.currentTarget.style.color = 'var(--color-text-primary)'
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'transparent'
              e.currentTarget.style.color = 'var(--color-text-secondary)'
            }}
            title={t('a11y.attach_file') || 'Прикрепить файл'}
            aria-label={t('a11y.attach_file') || 'Прикрепить файл'}
            disabled={disabled}
            tabIndex={0}
          >
            <FiPaperclip className="w-5 h-5" aria-hidden="true" />
          </button>

          {/* Emoji */}
          <button
            className="p-2 rounded-lg transition-colors"
            style={{
              color: 'var(--color-text-secondary)',
            }}
            onMouseEnter={(e) => {
              e.currentTarget.style.backgroundColor = 'var(--color-bg-tertiary)'
              e.currentTarget.style.color = 'var(--color-text-primary)'
            }}
            onMouseLeave={(e) => {
              e.currentTarget.style.backgroundColor = 'transparent'
              e.currentTarget.style.color = 'var(--color-text-secondary)'
            }}
            title={t('a11y.insert_emoji') || 'Эмодзи'}
            aria-label={t('a11y.insert_emoji') || 'Эмодзи'}
            disabled={disabled}
            tabIndex={0}
          >
            <FiSmile className="w-5 h-5" aria-hidden="true" />
          </button>

          {/* Input field */}
          <label htmlFor="message-input" className="sr-only">
            {t('message.type_message')}
          </label>
          <input
            ref={inputRef}
            id="message-input"
            type="text"
            value={inputValue}
            onChange={(e) => setInputValue(e.target.value)}
            onKeyDown={handleKeyPress}
            placeholder={t('message.type_message')}
            disabled={disabled}
            className={clsx(
              'flex-1 px-4 py-3 rounded-xl transition-colors',
              'focus:outline-none focus:ring-2',
              disabled && 'opacity-50 cursor-not-allowed'
            )}
            style={{
              backgroundColor: 'var(--color-bg-tertiary)',
              color: 'var(--color-text-primary)',
              border: '1px solid var(--color-border)',
            }}
            aria-describedby="encryption-hint"
            autoComplete="off"
          />

          {/* Send button */}
          <button
            onClick={handleSend}
            disabled={!canSend}
            className={clsx(
              'p-3 rounded-xl transition-all',
              canSend
                ? 'shadow-lg'
                : 'cursor-not-allowed'
            )}
            style={{
              backgroundColor: canSend
                ? 'var(--color-accent)'
                : 'var(--color-bg-tertiary)',
              color: canSend ? 'white' : 'var(--color-text-muted)',
              boxShadow: canSend
                ? '0 4px 14px var(--color-accent)'
                : 'none',
            }}
            onMouseEnter={(e) => {
              if (canSend) {
                e.currentTarget.style.backgroundColor = 'var(--color-accent-hover)'
              }
            }}
            onMouseLeave={(e) => {
              if (canSend) {
                e.currentTarget.style.backgroundColor = 'var(--color-accent)'
              }
            }}
            title={t('common.send')}
            aria-label={`${t('common.send')} (${inputValue.length} символов)`}
          >
            <FiSend className="w-5 h-5" aria-hidden="true" />
          </button>
        </div>

        {/* Encryption indicator */}
        <div
          id="encryption-hint"
          className="mt-2 flex items-center justify-center gap-2 text-xs"
          style={{ color: 'var(--color-text-muted)' }}
          role="note"
        >
          <FiShield
            className="w-3 h-3"
            style={{ color: 'var(--color-success)' }}
            aria-hidden="true"
          />
          <span>
            {t('message.encrypted')}{' '}
            <span style={{ color: 'var(--color-success)', fontWeight: 500 }}>
              {encryptionMethod}
            </span>{' '}
            {t('a11y.before_send') || 'перед отправкой'}
          </span>
        </div>
      </div>
    )
  }
)
