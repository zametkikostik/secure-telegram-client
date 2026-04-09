/**
 * QuoteDisplay — Компонент отображения котировок
 *
 * Показывает:
 * - Текущие котировки для обмена
 * - Рыночные цены
 * - Изменения за 24ч
 * - Сравнение предложений от разных провайдеров
 */

import React from 'react'
import clsx from 'clsx'
import {
  type AbcexQuoteData,
  type BitgetPriceData,
  type SwapQuoteData,
  type BitgetBalanceData,
} from '../../types/web3'
import { bpsToPercent } from '../../services/web3Service'

// ============================================================================
// Quote Card — Карточка котировки
// ============================================================================

interface QuoteCardProps {
  /** Заголовок карточки */
  title: string
  /** Источник котировки (0x, Abcex, Bitget) */
  source: string
  /** Иконка источника */
  icon?: React.ReactNode
}

/** Универсальная карточка котировки */
export const QuoteCard: React.FC<React.PropsWithChildren<QuoteCardProps>> = ({
  title,
  source,
  icon,
  children,
}) => {
  return (
    <div className="card bg-gradient-to-br from-bg-secondary to-bg-tertiary border border-bg-border rounded-xl p-5 space-y-4 hover:shadow-lg transition-shadow">
      {/* Заголовок */}
      <div className="flex items-center justify-between">
        <h4 className="font-semibold text-text-primary">{title}</h4>
        <div className="flex items-center gap-2">
          {icon && <span className="text-lg">{icon}</span>}
          <span className="text-xs text-text-secondary bg-bg-secondary px-2 py-1 rounded">
            {source}
          </span>
        </div>
      </div>

      {/* Содержимое */}
      <div className="space-y-3">{children}</div>
    </div>
  )
}

// ============================================================================
// SwapQuoteDisplay — Отображение котировки обмена
// ============================================================================

interface SwapQuoteDisplayProps {
  /** Данные котировки */
  quote: SwapQuoteData
  /** Символ токена продажи */
  sellTokenSymbol: string
  /** Символ токена покупки */
  buyTokenSymbol: string
  /** Десятичные знаки токена продажи */
  sellDecimals?: number
  /** Десятичные знаки токена покупки */
  buyDecimals?: number
}

/** Отображение котировки обмена токенов */
export const SwapQuoteDisplay: React.FC<SwapQuoteDisplayProps> = ({
  quote,
  sellTokenSymbol,
  buyTokenSymbol,
  sellDecimals = 18,
  buyDecimals = 18,
}) => {
  const sellAmountFormatted = (parseFloat(quote.sell_amount) / Math.pow(10, sellDecimals)).toFixed(6)
  const buyAmountFormatted = (parseFloat(quote.buy_amount) / Math.pow(10, buyDecimals)).toFixed(6)

  return (
    <QuoteCard title="Обмен токенов" source="0x Protocol" icon="🔄">
      {/* Направления обмена */}
      <div className="space-y-3">
        <div className="flex items-center justify-between bg-bg-primary/50 rounded-lg p-3">
          <div>
            <div className="text-xs text-text-secondary">Продаёте</div>
            <div className="text-lg font-semibold text-text-primary font-mono">
              {sellAmountFormatted}
            </div>
          </div>
          <div className="text-2xl font-bold text-primary-500">{sellTokenSymbol}</div>
        </div>

        <div className="flex justify-center">
          <svg className="w-6 h-6 text-primary-500" fill="none" stroke="currentColor" viewBox="0 0 24 24">
            <path strokeLinecap="round" strokeLinejoin="round" strokeWidth={2} d="M19 14l-7 7m0 0l-7-7m7 7V3" />
          </svg>
        </div>

        <div className="flex items-center justify-between bg-accent/10 rounded-lg p-3">
          <div>
            <div className="text-xs text-text-secondary">Получаете</div>
            <div className="text-lg font-semibold text-accent font-mono">
              {buyAmountFormatted}
            </div>
          </div>
          <div className="text-2xl font-bold text-accent">{buyTokenSymbol}</div>
        </div>
      </div>

      {/* Детали */}
      <div className="space-y-2 text-sm">
        <div className="flex justify-between">
          <span className="text-text-secondary">Курс:</span>
          <span className="text-text-primary font-mono">{quote.price}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-text-secondary">Газ (оценка):</span>
          <span className="text-text-primary font-mono">{quote.gas_estimate}</span>
        </div>
        <div className="flex justify-between">
          <span className="text-text-secondary">Комиссия:</span>
          <span className="text-text-primary font-mono">{bpsToPercent(quote.fee_bps)}</span>
        </div>
      </div>
    </QuoteCard>
  )
}

