/**
 * Bitget Types — Расширенные типы для trading на бирже Bitget
 *
 * Эти типы описывают все аспекты Bitget integration:
 * - Информация об аккаунте
 * - История ордеров
 * - Свечные графики
 * - WebSocket сообщения
 * - Позиции (фьючерсы)
 */

// ============================================================================
// Account Information
// ============================================================================

/** Типы аккаунтов */
export type BitgetAccountType = 'spot' | 'mix_contract' | 'usdt_future' | 'coin_future'

/** Общая информация об аккаунте */
export interface BitgetAccountInfo {
  /** ID аккаунта */
  account_id: string
  /** Тип аккаунта */
  account_type: BitgetAccountType
  /** Общий баланс в USD */
  total_usd: string
  /** Общий баланс в BTC */
  total_btc: string
  /** Общий баланс */
  total_equity: string
  /** Доступный баланс */
  available_balance: string
  /** Заблокированный баланс */
  frozen_balance: string
  /** Нереализованный P&L */
  unrealized_pnl: string
  /** Реализованный P&L */
  realized_pnl: string
  /** Маржа */
  margin: string
  /** Время обновления */
  update_time: number
}

/** Ответ с информацией об аккаунте */
export interface BitgetAccountInfoResponse {
  success: boolean
  account?: BitgetAccountInfo | null
  error?: string | null
}

/** Баланс по отдельной валюте */
export interface BitgetCurrencyBalance {
  /** Валюта */
  currency: string
  /** Доступный баланс */
  available: string
  /** Заблокированный баланс */
  frozen: string
  /** Баланс ордера */
  orderFrozen: string
  /** Баланс маржи */
  marginFrozen: string
  /** Unrealized P&L */
  unrealizedPnl: string
  /** Доступно для вывода */
  availableWithdraw: string
}

/** Ответ с балансами всех валют */
export interface BitgetBalancesResponse {
  success: boolean
  balances?: BitgetCurrencyBalance[] | null
  error?: string | null
}

// ============================================================================
// Order History
// ============================================================================

/** Запрос истории ордеров */
export interface BitgetOrderHistoryRequest {
  /** Торговая пара */
  symbol: string
  /** Тип ордера */
  order_type?: BitgetOrderType
  /** Сторона */
  side?: BitgetOrderSide
  /** Статус */
  status?: BitgetOrderStatus
  /** Время начала (timestamp ms) */
  startTime?: number
  /** Время конца (timestamp ms) */
  endTime?: number
  /** Смещение */
  offset?: number
  /** Лимит */
  limit?: number
}

/** Статусы ордеров Bitget */
export enum BitgetOrderStatus {
  /** Инициализирован */
  Init = 'init',
  /** В очереди */
  Queued = 'queued',
  /** Не отправлен */
  NotSubmitted = 'not_submitted',
  /** Отправлен */
  Submitted = 'submitted',
  /** Частично исполнен */
  PartiallyFilled = 'partially_filled',
  /** Полностью исполнен */
  Filled = 'filled',
  /** Отменён */
  Cancelled = 'cancelled',
  /** Отмена не удалась */
  CancelFailed = 'cancel_failed',
  /** Ошибка */
  Error = 'error',
}

/** Ответ истории ордеров */
export interface BitgetOrderHistoryResponse {
  success: boolean
  orders?: BitgetOrderData[] | null
  /** Общее количество */
  total?: number
  /** Есть ли ещё */
  has_next?: boolean
  error?: string | null
}

/** Запрос открытых ордеров */
export interface BitgetOpenOrdersRequest {
  /** Торговая пара */
  symbol?: string
  /** Смещение */
  offset?: number
  /** Лимит */
  limit?: number
}

/** Ответ открытых ордеров */
export interface BitgetOpenOrdersResponse {
  success: boolean
  orders?: BitgetOrderData[] | null
  error?: string | null
}

/** Пакетная отмена ордеров */
export interface BitgetBatchCancelRequest {
  /** Торговая пара */
  symbol: string
  /** IDs ордеров */
  order_ids: string[]
}

/** Ответ пакетной отмены */
export interface BitgetBatchCancelResponse {
  success: boolean
  results?: Array<{
    order_id: string
    success: boolean
    error?: string | null
  }> | null
  error?: string | null
}

// ============================================================================
// Market Data & Tickers
// ============================================================================

/** Запрос тикеров */
export interface BitgetTickerRequest {
  /** Торговая пара */
  symbol?: string
  /** Список пар */
  symbols?: string[]
}

