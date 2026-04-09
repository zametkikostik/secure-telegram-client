/**
 * 0x Swap Integration - TypeScript Types
 * 
 * Типы для использования 0x Protocol API в UI
 */

/** Поддерживаемые сети */
export type ChainId = 1 | 137 | 42161 | 8453 | 10 | 56;

/** Запрос котировки */
export interface SwapQuoteRequest {
  chain_id: ChainId;
  sell_token: string;  // адрес токена
  buy_token: string;   // адрес токена
  sell_amount: string; // raw amount (wei)
  buy_amount?: string; // raw amount (wei)
  slippage_bps?: number; // basis points (100 = 1%)
  taker_address: string; // адрес кошелька
}

/** Данные котировки */
export interface QuoteData {
  sell_token: string;
  buy_token: string;
  sell_amount: string;
  buy_amount: string;
  price: string;
  gas_estimate: string;
  fee_bps: number;
  to: string;          // адрес контракта
  data: string;        // калldata
  value: string;       // ETH value
}

/** Ответ котировки */
export interface SwapQuoteResponse {
  success: boolean;
  quote?: QuoteData;
  error?: string;
}

/** Запрос на обмен */
export interface SwapExecuteRequest {
  chain_id: ChainId;
  sell_token: string;
  buy_token: string;
  sell_amount: string;
  taker_address: string;
  slippage_bps?: number;
}

/** Данные записи обмена */
export interface SwapRecordData {
  id: string;
  chain: string;
  from_token: string;
  to_token: string;
  from_amount: string;
  to_amount: string;
  price: string;
  gas_estimate: string;
  fee_bps: number;
  status: string;
}

/** Ответ на обмен */
export interface SwapExecuteResponse {
  success: boolean;
  swap_record?: SwapRecordData;
  error?: string;
}

/** Информация о токене */
export interface TokenInfo {
  address: string;
  symbol: string;
  name: string;
  decimals: number;
  logoURI?: string;
}

/** Статус allowance */
export interface AllowanceInfo {
  tokenAddress: string;
  spenderAddress: string;
  allowance: string;
}

// ============================================================================
// Примеры использования
// ============================================================================

/**
 * Пример: Получить котировку ETH -> USDC
 * 
 * ```typescript
 * import { invoke } from '@tauri-apps/api/core';
 * 
 * const response = await invoke<SwapQuoteResponse>('get_swap_quote', {
 *   request: {
 *     chain_id: 1,
 *     sell_token: '0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE', // ETH
 *     buy_token: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', // USDC
 *     sell_amount: '1000000000000000000', // 1 ETH
 *     taker_address: walletAddress,
 *     slippage_bps: 100, // 1%
 *   }
 * });
 * 
 * if (response.success && response.quote) {
 *   console.log(`Получите: ${response.quote.buy_amount} USDC`);
 *   console.log(`Gas: ${response.quote.gas_estimate}`);
 * }
 * ```
 */

/**
 * Пример: Выполнить обмен
 * 
 * ```typescript
 * const result = await invoke<SwapExecuteResponse>('execute_swap', {
 *   request: {
 *     chain_id: 1,
 *     sell_token: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2', // WETH
 *     buy_token: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', // USDC
 *     sell_amount: '1000000000000000000', // 1 WETH
 *     taker_address: walletAddress,
 *   }
 * });
 * 
 * if (result.success) {
 *   console.log('Swap ID:', result.swap_record?.id);
 *   console.log('Status:', result.swap_record?.status);
 * }
 * ```
 */

/**
 * Пример: Получить адрес для approve
 * 
 * ```typescript
 * const allowanceTarget = await invoke<string>('get_allowance_target', {
 *   tokenAddress: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', // USDC
 *   chainId: 1
 * });
 * 
 * // Затем вызовите approve через ethers.js или viem
 * await tokenContract.approve(allowanceTarget, amount);
 * ```
 */

/**
 * Пример: Рассчитать slippage
 * 
 * ```typescript
 * const slippage = await invoke<number>('calculate_slippage', {
 *   tokenSymbol: 'ETH',
 *   gasPriceGwei: 50
 * });
 * 
 * console.log(`Рекомендуемое slippage: ${slippage} bps (${slippage / 100}%`);
 * ```
 */

// ============================================================================
// Константы
// ============================================================================

/** Адреса популярных токенов на Ethereum Mainnet */
export const ETHEREUM_TOKENS = {
  WETH: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
  USDC: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
  USDT: '0xdAC17F958D2ee523a2206206994597C13D831ec7',
  DAI: '0x6B175474E89094C44Da98b954EedeAC495271d0F',
  WBTC: '0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599',
} as const;

/** Адреса популярных токенов на Polygon */
export const POLYGON_TOKENS = {
  WMATIC: '0x0d500B1d8E8eF31E21C99d1Db9A6444d3ADf1270',
  USDC: '0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359',
  USDT: '0xc2132D05D31c914a87C6611C10748AEb04B58e8F',
  DAI: '0x8f3Cf7ad23Cd3CaDbD9735AFf958023239c6A063',
} as const;

/** Адреса популярных токенов на Arbitrum */
export const ARBITRUM_TOKENS = {
  WETH: '0x82aF49447D8a07e3bd95BD0d56f35241523fBab1',
  USDC: '0xaf88d065e77c8cC2239327C5EDb3A432268e5831',
} as const;

/** Адреса популярных токенов на Base */
export const BASE_TOKENS = {
  WETH: '0x4200000000000000000000000000000000000006',
  USDC: '0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913',
} as const;

/** Адрес ETH для swaps */
export const ETH_ADDRESS = '0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE';

/** Рекомендуемые slippage для разных типов токенов */
export const RECOMMENDED_SLIPPAGE = {
  stablecoins: 50,      // 0.5%
  majorTokens: 100,     // 1%
  midCapTokens: 200,    // 2%
  lowLiquidity: 300,    // 3%
} as const;
