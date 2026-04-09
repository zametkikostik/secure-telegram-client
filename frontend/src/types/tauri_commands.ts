/**
 * Tauri Command Constants — Константы имён Tauri команд
 *
 * Эти константы используются для вызова `invoke()` и помогают избежать опечаток.
 * Группируются по модулям: Abcex, Bitget, 0x Protocol, Wallet.
 *
 * Usage:
 * ```ts
 * import { TAURI_CMD_ABCEX_GET_QUOTE } from './tauri_commands'
 *
 * invoke(TAURI_CMD_ABCEX_GET_QUOTE, { request })
 * ```
 */

// ============================================================================
// Abcex Commands
// ============================================================================

/** Получить котировку для покупки */
export const TAURI_CMD_ABCEX_GET_QUOTE = 'abcex_get_quote'

/** Создать заказ на покупку */
export const TAURI_CMD_ABCEX_CREATE_ORDER = 'abcex_create_order'

/** Проверить статус заказа */
export const TAURI_CMD_ABCEX_GET_ORDER_STATUS = 'abcex_get_order_status'

/** Проверить KYC статус */
export const TAURI_CMD_ABCEX_CHECK_KYC = 'abcex_check_kyc'

/** Получить список криптовалют */
export const TAURI_CMD_ABCEX_GET_SUPPORTED_CRYPTOS = 'abcex_get_supported_cryptos'

/** Получить лимиты для страны */
export const TAURI_CMD_ABCEX_GET_LIMITS = 'abcex_get_limits'

/** Быстрая котировка */
export const TAURI_CMD_ABCEX_QUICK_QUOTE = 'abcex_quick_quote'

/** Рассчитать комиссию */
export const TAURI_CMD_ABCEX_CALCULATE_FEE = 'abcex_calculate_fee'

// ============================================================================
// Bitget Commands
// ============================================================================

/** Разместить ордер */
export const TAURI_CMD_BITGET_PLACE_ORDER = 'bitget_place_order'

/** Проверить статус ордера */
export const TAURI_CMD_BITGET_GET_ORDER_STATUS = 'bitget_get_order_status'

/** Отменить ордер */
export const TAURI_CMD_BITGET_CANCEL_ORDER = 'bitget_cancel_order'

/** Получить баланс */
export const TAURI_CMD_BITGET_GET_BALANCE = 'bitget_get_balance'

/** Получить рыночную цену */
export const TAURI_CMD_BITGET_GET_MARKET_PRICE = 'bitget_get_market_price'

/** Получить список торговых пар */
export const TAURI_CMD_BITGET_GET_SYMBOLS = 'bitget_get_symbols'

/** Быстрая покупка */
export const TAURI_CMD_BITGET_QUICK_BUY = 'bitget_quick_buy'

/** Быстрая продажа */
export const TAURI_CMD_BITGET_QUICK_SELL = 'bitget_quick_sell'

/** Рассчитать комиссию */
export const TAURI_CMD_BITGET_CALCULATE_FEE = 'bitget_calculate_fee'

// ============================================================================
// 0x Protocol (Swap) Commands
// ============================================================================

/** Получить котировку для обмена */
export const TAURI_CMD_GET_SWAP_QUOTE = 'get_swap_quote'

/** Выполнить обмен */
export const TAURI_CMD_EXECUTE_SWAP = 'execute_swap'

/** Рассчитать slippage */
export const TAURI_CMD_CALCULATE_SLIPPAGE = 'calculate_slippage'

/** Получить адрес для allowance */
export const TAURI_CMD_GET_ALLOWANCE_TARGET = 'get_allowance_target'

/** Быстрая котировка */
export const TAURI_CMD_QUICK_SWAP_QUOTE = 'quick_swap_quote'

// ============================================================================
// Wallet Commands
// ============================================================================

/** Генерировать кошелёк */
export const TAURI_CMD_WALLET_GENERATE = 'wallet_generate'

/** Импортировать приватный ключ */
export const TAURI_CMD_WALLET_IMPORT_PRIVATE_KEY = 'wallet_import_private_key'

/** Получить адрес кошелька */
export const TAURI_CMD_WALLET_GET_ADDRESS = 'wallet_get_address'

/** Подписать сообщение */
export const TAURI_CMD_WALLET_SIGN_MESSAGE = 'wallet_sign_message'

/** Получить баланс */
export const TAURI_CMD_WALLET_GET_BALANCE = 'wallet_get_balance'

