/**
 * Abcex Types — Расширенные типы для покупки криптовалюты за фиат
 *
 * Эти типы описывают все аспекты Abcex integration:
 * - Поддерживаемые страны и валюты
 * - Способы оплаты
 * - История заказов
 * - KYC верификация
 * - Лимиты
 */

// ============================================================================
// Supported Countries & Currencies
// ============================================================================

/** Информация о поддерживаемой стране */
export interface AbcexCountry {
  /** ISO 3166-1 alpha-2 код */
  code: string
  /** Название страны */
  name: string
  /** Флаг (emoji) */
  flag: string
  /** Доступные способы оплаты */
  payment_methods: AbcexPaymentMethod[]
  /** Валюты */
  currencies: string[]
}

/** Информация о фиатной валюте */
export interface AbcexFiatCurrency {
  /** Код валюты (USD, EUR, и т.д.) */
  code: string
  /** Символ ($, €, и т.д.) */
  symbol: string
  /** Название */
  name: string
  /** Минимальная сумма */
  min_amount: string
  /** Максимальная сумма */
  max_amount: string
  /** Доступные способы оплаты */
  payment_methods: string[]
}

/** Информация о криптовалюте */
export interface AbcexCryptoCurrency {
  /** Символ (BTC, ETH, и т.д.) */
  symbol: string
  /** Название */
  name: string
  /** Адрес контракта (если токен) */
  contract_address?: string | null
  /** Цепь */
  chain: string
  /** Минимальная сумма покупки */
  min_amount: string
  /** Точность */
  precision: number
}

/** Ответ со списком криптовалют */
export interface AbcexSupportedCryptosResponse {
  success: boolean
  cryptos?: AbcexCryptoCurrency[] | null
  error?: string | null
}

/** Ответ со списком стран */
export interface AbcexCountriesResponse {
  success: boolean
  countries?: AbcexCountry[] | null
  error?: string | null
}

// ============================================================================
// Payment Methods
// ============================================================================

/** Способы оплаты (расширенные) */
export type AbcexPaymentMethod =
  | 'credit_card'
  | 'debit_card'
  | 'sepa'
  | 'bank_transfer'
  | 'apple_pay'
  | 'google_pay'
  | 'pix'
  | 'fps'
  | 'interac'

/** Детали способа оплаты */
export interface AbcexPaymentMethodDetails {
  /** ID способа */
  id: AbcexPaymentMethod
  /** Название */
  name: string
  /** Описание */
  description: string
  /** Иконка */
  icon: string
  /** Комиссия (%) */
  fee_percent: string
  /** Время обработки */
  processing_time: string
  /** Доступен ли для страны */
  available: boolean
}

/** Ответ со способами оплаты */
export interface AbcexPaymentMethodsResponse {
  success: boolean
  methods?: AbcexPaymentMethodDetails[] | null
  error?: string | null
}

// ============================================================================
// Orders & History
// ============================================================================

/** Запрос истории заказов */
export interface AbcexOrderHistoryRequest {
  /** Смещение для пагинации */
  offset?: number
  /** Лимит */
  limit?: number
  /** Фильтр по статусу */
  status?: string
  /** Фильтр по криптовалюте */
  crypto_currency?: string
  /** Дата начала (timestamp ms) */
  from_date?: number
  /** Дата конца (timestamp ms) */
  to_date?: number
}

/** Элемент истории заказов */
export interface AbcexOrderHistoryItem {
  /** ID заказа */
  order_id: string
  /** ID котировки */
  quote_id: string
  /** Статус */
  status: string
  /** Фиатная валюта */
  fiat_currency: string
  /** Сумма фиата */
  fiat_amount: string
  /** Криптовалюта */
  crypto_currency: string
  /** Сумма криптовалюты */
  crypto_amount: string
  /** Курс */
  rate: string
  /** Комиссия */
  fee_amount: string
  /** Способ оплаты */
  payment_method: string
  /** Адрес депозита */
  deposit_address: string
  /** URL для оплаты */
  payment_url?: string | null
  /** TX hash (после выплаты) */
  tx_hash?: string | null
  /** Время создания */
  created_at: number
  /** Время обновления */
  updated_at: number
}

/** Ответ истории заказов */
export interface AbcexOrderHistoryResponse {
  success: boolean
  orders?: AbcexOrderHistoryItem[] | null
  /** Общее количество */
  total?: number
  error?: string | null
}

// ============================================================================
// KYC & Verification
// ============================================================================

/** Уровни KYC */
export type AbcexKycLevel = 'basic' | 'verified' | 'advanced'

/** Статус верификации */
export type AbcexVerificationStatus = 'pending' | 'approved' | 'rejected' | 'expired'

/** Расширенный запрос KYC */
export interface AbcexKycDetailsRequest {
  /** ID пользователя */
  user_id: string
  /** Тип документа */
  document_type?: 'passport' | 'id_card' | 'driving_license'
}

/** Информация о документе */
export interface AbcexDocumentInfo {
  /** Тип документа */
  type: string
  /** Статус верификации */
  status: AbcexVerificationStatus
  /** Дата загрузки */
  uploaded_at: number
  /** Причина отклонения */
  rejection_reason?: string | null
}

/** Расширенный ответ KYC */
export interface AbcexKycDetailsResponse {
  success: boolean
  kyc_status?: AbcexKycDataExtended | null
  error?: string | null
}

