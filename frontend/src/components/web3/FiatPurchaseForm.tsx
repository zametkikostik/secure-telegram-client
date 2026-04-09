/**
 * FiatPurchaseForm — Форма покупки криптовалюты за фиат через Abcex
 *
 * Позволяет пользователям:
 * - Выбрать фиатную валюту (USD, EUR, RUB)
 * - Выбрать криптовалюту (BTC, ETH, USDT)
 * - Выбрать способ оплаты
 * - Получить котировку
 * - Создать заказ на покупку
 */

import React, { useState, useEffect } from 'react'
import clsx from 'clsx'
import {
  type AbcexQuoteData,
  type AbcexQuoteResponse,
  type AbcexOrderResponse,
  type AbcexLimitsResponse,
} from '../../types/web3'
import {
  abcexGetQuote,
  abcexCreateOrder,
  abcexGetLimits,
  abcexGetSupportedCryptos,
} from '../../services/web3Service'

// Способы оплаты
const PAYMENT_METHODS = [
  { value: 'credit_card', label: '💳 Credit Card', icon: '💳' },
  { value: 'debit_card', label: '💳 Debit Card', icon: '💳' },
  { value: 'sepa', label: '🏦 SEPA Transfer', icon: '🏦' },
  { value: 'bank_transfer', label: '🏛️ Bank Transfer', icon: '🏛️' },
  { value: 'apple_pay', label: '🍎 Apple Pay', icon: '🍎' },
  { value: 'google_pay', label: '🤖 Google Pay', icon: '🤖' },
]

// Доступные фиатные валюты
const FIAT_CURRENCIES = [
  { value: 'USD', label: '🇺🇸 USD - US Dollar', symbol: '$' },
  { value: 'EUR', label: '🇪🇺 EUR - Euro', symbol: '€' },
  { value: 'GBP', label: '🇬🇧 GBP - British Pound', symbol: '£' },
  { value: 'RUB', label: '🇷🇺 RUB - Russian Ruble', symbol: '₽' },
]

// Криптовалюты по умолчанию
const DEFAULT_CRYPTOS = ['BTC', 'ETH', 'USDT', 'USDC', 'BNB']

interface FiatPurchaseFormProps {
  /** Адрес кошелька для получения криптовалюты */
  walletAddress: string
  /** Callback после успешного создания заказа */
  onSuccess?: (order: AbcexOrderResponse) => void
  /** Callback при ошибке */
  onError?: (error: string) => void
}

