# Abcex Integration — API Документация

## Обзор

Модуль для покупки криптовалюты через Abcex API.
Поддерживает фиатные валюты (USD, EUR, GBP, RUB) и 6 способов оплаты.

### Поддерживаемые фиатные валюты
- USD (US Dollar)
- EUR (Euro)
- GBP (British Pound)
- RUB (Russian Ruble)

### Способы оплаты
- Credit Card (`credit_card`)
- Debit Card (`debit_card`)
- SEPA (`sepa`)
- Bank Transfer (`bank_transfer`)
- Apple Pay (`apple_pay`)
- Google Pay (`google_pay`)

## Быстрый старт

### 1. Базовое использование

```rust
use messenger::web3::abcex::{AbcexClient, BuyQuoteRequest, PaymentMethod};

// Создать клиент без API ключа
let client = AbcexClient::new(None, None);

// Или с кастомной комиссией (2-3%)
let client = AbcexClient::with_fee(
    None,   // API key
    None,   // API secret
    200,    // 2% комиссия
);
```

### 2. Получение котировки

```rust
use messenger::web3::abcex::BuyQuoteBuilder;

// Пример: сколько BTC получим за 100 USD
let quote_request = BuyQuoteBuilder::new("USD", "100", "BTC")
    .payment_method(PaymentMethod::CreditCard)
    .country("US")
    .build();

let quote = client.get_buy_quote(quote_request).await?;

println!("Получите: {} BTC", quote.crypto_amount);
println!("Курс: {} USD/BTC", quote.rate);
println!("Комиссия: {}%", quote.fee_percent);
println!("Доступные методы: {:?}", quote.payment_methods);
```

### 3. Быстрая котировка (convenience function)

```rust
use messenger::web3::abcex::quick_buy_quote;

// Сколько BTC за 100 USD
let quote = quick_buy_quote(
    "USD",              // fiat currency
    "100",              // fiat amount
    "BTC",              // crypto
    PaymentMethod::CreditCard,
    None,               // API key
    None,               // API secret
).await?;

println!("Получите: {} BTC", quote.crypto_amount);
```

### 4. Создание заказа

```rust
use messenger::web3::abcex::PaymentMethod;

// Создать заказ на покупку
let order = client.create_buy_order(
    quote.quote_id,                          // ID котировки
    "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh".to_string(), // deposit address
    PaymentMethod::CreditCard,               // payment method
    "user@example.com".to_string(),          // user email
).await?;

println!("Order ID: {}", order.order_id);
println!("Status: {:?}", order.status);
println!("Payment URL: {:?}", order.payment_url);

// Перенаправить пользователя на страницу оплаты
if let Some(url) = order.payment_url {
    println!("Перейдите по ссылке: {}", url);
}
```

### 5. Полный процесс покупки

```rust
// 1. Проверить KYC
let kyc = client.check_kyc_status("user_123").await?;
if !kyc.verified {
    println!("Требуется верификация. Лимиты:");
    println!("  Дневной: {}", kyc.limits.daily_limit);
    println!("  Месячный: {}", kyc.limits.monthly_limit);
}

// 2. Получить котировку
let quote_request = BuyQuoteBuilder::new("USD", "100", "BTC")
    .payment_method(PaymentMethod::CreditCard)
    .country("US")
    .build();

let quote = client.get_buy_quote(quote_request).await?;
println!("Получите: {} BTC за {} USD", quote.crypto_amount, quote.fiat_amount);

// 3. Создать заказ
let order = client.create_buy_order(
    quote.quote_id,
    "bc1q...".to_string(),
    PaymentMethod::CreditCard,
    "user@example.com".to_string(),
).await?;

// 4. Проверить статус (периодически)
let status = client.get_order_status(&order.order_id).await?;
println!("Статус: {:?}", status.status);
```

## Комиссии

### Структура комиссий Abcex

| Тип | Комиссия | Описание |
|-----|----------|----------|
| Major crypto (BTC, ETH, USDT) | 2% | Высокая ликвидность |
| Mid-cap tokens | 2.5% | Средняя ликвидность |
| Low liquidity | 3% | Низколиквидные |

