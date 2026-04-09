# 0x Protocol Swap Module

## 📋 Обзор

Модуль обмена токенов через 0x Protocol API для Secure Telegram Messenger.

### Ключевые возможности

✅ **Обмен токенов** через 0x Protocol API  
✅ **Комиссия 0.5-3%** (настраиваемая)  
✅ **Мультичейн** (Ethereum, Polygon, Arbitrum, Base, Optimism, BSC)  
✅ **Автоматический slippage** (на основе волатильности и gas)  
✅ **Tauri интегра** (готовые команды для UI)  
✅ **TypeScript типы** (для фронтенда)  
✅ **Полная документация** и примеры  

### Архитектура

```
messenger/src/web3/
├── swap.rs              # Основная логика 0x Protocol
├── swap_commands.rs     # Tauri команды для UI
├── swap_types.ts        # TypeScript типы для фронтенда
├── SWAP_README.md       # API документация
├── SWAP_EXAMPLES.md     # Примеры использования
└── SWAP_MODULE.md       # Этот файл
```

## 🚀 Быстрый старт

### 1. Rust (Backend)

```rust
use messenger::web3::swap::{ZeroExClient, QuoteBuilder};
use messenger::web3::Chain;

// Создать клиент с комиссией 1%
let client = ZeroExClient::with_fee(
    None,  // API key (опционально)
    "0xFeeRecipient".to_string(),
    100,   // 1% комиссия
);

// Получить котировку: 1 WETH -> USDC
let quote = client.get_quote(
    QuoteBuilder::new(
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    )
    .sell_amount("1000000000000000000") // 1 WETH
    .slippage_bps(100) // 1%
    .build(),
    Chain::Ethereum
).await?;

println!("Получите: {} USDC", quote.buy_amount);
```

### 2. TypeScript (UI)

```typescript
import { invoke } from '@tauri-apps/api/core';

// Получить котировку через Tauri команду
const response = await invoke<SwapQuoteResponse>('get_swap_quote', {
  request: {
    chain_id: 1, // Ethereum
    sell_token: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2',
    buy_token: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48',
    sell_amount: '1000000000000000000', // 1 WETH
    taker_address: walletAddress,
  }
});

if (response.success) {
  console.log('Получите:', response.quote!.buy_amount, 'USDC');
}
```

## 📦 Файлы модуля

### `swap.rs`
Основная логика работы с 0x Protocol API:
- `ZeroExClient` - клиент для взаимодействия с API
- `QuoteBuilder` - builder для создания запросов
- `get_quote()` - получение котировки
- `execute_swap()` - полный процесс обмена
- `calculate_slippage()` - автоматический расчет slippage
- `get_allowance_target()` - получение адреса для approve

### `swap_commands.rs`
Tauri команды для вызова из UI:
- `get_swap_quote` - получить котировку
- `execute_swap` - выполнить обмен
- `calculate_slippage` - рассчитать slippage
- `get_allowance_target` - получить адрес для approve
- `quick_swap_quote` - быстрая котировка (human-readable)

### `swap_types.ts`
TypeScript типы и константы для фронтенда:
- Все интерфейсы для запросов/ответов
- Адреса популярных токенов (WETH, USDC, USDT, DAI)
- Константы chains и slippage
- Примеры использования

## 🔧 Комиссии

### Структура комиссий

| Тип токена | Комиссия | Описание |
|------------|----------|----------|
| Major pairs | 0.5% | ETH/USDC, WBTC/USDC |
| Mid-cap tokens | 1-2% | Популярные альткоины |
| Low liquidity | до 3% | Низколиквидные токены |

### Настройка комиссии

```rust
// Комиссия в basis points (1% = 100 bps)
// Допустимый диапазон: 50-300 bps (0.5% - 3%)

let client = ZeroExClient::with_fee(
    None,
    "0xВашАдрес".to_string(),
    150, // 1.5%
);
```

## 🌐 Поддерживаемые сети

