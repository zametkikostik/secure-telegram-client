/**
 * Web3 Types — Экспорт всех Web3 типов
 *
 * Файлы:
 * - web3.ts: Базовые типы для Abcex, Bitget, 0x Protocol
 * - abcex_types.ts: Расширенные типы Abcex
 * - bitget_types.ts: Расширенные типы Bitget
 * - swap_types.ts: Расширенные типы 0x Protocol
 * - web3_common_types.ts: Общие типы (ошибки, статусы, API)
 * - tauri_commands.ts: Константы Tauri команд
 *
 * Usage:
 * ```ts
 * import { AbcexQuoteRequest, BitgetOrderRequest, SwapQuoteRequest } from '@/types/web3'
 * import { AbcexOrderHistoryItem } from '@/types/abcex_types'
 * import { BitgetAccountInfo } from '@/types/bitget_types'
 * import { TokenInfo } from '@/types/swap_types'
 * import { Web3Error, TransactionStatus } from '@/types/web3_common_types'
 * import { TAURI_COMMANDS } from '@/types/tauri_commands'
 * ```
 */

// ============================================================================
// Base Types (from web3.ts)
// ============================================================================

export {
  // Abcex
  type AbcexPaymentMethod,
  type AbcexQuoteRequest,
  type AbcexQuoteData,
  type AbcexQuoteResponse,
  type AbcexOrderRequest,
  type AbcexOrderData,
  type AbcexOrderResponse,
  type AbcexKycRequest,
  type AbcexKycData,
  type AbcexKycResponse,
  type AbcexLimitsRequest,
  type AbcexLimitsResponse,
  type AbcexQuickQuoteParams,
  // Bitget
  type BitgetOrderType,
  type BitgetOrderSide,
  type BitgetOrderRequest,
  type BitgetOrderData,
  type BitgetOrderResponse,
  type BitgetOrderStatusRequest,
  type BitgetCancelRequest,
  type BitgetBalanceRequest,
  type BitgetBalanceData,
  type BitgetBalanceResponse,
  type BitgetMarketPriceRequest,
  type BitgetPriceData,
  type BitgetMarketPriceResponse,
  // 0x Protocol
  type SwapQuoteRequest,
  type SwapQuoteData,
  type SwapQuoteResponse,
  type SwapExecuteRequest,
  type SwapRecordData,
  type SwapExecuteResponse,
  type QuickSwapQuoteParams,
  // Common
  ChainId,
  CHAIN_NAMES,
  CHAIN_SYMBOLS,
} from './web3'

// ============================================================================
// Abcex Extended Types
// ============================================================================

export {
  type AbcexCountry,
  type AbcexFiatCurrency,
  type AbcexCryptoCurrency,
  type AbcexSupportedCryptosResponse,
  type AbcexCountriesResponse,
  type AbcexPaymentMethodDetails,
  type AbcexPaymentMethodsResponse,
  type AbcexOrderHistoryRequest,
  type AbcexOrderHistoryItem,
  type AbcexOrderHistoryResponse,
  type AbcexKycLevel,
  type AbcexVerificationStatus,
  type AbcexKycDetailsRequest,
  type AbcexDocumentInfo,
  type AbcexKycDetailsResponse,
  type AbcexKycDataExtended,
  type AbcexLimitsDetails,
  type AbcexLimitsDetailsResponse,
  AbcexOrderStatus,
  type AbcexOrderEvent,
  type AbcexOrderTrackingResponse,
  type AbcexOrderDataWithEvents,
  type AbcexWebhookRequest,
  type AbcexWebhookData,
  type AbcexWebhookResponse,
  type AbcexQuoteFilter,
  type AbcexQuoteComparison,
} from './abcex_types'

// ============================================================================
// Bitget Extended Types
// ============================================================================

export {
  type BitgetAccountType,
  type BitgetAccountInfo,
  type BitgetAccountInfoResponse,
  type BitgetCurrencyBalance,
  type BitgetBalancesResponse,
  type BitgetOrderHistoryRequest,
  BitgetOrderStatus,
  type BitgetOrderHistoryResponse,
  type BitgetOpenOrdersRequest,
  type BitgetOpenOrdersResponse,
  type BitgetBatchCancelRequest,
  type BitgetBatchCancelResponse,
  type BitgetTickerRequest,
  type BitgetTickerData,
  type BitgetTickerResponse,
  type BitgetSymbolInfo,
  type BitgetSymbolsResponse,
  type BitgetCandlePeriod,
  type BitgetCandlestickRequest,
  type BitgetCandlestickData,
  type BitgetCandlestickResponse,
  type BitgetWebSocketMessage,
  type BitgetWebSocketBaseMessage,
  type BitgetWebSocketTickerMessage,
  type BitgetWebSocketCandleMessage,
  type BitgetWebSocketTradeMessage,
  type BitgetWebSocketDepthMessage,
  type BitgetWebSocketAccountMessage,
  type BitgetWebSocketSubscribeRequest,
  type BitgetWebSocketSubscribeResponse,
  type BitgetMarginMode,
  type BitgetPositionSide,
  type BitgetPositionData,
  type BitgetPositionsResponse,
  type BitgetLeverageRequest,
  type BitgetLeverageResponse,
  type BitgetOrderFilter,
  type BitgetTradingStats,
  type BitgetTradingStatsResponse,
} from './bitget_types'

// ============================================================================
// 0x Protocol Extended Types
// ============================================================================