/** Расширенные данные KYC */
export interface AbcexKycDataExtended {
  /** Верифицирован ли */
  verified: boolean
  /** Уровень верификации */
  level: AbcexKycLevel
  /** Статус */
  status: AbcexVerificationStatus
  /** Дневной лимит */
  daily_limit: string
  /** Месячный лимит */
  monthly_limit: string
  /** Осталось на сегодня */
  remaining_daily: string
  /** Осталось на месяц */
  remaining_monthly: string
  /** Загруженные документы */
  documents: AbcexDocumentInfo[]
  /** Дата верификации */
  verified_at?: number | null
}

// ============================================================================
// Limits
// ============================================================================

/** Расширенные лимиты */
export interface AbcexLimitsDetails {
  /** Страна */
  country: string
  /** Минимальная сумма */
  min_amount: string
  /** Максимальная сумма */
  max_amount: string
  /** Дневной лимит */
  daily_limit: string
  /** Месячный лимит */
  monthly_limit: string
  /** Использовано сегодня */
  used_today: string
  /** Использовано за месяц */
  used_monthly: string
  /** Доступно сегодня */
  available_today: string
  /** Доступно за месяц */
  available_monthly: string
  /** Валюта лимитов */
  currency: string
}

/** Ответ с расширенными лимитами */
export interface AbcexLimitsDetailsResponse {
  success: boolean
  limits?: AbcexLimitsDetails | null
  error?: string | null
}

// ============================================================================
// Order Status & Tracking
// ============================================================================

/** Статусы заказа Abcex */
export enum AbcexOrderStatus {
  /** Ожидает оплаты */
  WaitingPayment = 'waiting_payment',
  /** Оплата получена */
  PaymentReceived = 'payment_received',
  /** В обработке */
  Processing = 'processing',
  /** Выплачивается */
  PayingOut = 'payout_in_progress',
  /** Завершён */
  Completed = 'completed',
  /** Отменён */
  Cancelled = 'cancelled',
  /** Возврат */
  Refunded = 'refunded',
  /** Ошибка */
  Failed = 'failed',
}

/** События отслеживания заказа */
export interface AbcexOrderEvent {
  /** Тип события */
  event: string
  /** Описание */
  description: string
  /** Время события */
  timestamp: number
  /** Дополнительные данные */
  metadata?: Record<string, any> | null
}

/** Ответ с отслеживанием заказа */
export interface AbcexOrderTrackingResponse {
  success: boolean
  order?: AbcexOrderDataWithEvents | null
  error?: string | null
}

/** Данные заказа с событиями */
export interface AbcexOrderDataWithEvents {
  order_id: string
  quote_id: string
  status: AbcexOrderStatus
  fiat_currency: string
  fiat_amount: string
  crypto_currency: string
  crypto_amount: string
  rate: string
  fee_amount: string
  payment_method: string
  deposit_address: string
  payment_url?: string | null
  created_at: number
  updated_at: number
  /** Хронология событий */
  events: AbcexOrderEvent[]
}

// ============================================================================
// Webhooks & Notifications
// ============================================================================

/** Запрос на настройку webhook */
export interface AbcexWebhookRequest {
  /** URL для webhook */
  url: string
  /** События для подписки */
  events: string[]
  /** Секрет для проверки */
  secret?: string
}

/** Данные webhook */
export interface AbcexWebhookData {
  /** ID webhook */
  webhook_id: string
  /** URL */
  url: string
  /** События */
  events: string[]
  /** Активен ли */
  active: boolean
  /** Время создания */
  created_at: number
}

/** Ответ webhook */
export interface AbcexWebhookResponse {
  success: boolean
  webhook?: AbcexWebhookData | null
  error?: string | null
}

// ============================================================================
// Helper Types
// ============================================================================

/** Фильтр для запроса котировок */
export interface AbcexQuoteFilter {
  /** Сортировка по курсу */
  sort_by_rate?: 'asc' | 'desc'
  /** Только доступные способы оплаты */
  only_available_methods?: boolean
  /** Страна */
  country?: string
}

/** Сравнение котировок */
export interface AbcexQuoteComparison {
  /** Запрос */
  request: AbcexQuoteRequest
  /** Полученные котировки */
  quotes: AbcexQuoteData[]
  /** Лучшая котировка */
  best_quote?: AbcexQuoteData | null
}

// Re-export базовых типов из web3.ts для удобства
import type {
  AbcexQuoteRequest,
  AbcexQuoteData,
  AbcexQuoteResponse,
  AbcexOrderRequest,
  AbcexOrderData,
  AbcexOrderResponse,
  AbcexKycRequest,
  AbcexKycData,
  AbcexKycResponse,
  AbcexLimitsRequest,
  AbcexLimitsResponse,
  AbcexQuickQuoteParams,
} from './web3'

// Экспортируем для обратной совместимости
export {
  AbcexQuoteRequest,
  AbcexQuoteData,
  AbcexQuoteResponse,
  AbcexOrderRequest,
  AbcexOrderData,
  AbcexOrderResponse,
  AbcexKycRequest,
  AbcexKycData,
  AbcexKycResponse,
  AbcexLimitsRequest,
  AbcexLimitsResponse,
  AbcexQuickQuoteParams,
}
