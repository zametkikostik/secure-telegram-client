# 🛡️ Liberty Reach — Fortress Edition v0.4.0-stable

**Децентрализованный P2P протокол суверенной связи.**

*Built for freedom, encrypted for life.*

---

## 🚀 Mission: Sovereignty First

Liberty Reach — это не просто мессенджер. Это децентрализованная среда, где финансовый суверенитет (Polygon/Web3) встречается с абсолютной приватностью (libp2p Mesh).

### 💎 Key Features:

| Feature | Description |
|---------|-------------|
| **P2P Mesh Network** | Прямое соединение между узлами без центральных серверов |
| **The Fortress Security** | AES-256-GCM + X25519 Key Exchange + Double Ratchet |
| **Noise Protocol** | Транспортное шифрование (защита от DPI) |
| **Kademlia DHT** | Децентрализованное обнаружение узлов |
| **AI Integration** | Локальная Ollama (Qwen) + Cloud Fallback (OpenRouter) |
| **Financial Layer** | Polygon Web3 (ERC20/MATIC/NFT) для транзакций и AI-доступа |
| **Emergency Geo-Discovery** | Поиск близких через зашифрованные метаданные |
| **IPFS Content Routing** | Децентрализованное хранение файлов |

---

## ⚖️ GDPR & Privacy by Design (EU/Bulgaria)

Проект разработан с учетом строгих требований **GDPR** и законодательства **Болгарии** о защите данных:

| Принцип | Реализация |
|---------|------------|
| **Zero-Knowledge Architecture** | Мы не собираем, не храним и не передаем ваши данные |
| **Local Storage Only** | Все ключи и переписка находятся исключительно на устройствах |
| **Instant Purge** | Функция мгновенной очистки метаданных (`zeroize()`) |
| **No Central Servers** | Полностью децентрализованная архитектура |
| **E2EE by Default** | Все сообщения шифруются на устройстве отправителя |
| **Data Portability** | Экспорт идентичности через `identity.key` |

### Правовое основание:
- **Регламент (ЕС) 2016/679 (GDPR)** — ст. 5, 6, 25
- **Закон за защита на личните данни (България)** — чл. 1, 3
- **ePrivacy Directive 2002/58/EC** — конфиденциальность коммуникаций

---

## 🛠️ Installation

### Быстрый старт (Dev-Build)

```bash
# Клонирование репозитория
git clone https://github.com/zametkikostik/liberty-reach-messenger.git
cd liberty-reach-messenger

# Сборка релизной версии
cargo build --release --features "voice"

# Запуск
./target/release/liberty-reach-messenger
```

### Системные требования

| OS | Dependencies |
|----|-------------|
| **Ubuntu 22.04** | `pkg-config libssl-dev libasound2-dev` |
| **Windows 11** | Visual Studio 2022 + OpenSSL |
| **macOS** | Xcode Command Line Tools |

### Переменные окружения (.env.local)

```bash
# AI Integration
OLLAMA_MODEL=qwen2.5-coder:3b
OPENROUTER_API_KEY=sk-or-v1-...

# Web3 Monetization
WEB3_RPC_URL=https://polygon-rpc.com
WEB3_WALLET_ADDRESS=0x...
AI_ERC20_TOKEN=0x...      # Опционально
AI_NFT_CONTRACT=0x...     # Опционально

# Cloudflare Worker (Emergency Tracking)
SIGNALING_URL=https://liberty-reach-tracking.kostik.workers.dev
```

---

## 🏰 Security Architecture

### The Fortress Stack

```
┌─────────────────────────────────────────────────────────┐
│              Liberty Reach Security Stack               │
├─────────────────────────────────────────────────────────┤
│  Application  │  Double Ratchet + AES-256-GCM          │
│  Transport    │  Noise Protocol (libp2p-noise)         │
│  Network      │  libp2p + Kademlia DHT + mDNS          │
│  Identity     │  Ed25519 Signatures + X25519 DH        │
│  Memory       │  Zeroize (secure cleanup)              │
└─────────────────────────────────────────────────────────┘
```