export const FiatPurchaseForm: React.FC<FiatPurchaseFormProps> = ({
  walletAddress,
  onSuccess,
  onError,
}) => {
  // Состояния формы
  const [fiatCurrency, setFiatCurrency] = useState('USD')
  const [fiatAmount, setFiatAmount] = useState('')
  const [cryptoCurrency, setCryptoCurrency] = useState('BTC')
  const [paymentMethod, setPaymentMethod] = useState('credit_card')
  const [email, setEmail] = useState('')
  const [country, setCountry] = useState('US')

  // Состояния загрузки
  const [loading, setLoading] = useState(false)
  const [creating, setCreating] = useState(false)
  const [quote, setQuote] = useState<AbcexQuoteData | null>(null)
  const [error, setError] = useState<string | null>(null)
  const [limits, setLimits] = useState<Record<string, string> | null>(null)
  const [supportedCryptos, setSupportedCryptos] = useState<string[]>(DEFAULT_CRYPTOS)

  // Загрузить поддерживаемые криптовалюты и лимиты при монтировании
  useEffect(() => {
    const loadData = async () => {
      try {
        // Загрузить криптовалюты
        const cryptos = await abcexGetSupportedCryptos()
        if (cryptos.length > 0) {
          setSupportedCryptos(cryptos)
          setCryptoCurrency(cryptos[0])
        }

        // Загрузить лимиты
        const limitsResponse: AbcexLimitsResponse = await abcexGetLimits({
          country,
        })
        if (limitsResponse.success && limitsResponse.limits) {
          setLimits(limitsResponse.limits)
        }
      } catch (err) {
        console.warn('Failed to load Web3 data:', err)
      }
    }

    loadData()
  }, [country])

  // Получить котировку
  const handleGetQuote = async () => {
    if (!fiatAmount || parseFloat(fiatAmount) <= 0) {
      setError('Введите сумму для покупки')
      return
    }

    if (!email) {
      setError('Введите email для получения чека')
      return
    }

    setLoading(true)
    setError(null)

    try {
      const response: AbcexQuoteResponse = await abcexGetQuote({
        fiat_currency: fiatCurrency,
        fiat_amount: fiatAmount,
        crypto_currency: cryptoCurrency,
        payment_method: paymentMethod,
        country,
      })

      if (response.success && response.quote) {
        setQuote(response.quote)
      } else {
        setError(response.error || 'Не удалось получить котировку')
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Неизвестная ошибка'
      setError(message)
      onError?.(message)
    } finally {
      setLoading(false)
    }
  }

  // Создать заказ
  const handleCreateOrder = async () => {
    if (!quote) {
      setError('Нет активной котировки')
      return
    }

    if (!walletAddress) {
      setError('Подключите кошелёк')
      return
    }

    setCreating(true)
    setError(null)

    try {
      const response: AbcexOrderResponse = await abcexCreateOrder({
        quote_id: quote.quote_id,
        deposit_address: walletAddress,
        payment_method: paymentMethod,
        user_email: email,
      })

      if (response.success && response.order) {
        onSuccess?.(response)
        // Сбросить форму
        setFiatAmount('')
        setQuote(null)
      } else {
        setError(response.error || 'Ошибка создания заказа')
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Неизвестная ошибка'
      setError(message)
      onError?.(message)
    } finally {
      setCreating(false)
    }
  }

  // Проверить лимиты
  const isWithinLimits = (): boolean => {
    if (!limits || !fiatAmount) return true
    const amount = parseFloat(fiatAmount)
    const min = parseFloat(limits.min_amount || '0')
    const max = parseFloat(limits.max_amount || '999999999')
    return amount >= min && amount <= max
  }

  const fiatSymbol = FIAT_CURRENCIES.find((c) => c.value === fiatCurrency)?.symbol || '$'

  return (
    <div className="card p-6 space-y-6">
      {/* Заголовок */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-text-primary">Купить криптовалюту</h3>
        <span className="text-xs text-text-secondary bg-bg-secondary px-2 py-1 rounded">
          Abcex
        </span>
      </div>

      {/* Страна */}
      <div className="space-y-2">
        <label className="text-sm font-medium text-text-secondary">Страна</label>
        <select
          value={country}
          onChange={(e) => setCountry(e.target.value)}
          className="input-primary w-full"
        >
          <option value="US">🇺🇸 United States</option>
          <option value="DE">🇩🇪 Germany</option>
          <option value="FR">🇫🇷 France</option>
          <option value="GB">🇬🇧 United Kingdom</option>
          <option value="RU">🇷🇺 Russia</option>
          <option value="BG">🇧🇬 Bulgaria</option>
          <option value="KZ">🇰🇿 Kazakhstan</option>
          <option value="UA">🇺🇦 Ukraine</option>
        </select>
      </div>

      {/* Фиатная валюта и сумма */}
      <div className="space-y-2">
        <label className="text-sm font-medium text-text-secondary">Сумма</label>
        <div className="flex gap-2">
          <select
            value={fiatCurrency}
            onChange={(e) => setFiatCurrency(e.target.value)}
            className="input-primary w-1/3"
          >
            {FIAT_CURRENCIES.map((currency) => (
              <option key={currency.value} value={currency.value}>
                {currency.value}
              </option>
            ))}
          </select>
          <div className="relative flex-1">
            <span className="absolute left-3 top-1/2 -translate-y-1/2 text-text-secondary">
              {fiatSymbol}
            </span>
            <input
              type="number"
              value={fiatAmount}
              onChange={(e) => {
                setFiatAmount(e.target.value)
                setQuote(null)
              }}
              placeholder="0.00"
              className="input-primary w-full pl-8"
              step="any"
              min="0"
            />
          </div>
        </div>
      </div>

      {/* Криптовалюта */}
      <div className="space-y-2">
        <label className="text-sm font-medium text-text-secondary">Криптовалюта</label>
        <select
          value={cryptoCurrency}
          onChange={(e) => setCryptoCurrency(e.target.value)}
          className="input-primary w-full"
        >
          {supportedCryptos.map((crypto) => (
            <option key={crypto} value={crypto}>
              {crypto}
            </option>
          ))}
        </select>
      </div>

      {/* Способ оплаты */}
      <div className="space-y-2">
        <label className="text-sm font-medium text-text-secondary">Способ оплаты</label>
        <div className="grid grid-cols-2 gap-2">
          {PAYMENT_METHODS.map((method) => (
            <button
              key={method.value}
              onClick={() => setPaymentMethod(method.value)}
              className={clsx(
                'p-3 rounded-lg border-2 transition-all text-sm',
                paymentMethod === method.value
                  ? 'border-primary-500 bg-primary-500/10 text-text-primary'
                  : 'border-bg-tertiary bg-bg-secondary text-text-secondary hover:border-bg-border hover:bg-bg-tertiary'
              )}
            >
              {method.label}
            </button>
          ))}
        </div>
      </div>

      {/* Email */}
      <div className="space-y-2">
        <label className="text-sm font-medium text-text-secondary">
          Email <span className="text-text-secondary text-xs">(для чека)</span>
        </label>
        <input
          type="email"
          value={email}
          onChange={(e) => setEmail(e.target.value)}
          placeholder="your@email.com"
          className="input-primary w-full"
        />
      </div>

      {/* Кнопка получения котировки */}
      <button
        onClick={handleGetQuote}
        disabled={loading || !fiatAmount}
        className={clsx('btn-primary w-full', {
          'opacity-50 cursor-not-allowed': loading || !fiatAmount,
        })}
      >
        {loading ? (
          <span className="flex items-center justify-center gap-2">
            <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
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
            Получение котировки...
          </span>
        ) : (
          'Получить котировку'
        )}
      </button>

      {/* Отображение котировки */}
      {quote && (
        <div className="bg-bg-secondary rounded-lg p-4 space-y-4">
          <h4 className="font-medium text-text-primary">Котировка</h4>
          
          {/* Основная информация */}
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-text-secondary">Вы платите:</span>
              <span className="text-text-primary font-mono font-semibold">
                {quote.fiat_amount} {quote.fiat_currency}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-secondary">Вы получаете:</span>
              <span className="text-accent font-mono font-semibold">
                {quote.crypto_amount} {quote.crypto_currency}
              </span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-secondary">Курс:</span>
              <span className="text-text-primary font-mono">{quote.rate}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-secondary">Комиссия:</span>
              <span className="text-text-primary font-mono">{quote.fee_percent}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-secondary">Действует:</span>
              <span className="text-text-primary font-mono">{quote.expires_in} сек</span>
            </div>
          </div>

          {/* Лимиты */}
          {limits && (
            <div className="text-xs text-text-secondary bg-bg-tertiary rounded p-2">
              Лимиты: {limits.min_amount || 'N/A'} - {limits.max_amount || 'N/A'} {quote.fiat_currency}
            </div>
          )}

          {/* Проверка лимитов */}
          {!isWithinLimits() && (
            <div className="bg-yellow-500/10 border border-yellow-500/30 rounded-lg p-3 text-yellow-500 text-sm">
              ⚠️ Сумма выходит за установленные лимиты
            </div>
          )}

          {/* Кнопка создания заказа */}
          <button
            onClick={handleCreateOrder}
            disabled={creating || !isWithinLimits()}
            className={clsx('btn-primary w-full mt-4', {
              'opacity-50 cursor-not-allowed': creating || !isWithinLimits(),
            })}
          >
            {creating ? (
              <span className="flex items-center justify-center gap-2">
                <svg className="animate-spin h-5 w-5" viewBox="0 0 24 24">
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
                Создание заказа...
              </span>
            ) : (
              `Купить ${cryptoCurrency}`
            )}
          </button>
        </div>
      )}

      {/* Ошибка */}
      {error && (
        <div className="bg-red-500/10 border border-red-500/30 rounded-lg p-3 text-red-500 text-sm">
          {error}
        </div>
      )}
    </div>
  )
}
