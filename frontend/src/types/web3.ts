//! TypeScript типы для Web3 модулей (Abcex, Bitget, 0x Protocol)
//!
//! Эти типы соответствуют Tauri командам из Rust backend.
//! Используются для типизации вызовов `invoke()` из Tauri API.

// ============================================================================
// Abcex — Покупка криптовалюты за фиат
// ============================================================================

/** Способы оплаты Abcex */
export type AbcexPaymentMethod =
  | 'credit_card'
  | 'debit_card'
  | 'sepa'
  | 'bank_transfer'
  | 'apple_pay'
  | 'google_pay';

/** Запрос котировки Abcex */
export interface AbcexQuoteRequest {
  /** Валюта (USD, EUR, GBP, RUB) */
  fiat_currency: string;
  /** Сумма в фиате */
  fiat_amount: string;
  /** Криптовалюта (BTC, ETH, USDT) */
  crypto_currency: string;
  /** Способ оплаты (опционально) */
  payment_method?: string;
  /** Код страны (ISO 3166-1 alpha-2) */
  country?: string;
}

/** Данные котировки Abcex */
export interface AbcexQuoteData {
  /** ID котировки */
  quote_id: string;
  /** Валюта фиата */
  fiat_currency: string;
  /** Сумма фиата */
  fiat_amount: string;
  /** Криптовалюта */
  crypto_currency: string;
  /** Ожидаемая сумма криптовалюты */
  crypto_amount: string;
  /** Курс обмена */
  rate: string;
  /** Сумма комиссии */
  fee_amount: string;
  /** Процент комиссии */
  fee_percent: string;
  /** Доступные способы оплаты */
  payment_methods: string[];
  /** Время действия котировки (секунды) */
  expires_in: number;
  /** Минимальная сумма */
  min_amount: string;
  /** Максимальная сумма */
  max_amount: string;
}

/** Ответ котировки Abcex */
export interface AbcexQuoteResponse {
  success: boolean;
  quote?: AbcexQuoteData | null;
  error?: string | null;
}

/** Запрос на создание заказа Abcex */
export interface AbcexOrderRequest {
  /** ID котировки */
  quote_id: string;
  /** Адрес для получения криптовалюты */
  deposit_address: string;
  /** Способ оплаты */
  payment_method: string;
  /** Email пользователя */
  user_email: string;
}

/** Данные заказа Abcex */
export interface AbcexOrderData {
  /** ID заказа */
  order_id: string;
  /** ID котировки */
  quote_id: string;
  /** Статус заказа */
  status: string;
  /** Валюта фиата */
  fiat_currency: string;
  /** Сумма фиата */
  fiat_amount: string;
  /** Криптовалюта */
  crypto_currency: string;
  /** Сумма криптовалюты */
  crypto_amount: string;
  /** Курс */
  rate: string;
  /** Комиссия */
  fee_amount: string;
  /** Способ оплаты */
  payment_method: string;
  /** Адрес депозита */
  deposit_address: string;
  /** URL для оплаты */
  payment_url?: string | null;
  /** Время создания (timestamp ms) */
  created_at: number;
}

/** Ответ на создание заказа Abcex */
export interface AbcexOrderResponse {
  success: boolean;
  order?: AbcexOrderData | null;
  error?: string | null;
}

/** Запрос KYC статуса */
export interface AbcexKycRequest {
  /** ID пользователя */
  user_id: string;
}

/** Данные KYC */
export interface AbcexKycData {
  /** Верифицирован ли */
  verified: boolean;
  /** Уровень верификации */
  level: string;
  /** Дневной лимит */
  daily_limit: string;
  /** Месячный лимит */
  monthly_limit: string;
  /** Осталось на сегодня */
  remaining_daily: string;
  /** Осталось на месяц */
  remaining_monthly: string;
}

/** Ответ KYC */
export interface AbcexKycResponse {
  success: boolean;
  kyc_status?: AbcexKycData | null;
  error?: string | null;
}

