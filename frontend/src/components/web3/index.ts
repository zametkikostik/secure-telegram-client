/**
 * Web3 Components — Экспорт всех Web3 компонентов
 *
 * Forms:
 * - TokenSwapForm: Форма обмена токенов (0x Protocol)
 * - FiatPurchaseForm: Форма покупки за фиат (Abcex)
 *
 * Displays:
 * - QuoteDisplay: Отображение котировок
 * - TransactionHistory: История транзакций
 *
 * Usage:
 * ```tsx
 * import { TokenSwapForm, FiatPurchaseForm } from '@/components/web3'
 * ```
 */

// Forms
export { TokenSwapForm } from './TokenSwapForm'
export { FiatPurchaseForm } from './FiatPurchaseForm'

// Quote Displays
export {
  QuoteCard,
  SwapQuoteDisplay,
  FiatQuoteDisplay,
  MarketPriceDisplay,
  BalanceDisplay,
  MultiQuoteComparator,
} from './QuoteDisplay'

// Transaction History
export {
  TransactionHistory,
  TransactionCard,
  TransactionModal,
  TransactionType,
  type TransactionRecord,
} from './TransactionHistory'
