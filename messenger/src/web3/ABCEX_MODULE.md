# Abcex Integration Module — Покупка криптовалюты за фиат

## 📋 Обзор

Модуль для покупки криптовалюты через Abcex API с комиссией 2-3%.
Позволяет пользователям покупать BTC, ETH, USDT за фиатные валюты
(USD, EUR, GBP, RUB) через различные способы оплаты.

### Ключевые возможности

✅ **Покупка за фиат** — USD, EUR, GBP, RUB → BTC, ETH, USDT
✅ **6 способов оплаты** — Credit Card, Debit Card, SEPA, Bank Transfer, Apple Pay, Google Pay
✅ **Комиссия 2-3%** (настраиваемая)
✅ **KYC verification** — проверка статуса верификации
✅ **Лимиты по странам** — автоматическая проверка лимитов
✅ **Tauri интеграция** — 8 готовых команд для UI
✅ **TypeScript типы** — полная типизация для фронтенда
✅ **Полная документация** и примеры

### Архитектура

```
messenger/src/web3/
├── abcex.rs                # Основная логика Abcex API
├── abcex_commands.rs       # Tauri команды для UI
├── ABCEX_MODULE.md         # Этот файл
├── ABCEX_README.md         # API документация
├── ABCEX_EXAMPLES.md       # Примеры использования
└── ABCEX_SUMMARY.md        # Итоговый отчёт
```

## 🚀 Быстрый старт

### 1. Rust (Backend)

```rust
use messenger::web3::abcex::{AbcexClient, BuyQuoteRequest, PaymentMethod};

// Создать клиент с комиссией 2%
let client = AbcexClient::with_fee(
    None,  // API key (опционально)
    None,  // API secret
    200,   // 2% комиссия
);

// Получить котировку: 100 USD → BTC
let quote = client.get_buy_quote(BuyQuoteRequest {
    fiat_currency: "USD".to_string(),
    fiat_amount: "100".to_string(),
    crypto_currency: "BTC".to_string(),
    payment_method: Some("credit_card".to_string()),
    country: Some("US".to_string()),
}).await?;

println!("Получите: {} BTC", quote.crypto_amount);
println!("Курс: {}", quote.rate);
println!("Комиссия: {}%", quote.fee_percent);
```

### 2. TypeScript (UI)

```typescript
import { abcexGetQuote, abcexCreateOrder } from '@/services/web3Service';
import type { AbcexQuoteRequest, AbcexOrderRequest } from '@/types/web3';

// Получить котировку
const quoteResponse = await abcexGetQuote({
  fiat_currency: 'USD',
  fiat_amount: '100',
  crypto_currency: 'BTC',
  payment_method: 'credit_card',
  country: 'US',
});

if (quoteResponse.success) {
  const quote = quoteResponse.quote!;
  console.log('Получите:', quote.crypto_amount, 'BTC');

  // Создать заказ
  const order = await abcexCreateOrder({
    quote_id: quote.quote_id,
    deposit_address: 'bc1q...',
    payment_method: 'credit_card',
    user_email: 'user@example.com',
  });

  // Перенаправить на оплату
  if (order.order?.payment_url) {
    window.open(order.order.payment_url);
  }
}
```

## 📦 Файлы модуля

### `abcex.rs` (~714 строк)
Основная логика работы с Abcex API:
- `AbcexClient` — клиент для взаимодействия с API
- `BuyQuoteBuilder` — builder для создания запросов
- `get_buy_quote()` — получение котировки
- `create_buy_order()` — создание заказа
- `get_order_status()` — проверка статуса
- `check_kyc_status()` — KYC верификация
- `get_supported_cryptos()` — список криптовалют
- `get_limits()` — лимиты по странам
- `execute_buy()` — полный процесс покупки

### `abcex_commands.rs` (~411 строк)
Tauri команды для вызова из UI:
- `abcex_get_quote` — получить котировку
- `abcex_create_order` — создать заказ
- `abcex_get_order_status` — проверить статус заказа
- `abcex_check_kyc` — проверить KYC
- `abcex_get_supported_cryptos` — список криптовалют
- `abcex_get_limits` — лимиты для страны
- `abcex_quick_quote` — быстрая котировка
- `abcex_calculate_fee` — расчёт комиссии

### `frontend/src/types/web3.ts`
TypeScript типы для фронтенда:
- Все интерфейсы для запросов/ответов
- Типы payment methods
- Константы и утилиты

### `frontend/src/services/web3Service.ts`
Сервисные функции:
- 8 асинхронных функций для Abcex
- Helper функции форматирования
- Конвертация комиссий

## 🔧 Комиссии

### Структура комиссий

| Тип криптовалюты | Комиссия | Описание |
|------------------|----------|----------|
| Major (BTC, ETH, USDT) | 2% | Высокая ликвидность |
| Mid-cap | 2.5% | Средняя ликвидность |
| Low liquidity | 3% | Низколиквидные |

### Настройка комиссии

```rust
// Комиссия в basis points (1% = 100 bps)
// Допустимый диапазон: 200-300 bps (2% - 3%)

let client = AbcexClient::with_fee(
    None,
    None,
    250, // 2.5%
);

// Валидация — паника при недопустимом значении
// AbcexClient::with_fee(None, None, 100); // PANIC: слишком мало
// AbcexClient::with_fee(None, None, 400); // PANIC: слишком много
```

## 💳 Способы оплаты