/** Отключить кошелёк */
export const TAURI_CMD_WALLET_DISCONNECT = 'wallet_disconnect'

/** Проверить загружен ли кошелёк */
export const TAURI_CMD_WALLET_IS_LOADED = 'wallet_is_loaded'

/** Abcex котировка через wallet */
export const TAURI_CMD_WALLET_ABCEX_QUOTE = 'wallet_abcex_quote'

/** Bitget покупка через wallet */
export const TAURI_CMD_WALLET_BITGET_BUY = 'wallet_bitget_buy'

/** Bitget продажа через wallet */
export const TAURI_CMD_WALLET_BITGET_SELL = 'wallet_bitget_sell'

/** Bitget баланс через wallet */
export const TAURI_CMD_WALLET_BITGET_BALANCE = 'wallet_bitget_balance'

// ============================================================================
// ENS Commands
// ============================================================================

/** Разрешить ENS имя */
export const TAURI_CMD_ENS_RESOLVE = 'ens_resolve'

/** Обратный поиск ENS */
export const TAURI_CMD_ENS_REVERSE = 'ens_reverse'

/** Проверить является ли строка ENS именем */
export const TAURI_CMD_ENS_IS_ENS_NAME = 'ens_is_ens_name'

/** Форматировать для отображения */
export const TAURI_CMD_ENS_FORMAT_DISPLAY = 'ens_format_display'

// ============================================================================
// Token Commands
// ============================================================================

/** Получить список токенов */
export const TAURI_CMD_TOKENS_LIST = 'tokens_list'

/** Получить популярные токены */
export const TAURI_CMD_TOKENS_POPULAR = 'tokens_popular'

/** Поиск токенов */
export const TAURI_CMD_TOKENS_SEARCH = 'tokens_search'

/** Получить цену токена */
export const TAURI_CMD_TOKEN_PRICE = 'token_price'

/** Получить цены нескольких токенов */
export const TAURI_CMD_TOKEN_BATCH_PRICE = 'token_batch_price'

// ============================================================================
// Transaction Commands
// ============================================================================

/** Получить историю обменов */
export const TAURI_CMD_SWAP_HISTORY = 'swap_history'

/** Получить историю заказов Abcex */
export const TAURI_CMD_ABCEX_ORDER_HISTORY = 'abcex_order_history'

/** Получить историю ордеров Bitget */
export const TAURI_CMD_BITGET_ORDER_HISTORY = 'bitget_order_history'

/** Получить открытые ордера Bitget */
export const TAURI_CMD_BITGET_OPEN_ORDERS = 'bitget_open_orders'

// ============================================================================
// Gas Commands
// ============================================================================

/** Получить цены газа */
export const TAURI_CMD_GAS_PRICES = 'gas_prices'

/** Оценить газ для транзакции */
export const TAURI_CMD_GAS_ESTIMATE = 'gas_estimate'

// ============================================================================
// Network Commands
// ============================================================================

/** Получить информацию о сети */
export const TAURI_CMD_NETWORK_INFO = 'network_info'

/** Проверить здоровье сервисов */
export const TAURI_CMD_HEALTH_CHECK = 'health_check'

/** Получить конфигурацию Web3 */
export const TAURI_CMD_WEB3_CONFIG = 'web3_config'

// ============================================================================
// Helper Maps
// ============================================================================

