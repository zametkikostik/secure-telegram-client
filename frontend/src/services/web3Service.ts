/**
 * Web3 Service — вызов Tauri команд для обмена криптовалют
 *
 * Обёртка над Tauri `invoke()` для работы с:
 * - Abcex (покупка за фиат)
 * - Bitget (trading)
 * - 0x Protocol (DeFi swap)
 */

import { invoke } from '@tauri-apps/api/core'

// Types
import type {
  AbcexQuoteRequest,
  AbcexQuoteResponse,
  AbcexOrderRequest,
  AbcexOrderResponse,
  AbcexKycRequest,
  AbcexKycResponse,
  AbcexLimitsRequest,
  AbcexLimitsResponse,
  AbcexQuickQuoteParams,
  BitgetOrderRequest,
  BitgetOrderResponse,
  BitgetOrderStatusRequest,
  BitgetCancelRequest,
  BitgetBalanceRequest,
  BitgetBalanceResponse,
  BitgetMarketPriceRequest,
  BitgetMarketPriceResponse,
  SwapQuoteRequest,
  SwapQuoteResponse,
  SwapExecuteRequest,
  SwapExecuteResponse,
  QuickSwapQuoteParams,
} from '../types/web3'

// ============================================================================
// Abcex Service — Покупка криптовалюты за фиат
// ============================================================================

/** Получить котировку для покупки */
export async function abcexGetQuote(
  request: AbcexQuoteRequest
): Promise<AbcexQuoteResponse> {
  return invoke<AbcexQuoteResponse>('abcex_get_quote', { request })
}

/** Создать заказ на покупку */
export async function abcexCreateOrder(
  request: AbcexOrderRequest
): Promise<AbcexOrderResponse> {
  return invoke<AbcexOrderResponse>('abcex_create_order', { request })
}

/** Проверить статус заказа */
export async function abcexGetOrderStatus(
  orderId: string
): Promise<AbcexOrderResponse> {
  return invoke<AbcexOrderResponse>('abcex_get_order_status', { orderId })
}

/** Проверить KYC статус */
export async function abcexCheckKyc(
  request: AbcexKycRequest
): Promise<AbcexKycResponse> {
  return invoke<AbcexKycResponse>('abcex_check_kyc', { request })
}

/** Получить список криптовалют */
export async function abcexGetSupportedCryptos(): Promise<string[]> {
  return invoke<string[]>('abcex_get_supported_cryptos')
}

/** Получить лимиты для страны */
export async function abcexGetLimits(
  request: AbcexLimitsRequest
): Promise<AbcexLimitsResponse> {
  return invoke<AbcexLimitsResponse>('abcex_get_limits', { request })
}

/** Быстрая котировка */
export async function abcexQuickQuote(
  params: AbcexQuickQuoteParams
): Promise<string> {
  return invoke<string>('abcex_quick_quote', {
    fiatCurrency: params.fiat_currency,
    fiatAmount: params.fiat_amount,
    cryptoCurrency: params.crypto_currency,
    paymentMethod: params.payment_method,
    country: params.country,
  })
}

/** Рассчитать комиссию */
export async function abcexCalculateFee(
  amount: string,
  feeBps: number
): Promise<string> {
  return invoke<string>('abcex_calculate_fee', { amount, feeBps })
}

// ============================================================================
// Bitget Service — Trading (биржа)
// ============================================================================

/** Разместить ордер */
export async function bitgetPlaceOrder(
  request: BitgetOrderRequest
): Promise<BitgetOrderResponse> {
  return invoke<BitgetOrderResponse>('bitget_place_order', { request })
}

/** Проверить статус ордера */
export async function bitgetGetOrderStatus(
  request: BitgetOrderStatusRequest
): Promise<BitgetOrderResponse> {
  return invoke<BitgetOrderResponse>('bitget_get_order_status', { request })
}

/** Отменить ордер */
export async function bitgetCancelOrder(
  request: BitgetCancelRequest
): Promise<BitgetOrderResponse> {
  return invoke<BitgetOrderResponse>('bitget_cancel_order', { request })
}

/** Получить баланс */
export async function bitgetGetBalance(
  request: BitgetBalanceRequest
): Promise<BitgetBalanceResponse> {
  return invoke<BitgetBalanceResponse>('bitget_get_balance', { request })
}

/** Получить рыночную цену */
export async function bitgetGetMarketPrice(
  request: BitgetMarketPriceRequest
): Promise<BitgetMarketPriceResponse> {
  return invoke<BitgetMarketPriceResponse>('bitget_get_market_price', {
    request,
  })
}

/** Получить список торговых пар */
export async function bitgetGetSymbols(): Promise<string[]> {
  return invoke<string[]>('bitget_get_symbols')
}

/** Быстрая покупка (market order) */
export async function bitgetQuickBuy(
  symbol: string,
  quoteAmount: string
): Promise<BitgetOrderResponse> {
  return invoke<BitgetOrderResponse>('bitget_quick_buy', {
    symbol,
    quoteAmount,
  })
}

/** Быстрая продажа (market order) */
export async function bitgetQuickSell(
  symbol: string,
  amount: string
): Promise<BitgetOrderResponse> {
  return invoke<BitgetOrderResponse>('bitget_quick_sell', { symbol, amount })
}

/** Рассчитать комиссию */
export async function bitgetCalculateFee(
  amount: string,
  feeBps: number
): Promise<string> {
  return invoke<string>('bitget_calculate_fee', { amount, feeBps })
}

// ============================================================================
// 0x Protocol Service — DeFi Swap
// ============================================================================

/** Получить котировку для обмена */
export async function getSwapQuote(
  request: SwapQuoteRequest
): Promise<SwapQuoteResponse> {
  return invoke<SwapQuoteResponse>('get_swap_quote', { request })
}

/** Выполнить обмен */
export async function executeSwap(
  request: SwapExecuteRequest
): Promise<SwapExecuteResponse> {
  return invoke<SwapExecuteResponse>('execute_swap', { request })
}

/** Рассчитать slippage */
export async function calculateSlippage(
  tokenSymbol: string,
  gasPriceGwei: number
): Promise<number> {
  return invoke<number>('calculate_slippage', {
    tokenSymbol,
    gasPriceGwei,
  })
}

/** Получить адрес для allowance */
export async function getAllowanceTarget(
  tokenAddress: string,
  chainId: number
): Promise<string> {
  return invoke<string>('get_allowance_target', { tokenAddress, chainId })
}

/** Быстрая котировка */
export async function quickSwapQuote(
  params: QuickSwapQuoteParams
): Promise<string> {
  return invoke<string>('quick_swap_quote', {
    sellToken: params.sell_token,
    buyToken: params.buy_token,
    sellAmount: params.sell_amount,
    decimals: params.decimals,
    chainId: params.chain_id,
  })
}

// ============================================================================
// Helper Functions
// ============================================================================

/** Форматировать сумму с символами */
export function formatTokenAmount(
  amount: string,
  symbol: string,
  decimals = 2
): string {
  const num = parseFloat(amount)
  if (isNaN(num)) return `0 ${symbol}`
  return `${num.toFixed(decimals)} ${symbol}`
}

/** Конвертировать basis points в проценты */
export function bpsToPercent(bps: number): string {
  return `${(bps / 100).toFixed(1)}%`
}

/** Конвертировать проценты в basis points */
export function percentToBps(percent: number): number {
  return Math.round(percent * 100)
}
