import { FiShield, FiWifi, FiCloud, FiLock, FiCheck } from 'react-icons/fi'
import clsx from 'clsx'

export type EncryptionStatus = 'encrypting' | 'encrypted' | 'error'
export type TransportRoute = 'p2p' | 'cloudflare' | 'unknown'
export type DeliveryStatus = 'sending' | 'sent' | 'delivered' | 'read' | 'failed'

// ============================================================================
// Encryption Badge
// ============================================================================

interface EncryptionBadgeProps {
  status?: EncryptionStatus
  method?: string
  size?: 'sm' | 'md'
  showLabel?: boolean
}

export function EncryptionBadge({
  status = 'encrypted',
  method = 'X25519+Kyber1024',
  size = 'sm',
  showLabel = true,
}: EncryptionBadgeProps) {
  const sizeClasses = {
    sm: 'px-1.5 py-0.5 text-xs',
    md: 'px-2.5 py-1 text-sm',
  }

  const iconSize = {
    sm: 'w-3 h-3',
    md: 'w-4 h-4',
  }

  if (status === 'error') {
    return (
      <span
        className={clsx(
          'inline-flex items-center gap-1 rounded font-medium',
          'bg-red-500/20 text-red-400 border border-red-500/30',
          sizeClasses[size]
        )}
        title="Ошибка шифрования"
      >
        <FiShield className={clsx(iconSize[size], 'text-red-400')} />
        {showLabel && <span>Ошибка</span>}
      </span>
    )
  }

  if (status === 'encrypting') {
    return (
      <span
        className={clsx(
          'inline-flex items-center gap-1 rounded font-medium',
          'bg-yellow-500/20 text-yellow-400 border border-yellow-500/30',
          sizeClasses[size]
        )}
        title="Шифрование..."
      >
        <div className={clsx('w-3 h-3 border-2 border-yellow-400 border-t-transparent rounded-full animate-spin', iconSize[size])} />
        {showLabel && <span>Шифрование...</span>}
      </span>
    )
  }

  return (
    <span
      className={clsx(
        'inline-flex items-center gap-1 rounded font-medium',
        'bg-blue-500/20 text-blue-400 border border-blue-500/30',
        sizeClasses[size]
      )}
      title={`Зашифровано: ${method}`}
    >
      <FiShield className={clsx(iconSize[size], 'text-blue-400')} />
      {showLabel && <span className="hidden sm:inline">{method}</span>}
    </span>
  )
}

// ============================================================================
// Route Indicator
// ============================================================================

interface RouteIndicatorProps {
  route: TransportRoute
  size?: 'sm' | 'md'
  showLabel?: boolean
}

export function RouteIndicator({
  route,
  size = 'sm',
  showLabel = true,
}: RouteIndicatorProps) {
  const sizeClasses = {
    sm: 'px-1.5 py-0.5 text-xs',
    md: 'px-2.5 py-1 text-sm',
  }

  const iconSize = {
    sm: 'w-3 h-3',
    md: 'w-4 h-4',
  }

  if (route === 'unknown') return null

  if (route === 'p2p') {
    return (
      <span
        className={clsx(
          'inline-flex items-center gap-1 rounded font-medium',
          'bg-green-500/20 text-green-400 border border-green-500/30',
          sizeClasses[size]
        )}
        title="P2P соединение (прямое)"
      >
        <FiWifi className={clsx(iconSize[size], 'text-green-400')} />
        {showLabel && <span className="hidden sm:inline">P2P</span>}
      </span>
    )
  }

  return (
    <span
      className={clsx(
        'inline-flex items-center gap-1 rounded font-medium',
        'bg-orange-500/20 text-orange-400 border border-orange-500/30',
        sizeClasses[size]
      )}
      title="Cloudflare fallback"
    >
      <FiCloud className={clsx(iconSize[size], 'text-orange-400')} />
      {showLabel && <span className="hidden sm:inline">Cloud</span>}
    </span>
  )
}

// ============================================================================
// Delivery Status
// ============================================================================

interface DeliveryStatusProps {
  status: DeliveryStatus
  showLabel?: boolean
  size?: 'sm' | 'md'
}