/** Маппинг всех команд для удобства */
export const TAURI_COMMANDS = {
  // Abcex
  ABCEX_GET_QUOTE: TAURI_CMD_ABCEX_GET_QUOTE,
  ABCEX_CREATE_ORDER: TAURI_CMD_ABCEX_CREATE_ORDER,
  ABCEX_GET_ORDER_STATUS: TAURI_CMD_ABCEX_GET_ORDER_STATUS,
  ABCEX_CHECK_KYC: TAURI_CMD_ABCEX_CHECK_KYC,
  ABCEX_GET_SUPPORTED_CRYPTOS: TAURI_CMD_ABCEX_GET_SUPPORTED_CRYPTOS,
  ABCEX_GET_LIMITS: TAURI_CMD_ABCEX_GET_LIMITS,
  ABCEX_QUICK_QUOTE: TAURI_CMD_ABCEX_QUICK_QUOTE,
  ABCEX_CALCULATE_FEE: TAURI_CMD_ABCEX_CALCULATE_FEE,

  // Bitget
  BITGET_PLACE_ORDER: TAURI_CMD_BITGET_PLACE_ORDER,
  BITGET_GET_ORDER_STATUS: TAURI_CMD_BITGET_GET_ORDER_STATUS,
  BITGET_CANCEL_ORDER: TAURI_CMD_BITGET_CANCEL_ORDER,
  BITGET_GET_BALANCE: TAURI_CMD_BITGET_GET_BALANCE,
  BITGET_GET_MARKET_PRICE: TAURI_CMD_BITGET_GET_MARKET_PRICE,
  BITGET_GET_SYMBOLS: TAURI_CMD_BITGET_GET_SYMBOLS,
  BITGET_QUICK_BUY: TAURI_CMD_BITGET_QUICK_BUY,
  BITGET_QUICK_SELL: TAURI_CMD_BITGET_QUICK_SELL,
  BITGET_CALCULATE_FEE: TAURI_CMD_BITGET_CALCULATE_FEE,

  // Swap
  GET_SWAP_QUOTE: TAURI_CMD_GET_SWAP_QUOTE,
  EXECUTE_SWAP: TAURI_CMD_EXECUTE_SWAP,
  CALCULATE_SLIPPAGE: TAURI_CMD_CALCULATE_SLIPPAGE,
  GET_ALLOWANCE_TARGET: TAURI_CMD_GET_ALLOWANCE_TARGET,
  QUICK_SWAP_QUOTE: TAURI_CMD_QUICK_SWAP_QUOTE,

  // Wallet
  WALLET_GENERATE: TAURI_CMD_WALLET_GENERATE,
  WALLET_IMPORT_PRIVATE_KEY: TAURI_CMD_WALLET_IMPORT_PRIVATE_KEY,
  WALLET_GET_ADDRESS: TAURI_CMD_WALLET_GET_ADDRESS,
  WALLET_SIGN_MESSAGE: TAURI_CMD_WALLET_SIGN_MESSAGE,
  WALLET_GET_BALANCE: TAURI_CMD_WALLET_GET_BALANCE,
  WALLET_DISCONNECT: TAURI_CMD_WALLET_DISCONNECT,
  WALLET_IS_LOADED: TAURI_CMD_WALLET_IS_LOADED,
  WALLET_ABCEX_QUOTE: TAURI_CMD_WALLET_ABCEX_QUOTE,
  WALLET_BITGET_BUY: TAURI_CMD_WALLET_BITGET_BUY,
  WALLET_BITGET_SELL: TAURI_CMD_WALLET_BITGET_SELL,
  WALLET_BITGET_BALANCE: TAURI_CMD_WALLET_BITGET_BALANCE,

  // ENS
  ENS_RESOLVE: TAURI_CMD_ENS_RESOLVE,
  ENS_REVERSE: TAURI_CMD_ENS_REVERSE,
  ENS_IS_ENS_NAME: TAURI_CMD_ENS_IS_ENS_NAME,
  ENS_FORMAT_DISPLAY: TAURI_CMD_ENS_FORMAT_DISPLAY,

  // Tokens
  TOKENS_LIST: TAURI_CMD_TOKENS_LIST,
  TOKENS_POPULAR: TAURI_CMD_TOKENS_POPULAR,
  TOKENS_SEARCH: TAURI_CMD_TOKENS_SEARCH,
  TOKEN_PRICE: TAURI_CMD_TOKEN_PRICE,
  TOKEN_BATCH_PRICE: TAURI_CMD_TOKEN_BATCH_PRICE,

  // Transactions
  SWAP_HISTORY: TAURI_CMD_SWAP_HISTORY,
  ABCEX_ORDER_HISTORY: TAURI_CMD_ABCEX_ORDER_HISTORY,
  BITGET_ORDER_HISTORY: TAURI_CMD_BITGET_ORDER_HISTORY,
  BITGET_OPEN_ORDERS: TAURI_CMD_BITGET_OPEN_ORDERS,

  // Gas
  GAS_PRICES: TAURI_CMD_GAS_PRICES,
  GAS_ESTIMATE: TAURI_CMD_GAS_ESTIMATE,

  // Network
  NETWORK_INFO: TAURI_CMD_NETWORK_INFO,
  HEALTH_CHECK: TAURI_CMD_HEALTH_CHECK,
  WEB3_CONFIG: TAURI_CMD_WEB3_CONFIG,
} as const

/** Тип всех команд */
export type TauriCommandKey = keyof typeof TAURI_COMMANDS

/** Получить имя команды */
export function getTauriCommandName(key: TauriCommandKey): string {
  return TAURI_COMMANDS[key]
}