/** Данные тикера */
export interface BitgetTickerData {
  /** Торговая пара */
  symbol: string
  /** Последняя цена */
  last: string
  /** Цена открытия 24ч */
  open: string
  /** Максимум 24ч */
  high_24h: string
  /** Минимум 24ч */
  low_24h: string
  /** Объём 24ч (base) */
  volume_24h: string
  /** Объём 24ч (quote) */
  quote_volume_24h: string
  /** Изменение 24ч (%) */
  change_24h: string
  /** Изменение 24ч (absolute) */
  change_24h_absolute: string
  /** Бид цена */
  bid: string
  /** Аск цена */
  ask: string
  /** Timestamp */
  timestamp: number
}

/** Ответ тикеров */
export interface BitgetTickerResponse {
  success: boolean
  tickers?: BitgetTickerData[] | null
  error?: string | null
}

/** Информация о торговой паре */
export interface BitgetSymbolInfo {
  /** Торговая пара */
  symbol: string
  /** Базовая валюта */
  base_currency: string
  /** Котируемая валюта */
  quote_currency: string
  /** Минимальный размер ордера */
  min_trade_amount: string
  /** Точность цены */
  price_precision: number
  /** Точность количества */
  quantity_precision: number
  /** Статус торговли */
  status: 'online' | 'offline' | 'suspend'
  /** Минимальный шаг цены */
  price_step: string
}

/** Ответ со списком торговых пар */
export interface BitgetSymbolsResponse {
  success: boolean
  symbols?: BitgetSymbolInfo[] | null
  error?: string | null
}

// ============================================================================
// Candlestick / K-Line Data
// ============================================================================

/** Период свечей */
export type BitgetCandlePeriod =
  | '1min'
  | '3min'
  | '5min'
  | '15min'
  | '30min'
  | '1H'
  | '2H'
  | '4H'
  | '6H'
  | '12H'
  | '1D'
  | '3D'
  | '1W'
  | '1M'

/** Запрос свечей */
export interface BitgetCandlestickRequest {
  /** Торговая пара */
  symbol: string
  /** Период */
  period: BitgetCandlePeriod
  /** Время начала (timestamp ms) */
  startTime?: number
  /** Время конца (timestamp ms) */
  endTime?: number
  /** Лимит */
  limit?: number
}

/** Данные свечи */
export interface BitgetCandlestickData {
  /** Timestamp начала свечи */
  timestamp: number
  /** Цена открытия */
  open: string
  /** Максимум */
  high: string
  /** Минимум */
  low: string
  /** Цена закрытия */
  close: string
  /** Объём (base) */
  volume: string
  /** Объём (quote) */
  quoteVolume: string
}

/** Ответ свечей */
export interface BitgetCandlestickResponse {
  success: boolean
  candles?: BitgetCandlestickData[] | null
  error?: string | null
}

// ============================================================================
// Trading Pairs & Symbols
// ============================================================================

/** Запрос торговых пар */
export interface BitgetSymbolsRequest {
  /** Тип продукта */
  product_type?: 'umcbl' | 'dmcbl' | 'sumcbl' | 'sp'
}

/** Ответ торговых пар */
export interface BitgetSymbolsResponse {
  success: boolean
  symbols?: BitgetSymbolInfo[] | null
  error?: string | null
}

// ============================================================================
// WebSocket
// ============================================================================

/** Типы WebSocket сообщений */
export type BitgetWebSocketMessage =
  | BitgetWebSocketTickerMessage
  | BitgetWebSocketCandleMessage
  | BitgetWebSocketTradeMessage
  | BitgetWebSocketDepthMessage
  | BitgetWebSocketAccountMessage

/** Базовое WebSocket сообщение */
export interface BitgetWebSocketBaseMessage {
  /** Тип канала */
  channel: string
  /** Действие */
  action: 'subscribe' | 'unsubscribe' | 'push'
  /** Данные */
  data: any
  /** Timestamp */
  timestamp: number
}

/** WebSocket сообщение тикера */
export interface BitgetWebSocketTickerMessage extends BitgetWebSocketBaseMessage {
  channel: 'ticker'
  data: BitgetTickerData
}

/** WebSocket сообщение свечей */
export interface BitgetWebSocketCandleMessage extends BitgetWebSocketBaseMessage {
  channel: 'candle'
  data: BitgetCandlestickData
}

/** WebSocket сообщение сделок */
export interface BitgetWebSocketTradeMessage extends BitgetWebSocketBaseMessage {
  channel: 'trade'
  data: {
    symbol: string
    side: string
    size: string
    price: string
    timestamp: number
    trade_id: string
  }
}

