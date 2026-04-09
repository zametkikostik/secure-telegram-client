/**
 * TransactionHistory — История транзакций
 *
 * Отображает:
 * - Обмены токенов (0x Protocol)
 * - Покупки за фиат (Abcex)
 * - Торговые операции (Bitget)
 * - Фильтрация и сортировка
 */

import React, { useState, useMemo } from 'react'
import clsx from 'clsx'
import {
  type AbcexOrderData,
  type BitgetOrderData,
  type SwapRecordData,
} from '../../types/web3'

// ============================================================================
// Типы транзакций
// ============================================================================

/** Тип транзакции */
export enum TransactionType {
  Swap = 'swap',
  Purchase = 'purchase',
  Trade = 'trade',
}

/** Единый интерфейс транзакции */
export interface TransactionRecord {
  id: string
  type: TransactionType
  timestamp: number
  status: string
  from?: string
  to?: string
  amount?: string
  fee?: string
  details: AbcexOrderData | BitgetOrderData | SwapRecordData | null
  raw?: any
}

// ============================================================================
// Helpers
// ============================================================================

/** Форматировать timestamp */
function formatTimestamp(ts: number): string {
  const date = new Date(ts)
  return date.toLocaleString('ru-RU', {
    day: '2-digit',
    month: '2-digit',
    year: '2-digit',
    hour: '2-digit',
    minute: '2-digit',
  })
}

/** Получить иконку типа транзакции */
function getTypeIcon(type: TransactionType): React.ReactNode {
  switch (type) {
    case TransactionType.Swap:
      return '🔄'
    case TransactionType.Purchase:
      return '💰'
    case TransactionType.Trade:
      return '📈'
  }
}

/** Получить цвет статуса */
function getStatusColor(status: string): string {
  const lower = status.toLowerCase()
  if (lower.includes('success') || lower.includes('confirmed') || lower.includes('filled')) {
    return 'text-green-500 bg-green-500/10'
  }
  if (lower.includes('pending') || lower.includes('waiting')) {
    return 'text-yellow-500 bg-yellow-500/10'
  }
  if (lower.includes('fail') || lower.includes('cancel') || lower.includes('error')) {
    return 'text-red-500 bg-red-500/10'
  }
  return 'text-text-secondary bg-bg-secondary'
}

// ============================================================================
// TransactionCard — Карточка транзакции
// ============================================================================

interface TransactionCardProps {
  /** Запись транзакции */
  transaction: TransactionRecord
  /** Callback при клике */
  onClick?: (transaction: TransactionRecord) => void
}

/** Карточка отдельной транзакции */
export const TransactionCard: React.FC<TransactionCardProps> = ({
  transaction,
  onClick,
}) => {
  const handleClick = () => onClick?.(transaction)

  return (
    <div
      className={clsx(
        'bg-bg-secondary hover:bg-bg-tertiary border border-bg-border rounded-lg p-4 cursor-pointer transition-all',
        'hover:shadow-md hover:border-primary-500/30'
      )}
      onClick={handleClick}
    >
      {/* Заголовок */}
      <div className="flex items-center justify-between mb-3">
        <div className="flex items-center gap-2">
          <span className="text-2xl">{getTypeIcon(transaction.type)}</span>
          <div>
            <div className="font-medium text-text-primary capitalize">
              {transaction.type === TransactionType.Swap && 'Обмен'}
              {transaction.type === TransactionType.Purchase && 'Покупка'}
              {transaction.type === TransactionType.Trade && 'Торговля'}
            </div>
            <div className="text-xs text-text-secondary">
              {formatTimestamp(transaction.timestamp)}
            </div>
          </div>
        </div>
        <span
          className={clsx(
            'text-xs font-medium px-2 py-1 rounded',
            getStatusColor(transaction.status)
          )}
        >
          {transaction.status}
        </span>
      </div>

      {/* Детали транзакции */}
      {transaction.type === TransactionType.Swap && transaction.details && (
        <SwapTransactionDetails details={transaction.details as SwapRecordData} />
      )}
      {transaction.type === TransactionType.Purchase && transaction.details && (
        <PurchaseTransactionDetails details={transaction.details as AbcexOrderData} />
      )}
      {transaction.type === TransactionType.Trade && transaction.details && (
        <TradeTransactionDetails details={transaction.details as BitgetOrderData} />
      )}
    </div>
  )
}