| Сеть | Chain ID | Статус |
|------|----------|--------|
| Ethereum Mainnet | 1 | ✅ |
| Polygon | 137 | ✅ |
| Arbitrum One | 42161 | ✅ |
| Base | 8453 | ✅ |
| Optimism | 10 | ✅ |
| BSC | 56 | ✅ |

## 📚 Документация

- **[SWAP_README.md](./SWAP_README.md)** - Полная API документация
- **[SWAP_EXAMPLES.md](./SWAP_EXAMPLES.md)** - Примеры кода
- **[swap_types.ts](./swap_types.ts)** - TypeScript типы

## 🧪 Тестирование

```bash
# Запустить все тесты
cargo test --features web3 swap::

# Запустить конкретный тест
cargo test --features web3 swap::tests::test_quote_builder_basic

# Проверить компиляцию
cargo check --features web3

# Build
cargo build --features web3
```

## 🔐 Безопасность

### Чеклист безопасности

1. ✅ Валидация адресов токенов (format: 0x..., length: 42)
2. ✅ Проверка сумм (positive, valid u128)
3. ✅ Ограничение комиссии (0.5-3%)
4. ✅ Slippage protection (настраиваемый)
5. ⚠️ Allowance check (требуется реализация)
6. ⚠️ Balance check (требуется реализация)

### TODO для Production

- [ ] Реализовать `check_balance()` через RPC
- [ ] Интеграция с кошельком для подписания транзакций
- [ ] Обработка ошибок и retries
- [ ] Логирование всех операций
- [ ] Мониторинг и алерты

## 📋 Интеграция с приложением

### 1. Регистрация команд в main.rs

```rust
use messenger::web3::swap_commands::register_swap_commands;

fn main() {
    let app = tauri::Builder::default()
        // ... другие настройки
    
    // Зарегистрировать swap команды
    let app = register_swap_commands(app);
    
    app.run(tauri::generate_context!())
}
```

### 2. Вызов из UI

```typescript
// Импорт типов
import type { SwapQuoteRequest, SwapQuoteResponse } from './web3/swap_types';

// Вызов команды
const quote = await invoke<SwapQuoteResponse>('get_swap_quote', {
  request: { ... }
});
```

### 3. Обработка транзакции

```typescript
// После получения quote:
// 1. Проверить allowance
const allowanceTarget = await invoke<string>('get_allowance_target', {
  tokenAddress: quote.sell_token,
  chainId: quote.chain_id
});

// 2. Вызвать approve через кошелек
await tokenContract.approve(allowanceTarget, sellAmount);

// 3. Отправить swap транзакцию
await wallet.sendTransaction({
  to: quote.to,
  data: quote.data,
  value: quote.value
});
```

## 🎯 Roadmap

### Phase 1 (✅ Done)
- [x] Базовая интеграция с 0x API
- [x] Quote API
- [x] Tauri команды
- [x] TypeScript типы
- [x] Документация

### Phase 2 (🚧 In Progress)
- [ ] Balance check через RPC
- [ ] Allowance check автоматизация
- [ ] Transaction status tracking
- [ ] История обменов

### Phase 3 (📋 Planned)
- [ ] Limit orders
- [ ] DCA (Dollar Cost Averaging)
- [ ] Portfolio rebalancing
- [ ] Price alerts
- [ ] Analytics dashboard

## 🐛 Troubleshooting

### Ошибка: "Insufficient allowance"

```typescript
// Решение: Вызвать approve перед swap
await tokenContract.approve(quote.allowanceTarget, sellAmount);
```

### Ошибка: "Slippage exceeded"

```rust
// Решение: Увеличить slippage
.slippage_bps(200) // 2% вместо 1%
```

### Ошибка: "Unsupported chain"

```rust
// Проверить поддерживаемые сети
assert!(Chain::from_chain_id(chain_id).is_some());
```

## 📞 Support & Resources

- **0x Docs**: https://docs.0x.org/
- **API Key**: https://0x.org/docs/getting-started
- **Discord**: https://discord.gg/0xProject
- **Issues**: GitHub Issues

## 📄 License

MIT License - см. файл LICENSE

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