export {
  type TokenInfo,
  type TokensListRequest,
  type TokensListResponse,
  type PopularTokensResponse,
  type TokenSearchRequest,
  type TokenSearchResponse,
  type SwapHistoryRequest,
  type SwapHistoryItem,
  SwapTransactionStatus,
  type SwapHistoryResponse,
  type ApproveTransactionRequest,
  type ApproveTransactionData,
  type ApproveTransactionResponse,
  type AllowanceRequest,
  type AllowanceResponse,
  type ApprovalCheckRequest,
  type ApprovalCheckResponse,
  type GasPriority,
  type GasPriceData,
  type GasPricesResponse,
  type GasPricesRequest,
  type SwapRouteData,
  type SwapPoolHop,
  type SwapRouteResponse,
  type ExtendedSwapQuoteData,
  type ExtendedSwapQuoteResponse,
  type TokenPriceRequest,
  type TokenPriceData,
  type TokenPriceResponse,
  type BatchTokenPriceRequest,
  type BatchTokenPriceResponse,
  type UserSwapStats,
  type UserSwapStatsResponse,
  type ProtocolStats,
  type ProtocolStatsResponse,
  type SwapQuoteFilter,
  type SwapQuoteComparison,
} from './swap_types'

// ============================================================================
// Common Types
// ============================================================================

export {
  Web3ErrorType,
  type Web3Error,
  type Web3ErrorResponse,
  type Web3ApiResponse,
  type PaginatedResponse,
  TransactionStatus,
  type TransactionInfo,
  type TransactionInfoResponse,
  WalletConnectionState,
  type WalletConnectionInfo,
  NetworkStatus,
  type NetworkInfo,
  type NetworkInfoResponse,
  type EnsResolveRequest,
  type EnsRecord,
  type EnsResolveResponse,
  type EnsReverseRequest,
  type GasEstimate,
  type GasEstimateResponse,
  type TokenBalance,
  type NativeBalance,
  type Portfolio,
  type PortfolioResponse,
  type RateLimitInfo,
  type RateLimitResponse,
  type Web3Config,
  type Web3ConfigResponse,
  type ComponentStatus,
  type Web3HealthStatus,
  type Web3HealthResponse,
} from './web3_common_types'

// ============================================================================
// Tauri Commands
// ============================================================================

export {
  // Abcex
  TAURI_CMD_ABCEX_GET_QUOTE,
  TAURI_CMD_ABCEX_CREATE_ORDER,
  TAURI_CMD_ABCEX_GET_ORDER_STATUS,
  TAURI_CMD_ABCEX_CHECK_KYC,
  TAURI_CMD_ABCEX_GET_SUPPORTED_CRYPTOS,
  TAURI_CMD_ABCEX_GET_LIMITS,
  TAURI_CMD_ABCEX_QUICK_QUOTE,
  TAURI_CMD_ABCEX_CALCULATE_FEE,
  // Bitget
  TAURI_CMD_BITGET_PLACE_ORDER,
  TAURI_CMD_BITGET_GET_ORDER_STATUS,
  TAURI_CMD_BITGET_CANCEL_ORDER,
  TAURI_CMD_BITGET_GET_BALANCE,
  TAURI_CMD_BITGET_GET_MARKET_PRICE,
  TAURI_CMD_BITGET_GET_SYMBOLS,
  TAURI_CMD_BITGET_QUICK_BUY,
  TAURI_CMD_BITGET_QUICK_SELL,
  TAURI_CMD_BITGET_CALCULATE_FEE,
  // Swap
  TAURI_CMD_GET_SWAP_QUOTE,
  TAURI_CMD_EXECUTE_SWAP,
  TAURI_CMD_CALCULATE_SLIPPAGE,
  TAURI_CMD_GET_ALLOWANCE_TARGET,
  TAURI_CMD_QUICK_SWAP_QUOTE,
  // Wallet
  TAURI_CMD_WALLET_GENERATE,
  TAURI_CMD_WALLET_IMPORT_PRIVATE_KEY,
  TAURI_CMD_WALLET_GET_ADDRESS,
  TAURI_CMD_WALLET_SIGN_MESSAGE,
  TAURI_CMD_WALLET_GET_BALANCE,
  TAURI_CMD_WALLET_DISCONNECT,
  TAURI_CMD_WALLET_IS_LOADED,
  TAURI_CMD_WALLET_ABCEX_QUOTE,
  TAURI_CMD_WALLET_BITGET_BUY,
  TAURI_CMD_WALLET_BITGET_SELL,
  TAURI_CMD_WALLET_BITGET_BALANCE,
  // ENS
  TAURI_CMD_ENS_RESOLVE,
  TAURI_CMD_ENS_REVERSE,
  TAURI_CMD_ENS_IS_ENS_NAME,
  TAURI_CMD_ENS_FORMAT_DISPLAY,
  // Tokens
  TAURI_CMD_TOKENS_LIST,
  TAURI_CMD_TOKENS_POPULAR,
  TAURI_CMD_TOKENS_SEARCH,
  TAURI_CMD_TOKEN_PRICE,
  TAURI_CMD_TOKEN_BATCH_PRICE,
  // Transactions
  TAURI_CMD_SWAP_HISTORY,
  TAURI_CMD_ABCEX_ORDER_HISTORY,
  TAURI_CMD_BITGET_ORDER_HISTORY,
  TAURI_CMD_BITGET_OPEN_ORDERS,
  // Gas
  TAURI_CMD_GAS_PRICES,
  TAURI_CMD_GAS_ESTIMATE,
  // Network
  TAURI_CMD_NETWORK_INFO,
  TAURI_CMD_HEALTH_CHECK,
  TAURI_CMD_WEB3_CONFIG,
  // Helper
  TAURI_COMMANDS,
  type TauriCommandKey,
  getTauriCommandName,
} from './tauri_commands'