// ============================================================================
// FiatQuoteDisplay — Отображение котировки покупки за фиат
// ============================================================================

interface FiatQuoteDisplayProps {
  /** Данные котировки */
  quote: AbcexQuoteData
}

/** Отображение котировки покупки за фиат */
export const FiatQuoteDisplay: React.FC<FiatQuoteDisplayProps> = ({ quote }) => {
  return (
    <QuoteCard title="Покупка криптовалюты" source="Abcex" icon="💰">
      {/* Основная информация */}
      <div className="space-y-3">
        <div className="bg-bg-primary/50 rounded-lg p-4 space-y-2">
          <div className="flex justify-between items-center">
            <span className="text-text-secondary">Вы платите:</span>
            <span className="text-xl font-bold text-text-primary font-mono">
              {quote.fiat_amount} {quote.fiat_currency}
            </span>
          </div>
          <div className="flex justify-between items-center">
            <span className="text-text-secondary">Вы получаете:</span>
            <span className="text-xl font-bold text-accent font-mono">
              {quote.crypto_amount} {quote.crypto_currency}
            </span>
          </div>
        </div>

        {/* Курс и комиссия */}
        <div className="grid grid-cols-2 gap-3 text-sm">
          <div className="bg-bg-tertiary rounded-lg p-3">
            <div className="text-xs text-text-secondary">Курс</div>
            <div className="text-sm font-semibold text-text-primary font-mono">{quote.rate}</div>
          </div>
          <div className="bg-bg-tertiary rounded-lg p-3">
            <div className="text-xs text-text-secondary">Комиссия</div>
            <div className="text-sm font-semibold text-text-primary font-mono">{quote.fee_percent}</div>
          </div>
        </div>

        {/* Способы оплаты */}
        <div className="space-y-1">
          <div className="text-xs text-text-secondary">Способы оплаты:</div>
          <div className="flex flex-wrap gap-1">
            {quote.payment_methods.map((method) => (
              <span
                key={method}
                className="text-xs bg-bg-secondary text-text-secondary px-2 py-1 rounded"
              >
                {method}
              </span>
            ))}
          </div>
        </div>

        {/* Время действия и лимиты */}
        <div className="flex justify-between text-xs text-text-secondary">
          <span>⏱️ Действует: {quote.expires_in} сек</span>
          <span>📊 {quote.min_amount} - {quote.max_amount} {quote.fiat_currency}</span>
        </div>
      </div>
    </QuoteCard>
  )
}

// ============================================================================
// MarketPriceDisplay — Отображение рыночной цены
// ============================================================================

interface MarketPriceDisplayProps {
  /** Данные рыночной цены */
  price: BitgetPriceData
  /** Показывать изменение в процентах */
  showChange?: boolean
}

/** Отображение рыночной цены */
export const MarketPriceDisplay: React.FC<MarketPriceDisplayProps> = ({
  price,
  showChange = true,
}) => {
  const changePercent = parseFloat(price.change_24h)
  const isPositive = changePercent >= 0

  return (
    <div className="bg-bg-secondary rounded-lg p-4 space-y-3">
      {/* Заголовок */}
      <div className="flex items-center justify-between">
        <h4 className="font-semibold text-text-primary">{price.symbol}</h4>
        <span className="text-xs text-text-secondary bg-bg-tertiary px-2 py-1 rounded">
          Bitget
        </span>
      </div>

      {/* Цена */}
      <div className="text-3xl font-bold text-text-primary font-mono">{price.price}</div>

      {/* Изменение за 24ч */}
      {showChange && (
        <div className={clsx('text-sm font-medium', isPositive ? 'text-green-500' : 'text-red-500')}>
          {isPositive ? '↑' : '↓'} {Math.abs(changePercent).toFixed(2)}%
        </div>
      )}

      {/* Статистика 24ч */}
      <div className="grid grid-cols-2 gap-2 text-xs">
        <div className="bg-bg-tertiary rounded p-2">
          <div className="text-text-secondary">Максимум</div>
          <div className="font-mono text-text-primary">{price.high_24h}</div>
        </div>
        <div className="bg-bg-tertiary rounded p-2">
          <div className="text-text-secondary">Минимум</div>
          <div className="font-mono text-text-primary">{price.low_24h}</div>
        </div>
        <div className="bg-bg-tertiary rounded p-2 col-span-2">
          <div className="text-text-secondary">Объём 24ч</div>
          <div className="font-mono text-text-primary">{price.volume_24h}</div>
        </div>
      </div>
    </div>
  )
}

