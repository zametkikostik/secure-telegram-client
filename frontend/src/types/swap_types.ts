/**
 * 0x Protocol Types — Расширенные типы для DeFi Swap
 *
 * Эти типы описывают все аспекты 0x Protocol integration:
 * - Информация о токенах
 * - История обменов
 * - Approval транзакции
 * - Цены газа
 * - Маршруты обмена
 */

// ============================================================================
// Token Information
// ============================================================================

/** Информация о токене */
export interface TokenInfo {
  /** Адрес токена */
  address: string
  /** Символ */
  symbol: string
  /** Название */
  name: string
  /** Десятичные знаки */
  decimals: number
  /** Логотип URL */
  logo_uri?: string | null
  /** Теги (stablecoin, verified, и т.д.) */
  tags?: string[] | null
  /** Цепь */
  chain_id: number
}

/** Запрос списка токенов */
export interface TokensListRequest {
  /** ID цепи */
  chain_id: number
  /** Фильтр по тегу */
  tag?: string
  /** Смещение */
  offset?: number
  /** Лимит */
  limit?: number
}

/** Ответ списка токенов */
export interface TokensListResponse {
  success: boolean
  tokens?: TokenInfo[] | null
  /** Общее количество */
  total?: number
  error?: string | null
}

/** Популярные токены */
export interface PopularTokensResponse {
  success: boolean
  tokens?: TokenInfo[] | null
  error?: string | null
}

/** Поиск токенов */
export interface TokenSearchRequest {
  /** ID цепи */
  chain_id: number
  /** Поисковый запрос */
  query: string
  /** Лимит */
  limit?: number
}

/** Ответ поиска токенов */
export interface TokenSearchResponse {
  success: boolean
  tokens?: TokenInfo[] | null
  error?: string | null
}

// ============================================================================
// Swap History
// ============================================================================

/** Запрос истории обменов */
export interface SwapHistoryRequest {
  /** Адрес пользователя */
  user_address: string
  /** ID цепи */
  chain_id?: number
  /** Токен продажи */
  from_token?: string
  /** Токен покупки */
  to_token?: string
  /** Время начала */
  from_time?: number
  /** Время конца */
  to_time?: number
  /** Смещение */
  offset?: number
  /** Лимит */
  limit?: number
}

/** Элемент истории обмена */
export interface SwapHistoryItem {
  /** ID транзакции */
  tx_hash: string
  /** ID цепи */
  chain_id: number
  /** Токен продажи */
  from_token: string
  /** Токен покупки */
  to_token: string
  /** Сумма продажи */
  from_amount: string
  /** Сумма покупки */
  to_amount: string
  /** Цена */
  price: string
  /** Газ */
  gas_used: string
  /** Комиссия */
  fee_bps: number
  /** Статус */
  status: SwapTransactionStatus
  /** Время */
  timestamp: number
  /** Блок */
  block_number?: number | null
}

/** Статусы транзакции обмена */
export enum SwapTransactionStatus {
  /** Ожидает */
  Pending = 'pending',
  /** Отправлен */
  Submitted = 'submitted',
  /** Подтверждён */
  Confirmed = 'confirmed',
  /** Не удался */
  Failed = 'failed',
  /** Отменён */
  Cancelled = 'cancelled',
}

/** Ответ истории обменов */
export interface SwapHistoryResponse {
  success: boolean
  swaps?: SwapHistoryItem[] | null
  /** Общее количество */
  total?: number
  error?: string | null
}

// ============================================================================
// Approval Transactions
// ============================================================================

/** Запрос approval транзакции */
export interface ApproveTransactionRequest {
  /** Адрес токена */
  token_address: string
  /** ID цепи */
  chain_id: number
  /** Адрес владельца (spender) */
  spender_address: string
  /** Сумма для approval (в wei) */
  amount?: string
  /** Адрес пользователя */
  user_address: string
}

/** Данные approval транзакции */
export interface ApproveTransactionData {
  /** Адрес для отправки */
  to: string
  /** Данные транзакции */
  data: string
  /** Значение (ETH) */
  value: string
  /** Оценка газа */
  gas_estimate: string
  /** Цена газа */
  gas_price: string
}

/** Ответ approval транзакции */
export interface ApproveTransactionResponse {
  success: boolean
  approval?: ApproveTransactionData | null
  error?: string | null
}

/** Проверка статуса approval */
export interface AllowanceRequest {
  /** Адрес токена */
  token_address: string
  /** ID цепи */
  chain_id: number
  /** Адрес владельца */
  owner_address: string
  /** Адрес spender */
  spender_address: string
}

/** Ответ allowance */
export interface AllowanceResponse {
  success: boolean
  /** Текущий allowance (в wei) */
  allowance?: string | null
  error?: string | null
}

/** Проверка требуется ли approval */
export interface ApprovalCheckRequest {
  /** Адрес токена */
  token_address: string
  /** ID цепи */
  chain_id: number
  /** Сумма (в wei) */
  amount: string
  /** Адрес пользователя */
  user_address: string
}

/** Ответ проверки approval */
export interface ApprovalCheckResponse {
  success: boolean
  /** Требуется ли approval */
  needs_approval?: boolean | null
  /** Текущий allowance */
  current_allowance?: string | null
  /** Недостаточно ли allowance */
  insufficient_allowance?: boolean | null
  error?: string | null
}

// ============================================================================
// Gas Prices
// ============================================================================

/** Уровни приоритета газа */
export type GasPriority = 'slow' | 'standard' | 'fast' | 'instant'

/** Данные цены газа */
export interface GasPriceData {
  /** Цена газа (gwei) */
  price_gwei: string
  /** Оценка времени (секунды) */
  estimate_seconds: number
  /** Приоритет */
  priority: GasPriority
}