| Способ | Код | Скорость | Комиссия |
|--------|-----|----------|----------|
| Credit Card | `credit_card` | Мгновенно | +1% |
| Debit Card | `debit_card` | Мгновенно | +1% |
| SEPA | `sepa` | 1-2 дня | +0.5% |
| Bank Transfer | `bank_transfer` | 2-3 дня | +0.3% |
| Apple Pay | `apple_pay` | Мгновенно | +1.2% |
| Google Pay | `google_pay` | Мгновенно | +1.2% |

## 🌐 Поддерживаемые валюты

### Фиатные валюты

| Валюта | Код | Страны |
|--------|-----|--------|
| US Dollar | USD | США, международные |
| Euro | EUR | Евросоюз |
| British Pound | GBP | Великобритания |
| Russian Ruble | RUB | Россия |

### Криптовалюты

BTC, ETH, USDT, USDC, DAI, и другие (динамический список через API)

## 📚 Документация

- **[ABCEX_README.md](./ABCEX_README.md)** — Полная API документация
- **[ABCEX_EXAMPLES.md](./ABCEX_EXAMPLES.md)** — Примеры кода
- **[ABCEX_SUMMARY.md](./ABCEX_SUMMARY.md)** — Итоговый отчёт
- **[web3.ts](../../../../frontend/src/types/web3.ts)** — TypeScript типы
- **[web3Service.ts](../../../../frontend/src/services/web3Service.ts)** — Сервис

## 🧪 Тестирование

```bash
# Запустить все тесты
cargo test --features web3 abcex::

# Запустить конкретный тест
cargo test --features web3 abcex::tests::test_buy_quote_builder_basic

# Проверить компиляцию
cargo check --features web3

# Build
cargo build --features web3
```

## 🔐 Безопасность

### Чеклист безопасности

1. ✅ Валидация сумм (positive, valid amounts)
2. ✅ Ограничение комиссии (2-3%)
3. ✅ KYC verification support
4. ✅ Лимиты по странам
5. ✅ Type-safe API (Rust + TypeScript)
6. ⚠️ Rate limiting (требуется реализация)
7. ⚠️ Order expiration handling

### TODO для Production

- [ ] Rate limiting для API calls
- [ ] Order expiration handling
- [ ] Retry logic для failed requests
- [ ] Webhook notifications для status updates
- [ ] Transaction logging и auditing
- [ ] User confirmation dialogs в UI

## 📋 Интеграция с приложением

### 1. Регистрация команд в main.rs

```rust
use messenger::web3::abcex_commands::register_abcex_commands;

fn main() {
    let app = tauri::Builder::default()
        // ... другие настройки

    // Зарегистрировать Abcex команды
    let app = register_abcex_commands(app);

    app.run(tauri::generate_context!())
}
```

### 2. Вызов из UI

```typescript
import type { AbcexQuoteRequest, AbcexQuoteResponse } from '@/types/web3';

const response = await invoke<AbcexQuoteResponse>('abcex_get_quote', {
  request: {
    fiat_currency: 'USD',
    fiat_amount: '100',
    crypto_currency: 'BTC',
    payment_method: 'credit_card',
  }
});
```

### 3. Полный процесс покупки

```typescript
// 1. Проверить KYC
const kyc = await abcexCheckKyc({ user_id });
if (!kyc.kyc_status?.verified) {
  // Показать форму KYC
}

// 2. Получить котировку
const quote = await abcexGetQuote({ ... });

// 3. Создать заказ
const order = await abcexCreateOrder({
  quote_id: quote.quote!.quote_id,
  deposit_address: userWalletAddress,
  payment_method: 'credit_card',
  user_email: userEmail,
});

// 4. Перенаправить на оплату
if (order.order?.payment_url) {
  window.open(order.order.payment_url);
}

// 5. Проверить статус (периодически)
const status = await abcexGetOrderStatus(order.order.order_id);
```

## 🎯 Roadmap

### Phase 1 (✅ Done)
- [x] Базовая интеграция с Abcex API
- [x] Quote API
- [x] Order management
- [x] KYC status check
- [x] Tauri команды
- [x] TypeScript типы
- [x] Документация

### Phase 2 (🚧 In Progress)
- [ ] Rate limiting
- [ ] Order expiration handling
- [ ] Webhook notifications
- [ ] Order history tracking

### Phase 3 (📋 Planned)
- [ ] Recurring buys (DCA)
- [ ] Price alerts
- [ ] Multi-currency support
- [ ] Analytics dashboard
- [ ] Referral program

## 🐛 Troubleshooting

### Ошибка: "Invalid payment method"

```typescript
// Убедитесь в правильном формате
payment_method: 'credit_card',  // ✅
payment_method: 'CreditCard',   // ❌
```

### Ошибка: "Quote expired"

```typescript
// Котировка действует ограниченное время
// Пересоздайте котировку перед заказом
const freshQuote = await abcexGetQuote({ ... });
```

### Ошибка: "KYC required"

```typescript
// Проверьте KYC перед заказом
const kyc = await abcexCheckKyc({ user_id });
if (!kyc.kyc_status?.verified) {
  // Начать процесс верификации
}
```

## 📞 Support & Resources

- **Abcex Docs**: https://abcex.io/docs/api
- **API Key**: https://abcex.io/api
- **Support**: support@abcex.io
- **Issues**: GitHub Issues

## 📄 License

MIT License — см. файл LICENSE

## 🤝 Contributing

1. Fork репозиторий
2. Создать feature branch
3. Внести изменения
4. Добавить тесты
5. Создать Pull Request

---

**Версия**: 1.0.0
**Дата**: Апрель 2026
**Автор**: Secure Messenger Team