/** WebSocket message стакана */
export interface BitgetWebSocketDepthMessage extends BitgetWebSocketBaseMessage {
  channel: 'depth'
  data: {
    symbol: string
    bids: Array<[string, string]> // [price, size]
    asks: Array<[string, string]>
    timestamp: number
  }
}

/** WebSocket сообщение аккаунта */
export interface BitgetWebSocketAccountMessage extends BitgetWebSocketBaseMessage {
  channel: 'account'
  data: {
    currency: string
    available: string
    frozen: string
    change: string
    timestamp: number
  }
}

/** Запрос подписки WebSocket */
export interface BitgetWebSocketSubscribeRequest {
  /** Каналы */
  channels: string[]
  /** Торговые пары */
  symbols: string[]
  /** Callback при получении данных */
  callback?: (message: BitgetWebSocketMessage) => void
}

/** Ответ подписки WebSocket */
export interface BitgetWebSocketSubscribeResponse {
  success: boolean
  subscription_id?: string | null
  error?: string | null
}

// ============================================================================
// Futures / Positions (Optional)
// ============================================================================

/** Типы маржи */
export type BitgetMarginMode = 'crossed' | 'fixed'

/** Типы позиции */
export type BitgetPositionSide = 'long' | 'short'

/** Данные позиции */
export interface BitgetPositionData {
  /** Торговая пара */
  symbol: string
  /** Маржа режим */
  margin_mode: BitgetMarginMode
  /** Сторона позиции */
  position_side: BitgetPositionSide
  /** Количество */
  available: string
  /** Средняя цена входа */
  average_open_price: string
  /** Нереализованный P&L */
  unrealized_pnl: string
  /** Реализованный P&L */
  realized_pnl: string
  /** Маржа */
  margin: string
  /** Плечо */
  leverage: string
  /** Ликвидационная цена */
  liquidation_price: string
  /** Время обновления */
  update_time: number
}

/** Ответ с позициями */
export interface BitgetPositionsResponse {
  success: boolean
  positions?: BitgetPositionData[] | null
  error?: string | null
}

/** Запрос изменения плеча */
export interface BitgetLeverageRequest {
  /** Торговая пара */
  symbol: string
  /** Плечо */
  leverage: string
  /** Режим маржи */
  margin_mode: BitgetMarginMode
  /** Сторона */
  position_side?: BitgetPositionSide
}

/** Ответ изменения плеча */
export interface BitgetLeverageResponse {
  success: boolean
  leverage?: string | null
  error?: string | null
}

// ============================================================================
// Helper Types
// ============================================================================

/** Фильтр для запроса ордеров */
export interface BitgetOrderFilter {
  /** Сортировка */
  sort_by?: 'timestamp' | 'price' | 'amount'
  /** Порядок сортировки */
  sort_order?: 'asc' | 'desc'
  /** Фильтр по статусу */
  status_filter?: BitgetOrderStatus[]
  /** Фильтр по стороне */
  side_filter?: BitgetOrderSide[]
}

/** Статистика торговли */
export interface BitgetTradingStats {
  /** Всего ордеров */
  total_orders: number
  /** Исполнено */
  filled_orders: number
  /** Отменено */
  cancelled_orders: number
  /** Объём торговли 24ч */
  volume_24h: string
  /** Прибыль/убыток 24ч */
  pnl_24h: string
  /** Win rate */
  win_rate: string
}

/** Ответ статистики */
export interface BitgetTradingStatsResponse {
  success: boolean
  stats?: BitgetTradingStats | null
  error?: string | null
}

// Re-export базовых типов из web3.ts
import type {
  BitgetOrderType,
  BitgetOrderSide,
  BitgetOrderRequest,
  BitgetOrderData,
  BitgetOrderResponse,
  BitgetOrderStatusRequest,
  BitgetCancelRequest,
  BitgetBalanceRequest,
  BitgetBalanceData,
  BitgetBalanceResponse,
  BitgetMarketPriceRequest,
  BitgetPriceData,
  BitgetMarketPriceResponse,
} from './web3'

// Экспортируем для обратной совместимости
export {
  BitgetOrderType,
  BitgetOrderSide,
  BitgetOrderRequest,
  BitgetOrderData,
  BitgetOrderResponse,
  BitgetOrderStatusRequest,
  BitgetCancelRequest,
  BitgetBalanceRequest,
  BitgetBalanceData,
  BitgetBalanceResponse,
  BitgetMarketPriceRequest,
  BitgetPriceData,
  BitgetMarketPriceResponse,
}