// ============================================================================
// Transaction Details Components
// ============================================================================

interface SwapTransactionDetailsProps {
  details: SwapRecordData
}

const SwapTransactionDetails: React.FC<SwapTransactionDetailsProps> = ({ details }) => {
  return (
    <div className="space-y-2 text-sm">
      <div className="flex justify-between">
        <span className="text-text-secondary">Из:</span>
        <span className="text-text-primary font-mono">
          {details.from_amount} {details.from_token}
        </span>
      </div>
      <div className="flex justify-between">
        <span className="text-text-secondary">В:</span>
        <span className="text-accent font-mono">
          {details.to_amount} {details.to_token}
        </span>
      </div>
      <div className="flex justify-between">
        <span className="text-text-secondary">Цена:</span>
        <span className="text-text-primary font-mono">{details.price}</span>
      </div>
      {details.fee_bps > 0 && (
        <div className="flex justify-between">
          <span className="text-text-secondary">Комиссия:</span>
          <span className="text-text-primary font-mono">{(details.fee_bps / 100).toFixed(1)}%</span>
        </div>
      )}
    </div>
  )
}

interface PurchaseTransactionDetailsProps {
  details: AbcexOrderData
}

const PurchaseTransactionDetails: React.FC<PurchaseTransactionDetailsProps> = ({ details }) => {
  return (
    <div className="space-y-2 text-sm">
      <div className="flex justify-between">
        <span className="text-text-secondary">Сумма:</span>
        <span className="text-text-primary font-mono">
          {details.fiat_amount} {details.fiat_currency}
        </span>
      </div>
      <div className="flex justify-between">
        <span className="text-text-secondary">Крипта:</span>
        <span className="text-accent font-mono">
          {details.crypto_amount} {details.crypto_currency}
        </span>
      </div>
      <div className="flex justify-between">
        <span className="text-text-secondary">Курс:</span>
        <span className="text-text-primary font-mono">{details.rate}</span>
      </div>
      <div className="flex justify-between">
        <span className="text-text-secondary">Способ оплаты:</span>
        <span className="text-text-primary">{details.payment_method}</span>
      </div>
    </div>
  )
}

interface TradeTransactionDetailsProps {
  details: BitgetOrderData
}

const TradeTransactionDetails: React.FC<TradeTransactionDetailsProps> = ({ details }) => {
  return (
    <div className="space-y-2 text-sm">
      <div className="flex justify-between">
        <span className="text-text-secondary">Пара:</span>
        <span className="text-text-primary font-semibold">{details.symbol}</span>
      </div>
      <div className="flex justify-between">
        <span className="text-text-secondary">Тип:</span>
        <span className={clsx(
          'text-text-primary',
          details.side === 'buy' ? 'text-green-500' : 'text-red-500'
        )}>
          {details.side === 'buy' ? 'Покупка' : 'Продажа'}
        </span>
      </div>
      <div className="flex justify-between">
        <span className="text-text-secondary">Количество:</span>
        <span className="text-text-primary font-mono">{details.amount}</span>
      </div>
      <div className="flex justify-between">
        <span className="text-text-secondary">Цена:</span>
        <span className="text-text-primary font-mono">{details.price}</span>
      </div>
      {details.fee && (
        <div className="flex justify-between">
          <span className="text-text-secondary">Комиссия:</span>
          <span className="text-text-primary font-mono">{details.fee} {details.fee_currency}</span>
        </div>
      )}
    </div>
  )
}

// ============================================================================
// TransactionHistory — Основной компонент
// ============================================================================

