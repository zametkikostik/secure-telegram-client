# 0x Swap Integration - Примеры использования

## 1. Базовый пример: Получение котировки

```rust
use messenger::web3::swap::{ZeroExClient, QuoteBuilder};
use messenger::web3::Chain;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Создать клиент
    let client = ZeroExClient::new(None);
    
    // Получить котировку: 1 WETH -> USDC
    let quote = client.get_quote(
        QuoteBuilder::new(
            "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
            "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        )
        .sell_amount("1000000000000000000") // 1 WETH (18 decimals)
        .slippage_bps(100) // 1%
        .build(),
        Chain::Ethereum
    ).await?;
    
    println!("Sell: 1 WETH");
    println!("Buy: {} USDC", quote.buy_amount);
    println!("Gas: {}", quote.gas);
    println!("Price: {}", quote.price);
    
    Ok(())
}
```

## 2. Быстрая котировка (human-readable)

```rust
use messenger::web3::swap::quick_quote;
use messenger::web3::Chain;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Сколько USDC получим за 1.5 ETH?
    let result = quick_quote(
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "1.5",           // 1.5 ETH (human-readable)
        18,              // decimals для ETH
        Chain::Ethereum,
        None,
    ).await?;
    
    println!("За 1.5 ETH получите: {} USDC", result);
    
    Ok(())
}
```

## 3. Полный процесс обмена

```rust
use messenger::web3::swap::{ZeroExClient, QuoteBuilder};
use messenger::web3::Chain;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ZeroExClient::with_fee(
        None,
        "0xВашАдресДляКомиссий".to_string(),
        100, // 1% комиссия
    );
    
    // Шаг 1: Получить котировку
    let swap_record = client.execute_swap(
        Chain::Ethereum,
        "0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2", // WETH
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        "1000000000000000000",  // 1 WETH
        "0xАдресКошелька",
        Some(100), // slippage 1%
    ).await?;
    
    println!("Swap ID: {}", swap_record.id);
    println!("Status: {:?}", swap_record.status);
    println!("Expected: {} USDC", swap_record.expected_to_amount);
    
    Ok(())
}
```

## 4. Проверка allowance и approve

```rust
use messenger::web3::swap::ZeroExClient;
use messenger::web3::Chain;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = ZeroExClient::new(None);
    
    // Получить адрес контракта для approve
    let allowance_target = client.get_allowance_target(
        "0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48", // USDC
        Chain::Ethereum
    ).await?;
    
    println!("Allowance target: {}", allowance_target);
    
    // Затем нужно вызвать approve через кошелек:
    // await usdcContract.approve(allowanceTarget, amount);
    
    Ok(())
}
```

## 5. Автоматический расчет slippage

```rust
use messenger::web3::swap::ZeroExClient;

fn main() {
    let client = ZeroExClient::new(None);
    
    // Slippage для разных токенов
    let usdc_slippage = client.calculate_slippage("USDC", 30);
    let eth_slippage = client.calculate_slippage("ETH", 60);
    let unknown_slippage = client.calculate_slippage("UNKNOWN", 150);
    
    println!("USDC slippage: {} bps ({}%)", usdc_slippage, usdc_slippage as f64 / 100.0);
    println!("ETH slippage: {} bps ({}%)", eth_slippage, eth_slippage as f64 / 100.0);
    println!("UNKNOWN slippage: {} bps ({}%)", unknown_slippage, unknown_slippage as f64 / 100.0);
}
```

## 6. Интеграция с Tauri UI

### Rust (уже реализовано)

```rust
// В src/main.rs или src/lib.rs
use messenger::web3::swap_commands::register_swap_commands;

fn main() {
    let app = tauri::Builder::default()
        // ... другие настройки
    
    // Зарегистрировать swap команды
    let app = register_swap_commands(app);
    
    app.run(tauri::generate_context!())
}
```

### TypeScript/JavaScript (UI)

```typescript
import { invoke } from '@tauri-apps/api/core';
import type { SwapQuoteRequest, SwapQuoteResponse } from './swap_types';

// Получить котировку
async function getQuote() {
  const response = await invoke<SwapQuoteResponse>('get_swap_quote', {
    request: {
      chain_id: 1, // Ethereum
      sell_token: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2', // WETH
      buy_token: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', // USDC
      sell_amount: '1000000000000000000', // 1 WETH
      taker_address: walletAddress,
      slippage_bps: 100, // 1%
    }
  });

  if (response.success && response.quote) {
    console.log(`Получите: ${response.quote.buy_amount} USDC`);
    console.log(`Gas: ${response.quote.gas_estimate}`);
    return response.quote;
  } else {
    console.error('Error:', response.error);
    throw new Error(response.error);
  }
}

// Выполнить обмен
async function executeSwap(quote: QuoteData) {
  const result = await invoke<SwapExecuteResponse>('execute_swap', {
    request: {
      chain_id: 1,
      sell_token: quote.sell_token,
      buy_token: quote.buy_token,
      sell_amount: quote.sell_amount,
      taker_address: walletAddress,
      slippage_bps: 100,
    }
  });

  if (result.success) {
    console.log('Swap completed!');
    console.log('Record:', result.swap_record);
    return result.swap_record;
  } else {
    console.error('Swap failed:', result.error);
    throw new Error(result.error);
  }
}

// Получить адрес для approve
async function getAllowanceTarget(tokenAddress: string) {
  const allowanceTarget = await invoke<string>('get_allowance_target', {
    tokenAddress,
    chainId: 1
  });
  
  return allowanceTarget;
}

// Рассчитать slippage
async function calculateSlippage(tokenSymbol: string, gasPrice: number) {
  const slippage = await invoke<number>('calculate_slippage', {
    tokenSymbol,
    gasPriceGwei: gasPrice
  });
  
  return slippage; // в basis points
}
```