/** Запрос лимитов */
export interface AbcexLimitsRequest {
  /** Код страны */
  country: string;
}

/** Ответ с лимитами */
export interface AbcexLimitsResponse {
  success: boolean;
  limits?: Record<string, string> | null;
  error?: string | null;
}

/** Быстрая котировка */
export interface AbcexQuickQuoteParams {
  fiat_currency: string;
  fiat_amount: string;
  crypto_currency: string;
  payment_method: string;
  country?: string;
}

// ============================================================================
// Bitget — Trading (биржа)
// ============================================================================

/** Тип ордера */
export type BitgetOrderType = 'market' | 'limit';

/** Сторона ордера */
export type BitgetOrderSide = 'buy' | 'sell';

/** Запрос на размещение ордера */
export interface BitgetOrderRequest {
  /** Торговая пара (BTCUSDT, ETHUSDT) */
  symbol: string;
  /** Сторона (buy/sell) */
  side: BitgetOrderSide;
  /** Тип ордера (market/limit) */
  order_type: BitgetOrderType;
  /** Количество криптовалюты (для limit ордеров) */
  amount?: string;
  /** Сумма в quote валюте (для market ордеров) */
  quote_amount?: string;
  /** Цена (для limit ордеров) */
  price?: string;
  /** Клиентский ID ордера */
  client_order_id?: string;
}

/** Данные ордера Bitget */
export interface BitgetOrderData {
  /** ID ордера */
  order_id: string;
  /** Клиентский ID */
  client_oid?: string | null;
  /** Торговая пара */
  symbol: string;
  /** Сторона */
  side: string;
  /** Тип ордера */
  order_type: string;
  /** Количество */
  amount: string;
  /** Цена */
  price: string;
  /** Заполненное количество */
  filled_amount: string;
  /** Средняя цена */
  average_price: string;
  /** Статус */
  status: string;
  /** Комиссия */
  fee: string;
  /** Валюта комиссии */
  fee_currency: string;
  /** Время создания (timestamp ms) */
  created_at: number;
}

/** Ответ на размещение ордера */
export interface BitgetOrderResponse {
  success: boolean;
  order?: BitgetOrderData | null;
  error?: string | null;
}

/** Запрос статуса ордера */
export interface BitgetOrderStatusRequest {
  /** Торговая пара */
  symbol: string;
  /** ID ордера */
  order_id: string;
}

/** Запрос на отмену ордера */
export interface BitgetCancelRequest {
  /** Торговая пара */
  symbol: string;
  /** ID ордера */
  order_id: string;
}

/** Запрос баланса */
export interface BitgetBalanceRequest {
  /** Валюта (USDT, BTC, ETH) */
  currency: string;
}

/** Данные баланса */
export interface BitgetBalanceData {
  /** Доступный баланс */
  available_balance: string;
  /** Заблокированный баланс */
  frozen_balance: string;
  /** Общий баланс */
  total_balance: string;
  /** Валюта */
  currency: string;
}

/** Ответ баланса */
export interface BitgetBalanceResponse {
  success: boolean;
  balance?: BitgetBalanceData | null;
  error?: string | null;
}

/** Запрос рыночной цены */
export interface BitgetMarketPriceRequest {
  /** Торговая пара */
  symbol: string;
}

/** Данные рыночной цены */
export interface BitgetPriceData {
  /** Торговая пара */
  symbol: string;
  /** Текущая цена */
  price: string;
  /** Максимум 24ч */
  high_24h: string;
  /** Минимум 24ч */
  low_24h: string;
  /** Объём 24ч */
  volume_24h: string;
  /** Изменение 24ч */
  change_24h: string;
  /** Timestamp */
  timestamp: number;
}

/** Ответ рыночной цены */
export interface BitgetMarketPriceResponse {
  success: boolean;
  price?: BitgetPriceData | null;
  error?: string | null;
}