/** Ответ цен газа */
export interface GasPricesResponse {
  success: boolean
  prices?: GasPriceData[] | null
  /** Базовая цена */
  base_price?: string | null
  error?: string | null
}

/** Запрос цен газа */
export interface GasPricesRequest {
  /** ID цепи */
  chain_id: number
}

// ============================================================================
// Swap Routes
// ============================================================================

/** Данные маршрута обмена */
export interface SwapRouteData {
  /** Маршрут (список пулов) */
  route: SwapPoolHop[]
  /** Ожидаемый выход */
  expected_output: string
  /** Минимальный выход (с slippage) */
  minimum_output: string
  /** Цена воздействия */
  price_impact: string
  /** Общая комиссия */
  total_fees: string
}

/** Шаг маршрута через пул */
export interface SwapPoolHop {
  /** Протокол (Uniswap, Sushiswap, и т.д.) */
  protocol: string
  /** Пул адрес */
  pool_address: string
  /** Токен входа */
  from_token: string
  /** Токен выхода */
  to_token: string
  /** Сумма входа */
  from_amount: string
  /** Сумма выхода */
  to_amount: string
}

/** Ответ маршрута */
export interface SwapRouteResponse {
  success: boolean
  route?: SwapRouteData | null
  error?: string | null
}

// ============================================================================
// Quote Extensions
// ============================================================================

/** Расширенная котировка с маршрутом */
export interface ExtendedSwapQuoteData {
  /** Базовая котировка */
  quote: SwapQuoteData
  /** Маршрут */
  route: SwapRouteData
  /** Требуется ли approval */
  needs_approval: boolean
  /** Данные approval (если требуется) */
  approval_data?: ApproveTransactionData | null
  /** Цены газа */
  gas_prices: GasPriceData[]
  /** Оценка времени */
  estimated_time_seconds: number
}

/** Расширенный ответ котировки */
export interface ExtendedSwapQuoteResponse {
  success: boolean
  quote?: ExtendedSwapQuoteData | null
  error?: string | null
}

// ============================================================================
// Price API
// ============================================================================

/** Запрос цены токена */
export interface TokenPriceRequest {
  /** Адрес токена */
  token_address: string
  /** ID цепи */
  chain_id: number
  /** Валюта цены (USD, ETH, и т.д.) */
  currency?: string
}

/** Данные цены токена */
export interface TokenPriceData {
  /** Адрес токена */
  address: string
  /** Цена */
  price: string
  /** Валюта */
  currency: string
  /** Изменение 24ч (%) */
  change_24h: string
  /** Объём 24ч */
  volume_24h: string
  /** Market cap */
  market_cap: string
  /** Timestamp */
  timestamp: number
}

/** Ответ цены токена */
export interface TokenPriceResponse {
  success: boolean
  price?: TokenPriceData | null
  error?: string | null
}

/** Запрос цен нескольких токенов */
export interface BatchTokenPriceRequest {
  /** Запросы цен */
  requests: TokenPriceRequest[]
}

/** Ответ цен нескольких токенов */
export interface BatchTokenPriceResponse {
  success: boolean
  prices?: TokenPriceData[] | null
  error?: string | null
}

// ============================================================================
// Statistics & Analytics
// ============================================================================

/** Статистика обменов пользователя */
export interface UserSwapStats {
  /** Адрес пользователя */
  user_address: string
  /** Всего обменов */
  total_swaps: number
  /** Объём торговли */
  total_volume_usd: string
  /** Сэкономлено на комиссиях */
  fees_saved_usd: string
  /** Любимая цепь */
  favorite_chain: string
  /** Любимый токен */
  favorite_token: string
  /** Средний slippage */
  avg_slippage: string
}

/** Ответ статистики пользователя */
export interface UserSwapStatsResponse {
  success: boolean
  stats?: UserSwapStats | null
  error?: string | null
}

/** Общая статистика протокола */
export interface ProtocolStats {
  /** Объём 24ч */
  volume_24h: string
  /** Объём всего */
  volume_total: string
  /** Всего обменов */
  total_swaps: number
  /** Уникальных пользователей */
  unique_users: number
  /** Средняя комиссия */
  avg_fee_bps: number
  /** Активных цепей */
  active_chains: number
}

/** Ответ статистики протокола */
export interface ProtocolStatsResponse {
  success: boolean
  stats?: ProtocolStats | null
  error?: string | null
}

// ============================================================================
// Helper Types
// ============================================================================

/** Фильтр для поиска котировок */
export interface SwapQuoteFilter {
  /** Максимальный slippage (%) */
  max_slippage?: number
  /** Максимальная комиссия */
  max_fee_bps?: number
  /** Минимальная ликвидность */
  min_liquidity?: string
  /** Предпочтительные протоколы */
  preferred_protocols?: string[]
}

/** Сравнение котировок */
export interface SwapQuoteComparison {
  /** Запрос */
  request: SwapQuoteRequest
  /** Котировки от разных источников */
  quotes: Array<{
    source: string
    quote: SwapQuoteData
    route: SwapRouteData
  }>
  /** Лучшая котировка */
  best_quote?: SwapQuoteData | null
}

// Re-export базовых типов из web3.ts
import type {
  SwapQuoteRequest,
  SwapQuoteData,
  SwapQuoteResponse,
  SwapExecuteRequest,
  SwapRecordData,
  SwapExecuteResponse,
  QuickSwapQuoteParams,
} from './web3'

// Экспортируем для обратной совместимости
export {
  SwapQuoteRequest,
  SwapQuoteData,
  SwapQuoteResponse,
  SwapExecuteRequest,
  SwapRecordData,
  SwapExecuteResponse,
  QuickSwapQuoteParams,
}