## 7. React Component Example

```tsx
import React, { useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { QuoteData, SwapQuoteResponse } from './swap_types';

const SwapComponent: React.FC = () => {
  const [sellAmount, setSellAmount] = useState('1.0');
  const [quote, setQuote] = useState<QuoteData | null>(null);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const getQuote = async () => {
    setLoading(true);
    setError(null);
    
    try {
      // Конвертировать в raw amount (18 decimals)
      const rawAmount = (parseFloat(sellAmount) * 1e18).toString();
      
      const response = await invoke<SwapQuoteResponse>('get_swap_quote', {
        request: {
          chain_id: 1,
          sell_token: '0xC02aaA39b223FE8D0A0e5C4F27eAD9083C756Cc2', // WETH
          buy_token: '0xA0b86991c6218b36c1d19D4a2e9Eb0cE3606eB48', // USDC
          sell_amount: rawAmount,
          taker_address: walletAddress,
          slippage_bps: 100,
        }
      });

      if (response.success && response.quote) {
        setQuote(response.quote);
      } else {
        setError(response.error || 'Failed to get quote');
      }
    } catch (err) {
      setError(err.message);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="swap-container">
      <h2>Swap Tokens</h2>
      
      <div className="swap-form">
        <input
          type="number"
          value={sellAmount}
          onChange={(e) => setSellAmount(e.target.value)}
          placeholder="Amount (WETH)"
        />
        
        <button onClick={getQuote} disabled={loading}>
          {loading ? 'Getting Quote...' : 'Get Quote'}
        </button>
        
        {quote && (
          <div className="quote-info">
            <h3>Quote:</h3>
            <p>You will receive: {(parseFloat(quote.buy_amount) / 1e6).toFixed(2)} USDC</p>
            <p>Price: {quote.price}</p>
            <p>Gas Estimate: {quote.gas_estimate}</p>
            <p>Fee: {quote.fee_bps / 100}%</p>
            
            <button onClick={() => executeSwap(quote)}>
              Execute Swap
            </button>
          </div>
        )}
        
        {error && (
          <div className="error">
            <p>Error: {error}</p>
          </div>
        )}
      </div>
      
      <style>{`
        .swap-container {
          max-width: 500px;
          margin: 0 auto;
          padding: 20px;
        }
        
        .swap-form {
          display: flex;
          flex-direction: column;
          gap: 10px;
        }
        
        .quote-info {
          background: #f0f0f0;
          padding: 15px;
          border-radius: 8px;
          margin-top: 10px;
        }
        
        .error {
          background: #ffcccc;
          padding: 10px;
          border-radius: 8px;
          color: #cc0000;
        }
      `}</style>
    </div>
  );
};

export default SwapComponent;
```

## 8. Конфигурация

### Добавить в `.env`

```env
# 0x Protocol API Key (опционально, для production)
ZEROEX_API_KEY=your_api_key_here

# Адрес для получения комиссий
ZEROEX_FEE_RECIPIENT=0x0000000000000000000000000000000000000000

# Комиссия в basis points (100 = 1%)
ZEROEX_FEE_BPS=100
```

### Добавить в `Cargo.toml`

```toml
[dependencies]
# Уже есть:
reqwest = "0.11"
serde = "1.0"
serde_json = "1.0"
tracing = "0.1"
uuid = "1.0"

# Web3 feature (опционально):
ethers = { version = "2.0", optional = true }
```

## 9. Тестирование

```bash
# Запустить unit тесты
cargo test --features web3 swap::

# Запустить конкретный тест
cargo test --features web3 swap::tests::test_quote_builder_basic

# Проверить компиляцию
cargo check --features web3

# Build
cargo build --features web3
```

## 10. Production Deployment

### Чеклист

1. ✅ Получить API ключ на https://0x.org/
2. ✅ Настроить fee recipient адрес
3. ✅ Установить комиссию (50-300 bps)
4. ✅ Протестировать на testnet (Goerli, Sepolia)
5. ✅ Проверить все supported chains
6. ✅ Добавить обработку ошибок
7. ✅ Настроить мониторинг и логи
8. ✅ Проверить безопасность (адреса, суммы)

### Получение API ключа

1. Зарегистрироваться на https://0x.org/
2. Перейти в Dashboard → API Keys
3. Создать новый ключ
4. Добавить в `.env`:

```env
ZEROEX_API_KEY=your_production_api_key
```

## 11. Troubleshooting

### Ошибка: "Insufficient allowance"

**Причина**: Контракт 0x не имеет разрешения на использование токенов

**Решение**:
```typescript
// Вызвать approve перед swap
await tokenContract.approve(
  quote.allowanceTarget,
  sellAmount
);
```

### Ошибка: "Slippage exceeded"

**Причина**: Цена изменилась больше чем допустимое slippage

**Решение**:
```rust
// Увеличить slippage
.slippage_bps(200) // 2% вместо 1%
```

### Ошибка: "Insufficient balance"

**Причина**: Недостаточно токенов для обмена

**Решение**: Проверить баланс перед swap

## 12. Ссылки

- [0x Protocol Documentation](https://docs.0x.org/)
- [0x Swap API Reference](https://docs.0x.org/0x-api-swap/introduction)
- [Get 0x API Key](https://0x.org/docs/getting-started)
- [Slippage Explained](https://docs.0x.org/0x-api-swap/guides/slippage)
