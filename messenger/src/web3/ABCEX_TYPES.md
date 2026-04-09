# Abcex — TypeScript типы и сервис

## 📋 Обзор

Полная документация по TypeScript типам и сервисным функциям для интеграции
с Abcex API через Tauri команды.

### Файлы

- **Типы**: `frontend/src/types/web3.ts`
- **Сервис**: `frontend/src/services/web3Service.ts`

---

## 🎨 Типы данных

### AbcexPaymentMethod

```typescript
export type AbcexPaymentMethod =
  | 'credit_card'      // Кредитная карта
  | 'debit_card'       // Дебетовая карта
  | 'sepa'             // SEPA перевод
  | 'bank_transfer'    // Банковский перевод
  | 'apple_pay'        // Apple Pay
  | 'google_pay';      // Google Pay
```

### AbcexQuoteRequest

Запрос на получение котировки.

```typescript
export interface AbcexQuoteRequest {
  /** Валюта фиата (USD, EUR, GBP, RUB) */
  fiat_currency: string;

  /** Сумма в фиате */
  fiat_amount: string;

  /** Криптовалюта (BTC, ETH, USDT) */
  crypto_currency: string;

  /** Способ оплаты (опционально) */
  payment_method?: string;

  /** Код страны (ISO 3166-1 alpha-2) */
  country?: string;
}
```

**Пример использования:**

```typescript
const request: AbcexQuoteRequest = {
  fiat_currency: 'USD',
  fiat_amount: '100',
  crypto_currency: 'BTC',
  payment_method: 'credit_card',
  country: 'US',
};
```

### AbcexQuoteData

Данные котировки.

```typescript
export interface AbcexQuoteData {
  /** ID котировки */
  quote_id: string;

  /** Валюта фиата */
  fiat_currency: string;

  /** Сумма фиата */
  fiat_amount: string;

  /** Криптовалюта */
  crypto_currency: string;

  /** Ожидаемая сумма криптовалюты */
  crypto_amount: string;

  /** Курс обмена */
  rate: string;

  /** Сумма комиссии */
  fee_amount: string;

  /** Процент комиссии */
  fee_percent: string;

  /** Доступные способы оплаты */
  payment_methods: string[];

  /** Время действия котировки (секунды) */
  expires_in: number;

  /** Минимальная сумма */
  min_amount: string;

  /** Максимальная сумма */
  max_amount: string;
}
```

### AbcexQuoteResponse

Ответ на запрос котировки.

```typescript
export interface AbcexQuoteResponse {
  success: boolean;
  quote?: AbcexQuoteData | null;
  error?: string | null;
}
```

**Пример использования:**

```typescript
const response = await abcexGetQuote(request);

if (response.success && response.quote) {
  const quote = response.quote;
  console.log(`Получите: ${quote.crypto_amount} ${quote.crypto_currency}`);
  console.log(`Курс: ${quote.rate}`);
  console.log(`Комиссия: ${quote.fee_percent}%`);
} else {
  console.error('Ошибка:', response.error);
}
```

### AbcexOrderRequest

Запрос на создание заказа.

```typescript
export interface AbcexOrderRequest {
  /** ID котировки */
  quote_id: string;

  /** Адрес для получения криптовалюты */
  deposit_address: string;

  /** Способ оплаты */
  payment_method: string;

  /** Email пользователя */
  user_email: string;
}
```

**Пример использования:**

```typescript
const orderRequest: AbcexOrderRequest = {
  quote_id: 'quote_abc123',
  deposit_address: 'bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh',
  payment_method: 'credit_card',
  user_email: 'user@example.com',
};
```

### AbcexOrderData

Данные заказа.

```typescript
export interface AbcexOrderData {
  /** ID заказа */
  order_id: string;

  /** ID котировки */
  quote_id: string;

  /** Статус заказа */
  status: string;

  /** Валюта фиата */
  fiat_currency: string;

  /** Сумма фиата */
  fiat_amount: string;

  /** Криптовалюта */
  crypto_currency: string;

  /** Сумма криптовалюты */
  crypto_amount: string;

  /** Курс */
  rate: string;

  /** Комиссия */
  fee_amount: string;

  /** Способ оплаты */
  payment_method: string;

  /** Адрес депозита */
  deposit_address: string;

  /** URL для оплаты */
  payment_url?: string | null;

  /** Время создания (timestamp ms) */
  created_at: number;
}
```

### AbcexOrderResponse

Ответ на создание заказа.

```typescript
export interface AbcexOrderResponse {
  success: boolean;
  order?: AbcexOrderData | null;
  error?: string | null;
}
```

### AbcexKycRequest

