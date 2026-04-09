/**
 * Web3 Common Types — Общие типы для всех Web3 операций
 *
 * Эти типы описывают:
 * -统一ные ошибки
 * - Статусы транзакций
 * - Обёртки API ответов
 * - Состояния кошелька
 * - Сетевые статусы
 */

// ============================================================================
// Error Types
// ============================================================================

/** Типы Web3 ошибок */
export enum Web3ErrorType {
  /** Кошелёк не подключён */
  NotConnected = 'not_connected',
  /** Ошибка кошелька */
  Wallet = 'wallet',
  /** Сетевая ошибка */
  Network = 'network',
  /** Транзакция не удалась */
  TransactionFailed = 'transaction_failed',
  /** Ошибка подписи */
  SigningFailed = 'signing_failed',
  /** Недостаточно средств */
  InsufficientBalance = 'insufficient_balance',
  /** Ошибка ENS */
  EnsFailed = 'ens_failed',
  /** Неподдерживаемая цепь */
  UnsupportedChain = 'unsupported_chain',
  /** Ошибка RPC */
  Rpc = 'rpc',
  /** Некорректный адрес */
  InvalidAddress = 'invalid_address',
  /** Пользователь отклонил */
  UserRejected = 'user_rejected',
  /** Таймаут */
  Timeout = 'timeout',
  /** Ошибка валидации */
  ValidationError = 'validation_error',
  /** Ошибка API */
  ApiError = 'api_error',
}

/**统一ная Web3 ошибка */
export interface Web3Error {
  /** Тип ошибки */
  type: Web3ErrorType
  /** Сообщение */
  message: string
  /** Код ошибки (если есть) */
  code?: number | null
  /** Дополнительные данные */
  details?: Record<string, any> | null
  /** Оригинальная ошибка */
  originalError?: any | null
}

/** Ответ с ошибкой */
export interface Web3ErrorResponse {
  success: false
  error: Web3Error
}

// ============================================================================
// API Response Wrapper
// ============================================================================

/**统一ная обёртка API ответа */
export interface Web3ApiResponse<T = any> {
  /** Успех */
  success: boolean
  /** Данные (если успех) */
  data?: T | null
  /** Ошибка (если провал) */
  error?: Web3Error | null
  /** Метаданные ответа */
  metadata?: Record<string, any> | null
}

/** Пагинированный ответ */
export interface PaginatedResponse<T> {
  /** Данные */
  items: T[]
  /** Общее количество */
  total: number
  /** Смещение */
  offset: number
  /** Лимит */
  limit: number
  /** Есть ли ещё */
  has_next: boolean
}

// ============================================================================
// Transaction Status
// ============================================================================

/** Статусы транзакции */
export enum TransactionStatus {
  /** Инициализирован */
  Initiated = 'initiated',
  /** Ожидает подписи */
  AwaitingSignature = 'awaiting_signature',
  /** Отправлен в сеть */
  Submitted = 'submitted',
  /** В ожидании подтверждения */
  Pending = 'pending',
  /** Подтверждён */
  Confirmed = 'confirmed',
  /** Не удался */
  Failed = 'failed',
  /** Отменён */
  Cancelled = 'cancelled',
  /** Возврат */
  Refunded = 'refunded',
}

/** Информация о транзакции */
export interface TransactionInfo {
  /** Hash транзакции */
  tx_hash: string
  /** ID цепи */
  chain_id: number
  /** Статус */
  status: TransactionStatus
  /** От */
  from: string
  /** Кому */
  to: string
  /** Значение */
  value: string
  /** Газ */
  gas_used?: string | null
  /** Цена газа */
  gas_price?: string | null
  /** Nonce */
  nonce?: number | null
  /** Блок */
  block_number?: number | null
  /** Время */
  timestamp: number
  /** Тип транзакции */
  type: string
  /** Данные */
  data?: string | null
}

/** Ответ с информацией о транзакции */
export interface TransactionInfoResponse {
  success: boolean
  transaction?: TransactionInfo | null
  error?: string | null
}

// ============================================================================
// Wallet Connection State
// ============================================================================

/** Состояния подключения кошелька */
export enum WalletConnectionState {
  /** Отключён */
  Disconnected = 'disconnected',
  /** Подключение */
  Connecting = 'connecting',
  /** Подключён */
  Connected = 'connected',
  /** Ошибка подключения */
  ConnectionError = 'connection_error',
  /** Переключение сети */
  SwitchingNetwork = 'switching_network',
}

/** Информация о подключении кошелька */
export interface WalletConnectionInfo {
  /** Состояние */
  state: WalletConnectionState
  /** Адрес */
  address?: string | null
  /** Цепь */
  chain_id?: number | null
  /** Тип кошелька (MetaMask, WalletConnect, и т.д.) */
  wallet_type?: string | null
  /** Время подключения */
  connected_at?: number | null
}

// ============================================================================
// Network Status
// ============================================================================

/** Статусы сети */
export enum NetworkStatus {
  /** Подключён */
  Connected = 'connected',
  /** Подключение */
  Connecting = 'connecting',
  /** Переподключение */
  Reconnecting = 'reconnecting',
  /** Отключён */
  Disconnected = 'disconnected',
  /** Ошибка */
  Error = 'error',
}

/** Информация о сети */
export interface NetworkInfo {
  /** Статус */
  status: NetworkStatus
  /** ID цепи */
  chain_id: number
  /** Название цепи */
  chain_name: string
  /** RPC URL */
  rpc_url: string
  /** Блок */
  latest_block?: number | null
  /** Пинг (ms) */
  ping?: number | null
  /** Время последней проверки */
  last_checked: number
}

