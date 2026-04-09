# Abcex — Интеграция с UI

## 📋 Обзор

Полное руководство по интеграции Abcex модуля с фронтендом приложения
через Tauri IPC.

### Архитектура

```
Frontend (React/TypeScript)
  ↓
web3Service.ts (invoke)
  ↓
Tauri IPC
  ↓
abcex_commands.rs (Tauri commands)
  ↓
abcex.rs (AbcexClient)
  ↓
Abcex API (https://api.abcex.io/v1)
```

---

## 🔧 Настройка

### 1. Регистрация команд (main.rs)

Команды уже зарегистрированы в `messenger/src/main.rs`:

```rust
#[cfg(feature = "web3")]
{
    use secure_messenger_lib::web3;

    builder = web3::abcex_commands::register_abcex_commands(builder);
}
```

### 2. Импорты в TypeScript

```typescript
// Типы
import type {
  AbcexQuoteRequest,
  AbcexQuoteResponse,
  AbcexQuoteData,
  AbcexOrderRequest,
  AbcexOrderResponse,
  AbcexOrderData,
  AbcexPaymentMethod,
} from '@/types/web3';

// Сервис
import {
  abcexGetQuote,
  abcexCreateOrder,
  abcexGetOrderStatus,
  abcexCheckKyc,
  abcexGetSupportedCryptos,
  abcexGetLimits,
  abcexQuickQuote,
  abcexCalculateFee,
} from '@/services/web3Service';
```

---

## 📱 Компоненты UI

### BuyCryptoWidget.tsx

Полный виджет покупки криптовалюты.