### Cryptographic Primitives

| Algorithm | Purpose | Security Level |
|-----------|---------|----------------|
| **AES-256-GCM** | Message encryption | 256-bit |
| **X25519** | Key exchange | 128-bit |
| **Ed25519** | Digital signatures | 128-bit |
| **HKDF-SHA256** | Key derivation | 256-bit |
| **HMAC-SHA256** | API authentication | 256-bit |

---

## 🌐 Cloudflare Worker (Emergency Tracking)

### Развертывание

```bash
cd cloudflare-worker

# Автоматический деплой
./deploy_worker.sh

# Ручная настройка
wrangler login
wrangler kv:namespace create PEER_METADATA
wrangler kv:namespace create SIGNALING
wrangler secret put HMAC_SECRET
wrangler deploy
```

### API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/metadata` | POST | Update peer metadata (HMAC signed) |
| `/metadata/:peer_id` | GET | Get peer metadata |
| `/signal/offer` | POST | WebRTC SDP offer |
| `/signal/answer` | POST | WebRTC SDP answer |
| `/signal/ice-candidate` | POST | ICE candidate |
| `/signal/:peer_id` | GET | Get signaling messages |

---

## 🇧🇬 Legal & Privacy (BG)

### Политика за поверителност

> **Liberty Reach е децентрализирано приложение.**
>
> Всички данни се съхраняват локално на вашето устройство. Ние не събираме лична информация, IP адреси или метаданни на централни сървъри.
>
> В съответствие с **Регламент (ЕС) 2016/679 (GDPR)**, вие имате пълен контрол върху вашите ключове и история на съобщенията.
>
> **Използвайки това приложение, вие поемате пълна отговорност за сигурността на вашите частни ключи.**

### Вашите права по GDPR:

1. **Право на достъп** — Преглед на всички съхранявани данни
2. **Право на изтриване** — Мигновено изтриване чрез `zeroize()`
3. **Право на преносимост** — Експорт на `identity.key`
4. **Право на възражение** — Деактивация на Family Safety

---

## 🧪 Testing

```bash
# Все тесты
cargo test --no-default-features

# Тесты с voice feature
cargo test --features voice

# Тесты конкретного модуля
cargo test crypto
cargo test ratchet
cargo test ai
```

**Test Coverage:** 26 passed ✅

---

## 📦 Release Build (GitHub Actions)

Автоматична сборка при създаване на таг:

```bash
git tag v0.4.0-fortress-stable
git push origin --tags
```

GitHub Actions ще създаде:
- Linux binary (stripped)
- Windows binary (stripped)
- SHA256 checksums

---

## 🤝 Contributing

### Security Guidelines

1. **No hardcoded secrets** — Използвайте `.env.local`
2. **Zeroize sensitive data** — Изчиствайте ключовете от паметта
3. **Constant-time comparisons** — За криптографски операции
4. **Validate all input** — Никога не се доверявайте на външни данни

### Reporting Vulnerabilities

Вижте **[SECURITY.md](SECURITY.md)** за процес на докладване на уязвимости.

---

## 📄 License

**MIT License** — Свободното ПО е основа на цифровия суверенитет.

---

## 👤 Author

**Konstantin** — Decentralized Systems Developer

- **Email**: zametkikostik@gmail.com
- **GitHub**: https://github.com/zametkikostik
- **Telegram**: @liberty_reach_support

---

## 🙏 Acknowledgments

- **libp2p** — Децентрализирани мрежови протоколи
- **Ollama** — Локални LLM модели
- **Cloudflare Workers** — Edge computing
- **Polygon** — Мащабируем блокчейн
- **Open Source Community** — Криптографски библиотеки

---

*Liberty Reach — защото свободата на връзката е фундаментално човешко право.*

**Version:** v0.4.0-fortress-stable  
**Build Date:** March 12, 2026  
**Security Audit:** Internal ✅