interface TransactionHistoryProps {
  /** Обмены токенов */
  swaps?: SwapRecordData[]
  /** Покупки за фиат */
  purchases?: AbcexOrderData[]
  /** Торговые операции */
  trades?: BitgetOrderData[]
  /** Callback при клике на транзакцию */
  onTransactionClick?: (transaction: TransactionRecord) => void
  /** Максимальное количество транзакций для показа */
  limit?: number
  /** Показать фильтр */
  showFilter?: boolean
  /** Пустое состояние */
  emptyMessage?: string
}

/** История всех транзакций */
export const TransactionHistory: React.FC<TransactionHistoryProps> = ({
  swaps = [],
  purchases = [],
  trades = [],
  onTransactionClick,
  limit = 20,
  showFilter = true,
  emptyMessage = 'Нет транзакций',
}) => {
  const [filter, setFilter] = useState<TransactionType | 'all'>('all')
  const [sortBy, setSortBy] = useState<'newest' | 'oldest'>('newest')

  // Преобразовать все транзакции в единый формат
  const allTransactions = useMemo((): TransactionRecord[] => {
    const transactions: TransactionRecord[] = []

    // Добавить обмены
    swaps.forEach((swap) => {
      transactions.push({
        id: swap.id,
        type: TransactionType.Swap,
        timestamp: new Date().getTime(), // TODO: получить из backend
        status: swap.status,
        from: swap.from_token,
        to: swap.to_token,
        amount: swap.from_amount,
        fee: `${swap.fee_bps / 100}%`,
        details: swap,
        raw: swap,
      })
    })

    // Добавить покупки
    purchases.forEach((purchase) => {
      transactions.push({
        id: purchase.order_id,
        type: TransactionType.Purchase,
        timestamp: purchase.created_at,
        status: purchase.status,
        from: purchase.fiat_currency,
        to: purchase.crypto_currency,
        amount: purchase.fiat_amount,
        fee: purchase.fee_amount,
        details: purchase,
        raw: purchase,
      })
    })

    // Добавить торговые операции
    trades.forEach((trade) => {
      transactions.push({
        id: trade.order_id,
        type: TransactionType.Trade,
        timestamp: trade.created_at,
        status: trade.status,
        from: trade.symbol,
        amount: trade.amount,
        fee: `${trade.fee} ${trade.fee_currency}`,
        details: trade,
        raw: trade,
      })
    })

    // Сортировать
    transactions.sort((a, b) => {
      return sortBy === 'newest' ? b.timestamp - a.timestamp : a.timestamp - b.timestamp
    })

    // Применить фильтр
    const filtered = filter === 'all'
      ? transactions
      : transactions.filter((t) => t.type === filter)

    // Обрезать до лимита
    return filtered.slice(0, limit)
  }, [swaps, purchases, trades, filter, sortBy, limit])

  // Статистика
  const stats = useMemo(() => {
    return {
      total: allTransactions.length,
      swaps: allTransactions.filter((t) => t.type === TransactionType.Swap).length,
      purchases: allTransactions.filter((t) => t.type === TransactionType.Purchase).length,
      trades: allTransactions.filter((t) => t.type === TransactionType.Trade).length,
    }
  }, [allTransactions])

  return (
    <div className="space-y-4">
      {/* Заголовок и статистика */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-text-primary">История транзакций</h3>
        <div className="flex items-center gap-3 text-sm text-text-secondary">
          <span>Всего: {stats.total}</span>
          {stats.swaps > 0 && <span>🔄 {stats.swaps}</span>}
          {stats.purchases > 0 && <span>💰 {stats.purchases}</span>}
          {stats.trades > 0 && <span>📈 {stats.trades}</span>}
        </div>
      </div>

      {/* Фильтры */}
      {showFilter && (
        <div className="flex gap-2">
          {/* Тип транзакции */}
          <select
            value={filter}
            onChange={(e) => setFilter(e.target.value as any)}
            className="input-primary flex-1"
          >
            <option value="all">Все</option>
            <option value={TransactionType.Swap}>Обмены</option>
            <option value={TransactionType.Purchase}>Покупки</option>
            <option value={TransactionType.Trade}>Торговля</option>
          </select>

          {/* Сортировка */}
          <select
            value={sortBy}
            onChange={(e) => setSortBy(e.target.value as any)}
            className="input-primary w-40"
          >
            <option value="newest">Сначала новые</option>
            <option value="oldest">Сначала старые</option>
          </select>
        </div>
      )}

      {/* Список транзакций */}
      {allTransactions.length > 0 ? (
        <div className="space-y-3">
          {allTransactions.map((transaction) => (
            <TransactionCard
              key={transaction.id}
              transaction={transaction}
              onClick={onTransactionClick}
            />
          ))}
        </div>
      ) : (
        <div className="text-center py-12 text-text-secondary">
          <div className="text-4xl mb-2">📭</div>
          <div>{emptyMessage}</div>
        </div>
      )}
    </div>
  )
}

// ============================================================================
// TransactionModal — Модальное окно с деталями транзакции
// ============================================================================

interface TransactionModalProps {
  /** Транзакция */
  transaction: TransactionRecord | null
  /** Закрыть модалку */
  onClose: () => void
}

/** Модальное окно с полной информацией о транзакции */
export const TransactionModal: React.FC<TransactionModalProps> = ({
  transaction,
  onClose,
}) => {
  if (!transaction) return null

  return (
    <div className="fixed inset-0 bg-black/50 backdrop-blur-sm z-50 flex items-center justify-center p-4">
      <div className="bg-bg-primary border border-bg-border rounded-xl p-6 max-w-lg w-full max-h-[90vh] overflow-y-auto">
        {/* Заголовок */}
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-2">
            <span className="text-2xl">{getTypeIcon(transaction.type)}</span>
            <h3 className="text-lg font-semibold text-text-primary">
              Детали транзакции
            </h3>
          </div>
          <button
            onClick={onClose}
            className="text-text-secondary hover:text-text-primary transition-colors"
          >
            <svg className="w-6 h-6" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        {/* Информация */}
        <div className="space-y-3 text-sm">
          <div className="flex justify-between">
            <span className="text-text-secondary">ID:</span>
            <span className="text-text-primary font-mono text-xs">{transaction.id}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-text-secondary">Тип:</span>
            <span className="text-text-primary capitalize">{transaction.type}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-text-secondary">Время:</span>
            <span className="text-text-primary">{formatTimestamp(transaction.timestamp)}</span>
          </div>
          <div className="flex justify-between">
            <span className="text-text-secondary">Статус:</span>
            <span className={clsx('px-2 py-1 rounded', getStatusColor(transaction.status))}>
              {transaction.status}
            </span>
          </div>

          {/* Разделитель */}
          <div className="border-t border-bg-border my-4" />

          {/* Специфичные детали */}
          {transaction.from && (
            <div className="flex justify-between">
              <span className="text-text-secondary">Из:</span>
              <span className="text-text-primary font-mono">{transaction.from}</span>
            </div>
          )}
          {transaction.to && (
            <div className="flex justify-between">
              <span className="text-text-secondary">В:</span>
              <span className="text-accent font-mono">{transaction.to}</span>
            </div>
          )}
          {transaction.amount && (
            <div className="flex justify-between">
              <span className="text-text-secondary">Сумма:</span>
              <span className="text-text-primary font-mono font-semibold">{transaction.amount}</span>
            </div>
          )}
          {transaction.fee && (
            <div className="flex justify-between">
              <span className="text-text-secondary">Комиссия:</span>
              <span className="text-text-primary font-mono">{transaction.fee}</span>
            </div>
          )}
        </div>

        {/* Кнопка закрытия */}
        <button onClick={onClose} className="btn-primary w-full mt-6">
          Закрыть
        </button>
      </div>
    </div>
  )
}