### Настройка комиссии

```rust
// Комиссия в basis points (1% = 100 bps)
let fee_bps = 200; // 2%

// Допустимый диапазон: 200-300 bps (2% - 3%)
let client = AbcexClient::with_fee(
    None,
    None,
    250, // 2.5%
);

// calculate_fee helper
use messenger::web3::abcex::calculate_fee;
let fee = calculate_fee("100", 200)?; // "2.00" (2% от $100)
let fee = calculate_fee("500", 250)?; // "12.50" (2.5% от $500)
```

## Типы данных

### BuyQuoteRequest

Запрос на получение котировки:

```rust
pub struct BuyQuoteRequest {
    pub fiat_currency: String,       // USD, EUR, GBP, RUB
    pub fiat_amount: String,          // Сумма в фиате
    pub crypto_currency: String,      // BTC, ETH, USDT
    pub payment_method: Option<String>, // credit_card, sepa, etc
    pub country: Option<String>,      // US, RU, DE, etc (ISO 3166-1)
}
```

### BuyQuoteResponse

Ответ с котировкой:

```rust
pub struct BuyQuoteResponse {
    pub quote_id: String,             // ID котировки
    pub fiat_currency: String,        // USD
    pub fiat_amount: String,          // 100
    pub crypto_currency: String,      // BTC
    pub crypto_amount: String,        // 0.0015
    pub rate: String,                 // 66666.67
    pub fee_amount: String,           // 2.00
    pub fee_percent: String,          // 2.0
    pub payment_methods: Vec<String>, // ["credit_card", "debit_card"]
    pub expires_in: u64,              // Время действия (секунды)
    pub min_amount: String,           // Минимальная сумма
    pub max_amount: String,           // Максимальная сумма
}
```

### BuyOrder

Информация о заказе:

```rust
pub struct BuyOrder {
    pub order_id: String,             // ID заказа
    pub quote_id: String,             // ID котировки
    pub status: BuyOrderStatus,       // Статус
    pub fiat_currency: String,
    pub fiat_amount: String,
    pub crypto_currency: String,
    pub crypto_amount: String,
    pub rate: String,
    pub fee_amount: String,
    pub payment_method: String,
    pub deposit_address: String,      // Куда отправить crypto
    pub payment_url: Option<String>,  // Ссылка на оплату
    pub created_at: u64,              // Timestamp
    pub updated_at: u64,              // Timestamp
}
```

### BuyOrderStatus

Статус заказа:

```rust
pub enum BuyOrderStatus {
    Pending,                          // Ожидает оплаты
    Processing,                       // Обработка
    CryptoSent { tx_hash: String },   // Crypto отправлен
    Completed,                        // Завершен
    Cancelled,                        // Отменен
    Failed { error: String },         // Ошибка
}
```

### KycStatus

KYC информация:

```rust
pub struct KycStatus {
    pub verified: bool,               // Верифицирован
    pub level: String,                // Уровень
    pub limits: KycLimits,
}

pub struct KycLimits {
    pub daily_limit: String,          // Дневной лимит
    pub monthly_limit: String,        // Месячный лимит
    pub remaining_daily: String,      // Осталось сегодня
    pub remaining_monthly: String,    // Осталось в этом месяце
}
```

## Payment Methods

### Перечисление PaymentMethod

```rust
pub enum PaymentMethod {
    CreditCard,    // Кредитная карта
    DebitCard,     // Дебетовая карта
    SEPA,          // SEPA перевод (Европа)
    BankTransfer,  // Банковский перевод
    ApplePay,      // Apple Pay
    GooglePay,     // Google Pay
}
```

### Строковые значения

```rust
PaymentMethod::CreditCard.as_str()   // "credit_card"
PaymentMethod::DebitCard.as_str()    // "debit_card"
PaymentMethod::SEPA.as_str()         // "sepa"
PaymentMethod::BankTransfer.as_str() // "bank_transfer"
PaymentMethod::ApplePay.as_str()     // "apple_pay"
PaymentMethod::GooglePay.as_str()    // "google_pay"
```

