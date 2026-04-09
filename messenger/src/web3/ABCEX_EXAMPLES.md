# Abcex Integration — Примеры использования

## 📋 Содержание

1. [Базовая котировка](#1-базовая-котировка)
2. [Полный процесс покупки](#2-полный-процесс-покупки)
3. [Проверка KYC](#3-проверка-kyc)
4. [Лимиты по странам](#4-лимиты-по-странам)
5. [Список криптовалют](#5-список-криптовалют)
6. [Расчёт комиссии](#6-расчёт-комиссии)
7. [Проверка статуса заказа](#7-проверка-статуса-заказа)
8. [Tauri команды из UI](#8-tauri-команды-из-ui)
9. [React компонент](#9-react-компонент)
10. [Обработка ошибок](#10-обработка-ошибок)
11. [Builder pattern](#11-builder-pattern)
12. [Сравнение способов оплаты](#12-сравнение-способов-оплаты)

---

## 1. Базовая котировка

### Rust

```rust
use messenger::web3::abcex::{AbcexClient, BuyQuoteRequest, PaymentMethod};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AbcexClient::new(None, None);

    // Получить котировку: 100 USD → BTC
    let request = BuyQuoteRequest {
        fiat_currency: "USD".to_string(),
        fiat_amount: "100".to_string(),
        crypto_currency: "BTC".to_string(),
        payment_method: Some("credit_card".to_string()),
        country: Some("US".to_string()),
    };

    let quote = client.get_buy_quote(request).await?;

    println!("💰 Котировка Abcex");
    println!("─────────────────────────");
    println!("Платите: {} {}", quote.fiat_amount, quote.fiat_currency);
    println!("Получаете: {} {}", quote.crypto_amount, quote.crypto_currency);
    println!("Курс: {} {}/{}", quote.rate, quote.fiat_currency, quote.crypto_currency);
    println!("Комиссия: {}%", quote.fee_percent);
    println!("Действует: {} секунд", quote.expires_in);
    println!("Минимум: {} {}", quote.min_amount, quote.fiat_currency);
    println!("Максимум: {} {}", quote.max_amount, quote.fiat_currency);
    println!("Способы оплаты: {:?}", quote.payment_methods);

    Ok(())
}
```

### TypeScript (UI)

```typescript
import { abcexGetQuote } from '@/services/web3Service';
import type { AbcexQuoteRequest } from '@/types/web3';

async function getQuote() {
  const request: AbcexQuoteRequest = {
    fiat_currency: 'USD',
    fiat_amount: '100',
    crypto_currency: 'BTC',
    payment_method: 'credit_card',
    country: 'US',
  };

  const response = await abcexGetQuote(request);

  if (response.success && response.quote) {
    const quote = response.quote;
    console.log(`Получите: ${quote.crypto_amount} BTC`);
    console.log(`Курс: ${quote.rate} USD/BTC`);
    console.log(`Комиссия: ${quote.fee_percent}%`);
  } else {
    console.error('Ошибка:', response.error);
  }
}
```

---

## 2. Полный процесс покупки

### Rust

```rust
use messenger::web3::abcex::{AbcexClient, BuyQuoteBuilder, PaymentMethod};

async fn buy_crypto(
    fiat_currency: &str,
    fiat_amount: &str,
    crypto: &str,
    deposit_address: &str,
    user_email: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = AbcexClient::with_fee(None, None, 200); // 2%

    // 1. Получить котировку
    let request = BuyQuoteBuilder::new(fiat_currency, fiat_amount, crypto)
        .payment_method(PaymentMethod::CreditCard)
        .build();

    let quote = client.get_buy_quote(request).await?;
    println!("📊 Котировка получена");
    println!("  Платите: {} {}", quote.fiat_amount, quote.fiat_currency);
    println!("  Получаете: {} {}", quote.crypto_amount, quote.crypto_currency);

    // 2. Создать заказ
    let order = client.create_buy_order(
        quote.quote_id,
        deposit_address.to_string(),
        PaymentMethod::CreditCard,
        user_email.to_string(),
    ).await?;

    println!("✅ Заказ создан");
    println!("  Order ID: {}", order.order_id);
    println!("  Статус: {:?}", order.status);

    // 3. Payment URL для редиректа
    if let Some(url) = &order.payment_url {
        println!("🔗 Оплата: {}", url);
        // В реальном приложении: open_url(url);
    }

    Ok(())
}

// Использование
buy_crypto(
    "USD",
    "100",
    "BTC",
    "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh",
    "user@example.com",
).await?;
```

---

## 3. Проверка KYC

### Rust

```rust
use messenger::web3::abcex::AbcexClient;

async fn check_user_kyc(user_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = AbcexClient::new(None, None);

    let kyc = client.check_kyc_status(user_id).await?;

    println!("🔐 KYC Статус");
    println!("─────────────────────────");
    println!("Верифицирован: {}", if kyc.verified { "✅" } else { "❌" });
    println!("Уровень: {}", kyc.level);
    println!();
    println!("Лимиты:");
    println!("  Дневной: {}/{}", kyc.limits.remaining_daily, kyc.limits.daily_limit);
    println!("  Месячный: {}/{}", kyc.limits.remaining_monthly, kyc.limits.monthly_limit);

    // Проверить, достаточно ли лимитов
    let daily_remaining: f64 = kyc.limits.remaining_daily.parse().unwrap_or(0.0);
    if daily_remaining < 100.0 {
        println!("⚠️ Дневной лимит менее $100");
    }

    Ok(())
}

check_user_kyc("user_123").await?;
```

---

## 4. Лимиты по странам

### Rust

```rust
use messenger::web3::abcex::AbcexClient;

async fn show_country_limits(country: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = AbcexClient::new(None, None);

    let limits = client.get_limits(country).await?;

    println!("🌍 Лимиты для {}", country);
    println!("─────────────────────────");

    for (key, value) in &limits {
        println!("  {}: {}", key, value);
    }

    Ok(())
}

// Проверить для нескольких стран
show_country_limits("US").await?;
show_country_limits("DE").await?;
show_country_limits("RU").await?;
```

---

## 5. Список криптовалют

### Rust

```rust
use messenger::web3::abcex::AbcexClient;

async fn list_cryptos() -> Result<(), Box<dyn std::error::Error>> {
    let client = AbcexClient::new(None, None);

    let cryptos = client.get_supported_cryptos().await?;

    println!("💎 Доступные криптовалюты:");
    for crypto in &cryptos {
        println!("  • {}", crypto);
    }

    Ok(())
}

list_cryptos().await?;
```

### TypeScript (UI)

```typescript
import { abcexGetSupportedCryptos } from '@/services/web3Service';

async function showCryptos() {
  const cryptos = await abcexGetSupportedCryptos();
  console.log('Доступные криптовалюты:', cryptos);
}
```

---

## 6. Расчёт комиссии

### Rust

```rust
use messenger::web3::abcex::calculate_fee;

fn show_fees() {
    println!("💸 Комиссии Abcex");
    println!("─────────────────────────");

    // 2% комиссия
    let fee1 = calculate_fee("100", 200).unwrap();
    println!("$100 при 2%: комиссия ${}", fee1);

    // 2.5% комиссия
    let fee2 = calculate_fee("500", 250).unwrap();
    println!("$500 при 2.5%: комиссия ${}", fee2);

    // 3% комиссия
    let fee3 = calculate_fee("1000", 300).unwrap();
    println!("$1000 при 3%: комиссия ${}", fee3);
}

show_fees();
```

### TypeScript (UI)

```typescript
function calculateFee(amount: number, feePercent: number): number {
  return amount * (feePercent / 100);
}

// Примеры
console.log('$100 при 2%:', calculateFee(100, 2));    // $2
console.log('$500 при 2.5%:', calculateFee(500, 2.5)); // $12.5
```

---

## 7. Проверка статуса заказа

### Rust

```rust
use messenger::web3::abcex::AbcexClient;

async fn check_order(order_id: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = AbcexClient::new(None, None);

    let order = client.get_order_status(order_id).await?;

    println!("📦 Заказ {}", order.order_id);
    println!("─────────────────────────");
    println!("Статус: {:?}", order.status);
    println!("Платеж: {} {}", order.fiat_amount, order.fiat_currency);
    println!("Крипто: {} {}", order.crypto_amount, order.crypto_currency);
    println!("Создан: {}", order.created_at);
    println!("Обновлен: {}", order.updated_at);

    // Проверить статус
    match &order.status {
        messenger::web3::abcex::BuyOrderStatus::Pending => {
            println!("⏳ Ожидает оплаты");
        }
        messenger::web3::abcex::BuyOrderStatus::Processing => {
            println!("⚙️ В обработке");
        }
        messenger::web3::abcex::BuyOrderStatus::CryptoSent { tx_hash } => {
            println!("✅ Crypto отправлен");
            println!("TX: {}", tx_hash);
        }
        messenger::web3::abcex::BuyOrderStatus::Completed => {
            println!("✅ Завершен");
        }
        messenger::web3::abcex::BuyOrderStatus::Failed { error } => {
            println!("❌ Ошибка: {}", error);
        }
        _ => {}
    }

    Ok(())
}

check_order("order_abc123").await?;
```

---

## 8. Tauri команды из UI

### TypeScript — Все команды

```typescript
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

// 1. Получить котировку
const quote = await abcexGetQuote({
  fiat_currency: 'USD',
  fiat_amount: '100',
  crypto_currency: 'BTC',
  payment_method: 'credit_card',
  country: 'US',
});

// 2. Создать заказ
const order = await abcexCreateOrder({
  quote_id: quote.quote!.quote_id,
  deposit_address: 'bc1q...',
  payment_method: 'credit_card',
  user_email: 'user@example.com',
});

// 3. Проверить статус
const status = await abcexGetOrderStatus(order.order!.order_id);

// 4. Проверить KYC
const kyc = await abcexCheckKyc({ user_id: 'user_123' });

// 5. Список криптовалют
const cryptos = await abcexGetSupportedCryptos();

// 6. Лимиты
const limits = await abcexGetLimits({ country: 'US' });

// 7. Быстрая котировка
const quickQuote = await abcexQuickQuote({
  fiat_currency: 'EUR',
  fiat_amount: '500',
  crypto_currency: 'ETH',
  payment_method: 'sepa',
  country: 'DE',
});

// 8. Комиссия
const fee = await abcexCalculateFee('100', 200);
```

---

## 9. React компонент

### BuyCryptoForm.tsx

```tsx
import React, { useState } from 'react';
import { abcexGetQuote, abcexCreateOrder } from '@/services/web3Service';
import type {
  AbcexQuoteData,
  AbcexOrderData,
  AbcexPaymentMethod,
} from '@/types/web3';

const PAYMENT_METHODS: { value: AbcexPaymentMethod; label: string }[] = [
  { value: 'credit_card', label: '💳 Credit Card' },
  { value: 'debit_card', label: '💳 Debit Card' },
  { value: 'sepa', label: '🏦 SEPA' },
  { value: 'bank_transfer', label: '🏦 Bank Transfer' },
  { value: 'apple_pay', label: '🍎 Apple Pay' },
  { value: 'google_pay', label: '🔵 Google Pay' },
];

const CRYPTOS = ['BTC', 'ETH', 'USDT', 'USDC'];
const FIAT = ['USD', 'EUR', 'GBP', 'RUB'];

export function BuyCryptoForm() {
  const [fiatAmount, setFiatAmount] = useState('100');
  const [fiatCurrency, setFiatCurrency] = useState('USD');
  const [crypto, setCrypto] = useState('BTC');
  const [paymentMethod, setPaymentMethod] = useState<AbcexPaymentMethod>('credit_card');
  const [quote, setQuote] = useState<AbcexQuoteData | null>(null);
  const [order, setOrder] = useState<AbcexOrderData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleGetQuote = async () => {
    setLoading(true);
    setError(null);

    try {
      const response = await abcexGetQuote({
        fiat_currency: fiatCurrency,
        fiat_amount: fiatAmount,
        crypto_currency: crypto,
        payment_method: paymentMethod,
        country: 'US',
      });

      if (response.success && response.quote) {
        setQuote(response.quote);
      } else {
        setError(response.error || 'Не удалось получить котировку');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  const handleBuy = async () => {
    if (!quote) return;

    setLoading(true);
    setError(null);

    try {
      const response = await abcexCreateOrder({
        quote_id: quote.quote_id,
        deposit_address: 'bc1q...', // TODO: из кошелька пользователя
        payment_method: paymentMethod,
        user_email: 'user@example.com', // TODO: из профиля
      });

      if (response.success && response.order) {
        setOrder(response.order);

        // Перенаправить на оплату
        if (response.order.payment_url) {
          window.open(response.order.payment_url, '_blank');
        }
      } else {
        setError(response.error || 'Не удалось создать заказ');
      }
    } catch (err) {
      setError(err instanceof Error ? err.message : 'Unknown error');
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="buy-crypto-form">
      <h2>Купить криптовалюту</h2>

      <div className="form-group">
        <label>Сумма</label>
        <div className="amount-input">
          <input
            type="number"
            value={fiatAmount}
            onChange={(e) => setFiatAmount(e.target.value)}
            placeholder="100"
          />
          <select value={fiatCurrency} onChange={(e) => setFiatCurrency(e.target.value)}>
            {FIAT.map((f) => (
              <option key={f} value={f}>{f}</option>
            ))}
          </select>
        </div>
      </div>

      <div className="form-group">
        <label>Криптовалюта</label>
        <select value={crypto} onChange={(e) => setCrypto(e.target.value)}>
          {CRYPTOS.map((c) => (
            <option key={c} value={c}>{c}</option>
          ))}
        </select>
      </div>

      <div className="form-group">
        <label>Способ оплаты</label>
        <select
          value={paymentMethod}
          onChange={(e) => setPaymentMethod(e.target.value as AbcexPaymentMethod)}
        >
          {PAYMENT_METHODS.map((pm) => (
            <option key={pm.value} value={pm.value}>{pm.label}</option>
          ))}
        </select>
      </div>

      <button onClick={handleGetQuote} disabled={loading}>
        {loading ? 'Загрузка...' : 'Получить котировку'}
      </button>

      {error && <div className="error">{error}</div>}

      {quote && (
        <div className="quote-card">
          <h3>Котировка</h3>
          <div className="quote-details">
            <p>Платите: <strong>{quote.fiat_amount} {quote.fiat_currency}</strong></p>
            <p>Получаете: <strong>{quote.crypto_amount} {quote.crypto_currency}</strong></p>
            <p>Курс: {quote.rate}</p>
            <p>Комиссия: {quote.fee_percent}%</p>
            <p>Действует: {quote.expires_in} сек</p>
          </div>

          <button onClick={handleBuy} disabled={loading}>
            Купить
          </button>
        </div>
      )}

      {order && (
        <div className="order-card">
          <h3>Заказ создан</h3>
          <p>Order ID: {order.order_id}</p>
          <p>Статус: {order.status}</p>
          {order.payment_url && (
            <a href={order.payment_url} target="_blank" rel="noopener noreferrer">
              Перейти к оплате →
            </a>
          )}
        </div>
      )}
    </div>
  );
}
```

---

## 10. Обработка ошибок

### Rust

```rust
use messenger::web3::abcex::AbcexClient;
use messenger::web3::Web3Error;

async fn buy_with_error_handling() {
    let client = AbcexClient::new(None, None);

    let request = messenger::web3::abcex::BuyQuoteBuilder::new("USD", "100", "BTC")
        .payment_method(messenger::web3::abcex::PaymentMethod::CreditCard)
        .build();

    match client.get_buy_quote(request).await {
        Ok(quote) => {
            println!("✅ Успех: {} BTC", quote.crypto_amount);
        }
        Err(Web3Error::Network(err)) => {
            eprintln!("🌐 Ошибка сети: {}", err);
            // Retry logic
        }
        Err(Web3Error::Wallet(err)) => {
            eprintln!("💰 Ошибка кошелька: {}", err);
        }
        Err(e) => {
            eprintln!("❌ Ошибка: {}", e);
        }
    }
}
```

### TypeScript

```typescript
import { abcexGetQuote } from '@/services/web3Service';

async function buyWithErrorHandling() {
  try {
    const response = await abcexGetQuote({
      fiat_currency: 'USD',
      fiat_amount: '100',
      crypto_currency: 'BTC',
      payment_method: 'credit_card',
    });

    if (response.success && response.quote) {
      console.log('✅ Успех:', response.quote.crypto_amount, 'BTC');
    } else {
      console.error('❌ Ошибка Abcex:', response.error);
    }
  } catch (err) {
    if (err instanceof Error) {
      console.error('🌐 Network error:', err.message);
    } else {
      console.error('❌ Unknown error:', err);
    }
  }
}
```

---

## 11. Builder pattern

### Rust

```rust
use messenger::web3::abcex::{AbcexClient, BuyQuoteBuilder, PaymentMethod};

fn examples() {
    // Минимальный запрос
    let minimal = BuyQuoteBuilder::new("USD", "100", "BTC").build();
    assert_eq!(minimal.fiat_currency, "USD");
    assert_eq!(minimal.fiat_amount, "100");
    assert_eq!(minimal.crypto_currency, "BTC");
    assert!(minimal.payment_method.is_none());
    assert!(minimal.country.is_none());

    // С способом оплаты
    let with_payment = BuyQuoteBuilder::new("EUR", "500", "ETH")
        .payment_method(PaymentMethod::SEPA)
        .build();
    assert_eq!(with_payment.payment_method, Some("sepa".to_string()));

    // Полный запрос
    let full = BuyQuoteBuilder::new("USD", "1000", "BTC")
        .payment_method(PaymentMethod::CreditCard)
        .country("US")
        .build();
    assert_eq!(full.payment_method, Some("credit_card".to_string()));
    assert_eq!(full.country, Some("US".to_string()));
}
```

---

## 12. Сравнение способов оплаты

### Rust

```rust
use messenger::web3::abcex::{AbcexClient, BuyQuoteBuilder, PaymentMethod};

async fn compare_payment_methods() -> Result<(), Box<dyn std::error::Error>> {
    let client = AbcexClient::new(None, None);
    let methods = [
        PaymentMethod::CreditCard,
        PaymentMethod::DebitCard,
        PaymentMethod::SEPA,
        PaymentMethod::BankTransfer,
        PaymentMethod::ApplePay,
        PaymentMethod::GooglePay,
    ];

    println!("💳 Сравнение способов оплаты");
    println!("100 USD → BTC");
    println!("─────────────────────────────────────────");

    for method in methods {
        let request = BuyQuoteBuilder::new("USD", "100", "BTC")
            .payment_method(method)
            .country("US")
            .build();

        match client.get_buy_quote(request).await {
            Ok(quote) => {
                println!(
                    "{:15} | {} BTC | fee: {}% | rate: {}",
                    format!("{:?}", method),
                    quote.crypto_amount,
                    quote.fee_percent,
                    quote.rate
                );
            }
            Err(e) => {
                println!("{:15} | ❌ Error: {}", format!("{:?}", method), e);
            }
        }
    }

    Ok(())
}

compare_payment_methods().await?;
```

---

## 🎯 Полезные сниппеты

### Конвертация basis points в проценты

```rust
fn bps_to_percent(bps: u64) -> f64 {
    bps as f64 / 100.0
}

assert_eq!(bps_to_percent(200), 2.0);
assert_eq!(bps_to_percent(250), 2.5);
assert_eq!(bps_to_percent(300), 3.0);
```

### Форматирование сумм

```typescript
function formatCryptoAmount(amount: string, symbol: string): string {
  const num = parseFloat(amount);
  if (isNaN(num)) return `0 ${symbol}`;

  if (num < 0.0001) {
    return `${num.toPrecision(6)} ${symbol}`;
  }
  return `${num.toFixed(8)} ${symbol}`;
}

console.log(formatCryptoAmount('0.00152345', 'BTC')); // "0.00152345 BTC"
console.log(formatCryptoAmount('1.5', 'ETH'));        // "1.50000000 ETH"
```

---

## 📞 Ресурсы

- [Abcex API Docs](https://abcex.io/docs/api)
- [Abcex Website](https://abcex.io)
- [ISO 3166-1 Alpha-2](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2)