```typescript
export interface AbcexKycRequest {
  /** ID пользователя */
  user_id: string;
}
```

### AbcexKycData

Данные KYC верификации.

```typescript
export interface AbcexKycData {
  /** Верифицирован ли */
  verified: boolean;

  /** Уровень верификации */
  level: string;

  /** Дневной лимит */
  daily_limit: string;

  /** Месячный лимит */
  monthly_limit: string;

  /** Осталось на сегодня */
  remaining_daily: string;

  /** Осталось на месяц */
  remaining_monthly: string;
}
```

### AbcexKycResponse

```typescript
export interface AbcexKycResponse {
  success: boolean;
  kyc_status?: AbcexKycData | null;
  error?: string | null;
}
```

### AbcexLimitsRequest

```typescript
export interface AbcexLimitsRequest {
  /** Код страны */
  country: string;
}
```

### AbcexLimitsResponse

```typescript
export interface AbcexLimitsResponse {
  success: boolean;
  limits?: Record<string, string> | null;
  error?: string | null;
}
```

### AbcexQuickQuoteParams

```typescript
export interface AbcexQuickQuoteParams {
  fiat_currency: string;
  fiat_amount: string;
  crypto_currency: string;
  payment_method: string;
  country?: string;
}
```

---

## 🔧 Сервисные функции

### abcexGetQuote

Получить котировку для покупки.

```typescript
export async function abcexGetQuote(
  request: AbcexQuoteRequest
): Promise<AbcexQuoteResponse>
```

**Пример:**

```typescript
const response = await abcexGetQuote({
  fiat_currency: 'USD',
  fiat_amount: '100',
  crypto_currency: 'BTC',
  payment_method: 'credit_card',
  country: 'US',
});

if (response.success) {
  console.log('Получите:', response.quote?.crypto_amount, 'BTC');
}
```

### abcexCreateOrder

Создать заказ на покупку.

```typescript
export async function abcexCreateOrder(
  request: AbcexOrderRequest
): Promise<AbcexOrderResponse>
```

**Пример:**

```typescript
const order = await abcexCreateOrder({
  quote_id: 'quote_123',
  deposit_address: 'bc1q...',
  payment_method: 'credit_card',
  user_email: 'user@example.com',
});

if (order.success && order.order?.payment_url) {
  window.open(order.order.payment_url);
}
```

### abcexGetOrderStatus

Проверить статус заказа.

```typescript
export async function abcexGetOrderStatus(
  orderId: string
): Promise<AbcexOrderResponse>
```

**Пример:**

```typescript
const status = await abcexGetOrderStatus('order_abc123');

if (status.success) {
  console.log('Статус:', status.order?.status);
}
```

### abcexCheckKyc

Проверить KYC статус.

```typescript
export async function abcexCheckKyc(
  request: AbcexKycRequest
): Promise<AbcexKycResponse>
```

**Пример:**

```typescript
const kyc = await abcexCheckKyc({ user_id: 'user_123' });

if (kyc.success && kyc.kyc_status) {
  console.log('Верифицирован:', kyc.kyc_status.verified);
  console.log('Лимиты:', kyc.kyc_status.daily_limit);
}
```

### abcexGetSupportedCryptos

Получить список криптовалют.

```typescript
export async function abcexGetSupportedCryptos(): Promise<string[]>
```

**Пример:**

```typescript
const cryptos = await abcexGetSupportedCryptos();
console.log('Доступные:', cryptos);
// ['BTC', 'ETH', 'USDT', 'USDC', ...]
```

### abcexGetLimits

Получить лимиты для страны.

```typescript
export async function abcexGetLimits(
  request: AbcexLimitsRequest
): Promise<AbcexLimitsResponse>
```

**Пример:**

```typescript
const limits = await abcexGetLimits({ country: 'US' });

if (limits.success && limits.limits) {
  console.log('Daily limit:', limits.limits['daily_limit']);
}
```

### abcexQuickQuote

Быстрая котировка.

```typescript
export async function abcexQuickQuote(
  params: AbcexQuickQuoteParams
): Promise<string>
```

**Пример:**

```typescript
const quoteJson = await abcexQuickQuote({
  fiat_currency: 'EUR',
  fiat_amount: '500',
  crypto_currency: 'ETH',
  payment_method: 'sepa',
  country: 'DE',
});

const quote = JSON.parse(quoteJson);
console.log('Получите:', quote.crypto_amount, 'ETH');
```

### abcexCalculateFee

Рассчитать комиссию.

```typescript
export async function abcexCalculateFee(
  amount: string,
  feeBps: number
): Promise<string>
```

**Пример:**