```tsx
import React, { useState, useCallback } from 'react';
import {
  abcexGetQuote,
  abcexCreateOrder,
  abcexCheckKyc,
} from '@/services/web3Service';
import type {
  AbcexQuoteData,
  AbcexOrderData,
  AbcexPaymentMethod,
} from '@/types/web3';

// Константы
const PAYMENT_METHODS: { value: AbcexPaymentMethod; label: string }[] = [
  { value: 'credit_card', label: '💳 Credit Card' },
  { value: 'debit_card', label: '💳 Debit Card' },
  { value: 'sepa', label: '🏦 SEPA' },
  { value: 'bank_transfer', label: '🏦 Bank Transfer' },
  { value: 'apple_pay', label: '🍎 Apple Pay' },
  { value: 'google_pay', label: '🔵 Google Pay' },
];

const CRYPTOS = ['BTC', 'ETH', 'USDT', 'USDC', 'DAI'];
const FIAT_CURRENCIES = ['USD', 'EUR', 'GBP', 'RUB'];

interface BuyCryptoWidgetProps {
  userWalletAddress?: string;
  userEmail?: string;
  userId?: string;
  onOrderComplete?: (order: AbcexOrderData) => void;
}

type Step = 'input' | 'quote' | 'confirm' | 'processing' | 'success' | 'error';

export function BuyCryptoWidget({
  userWalletAddress,
  userEmail,
  userId,
  onOrderComplete,
}: BuyCryptoWidgetProps) {
  const [step, setStep] = useState<Step>('input');
  const [fiatAmount, setFiatAmount] = useState('100');
  const [fiatCurrency, setFiatCurrency] = useState('USD');
  const [crypto, setCrypto] = useState('BTC');
  const [paymentMethod, setPaymentMethod] = useState<AbcexPaymentMethod>('credit_card');
  const [quote, setQuote] = useState<AbcexQuoteData | null>(null);
  const [order, setOrder] = useState<AbcexOrderData | null>(null);
  const [error, setError] = useState<string | null>(null);

  const handleGetQuote = useCallback(async () => {
    setStep('processing');
    setError(null);

    try {
      // Проверить KYC
      if (userId) {
        const kyc = await abcexCheckKyc({ user_id: userId });
        if (kyc.success && kyc.kyc_status && !kyc.kyc_status.verified) {
          setError('Требуется верификация KYC');
          setStep('error');
          return;
        }
      }

      // Получить котировку
      const response = await abcexGetQuote({
        fiat_currency: fiatCurrency,
        fiat_amount: fiatAmount,
        crypto_currency: crypto,
        payment_method: paymentMethod,
        country: 'US',
      });

      if (response.success && response.quote) {
        setQuote(response.quote);
        setStep('quote');
      } else {
        setError(response.error || 'Не удалось получить котировку');
        setStep('error');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Неизвестная ошибка');
      setStep('error');
    }
  }, [fiatAmount, fiatCurrency, crypto, paymentMethod, userId]);

  const handleBuy = useCallback(async () => {
    if (!quote) return;

    setStep('processing');
    setError(null);

    try {
      const response = await abcexCreateOrder({
        quote_id: quote.quote_id,
        deposit_address: userWalletAddress || 'bc1q...',
        payment_method: paymentMethod,
        user_email: userEmail || 'user@example.com',
      });

      if (response.success && response.order) {
        setOrder(response.order);

        // Перенаправить на оплату
        if (response.order.payment_url) {
          window.open(response.order.payment_url, '_blank');
        }

        setStep('success');
        onOrderComplete?.(response.order);
      } else {
        setError(response.error || 'Не удалось создать заказ');
        setStep('error');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Неизвестная ошибка');
      setStep('error');
    }
  }, [quote, userWalletAddress, userEmail, paymentMethod, onOrderComplete]);

  const handleReset = useCallback(() => {
    setStep('input');
    setQuote(null);
    setOrder(null);
    setError(null);
  }, []);

  // Step: Input
  if (step === 'input') {
    return (
      <div className="buy-crypto-widget">
        <h2>Купить криптовалюту</h2>

        <div className="form-group">
          <label>Сумма</label>
          <div className="amount-input-group">
            <input
              type="number"
              value={fiatAmount}
              onChange={(e) => setFiatAmount(e.target.value)}
              placeholder="100"
              min="1"
            />
            <select
              value={fiatCurrency}
              onChange={(e) => setFiatCurrency(e.target.value)}
            >
              {FIAT_CURRENCIES.map((f) => (
                <option key={f} value={f}>
                  {f}
                </option>
              ))}
            </select>
          </div>
        </div>

        <div className="form-group">
          <label>Криптовалюта</label>
          <select value={crypto} onChange={(e) => setCrypto(e.target.value)}>
            {CRYPTOS.map((c) => (
              <option key={c} value={c}>
                {c}
              </option>
            ))}
          </select>
        </div>

        <div className="form-group">
          <label>Способ оплаты</label>
          <div className="payment-methods">
            {PAYMENT_METHODS.map((pm) => (
              <button
                key={pm.value}
                className={paymentMethod === pm.value ? 'active' : ''}
                onClick={() => setPaymentMethod(pm.value)}
              >
                {pm.label}
              </button>
            ))}
          </div>
        </div>

        <button
          className="btn-primary"
          onClick={handleGetQuote}
          disabled={!fiatAmount || parseFloat(fiatAmount) <= 0}
        >
          Получить котировку
        </button>
      </div>
    );
  }

  // Step: Quote
  if (step === 'quote' && quote) {
    return (
      <div className="buy-crypto-widget">
        <h2>Котировка</h2>

        <div className="quote-card">
          <div className="quote-header">
            <span className="quote-id">{quote.quote_id}</span>
            <span className="expires-in">⏱ {quote.expires_in}с</span>
          </div>

          <div className="quote-amounts">
            <div className="pay">
              <span className="label">Платите</span>
              <span className="amount">
                {quote.fiat_amount} {quote.fiat_currency}
              </span>
            </div>

            <div className="arrow">↓</div>

            <div className="receive">
              <span className="label">Получаете</span>
              <span className="amount">
                {quote.crypto_amount} {quote.crypto_currency}
              </span>
            </div>
          </div>

          <div className="quote-details">
            <div className="detail-row">
              <span>Курс</span>
              <span>
                {quote.rate} {quote.fiat_currency}/{quote.crypto_currency}
              </span>
            </div>
            <div className="detail-row">
              <span>Комиссия</span>
              <span>{quote.fee_percent}%</span>
            </div>
            <div className="detail-row">
              <span>Способ оплаты</span>
              <span>{paymentMethod}</span>
            </div>
          </div>

          <div className="quote-actions">
            <button className="btn-secondary" onClick={handleReset}>
              Отмена
            </button>
            <button className="btn-primary" onClick={handleBuy}>
              Купить
            </button>
          </div>
        </div>
      </div>
    );
  }

  // Step: Processing
  if (step === 'processing') {
    return (
      <div className="buy-crypto-widget">
        <div className="processing">
          <div className="spinner" />
          <p>Обработка...</p>
        </div>
      </div>
    );
  }

  // Step: Success
  if (step === 'success' && order) {
    return (
      <div className="buy-crypto-widget">
        <div className="success-card">
          <div className="success-icon">✅</div>
          <h3>Заказ создан!</h3>

          <div className="order-info">
            <p>
              <strong>Order ID:</strong> {order.order_id}
            </p>
            <p>
              <strong>Статус:</strong> {order.status}
            </p>
            <p>
              <strong>Сумма:</strong> {order.crypto_amount}{' '}
              {order.crypto_currency}
            </p>
          </div>

          {order.payment_url && (
            <a
              href={order.payment_url}
              target="_blank"
              rel="noopener noreferrer"
              className="btn-primary"
            >
              Перейти к оплате →
            </a>
          )}

          <button className="btn-secondary" onClick={handleReset}>
            Новая покупка
          </button>
        </div>
      </div>
    );
  }

  // Step: Error
  if (step === 'error') {
    return (
      <div className="buy-crypto-widget">
        <div className="error-card">
          <div className="error-icon">❌</div>
          <h3>Ошибка</h3>
          <p>{error}</p>
          <button className="btn-primary" onClick={handleReset}>
            Попробовать снова
          </button>
        </div>
      </div>
    );
  }

  return null;
}
```