export function DeliveryStatusIndicator({
  status,
  showLabel = true,
  size = 'sm',
}: DeliveryStatusProps) {
  const textSize = {
    sm: 'text-xs',
    md: 'text-sm',
  }

  const iconSize = {
    sm: 'w-4 h-4',
    md: 'w-5 h-5',
  }

  const labels: Record<DeliveryStatus, string> = {
    sending: 'Отправка...',
    sent: 'Отправлено',
    delivered: 'Доставлено',
    read: 'Прочитано',
    failed: 'Ошибка',
  }

  switch (status) {
    case 'sending':
      return (
        <span className={clsx('inline-flex items-center gap-1', textSize[size], 'text-gray-400')}>
          <div className="w-3 h-3 border-2 border-gray-400 border-t-transparent rounded-full animate-spin" />
          {showLabel && <span>{labels.sending}</span>}
        </span>
      )

    case 'sent':
      return (
        <span className={clsx('inline-flex items-center gap-1', textSize[size], 'text-gray-400')}>
          <FiCheck className={iconSize[size]} />
          {showLabel && <span>{labels.sent}</span>}
        </span>
      )

    case 'delivered':
      return (
        <span className={clsx('inline-flex items-center gap-1', textSize[size], 'text-gray-400')}>
          <span className="flex -space-x-2">
            <FiCheck className={iconSize[size]} />
            <FiCheck className={iconSize[size]} />
          </span>
          {showLabel && <span>{labels.delivered}</span>}
        </span>
      )

    case 'read':
      return (
        <span className={clsx('inline-flex items-center gap-1', textSize[size], 'text-blue-400')}>
          <span className="flex -space-x-2">
            <FiCheck className={iconSize[size]} />
            <FiCheck className={iconSize[size]} />
          </span>
          {showLabel && <span>{labels.read}</span>}
        </span>
      )

    case 'failed':
      return (
        <span className={clsx('inline-flex items-center gap-1', textSize[size], 'text-red-400')}>
          <span className="font-bold">!</span>
          {showLabel && <span>{labels.failed}</span>}
        </span>
      )
  }
}

// ============================================================================
// Connection Status Bar
// ============================================================================

interface ConnectionStatusBarProps {
  route: TransportRoute
  encryptionStatus: EncryptionStatus
}

export function ConnectionStatusBar({ route, encryptionStatus }: ConnectionStatusBarProps) {
  return (
    <div className="flex items-center gap-2">
      {/* Connection status */}
      <div
        className={clsx(
          'flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs border',
          route === 'p2p'
            ? 'bg-green-500/10 border-green-500/20 text-green-400'
            : route === 'cloudflare'
            ? 'bg-orange-500/10 border-orange-500/20 text-orange-400'
            : 'bg-gray-500/10 border-gray-500/20 text-gray-400'
        )}
      >
        {route === 'p2p' ? (
          <FiWifi className="w-3 h-3" />
        ) : route === 'cloudflare' ? (
          <FiCloud className="w-3 h-3" />
        ) : (
          <FiLock className="w-3 h-3" />
        )}
        <span>
          {route === 'p2p' ? 'P2P' : route === 'cloudflare' ? 'Cloudflare' : 'Подключение...'}
        </span>
      </div>

      {/* Encryption status */}
      <div
        className={clsx(
          'flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs border',
          encryptionStatus === 'encrypted'
            ? 'bg-blue-500/10 border-blue-500/20 text-blue-400'
            : encryptionStatus === 'encrypting'
            ? 'bg-yellow-500/10 border-yellow-500/20 text-yellow-400'
            : 'bg-red-500/10 border-red-500/20 text-red-400'
        )}
      >
        <FiShield className="w-3 h-3" />
        <span>
          {encryptionStatus === 'encrypted'
            ? 'E2EE'
            : encryptionStatus === 'encrypting'
            ? 'Шифрование...'
            : 'Ошибка'}
        </span>
      </div>
    </div>
  )
}

// ============================================================================
// Encryption Notice Banner
// ============================================================================

interface EncryptionNoticeProps {
  method?: string
  variant?: 'default' | 'compact'
}

export function EncryptionNotice({
  method = 'X25519+Kyber1024',
  variant = 'default',
}: EncryptionNoticeProps) {
  if (variant === 'compact') {
    return (
      <div className="flex justify-center">
        <div className="px-3 py-1.5 bg-gray-800/50 rounded-lg text-xs text-gray-500 flex items-center gap-1.5">
          <FiShield className="w-3 h-3 text-blue-400" />
          <span>E2EE: {method}</span>
        </div>
      </div>
    )
  }

  return (
    <div className="flex justify-center">
      <div className="px-4 py-2.5 bg-gradient-to-r from-blue-500/10 to-purple-500/10 border border-blue-500/20 rounded-lg text-xs text-gray-400 flex items-center gap-2">
        <FiShield className="w-4 h-4 text-blue-400" />
        <span>Сообщения защищены постквантовым E2EE</span>
        <span className="px-1.5 py-0.5 bg-blue-500/20 text-blue-400 rounded font-mono">
          {method}
        </span>
        <span className="text-gray-600">•</span>
        <span className="flex items-center gap-1">
          <FiLock className="w-3 h-3" />
          Только вы и получатель можете читать
        </span>
      </div>
    </div>
  )
}
