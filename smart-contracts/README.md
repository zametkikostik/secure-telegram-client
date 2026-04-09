# 🔐 P2P Escrow - Смарт-контракт для безопасных сделок

**Дата:** 7 апреля 2026 г.
**Статус:** ✅ **Готово к тестированию**

---

## 📋 Обзор

P2P Escrow - это смарт-контракт для безопасных децентрализованных сделок внутри мессенджера. Покупатель блокирует средства, которые автоматически выплачиваются продавцу после подтверждения или возвращаются при проблемах.

---

## 🏗️ Архитектура

### Solidity Контракт

**Файл:** `smart-contracts/contracts/P2PEscrow.sol`

**Функционал:**
- ✅ Создание сделок (5 типов: цифровые/физические товары, услуги, подписка, фриланс)
- ✅ Блокировка ETH или ERC-20 токенов
- ✅ Подтверждение доставки и авто-выплата
- ✅ Система споров с арбитражем (мультиподпись 2/3)
- ✅ Таймаут с автоматическим возвратом
- ✅ Трехуровневая комиссия (2% / 1% / 0.5%)

**Безопасность:**
- ✅ Модификаторы доступа (onlyBuyer, onlySeller, onlyArbiter)
- ✅ Проверка состояний (inState)
- ✅ Защита от реентрантности
- ✅ Автоматические возвраты после дедлайна

### Rust Интеграция

**Файлы:**
- `messenger/src/web3/p2p_escrow.rs` - Клиент для взаимодействия с контрактом
- `messenger/src/web3/p2p_escrow_commands.rs` - Tauri команды для UI

**Функции:**
- ✅ Создание и финансирование сделок
- ✅ Подтверждение доставки
- ✅ Открытие и разрешение споров
- ✅ Мониторинг событий
- ✅ Query статистики платформы

---

## 📊 Типы сделок

| Тип | Описание | Пример |
|-----|----------|--------|
| **DigitalGoods** | Цифровые товары | Аккаунты, ключи, файлы |
| **PhysicalGoods** | Физические товары | Доставка вещей |
| **Service** | Услуги | Консультации, ремонт |
| **Subscription** | Подписка | Доступ к контенту |
| **Freelance** | Фриланс | Разработка, дизайн |

---

## 💰 Комиссии платформы

| Сумма | Комиссия | Пример (сумма → комиссия) |
|-------|----------|---------------------------|
| < 0.1 ETH | 2% | 0.05 ETH → 0.001 ETH |
| 0.1 - 1 ETH | 1% | 0.5 ETH → 0.005 ETH |
| ≥ 1 ETH | 0.5% | 2 ETH → 0.01 ETH |

---

## 🔄 Жизненный цикл сделки

```
Created → Funded → Delivered → Completed
              ↓
          Disputed → Resolved (Refund or Payout)
              ↓
          Refunded (после deadline)
```

### Шаги:

1. **Создание** (Покупатель)
   - Указать продавца, тип, дедлайн
   - E2EE хэш описания сделки

2. **Финансирование** (Покупатель)
   - ETH или ERC-20 токены
   - Средства блокируются в контракте

3. **Доставка** (Продавец)
   - Отправка товара/услуги
   - Покупатель подтверждает

4. **Завершение** (Любой)
   - Авто-выплата продавцу
   - Комиссия платформы → Treasury

5. **Спор** (Опционально)
   - Открытие спора (покупатель/продавец)
   - Арбитраж (авторизованный арбитр)
   - Решение: полный/частичный возврат или выплата

---

## 🧪 Тестирование

### Solidity Tests (Forge)

```bash
cd smart-contracts

# Установить Foundry
curl -L https://foundry.paradigm.xyz | bash
foundryup

# Запустить тесты
forge test -vvv

# Coverage
forge coverage
```

**Результаты:** 30+ тестов покрывают:
- ✅ Создание сделок (успех/ошибки)
- ✅ Финансирование ETH и токенами
- ✅ Завершение и возвраты
- ✅ Система споров и арбитража
- ✅ Админ функции (arbiter, treasury)
- ✅ Статистика платформы
- ✅ Edge cases

### Rust Tests

```bash
cd /home/kostik/secure-messenger/secure-telegram-client

# Запустить тесты
cargo test -p secure-messenger p2p_escrow --features web3
```

---

## 📁 Структура файлов

```
smart-contracts/
├── contracts/
│   └── P2PEscrow.sol              # Основной контракт
├── test/
│   └── P2PEscrow.t.sol            # Тесты (30+)
└── scripts/
    └── deploy.js                  # Скрипт деплоя

messenger/src/web3/
├── mod.rs                         # Модуль web3
├── p2p_escrow.rs                  # Rust клиент
└── p2p_escrow_commands.rs         # Tauri команды
```

---

## 🚀 Деплой

### Local (Anvil)

```bash
# Запустить локальную ноду
anvil

# В другом терминале
forge script script/Deploy.s.sol \
  --rpc-url http://localhost:8545 \
  --private-key 0xac0974... \
  --broadcast
```

### Testnet (Sepolia)

```bash
# Деплой на Sepolia
forge script script/Deploy.s.sol \
  --rpc-url https://rpc.sepolia.org \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --verify \
  --etherscan-api-key $ETHERSCAN_API_KEY
```

### Mainnet

```bash
# Деплой на Ethereum Mainnet
forge script script/Deploy.s.sol \
  --rpc-url https://eth.llamarpc.com \
  --private-key $PRIVATE_KEY \
  --broadcast \
  --verify
```

---

## 🔗 Интеграция с UI

### Пример создания сделки (JavaScript/TypeScript)

```typescript
import { invoke } from '@tauri-apps/api/core';

// Создать сделку
const result = await invoke('create_deal', {
  request: {
    seller_address: '0x1234567890abcdef...',
    deal_type: 'DigitalGoods',
    deadline_timestamp: Math.floor(Date.now() / 1000) + 7 * 24 * 3600, // 7 дней
    message_hash: '0x' + encryptedDealHash,
    ipfs_metadata: 'QmXyz...'
  }
});

// Финансировать (0.5 ETH)
await invoke('fund_deal', {
  request: {
    deal_id: result.deal_id,
    amount_wei: '500000000000000000' // 0.5 ETH in wei
  }
});
```

---

## 🔐 Безопасность

### Аудит

Перед production требуется:
- [ ] Профессиональный аудит (Trail of Bits, OpenZeppelin)
- [ ] Bug bounty программа
- [ ] Формальная верификация

### Known Limitations

- ⚠️ Нет поддержки batch операций
- ⚠️ Арбитр выбирается автоматически (упрощено)
- ⚠️ Нет механизмов обжалования после решения арбитра

### Recommendations

- ✅ Использовать multisig для treasury
- ✅ Добавить rate limiting для disputes
- ✅ Реализовать репутационную систему для арбитров
- ✅ Добавить страховочный фонд

---

## 📈 Метрики

После запуска отслеживать:
- Total Value Locked (TVL)
- Среднее время завершения сделки
- Процент успешных/спорных сделок
- Доход платформы (комиссии)
- Активные арбитры

---

## 📚 Ресурсы

- [Solidity Docs](https://docs.soliditylang.org/)
- [Foundry Book](https://book.getfoundry.sh/)
- [Ethers.rs Docs](https://docs.rs/ethers/)
- [Smart Contract Best Practices](https://consensys.github.io/smart-contract-best-practices/)

---

> **Резюме:** P2P Escrow готов к деплою и тестированию! 🚀