/** Ответ с информацией о сети */
export interface NetworkInfoResponse {
  success: boolean
  network?: NetworkInfo | null
  error?: string | null
}

// ============================================================================
// ENS (Ethereum Name Service)
// ============================================================================

/** Запрос разрешения ENS имени */
export interface EnsResolveRequest {
  /** ENS имя или адрес */
  name_or_address: string
  /** ID цепи */
  chain_id?: number
}

/** Данные ENS записи */
export interface EnsRecord {
  /** ENS имя */
  name: string
  /** Адрес */
  address: string
  /** URL аватара */
  avatar_url?: string | null
  /** Описание */
  description?: string | null
  /** Контент хеш */
  content_hash?: string | null
}

/** Ответ разрешения ENS */
export interface EnsResolveResponse {
  success: boolean
  record?: EnsRecord | null
  error?: string | null
}

/** Запрос обратного поиска ENS */
export interface EnsReverseRequest {
  /** Адрес */
  address: string
  /** ID цепи */
  chain_id?: number
}

// ============================================================================
// Gas Estimation
// ============================================================================

/** Оценка газа */
export interface GasEstimate {
  /** Оценка газа (units) */
  gas_estimate: string
  /** Цена газа (gwei) */
  gas_price: string
  /** Максимальная цена газа (gwei, EIP-1559) */
  max_fee_per_gas?: string | null
  /** Приоритет (gwei, EIP-1559) */
  max_priority_fee?: string | null
  /** Общая стоимость (в нативном токене) */
  total_cost: string
  /** Общая стоимость (USD) */
  total_cost_usd: string
}

/** Ответ оценки газа */
export interface GasEstimateResponse {
  success: boolean
  estimate?: GasEstimate | null
  error?: string | null
}

// ============================================================================
// Balance & Portfolio
// ============================================================================

/** Баланс токена */
export interface TokenBalance {
  /** Адрес токена */
  token_address: string
  /** Символ */
  symbol: string
  /** Название */
  name: string
  /** Десятичные знаки */
  decimals: number
  /** Баланс (форматированный) */
  balance: string
  /** Баланс (raw, wei) */
  balance_raw: string
  /** Цена токена (USD) */
  price_usd: string
  /** Стоимость (USD) */
  value_usd: string
  /** Изменение 24ч (%) */
  change_24h: string
  /** Логотип */
  logo_uri?: string | null
}

/** Нативный баланс */
export interface NativeBalance {
  /** Цепь */
  chain_id: number
  /** Адрес */
  address: string
  /** Баланс (форматированный) */
  balance: string
  /** Баланс (wei) */
  balance_wei: string
  /** Символ */
  symbol: string
  /** Цена (USD) */
  price_usd: string
  /** Стоимость (USD) */
  value_usd: string
}

/** Общий портфель */
export interface Portfolio {
  /** Адрес */
  address: string
  /** Общий баланс (USD) */
  total_usd: string
  /** Изменение 24ч (%) */
  change_24h: string
  /** Балансы цепей */
  chains: Array<{
    chain_id: number
    chain_name: string
    balance_usd: string
  }>
  /** Балансы токенов */
  tokens: TokenBalance[]
  /** Нативные балансы */
  native_balances: NativeBalance[]
  /** Время обновления */
  updated_at: number
}

/** Ответ портфеля */
export interface PortfolioResponse {
  success: boolean
  portfolio?: Portfolio | null
  error?: string | null
}

// ============================================================================
// Rate Limiting & Quotas
// ============================================================================

/** Информация о лимитах */
export interface RateLimitInfo {
  /** Лимит запросов */
  limit: number
  /** Использовано */
  used: number
  /** Осталось */
  remaining: number
  /** Время сброса (timestamp ms) */
  reset_at: number
}

/** Ответ с лимитами */
export interface RateLimitResponse {
  success: boolean
  rate_limit?: RateLimitInfo | null
  error?: string | null
}

// ============================================================================
// Configuration
// ============================================================================

/** Конфигурация Web3 */
export interface Web3Config {
  /** API ключ */
  api_key?: string | null
  /** RPC URL */
  rpc_urls: Record<number, string>
  /** Получатель комиссии */
  fee_recipient: string
  /** Комиссия (basis points) */
  fee_bps: number
  /** Slippage по умолчанию */
  default_slippage_bps: number
  /** Таймаут запросов (ms) */
  request_timeout_ms: number
  /** Включить кэш */
  enable_cache: boolean
  /** Время жизни кэша (ms) */
  cache_ttl_ms: number
}

/** Ответ конфигурации */
export interface Web3ConfigResponse {
  success: boolean
  config?: Web3Config | null
  error?: string | null
}

// ============================================================================
// Health Check
// ============================================================================

/** Статус компонента */
export type ComponentStatus = 'healthy' | 'degraded' | 'down' | 'unknown'

/** Статусы Web3 сервисов */
export interface Web3HealthStatus {
  /** 0x Protocol */
  zerox_protocol: ComponentStatus
  /** Abcex */
  abcex: ComponentStatus
  /** Bitget */
  bitget: ComponentStatus
  /** RPC ноды */
  rpc_nodes: ComponentStatus
  /** ENS */
  ens: ComponentStatus
  /** Общее состояние */
  overall: ComponentStatus
  /** Время проверки */
  checked_at: number
}

/** Ответ проверки здоровья */
export interface Web3HealthResponse {
  success: boolean
  health?: Web3HealthStatus | null
  error?: string | null
}
