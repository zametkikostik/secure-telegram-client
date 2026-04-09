# 0x Protocol Swap Integration

## Обзор

Модуль для обмена токенов через 0x Protocol API с комиссией 0.5-3%.

### Поддерживаемые сети
- Ethereum Mainnet (chain_id: 1)
- Polygon (chain_id: 137)
- Arbitrum (chain_id: 42161)
- Base (chain_id: 8453)
- Optimism (chain_id: 10)
- BSC (chain_id: 56)

## Быстрый старт

### 1. Базовое использование

```rust
use messenger::web3::swap::{ZeroExClient, QuoteBuilder};
use messenger::web3::Chain;

// Создать клиент с комиссией 1% (100 bps)
let client = ZeroExClient::new(None); // без API ключа

// Или с кастомной комиссией
let client = ZeroExClient::with_fee(
    None,
    "0xВашАдресДляКомиссий".to_string(),
    100 // 1%
);
```

### 2. Получение котировки

```rust
// Пример: сколько USDC получим за 1 WETH
let quote_request = QuoteBuilder::new(
    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
)
.sell_amount("1000000000000000000") // 1 WETH (18 decimals)
.slippage_bps(100) // 1% проскальзывание
.taker_address("0xАдресВашегоКошелька")
.build();

let quote = client.get_quote(quote_request, Chain::Ethereum).await?;

println!("Получите: {} USDC", quote.buy_amount);
println!("Gas: {}", quote.gas);
```

### 3. Быстрая котировка (human-readable)

```rust
use messenger::web3::swap::quick_quote;

// Сколько USDC за 1 ETH
let result = quick_quote(
    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    "1.0",           // 1 ETH (human-readable)
    18,              // decimals для ETH
    Chain::Ethereum,
    None,            // API key
).await?;

println!("Получите: {} USDC", result);
```

### 4. Полный процесс обмена

```rust
let swap_record = client.execute_swap(
    Chain::Ethereum,
    "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // sell: WETH
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // buy: USDC
    "1000000000000000000",  // 1 WETH (raw amount)
    "0xАдресКошелька",
    Some(100),               // slippage 1%
).await?;

println!("Swap ID: {}", swap_record.id);
println!("Status: {:?}", swap_record.status);
```

## Комиссии

### Структура комиссий 0x Protocol

| Тип токена | Комиссия |
|------------|----------|
| Major pairs (ETH/USDC) | 0.5% |
| Mid-cap tokens | 1-2% |
| Low liquidity tokens | до 3% |

### Настройка комиссии

```rust
// Комиссия в basis points (1% = 100 bps)
let fee_bps = 100; // 1%

// Допустимый диапазон: 50-300 bps (0.5% - 3%)
let client = ZeroExClient::with_fee(
    None,
    "0xFeeRecipient".to_string(),
    150, // 1.5%
);
```

## Автоматический расчет Slippage

```rust
// Slippage рассчитывается автоматически на основе:
// - Волатильности токена (stablecoins: 0.2%, ETH: 0.5%, другие: 2%)
// - Текущей цены gas (высокий gas = больше slippage)

let slippage = client.calculate_slippage("USDC", 50); // 120 bps = 1.2%
let slippage = client.calculate_slippage("ETH", 150); // 200 bps = 2%
```

## Интеграция с кошельком

### 1. Проверка allowance

```rust
// Получить адрес контракта для approve
let allowance_target = client.get_allowance_target(
    "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
    Chain::Ethereum
).await?;

// Затем вызвать approve(allowance_target, amount) через кошелек
```

### 2. Отправка транзакции

```rust
// После получения quote:
// 1. Проверить allowance (если нужно - сделать approve)
// 2. Отправить транзакцию с данными из quote:
//    - to: quote.to
//    - data: quote.data
//    - value: quote.value (для ETH swaps)
```

## Примеры использования

### Обмен ETH -> USDC

```rust
let client = ZeroExClient::new(None);

let quote = client.get_quote(QuoteRequest {
    sell_token: "0xEeeeeEeeeEeEeeEeEeEeeEEEeeeeEeeeeeeeEEeE".to_string(), // ETH
    buy_token: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
    sell_amount: Some("1000000000000000000".to_string()), // 1 ETH
    buy_amount: None,
    slippage_bps: Some(100),
    taker_address: Some("0xВашКошелек".to_string()),
    fee_recipient: Some("0xFeeRecipient".to_string()),
    buy_token_percentage_fee: Some(100), // 1%
}, Chain::Ethereum).await?;
```

### Обмен USDC -> DAI

```rust
let quote = client.get_quote(QuoteRequest {
    sell_token: "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48".to_string(), // USDC
    buy_token: "0x6B175474E89094C44Da98b954EedeAC495271d0F".to_string(), // DAI
    sell_amount: Some("1000000000".to_string()), // 1000 USDC (6 decimals)
    buy_amount: None,
    slippage_bps: Some(50), // 0.5% для stablecoins
    taker_address: Some("0xВашКошелек".to_string()),
    fee_recipient: None,
    buy_token_percentage_fee: None,
}, Chain::Ethereum).await?;
```

## Типы данных

### QuoteRequest
- `sell_token`: Адрес токена для продажи
- `buy_token`: Адрес токена для покупки
- `sell_amount`: Сумма продажи (raw wei)
- `buy_amount`: Сумма покупки (raw wei)
- `slippage_bps`: Проскальзывание в basis points
- `taker_address`: Адрес кошелька
- `fee_recipient`: Получатель комиссии
- `buy_token_percentage_fee`: Комиссия в bps

### QuoteResponse
- `sell_amount`: Сумма продажи
- `buy_amount`: Ожидаемая сумма покупки
- `price`: Цена обмена
- `gas`: Оценка газа
- `to`: Адрес контракта для отправки
- `data`: Калldata транзакции
- `value`: ETH value (для ETH swaps)

### SwapRecord
- `id`: UUID записи
- `chain`: Сеть
- `from_token/to_token`: Токены
- `from_amount/to_amount`: Суммы
- `status`: Статус обмена
- `tx_hash`: Хэш транзакции

## Безопасность

### Важные моменты

1. **Проверка allowance**: Перед отправкой swap убедитесь, что контракт имеет allowance
2. **Slippage**: Используйте разумные значения (0.5-2% для большинства случаев)
3. **Gas price**: Проверяйте актуальную цену gas перед отправкой
4. **API Key**: Для production использования получите API ключ на https://0x.org/docs

### Валидация

```rust
// Проверяйте адреса токенов
assert!(address.starts_with("0x") && address.len() == 42);

// Проверяйте суммы
assert!(!amount.is_empty() && amount.parse::<u128>().is_ok());

// Проверяйте комиссию (0.5-3%)
assert!(fee_bps >= 50 && fee_bps <= 300);
```

## Тесты

```bash
cargo test --features web3 swap::
```

## API Endpoints

- **Quote**: `https://api.0x.org/swap/v1/quote`
- **Price**: `https://api.0x.org/swap/v1/price`
- **Allowance**: `https://api.0x.org/swap/v1/allowance`

## Ссылки

- [0x Protocol Docs](https://docs.0x.org/)
- [0x Swap API](https://docs.0x.org/0x-api-swap/introduction)
- [Get API Key](https://0x.org/docs/getting-started)
