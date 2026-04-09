/**
 * TokenSwapForm — Форма обмена токенов через 0x Protocol
 *
 * Позволяет пользователям:
 * - Выбрать сеть (Ethereum, Polygon, Arbitrum, и т.д.)
 * - Выбрать токены для обмена
 * - Ввести сумму
 * - Получить котировку
 * - Выполнить обмен
 */

import React, { useState, useCallback } from 'react'
import clsx from 'clsx'
import {
  ChainId,
  CHAIN_NAMES,
  type SwapQuoteData,
  type SwapQuoteResponse,
  type SwapExecuteResponse,
} from '../../types/web3'
import {
  getSwapQuote,
  executeSwap,
  bpsToPercent,
} from '../../services/web3Service'

// Доступные токены для каждой сети
const TOKENS_BY_CHAIN: Record<number, Array<{ address: string; symbol: string; decimals: number }>> = {
  [ChainId.Ethereum]: [
    { address: '0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE', symbol: 'ETH', decimals: 18 },
    { address: '0xdAC17F958D2ee523a2206206994597C13D831ec7', symbol: 'USDT', decimals: 6 },
    { address: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', symbol: 'USDC', decimals: 6 },
    { address: '0x6B175474E89094C44Da98b954EedeAC495271d0F', symbol: 'DAI', decimals: 18 },
    { address: '0x2260FAC5E5542a773Aa44fBCfeDf7C193bc2C599', symbol: 'WBTC', decimals: 8 },
  ],
  [ChainId.Polygon]: [
    { address: '0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE', symbol: 'MATIC', decimals: 18 },
    { address: '0xc2132D05D31c914a87C6611C10748AEb04B58e8F', symbol: 'USDT', decimals: 6 },
    { address: '0x3c499c542cEF5E3811e1192ce70d8cC03d5c3359', symbol: 'USDC', decimals: 6 },
  ],
  [ChainId.Arbitrum]: [
    { address: '0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE', symbol: 'ETH', decimals: 18 },
    { address: '0xFd086bC7CD5C481DCC9C85ebE478A1C0b69FCbb9', symbol: 'USDT', decimals: 6 },
  ],
  [ChainId.Base]: [
    { address: '0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE', symbol: 'ETH', decimals: 18 },
    { address: '0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913', symbol: 'USDC', decimals: 6 },
  ],
  [ChainId.Optimism]: [
    { address: '0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE', symbol: 'ETH', decimals: 18 },
    { address: '0x94b008aA00579c1307B0EF2c499aD98a8ce58e58', symbol: 'USDT', decimals: 6 },
  ],
  [ChainId.BSC]: [
    { address: '0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE', symbol: 'BNB', decimals: 18 },
    { address: '0x55d398326f99059fF775485246999027B3197955', symbol: 'USDT', decimals: 18 },
  ],
}

interface TokenSwapFormProps {
  /** Адрес кошелька пользователя */
  walletAddress: string
  /** Callback после успешного обмена */
  onSuccess?: (response: SwapExecuteResponse) => void
  /** Callback при ошибке */
  onError?: (error: string) => void
}

export const TokenSwapForm: React.FC<TokenSwapFormProps> = ({
  walletAddress,
  onSuccess,
  onError,
}) => {
  // Состояния формы
  const [chainId, setChainId] = useState<ChainId>(ChainId.Ethereum)
  const [sellToken, setSellToken] = useState(TOKENS_BY_CHAIN[ChainId.Ethereum][0])
  const [buyToken, setBuyToken] = useState(TOKENS_BY_CHAIN[ChainId.Ethereum][1])
  const [sellAmount, setSellAmount] = useState('')
  
  // Состояния загрузки
  const [loading, setLoading] = useState(false)
  const [executing, setExecuting] = useState(false)
  const [quote, setQuote] = useState<SwapQuoteData | null>(null)
  const [error, setError] = useState<string | null>(null)

  // Обновить токены при смене сети
  const handleChainChange = useCallback((newChainId: ChainId) => {
    setChainId(newChainId)
    const tokens = TOKENS_BY_CHAIN[newChainId] || []
    if (tokens.length >= 2) {
      setSellToken(tokens[0])
      setBuyToken(tokens[1])
    }
    setQuote(null)
    setSellAmount('')
  }, [])

  // Получить котировку
  const handleGetQuote = async () => {
    if (!sellAmount || parseFloat(sellAmount) <= 0) {
      setError('Введите сумму для обмена')
      return
    }

    setLoading(true)
    setError(null)

    try {
      const amountWei = (parseFloat(sellAmount) * Math.pow(10, sellToken.decimals)).toString()
      
      const response: SwapQuoteResponse = await getSwapQuote({
        chain_id: chainId,
        sell_token: sellToken.address,
        buy_token: buyToken.address,
        sell_amount: amountWei,
        taker_address: walletAddress,
        slippage_bps: 100, // 1% slippage
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

  // Выполнить обмен
  const handleExecuteSwap = async () => {
    if (!quote) {
      setError('Нет активной котировки')
      return
    }

    setExecuting(true)
    setError(null)

    try {
      const response: SwapExecuteResponse = await executeSwap({
        chain_id: chainId,
        sell_token: sellToken.address,
        buy_token: buyToken.address,
        sell_amount: quote.sell_amount,
        taker_address: walletAddress,
        slippage_bps: 100,
      })

      if (response.success) {
        onSuccess?.(response)
        // Сбросить форму
        setSellAmount('')
        setQuote(null)
      } else {
        setError(response.error || 'Ошибка выполнения обмена')
      }
    } catch (err) {
      const message = err instanceof Error ? err.message : 'Неизвестная ошибка'
      setError(message)
      onError?.(message)
    } finally {
      setExecuting(false)
    }
  }

  // Поменять токены местами
  const handleSwapTokens = () => {
    const temp = sellToken
    setSellToken(buyToken)
    setBuyToken(temp)
    setQuote(null)
  }

  const tokens = TOKENS_BY_CHAIN[chainId] || []

  return (
    <div className="card p-6 space-y-6">
      {/* Заголовок */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-text-primary">Обмен токенов</h3>
        <span className="text-xs text-text-secondary bg-bg-secondary px-2 py-1 rounded">
          0x Protocol
        </span>
      </div>

      {/* Выбор сети */}
      <div className="space-y-2">
        <label className="text-sm font-medium text-text-secondary">Сеть</label>
        <select
          value={chainId}
          onChange={(e) => handleChainChange(Number(e.target.value) as ChainId)}
          className="input-primary w-full"
        >
          {Object.values(ChainId)
            .filter((v) => typeof v === 'number')
            .map((id) => (
              <option key={id} value={id}>
                {CHAIN_NAMES[id as number] || id}
              </option>
            ))}
        </select>
      </div>

      {/* Токен продажи */}
      <div className="space-y-2">
        <label className="text-sm font-medium text-text-secondary">Продаёте</label>
        <div className="flex gap-2">
          <select
            value={sellToken.address}
            onChange={(e) => {
              const token = tokens.find((t) => t.address === e.target.value)
              if (token) setSellToken(token)
            }}
            className="input-primary w-1/3"
          >
            {tokens.map((token) => (
              <option key={token.address} value={token.address}>
                {token.symbol}
              </option>
            ))}
          </select>
          <input
            type="number"
            value={sellAmount}
            onChange={(e) => {
              setSellAmount(e.target.value)
              setQuote(null)
            }}
            placeholder="0.0"
            className="input-primary flex-1"
            step="any"
            min="0"
          />
        </div>
      </div>

      {/* Кнопка смены токенов */}
      <div className="flex justify-center">
        <button
          onClick={handleSwapTokens}
          className="btn-secondary p-2 rounded-full hover:bg-bg-tertiary transition-colors"
          title="Поменять токены местами"
        >
          <svg
            className="w-5 h-5 text-text-primary"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              strokeLinecap="round"
              strokeLinejoin="round"
              strokeWidth={2}
              d="M7 16V4m0 0L3 8m4-4l4 4m6 0v12m0 0l4-4m-4 4l-4-4"
            />
          </svg>
        </button>
      </div>

      {/* Токен покупки */}
      <div className="space-y-2">
        <label className="text-sm font-medium text-text-secondary">Покупаете</label>
        <select
          value={buyToken.address}
          onChange={(e) => {
            const token = tokens.find((t) => t.address === e.target.value)
            if (token) setBuyToken(token)
          }}
          className="input-primary w-full"
        >
          {tokens
            .filter((t) => t.address !== sellToken.address)
            .map((token) => (
              <option key={token.address} value={token.address}>
                {token.symbol}
              </option>
            ))}
        </select>
      </div>

      {/* Кнопка получения котировки */}
      <button
        onClick={handleGetQuote}
        disabled={loading || !sellAmount}
        className={clsx('btn-primary w-full', {
          'opacity-50 cursor-not-allowed': loading || !sellAmount,
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
        <div className="bg-bg-secondary rounded-lg p-4 space-y-3">
          <h4 className="font-medium text-text-primary">Котировка</h4>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-text-secondary">Курс:</span>
              <span className="text-text-primary font-mono">{quote.price}</span>
            </div>
            <div className="flex justify-between">
              <span className="text-text-secondary">Получите:</span>
              <span className="text-text-primary font-mono">
                {(parseFloat(quote.buy_amount) / Math.pow(10, buyToken.decimals)).toFixed(6)} {buyToken.symbol}
              </span>
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

          {/* Кнопка выполнения обмена */}
          <button
            onClick={handleExecuteSwap}
            disabled={executing}
            className={clsx('btn-primary w-full mt-4', {
              'opacity-50 cursor-not-allowed': executing,
            })}
          >
            {executing ? (
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
                Выполнение обмена...
              </span>
            ) : (
              'Обменять'
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