```typescript
const fee = await abcexCalculateFee('100', 200);
console.log('Комиссия:', fee); // "2.00"
```

---

## 🛠 Helper функции

### formatTokenAmount

Форматировать сумму с символами.

```typescript
export function formatTokenAmount(
  amount: string,
  symbol: string,
  decimals = 2
): string
```

**Примеры:**

```typescript
formatTokenAmount('0.00152345', 'BTC');  // "0.00 BTC"
formatTokenAmount('1.5', 'ETH', 4);      // "1.5000 ETH"
formatTokenAmount('100', 'USD');         // "100.00 USD"
```

### bpsToPercent

Конвертировать basis points в проценты.

```typescript
export function bpsToPercent(bps: number): string
```

**Примеры:**

```typescript
bpsToPercent(200);  // "2.0%"
bpsToPercent(250);  // "2.5%"
bpsToPercent(300);  // "3.0%"
```

### percentToBps

Конвертировать проценты в basis points.

```typescript
export function percentToBps(percent: number): number
```

**Примеры:**

```typescript
percentToBps(2.0);   // 200
percentToBps(2.5);   // 250
percentToBps(3.0);   // 300
```

---

## 📦 Импорты

```typescript
// Типы
import type {
  AbcexPaymentMethod,
  AbcexQuoteRequest,
  AbcexQuoteResponse,
  AbcexQuoteData,
  AbcexOrderRequest,
  AbcexOrderResponse,
  AbcexOrderData,
  AbcexKycRequest,
  AbcexKycResponse,
  AbcexKycData,
  AbcexLimitsRequest,
  AbcexLimitsResponse,
  AbcexQuickQuoteParams,
} from '@/types/web3';

// Сервисные функции
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

// Helpers
import { formatTokenAmount, bpsToPercent, percentToBps } from '@/services/web3Service';
```

---

## ⚠️ Обработка ошибок

### Типичные ошибки

```typescript
try {
  const response = await abcexGetQuote(request);

  if (!response.success) {
    // Ошибка от Abcex API
    console.error('Abcex error:', response.error);
    return;
  }

  // Успех
  console.log('Quote:', response.quote);
} catch (err) {
  // Tauri invoke error
  console.error('Tauri error:', err);
}
```

### Retry логика

```typescript
async function getQuoteWithRetry(
  request: AbcexQuoteRequest,
  retries = 3
): Promise<AbcexQuoteResponse> {
  for (let i = 0; i < retries; i++) {
    try {
      const response = await abcexGetQuote(request);
      if (response.success) {
        return response;
      }

      if (i === retries - 1) {
        return response; // Последний attempt
      }

      // Wait before retry
      await new Promise((resolve) => setTimeout(resolve, 1000 * (i + 1)));
    } catch (err) {
      if (i === retries - 1) {
        throw err;
      }
    }
  }

  throw new Error('Failed to get quote');
}
```

---

## 🎨 React пример

### Компонент выбора способа оплаты

```tsx
import React from 'react';
import type { AbcexPaymentMethod } from '@/types/web3';

const PAYMENT_METHODS: {
  value: AbcexPaymentMethod;
  label: string;
  icon: string;
  speed: string;
}[] = [
  { value: 'credit_card', label: 'Credit Card', icon: '💳', speed: 'Instant' },
  { value: 'debit_card', label: 'Debit Card', icon: '💳', speed: 'Instant' },
  { value: 'sepa', label: 'SEPA', icon: '🏦', speed: '1-2 days' },
  { value: 'bank_transfer', label: 'Bank Transfer', icon: '🏦', speed: '2-3 days' },
  { value: 'apple_pay', label: 'Apple Pay', icon: '🍎', speed: 'Instant' },
  { value: 'google_pay', label: 'Google Pay', icon: '🔵', speed: 'Instant' },
];

interface PaymentMethodSelectorProps {
  value: AbcexPaymentMethod;
  onChange: (method: AbcexPaymentMethod) => void;
}

export function PaymentMethodSelector({
  value,
  onChange,
}: PaymentMethodSelectorProps) {
  return (
    <div className="payment-methods">
      {PAYMENT_METHODS.map((method) => (
        <button
          key={method.value}
          className={value === method.value ? 'active' : ''}
          onClick={() => onChange(method.value)}
        >
          <span className="icon">{method.icon}</span>
          <span className="label">{method.label}</span>
          <span className="speed">{method.speed}</span>
        </button>
      ))}
    </div>
  );
}
```

---

## 📞 Ресурсы

- [Abcex API Docs](https://abcex.io/docs/api)
- [Abcex Website](https://abcex.io)
- [Tauri Invoke API](https://tauri.app/v1/api/js/#invoke)
