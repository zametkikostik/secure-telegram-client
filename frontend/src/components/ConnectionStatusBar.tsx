/**
 * Connection Status Bar Component
 *
 * Shows real-time WebSocket connection status
 * Provides visual feedback for connection state
 * Accessible to screen readers via aria-live region
 */

import React from 'react'
import { useTranslation } from 'react-i18next'
import { useChatStore } from '../services/chatStore'

const ConnectionStatusBar: React.FC = () => {
  const { t } = useTranslation()
  const isOnline = useChatStore((state) => state.isOnline)

  if (isOnline) {
    return null // Hide when online
  }

  const statusText = t('status.connecting')

  return (
    <div
      role="alert"
      aria-live="assertive"
      aria-atomic="true"
      className="px-4 py-2 text-center text-sm font-medium transition-colors duration-300"
      style={{
        backgroundColor: 'var(--color-warning)',
        color: 'white',
      }}
    >
      <span className="inline-flex items-center gap-2" aria-hidden="true">
        ⚡ {statusText}
      </span>
      {/* Screen reader only */}
      <span className="sr-only">{statusText}</span>
    </div>
  )
}

export default ConnectionStatusBar