---

## 📊 Стили (CSS)

```css
.buy-crypto-widget {
  max-width: 400px;
  margin: 0 auto;
  padding: 20px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 8px;
  font-weight: 500;
}

.amount-input-group {
  display: flex;
  gap: 8px;
}

.amount-input-group input {
  flex: 1;
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
  font-size: 16px;
}

.amount-input-group select {
  width: 100px;
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
}

.payment-methods {
  display: grid;
  grid-template-columns: repeat(2, 1fr);
  gap: 8px;
}

.payment-methods button {
  padding: 12px;
  border: 1px solid #ddd;
  border-radius: 8px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
}

.payment-methods button.active {
  border-color: #007bff;
  background: #f0f7ff;
}

.quote-card {
  padding: 16px;
  background: #f9f9f9;
  border-radius: 8px;
}

.quote-header {
  display: flex;
  justify-content: space-between;
  margin-bottom: 16px;
  font-size: 12px;
  color: #666;
}

.quote-amounts {
  display: flex;
  flex-direction: column;
  gap: 8px;
  margin-bottom: 16px;
}

.quote-amounts .pay,
.quote-amounts .receive {
  padding: 12px;
  background: white;
  border-radius: 8px;
}

.quote-amounts .arrow {
  text-align: center;
  font-size: 20px;
}

.detail-row {
  display: flex;
  justify-content: space-between;
  padding: 8px 0;
  border-bottom: 1px solid #eee;
}

.quote-actions,
.order-actions {
  display: flex;
  gap: 8px;
  margin-top: 16px;
}

.btn-primary,
.btn-secondary {
  flex: 1;
  padding: 12px;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-primary {
  background: #007bff;
  color: white;
}

.btn-primary:hover {
  background: #0056b3;
}

.btn-secondary {
  background: #eee;
  color: #333;
}

.btn-secondary:hover {
  background: #ddd;
}

.processing {
  text-align: center;
  padding: 40px;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 3px solid #eee;
  border-top-color: #007bff;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
}

@keyframes spin {
  to {
    transform: rotate(360deg);
  }
}

.success-card,
.error-card {
  text-align: center;
  padding: 40px 20px;
}

.success-icon,
.error-icon {
  font-size: 48px;
  margin-bottom: 16px;
}
```