// ============================================================================
// BalanceDisplay — Отображение баланса
// ============================================================================

interface BalanceDisplayProps {
  /** Данные баланса */
  balance: BitgetBalanceData
  /** Скрыть общий баланс */
  hideTotal?: boolean
}

/** Отображение баланса аккаунта */
export const BalanceDisplay: React.FC<BalanceDisplayProps> = ({
  balance,
  hideTotal = false,
}) => {
  return (
    <div className="bg-gradient-to-br from-primary-500/10 to-accent/10 border border-primary-500/30 rounded-xl p-5 space-y-4">
      {/* Заголовок */}
      <div className="flex items-center justify-between">
        <h4 className="font-semibold text-text-primary">Баланс</h4>
        <span className="text-2xl">💎</span>
      </div>

      {/* Валюта */}
      <div className="text-lg font-bold text-text-primary">{balance.currency}</div>

      {/* Балансы */}
      <div className="space-y-2">
        <div className="flex justify-between text-sm">
          <span className="text-text-secondary">Доступно:</span>
          <span className="text-text-primary font-mono font-semibold">
            {balance.available_balance}
          </span>
        </div>
        <div className="flex justify-between text-sm">
          <span className="text-text-secondary">В ордерах:</span>
          <span className="text-text-primary font-mono">{balance.frozen_balance}</span>
        </div>
        {!hideTotal && (
          <div className="flex justify-between text-sm pt-2 border-t border-bg-border">
            <span className="text-text-secondary font-medium">Общий:</span>
            <span className="text-text-primary font-mono font-bold">
              {balance.total_balance}
            </span>
          </div>
        )}
      </div>
    </div>
  )
}

// ============================================================================
// MultiQuoteComparator — Сравнение котировок от разных провайдеров
// ============================================================================

interface MultiQuoteComparatorProps {
  /** Котировка 0x Protocol */
  swapQuote?: SwapQuoteData
  /** Котировка Abcex */
  abcexQuote?: AbcexQuoteData
  /** Загрузка */
  loading?: boolean
}

/** Сравнение котировок от разных провайдеров */
export const MultiQuoteComparator: React.FC<MultiQuoteComparatorProps> = ({
  swapQuote,
  abcexQuote,
  loading = false,
}) => {
  if (loading) {
    return (
      <div className="space-y-4">
        <div className="text-center text-text-secondary">
          <svg className="animate-spin h-8 w-8 mx-auto mb-2" viewBox="0 0 24 24">
            <circle
              className="opacity-25"
              cx="12"
              cy="12"
              r="10"
              stroke="currentColor"
              strokeWidth="4"
              fill="none"
            />
            <path
              className="opacity-75"
              fill="currentColor"
              d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
            />
          </svg>
          Загрузка котировок...
        </div>
      </div>
    )
  }

  if (!swapQuote && !abcexQuote) {
    return (
      <div className="text-center text-text-secondary py-8">
        Нет доступных котировок
      </div>
    )
  }

  return (
    <div className="space-y-4">
      <h3 className="text-lg font-semibold text-text-primary">Сравнение котировок</h3>
      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
        {swapQuote && (
          <SwapQuoteDisplay
            quote={swapQuote}
            sellTokenSymbol="USDT"
            buyTokenSymbol="ETH"
          />
        )}
        {abcexQuote && <FiatQuoteDisplay quote={abcexQuote} />}
      </div>
    </div>
  )
}