## Builder Pattern

### BuyQuoteBuilder

Удобный builder для создания запросов:

```rust
use messenger::web3::abcex::{BuyQuoteBuilder, PaymentMethod};

// Минимальный запрос
let request = BuyQuoteBuilder::new("USD", "100", "BTC").build();

// Полный запрос
let request = BuyQuoteBuilder::new("EUR", "500", "ETH")
    .payment_method(PaymentMethod::SEPA)
    .country("DE")
    .build();
```

## API Endpoints

| Endpoint | Method | Описание |
|----------|--------|----------|
| `/v1/buy/quote` | GET | Получить котировку |
| `/v1/buy/order` | POST | Создать заказ |
| `/v1/buy/order/{id}` | GET | Статус заказа |
| `/v1/kyc/status` | GET | KYC статус |
| `/v1/supported/cryptos` | GET | Список криптовалют |
| `/v1/limits` | GET | Лимиты по стране |

### Base URL

```
https://api.abcex.io/v1
```

## Безопасность

### Важные моменты

1. **Проверка лимитов**: Перед заказом проверьте лимиты для вашей страны
2. **KYC**: Для крупных сумм требуется верификация
3. **Expiration**: Котировки действуют ограниченное время
4. **Payment URL**: Всегда перенаправляйте пользователя на payment_url

### Валидация

```rust
// Проверяйте суммы
assert!(!amount.is_empty());
assert!(amount.parse::<f64>().is_ok());
assert!(amount.parse::<f64>().unwrap() > 0.0);

// Проверяйте комиссию (2-3%)
assert!(fee_bps >= 200 && fee_bps <= 300);

// Проверяйте код страны (ISO 3166-1 alpha-2)
assert!(country.len() == 2);
```

### Обработка ошибок

```rust
match client.get_buy_quote(request).await {
    Ok(quote) => {
        println!("Получите: {} {}", quote.crypto_amount, quote.crypto_currency);
    }
    Err(Web3Error::Network(err)) => {
        eprintln!("Ошибка сети: {}", err);
    }
    Err(Web3Error::Wallet(err)) => {
        eprintln!("Ошибка кошелька: {}", err);
    }
    Err(e) => {
        eprintln!("Неизвестная ошибка: {}", e);
    }
}
```

## Тесты

```bash
# Запустить все тесты Abcex
cargo test --features web3 abcex::

# Проверить компиляцию
cargo check --features web3

# Build
cargo build --features web3
```

### Доступные тесты

- `test_buy_quote_builder_basic` — базовый builder
- `test_calculate_fee` — расчёт комиссии
- `test_payment_method_as_str` — строковые значения
- `test_fiat_currency_as_str` — валюты
- `test_abcex_client_creation` — создание клиента
- `test_abcex_client_with_fee` — кастомная комиссия
- `test_abcex_client_invalid_fee_too_low` — валидация (panic)
- `test_abcex_client_invalid_fee_too_high` — валидация (panic)
- `test_buy_quote_builder_defaults` — значения по умолчанию

## Примеры

### Обмен USD → BTC

```rust
let quote = quick_buy_quote(
    "USD",
    "100",
    "BTC",
    PaymentMethod::CreditCard,
    None,
    None,
).await?;

println!("{} USD → {} BTC", quote.fiat_amount, quote.crypto_amount);
```

### Обмен EUR → ETH через SEPA

```rust
let request = BuyQuoteBuilder::new("EUR", "500", "ETH")
    .payment_method(PaymentMethod::SEPA)
    .country("DE")
    .build();

let quote = client.get_buy_quote(request).await?;
```

### Проверка лимитов для страны

```rust
let limits = client.get_limits("US").await?;
println!("Daily limit: {:?}", limits.get("daily_limit"));
println!("Monthly limit: {:?}", limits.get("monthly_limit"));
```

## Ссылки

- [Abcex API Docs](https://abcex.io/docs/api)
- [Abcex Website](https://abcex.io)
- [ISO 3166-1](https://en.wikipedia.org/wiki/ISO_3166-1_alpha-2)