---

## 🔄 State management

### Использование с Zustand

```typescript
import { create } from 'zustand';
import { abcexGetQuote, abcexCreateOrder } from '@/services/web3Service';
import type { AbcexQuoteData, AbcexOrderData } from '@/types/web3';

interface AbcexStore {
  quote: AbcexQuoteData | null;
  order: AbcexOrderData | null;
  loading: boolean;
  error: string | null;

  getQuote: (request: AbcexQuoteRequest) => Promise<void>;
  createOrder: (request: AbcexOrderRequest) => Promise<void>;
  reset: () => void;
}

export const useAbcexStore = create<AbcexStore>((set) => ({
  quote: null,
  order: null,
  loading: false,
  error: null,

  getQuote: async (request) => {
    set({ loading: true, error: null });

    try {
      const response = await abcexGetQuote(request);

      if (response.success && response.quote) {
        set({ quote: response.quote, loading: false });
      } else {
        set({ error: response.error || 'Failed to get quote', loading: false });
      }
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Unknown error',
        loading: false,
      });
    }
  },

  createOrder: async (request) => {
    set({ loading: true, error: null });

    try {
      const response = await abcexCreateOrder(request);

      if (response.success && response.order) {
        set({ order: response.order, loading: false });
      } else {
        set({ error: response.error || 'Failed to create order', loading: false });
      }
    } catch (err) {
      set({
        error: err instanceof Error ? err.message : 'Unknown error',
        loading: false,
      });
    }
  },

  reset: () => set({ quote: null, order: null, loading: false, error: null }),
}));
```

---

## 🎯 Интеграция с кошельком

### Получение адреса депозита

```typescript
import { invoke } from '@tauri-apps/api/core';

async function getDepositAddress(crypto: string): Promise<string> {
  // Получить адрес из локального кошелька
  switch (crypto) {
    case 'BTC':
      return invoke<string>('get_btc_deposit_address');
    case 'ETH':
      return invoke<string>('get_eth_deposit_address');
    case 'USDT':
      // USDT на Ethereum
      return invoke<string>('get_eth_deposit_address');
    default:
      throw new Error(`Unsupported crypto: ${crypto}`);
  }
}

// Использование в BuyCryptoWidget
const handleBuy = async () => {
  const depositAddress = await getDepositAddress(crypto);

  const response = await abcexCreateOrder({
    quote_id: quote!.quote_id,
    deposit_address: depositAddress,
    payment_method: paymentMethod,
    user_email: userEmail,
  });

  // ...
};
```

---

## 📝 Логирование

### Client-side logging

```typescript
import { abcexGetQuote } from '@/services/web3Service';

async function getQuoteWithLogging(request: AbcexQuoteRequest) {
  console.log('📊 Abcex Quote Request:', request);

  const startTime = Date.now();
  const response = await abcexGetQuote(request);
  const duration = Date.now() - startTime;

  console.log('📊 Abcex Quote Response:', {
    ...response,
    duration: `${duration}ms`,
  });

  return response;
}
```

---

## 📞 Ресурсы

- [Tauri Invoke API](https://tauri.app/v1/api/js/#invoke)
- [React Documentation](https://react.dev/)
- [Zustand Documentation](https://zustand-demo.pmnd.rs/)