// ============================================================================
// 0x Protocol — DeFi Swap (обмен токенов)
// ============================================================================

/** Запрос котировки 0x Protocol */
export interface SwapQuoteRequest {
  /** ID цепи (1=Ethereum, 137=Polygon, и т.д.) */
  chain_id: number;
  /** Адрес токена продажи */
  sell_token: string;
  /** Адрес токена покупки */
  buy_token: string;
  /** Сумма продажи (в wei/минимальных единицах) */
  sell_amount: string;
  /** Сумма покупки (опционально) */
  buy_amount?: string;
  /** Slippage в basis points (100 = 1%) */
  slippage_bps?: number;
  /** Адрес кошелька пользователя */
  taker_address: string;
}

/** Данные котировки 0x */
export interface SwapQuoteData {
  /** Токен продажи */
  sell_token: string;
  /** Токен покупки */
  buy_token: string;
  /** Сумма продажи */
  sell_amount: string;
  /** Сумма покупки */
  buy_amount: string;
  /** Цена */
  price: string;
  /** Оценка газа */
  gas_estimate: string;
  /** Комиссия в basis points */
  fee_bps: number;
  /** Адрес для отправки транзакции */
  to: string;
  /** Данные транзакции (hex) */
  data: string;
  /** Значение (ETH для gas) */
  value: string;
}

/** Ответ котировки 0x */
export interface SwapQuoteResponse {
  success: boolean;
  quote?: SwapQuoteData | null;
  error?: string | null;
}

/** Запрос на обмен */
export interface SwapExecuteRequest {
  /** ID цепи */
  chain_id: number;
  /** Токен продажи */
  sell_token: string;
  /** Токен покупки */
  buy_token: string;
  /** Сумма продажи */
  sell_amount: string;
  /** Адрес пользователя */
  taker_address: string;
  /** Slippage в basis points */
  slippage_bps?: number;
}

/** Данные записи обмена */
export interface SwapRecordData {
  /** ID записи */
  id: string;
  /** Цепь */
  chain: string;
  /** Токен продажи */
  from_token: string;
  /** Токен покупки */
  to_token: string;
  /** Сумма продажи */
  from_amount: string;
  /** Сумма покупки */
  to_amount: string;
  /** Цена */
  price: string;
  /** Оценка газа */
  gas_estimate: string;
  /** Комиссия */
  fee_bps: number;
  /** Статус */
  status: string;
}

/** Ответ обмена */
export interface SwapExecuteResponse {
  success: boolean;
  swap_record?: SwapRecordData | null;
  error?: string | null;
}

/** Быстрая котировка */
export interface QuickSwapQuoteParams {
  sell_token: string;
  buy_token: string;
  sell_amount: string;
  decimals: number;
  chain_id: number;
}

// ============================================================================
// Общие утилиты
// ============================================================================

/** Поддерживаемые блокчейны */
export enum ChainId {
  Ethereum = 1,
  Polygon = 137,
  Arbitrum = 42161,
  Base = 8453,
  Optimism = 10,
  BSC = 56,
}

/** Названия блокчейнов */
export const CHAIN_NAMES: Record<number, string> = {
  [ChainId.Ethereum]: 'Ethereum Mainnet',
  [ChainId.Polygon]: 'Polygon',
  [ChainId.Arbitrum]: 'Arbitrum One',
  [ChainId.Base]: 'Base',
  [ChainId.Optimism]: 'Optimism',
  [ChainId.BSC]: 'BNB Smart Chain',
};

/** Символы нативных токенов */
export const CHAIN_SYMBOLS: Record<number, string> = {
  [ChainId.Ethereum]: 'ETH',
  [ChainId.Polygon]: 'MATIC',
  [ChainId.Arbitrum]: 'ETH',
  [ChainId.Base]: 'ETH',
  [ChainId.Optimism]: 'ETH',
  [ChainId.BSC]: 'BNB',
};
