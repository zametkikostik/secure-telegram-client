# 🔐 Secure Telegram Client

> **Приватный мессенджер с post-quantum шифрованием, P2P-сетью и полной независимостью от центральных серверов**

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org)
[![Android](https://img.shields.io/badge/Android-8.0+-green.svg)](https://www.android.com)
[![Web3](https://img.shields.io/badge/Web3-EVM--compatible-blue.svg)](https://ethereum.org)

---

## 📋 Содержание

<details>
<summary><b>Нажмите, чтобы развернуть</b></summary>

1. [О проекте](#-о-проекта)
2. [Возможности](#-возможности)
3. [Архитектура](#-архитектура)
4. [Быстрый старт](#-быстрый-старт)
5. [Установка](#-установка)
6. [Конфигурация](#-конфигурация)
7. [Монетизация](#-монетизация)
8. [RoadMap](#-roadmap)
9. [Поддержка](#-поддержка)

</details>

---

## 📖 О проекте

**Secure Telegram Client** — это децентрализованный мессенджер нового поколения, созданный для тех, кто ценит приватность и безопасность общения.

### Почему это важно?

| Проблема | Решение |
|----------|---------|
| Централизованные серверы хранят ваши данные | P2P-архитектура без единой точки контроля |
| Шифрование может быть взломано квантовыми компьютерами | Post-quantum криптография (Kyber1024) |
| Мессенджеры продают ваши данные | Никакой телеметрии, никаких трекеров |
| Сложность миграции из других мессенджеров | Автоматический импорт из Telegram и WhatsApp |
| Зависимость от инфраструктуры | Self-hosting за 1 клик |

### Ключевые преимущества

- 🛡️ **Безопасность** — шифрование на уровне военных стандартов
- 🌐 **Независимость** — работайте без центральных серверов
- 🤖 **AI-помощник** — перевод, саммаризация, генерация кода
- 💰 **Web3-интеграция** — обмен криптовалют прямо в чате
- 📱 **Кроссплатформенность** — Android, Desktop, Web

---

## ✨ Возможности

### 🔐 Безопасность и приватность

| Функция | Описание |
|---------|----------|
| **Post-quantum шифрование** | Алгоритм Kyber1024 — защита от квантовых атак |
| **Стеганография** | Скрытие данных в изображениях (LSB) |
| **Ed25519 подписи** | Криптографическая подпись сообщений |
| **JWT аутентификация** | Безопасные сессии пользователей |
| **Argon2 хэширование** | Надёжное хранение паролей |

### 💬 Чаты и общение

- ✅ **Приватные чаты** 1-на-1 с end-to-end шифрованием
- ✅ **Групповые чаты** до 1000 участников
- ✅ **Каналы** для массовых рассылок (broadcast)
- ✅ **AI авто-перевод** 100+ языков в реальном времени
- ✅ **Статусы прочтения** и индикаторы набора текста
- ✅ **Ответы на сообщения** и треды
- ✅ **Редактирование** и удаление сообщений
- ✅ **24-часовые сообщения** — автоудаление через 24 часа
- ✅ **Семейные статусы** — женат/замужем/встречаюсь с партнёром
- ✅ **Синхронизированные обои** — одинаковые обои у обоих собеседников (розы, природа, закаты и т.д.)
- ✅ **Закреплённые сообщения** — закрепление важных сообщений в чате
- ✅ **Избранные сообщения** — сохранение заметок и мыслей с тегами
- ✅ **Отложенные сообщения** — планирование отправки по времени
- ✅ **Стикеры** — библиотека стикеров и паки
- ✅ **GIF** — популярные GIF анимации
- ✅ **Эмодзи реакции** — реакции на сообщения (❤️👍😂)
- ✅ **Ночной режим** — тёмная тема для комфортного использования
- ✅ **Темы оформления** — светлая, тёмная, ночная темы
- ✅ **Био** — информация о пользователе в профиле
- ✅ **Демонстрация экрана** — показ экрана во время звонка

### 📞 Звонки и конференции

| Тип | Возможности |
|-----|-------------|
| **Аудио звонки** | WebRTC, HD-качество, шумоподавление |
| **Видео звонки** | До 1080p, адаптивный битрейт |
| **AI перевод** | Перевод речи в реальном времени |
| **Субтитры** | WebVTT, 100+ языков |
| **Рация** | Push-to-Talk для быстрой связи |
| **Конференции** | До 100 участников одновременно |

### 🤖 AI функции

- 🧠 **Qwen 3.5 интеграция** — мощный AI-ассистент
- 🌍 **Перевод текста** — 100+ языков
- 📝 **Саммаризация** — краткое содержание чатов
- 💻 **Генерация кода** — помощь разработчикам
- 🎤 **Speech-to-Text** — Vosk, офлайн-распознавание
- 🔊 **Text-to-Speech** — Qwen TTS, естественный голос
- 🎯 **Голосовые команды** — управление без рук

### 📲 Миграция из других мессенджеров

| Источник | Формат | AI перевод |
|----------|--------|------------|
| **Telegram** | JSON export | ✅ |
| **WhatsApp** | TXT export | ✅ |

### 💰 Web3 интеграции

- 🦊 **MetaMask** — встроенный кошелёк
- 0️⃣ **0x Protocol** — обмен токенов (0.5-3% комиссия)
- 🔄 **ABCEX API** — покупка криптовалюты (2-3%)
- 📊 **Bitget API** — биржевые операции (2-3%)
- 🤝 **P2P Escrow** — смарт-контракт для безопасных сделок (3%)
- 💸 **FeeSplitter** — автоматическое распределение комиссий

### 📡 P2P сеть

```
┌─────────────────────────────────────────┐
│         DECENTRALIZED NETWORK           │
├─────────────────────────────────────────┤
│ libp2p: TCP, QUIC, Noise, Yamux        │
│ Kademlia DHT для маршрутизации         │
│ Gossipsub для чатов                    │
│ mDNS для локального обнаружения        │
└─────────────────────────────────────────┘
```

### 📱 Клиенты

| Платформа | Статус | Технологии |
|-----------|--------|------------|
| **Desktop v1.0** | ✅ | Tauri (Windows, Mac, Linux) |
| **Desktop v2.0** | ✅ | Tauri v2 (Linux Mint оптимизировано) |
| **Enterprise** | ✅ | Axum + Tauri (SSO, аудит, compliance) |
| **Android** | ✅ | React Native (APK) |
| **Web** | ✅ | React + TypeScript |
| **iOS** | ⏳ | В разработке |

### 🔔 Уведомления

- ✅ Firebase Cloud Messaging
- ✅ Push-уведомления для Android
- ✅ WebSocket уведомления в реальном времени

### 🖥️ Self-Hosting

- ✅ Docker образ
- ✅ Docker Compose
- ✅ One-click install скрипт

### 🏢 Enterprise версия

- 🔐 **SSO** — OAuth2, SAML, LDAP, Kerberos
- 📊 **Аудит** — централизованное логирование, SIEM экспорт
- 👥 **Админ-панель** — управление пользователями и группами
- 🛡️ **Compliance** — GDPR, DLP, политики безопасности
- 📈 **Мониторинг** — Prometheus, Grafana, OpenTelemetry

---

## 🏗️ Архитектура

```
┌─────────────────────────────────────────────────────────────┐
│                    SECURE TELEGRAM CLIENT                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐        │
│  │   Desktop   │  │   Android   │  │     Web     │        │
│  │   (Tauri)   │  │  (React)    │  │  (React)    │        │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘        │
│         │                │                │                 │
│         └────────────────┴────────────────┘                 │
│                          │                                  │
│         ┌────────────────▼────────────────┐                 │
│         │      Cloudflare Worker          │                 │
│         │   (Push Notifications)          │                 │
│         └────────────────┬────────────────┘                 │
│                          │                                  │
│         ┌────────────────▼────────────────┐                 │
│         │        Backend (Axum)           │                 │
│         │  REST API + WebSocket + Files   │                 │
│         └────────────────┬────────────────┘                 │
│                          │                                  │
│    ┌─────────────────────┼─────────────────────┐            │
│    │                     │                     │            │
│    ▼                     ▼                     ▼            │
│ ┌────────┐       ┌────────────┐       ┌────────────┐       │
│ │ SQLite │       │ PostgreSQL │       │  uploads/  │       │
│ │  (dev) │       │ (prod)     │       │   files    │       │
│ └────────┘       └────────────┘       └────────────┘       │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐   │
│  │              P2P NETWORK (libp2p)                   │   │
│  │  Direct connections between clients (DHT, Gossip)   │   │
│  └─────────────────────────────────────────────────────┘   │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Структура проекта

```
secure-telegram-client/
├── messenger/                    # Rust + Tauri (Desktop)
│   ├── Cargo.toml
│   ├── Capfile.toml              # Tauri Mobile config
│   ├── tauri.conf.json
│   └── src/
│       ├── main.rs
│       ├── crypto/
│       │   ├── mod.rs
│       │   ├── pqcrypto.rs       # Kyber1024
│       │   └── steganography.rs  # LSB
│       ├── webrtc/
│       │   ├── mod.rs
│       │   ├── walkie_talkie.rs
│       │   ├── conference.rs
│       │   └── translator.rs
│       ├── p2p/
│       │   ├── mod.rs
│       │   └── libp2p.rs
│       ├── web3/
│       │   ├── mod.rs
│       │   ├── metamask.rs
│       │   ├── 0x_swap.rs
│       │   ├── abcex.rs
│       │   ├── bitget.rs
│       │   └── p2p_escrow.rs
│       ├── ai/
│       │   ├── mod.rs
│       │   ├── translator.rs
│       │   ├── assistant.rs
│       │   ├── speech_to_text.rs
│       │   ├── text_to_speech.rs
│       │   └── subtitles.rs
│       ├── telegram/
│       │   ├── mod.rs
│       │   ├── importer.rs
│       │   ├── migration.rs
│       │   └── whatsapp_importer.rs
│       └── chat/
│           ├── mod.rs
│           ├── private.rs
│           ├── group.rs
│           └── channel.rs
│
├── server/                       # Backend (Rust + Axum)
│   ├── Cargo.toml
│   ├── .env.example
│   └── src/
│       ├── main.rs
│       ├── db.rs                 # SQLite/PostgreSQL
│       ├── auth.rs               # JWT + Ed25519
│       ├── websocket.rs          # Real-time
│       ├── middleware.rs
│       └── api/
│           ├── mod.rs
│           ├── auth.rs
│           ├── users.rs
│           ├── chats.rs
│           ├── messages.rs
│           ├── files.rs
│           ├── web3.rs
│           └── ai.rs
│
├── frontend/                     # Web UI (React + TypeScript)
│   ├── package.json
│   ├── vite.config.ts
│   ├── tailwind.config.js
│   ├── index.html
│   └── src/
│       ├── main.tsx
│       ├── App.tsx
│       ├── api/
│       ├── store/
│       ├── hooks/
│       ├── components/
│       ├── pages/
│       └── styles/
│
├── mobile/                       # Android APK (React Native)
│   ├── package.json
│   ├── src/
│   │   └── App.tsx
│   └── android/
│       ├── build.gradle
│       ├── app/
│       │   ├── build.gradle
│       │   └── src/main/
│       │       ├── AndroidManifest.xml
│       │       └── java/io/libertyreach/
│       │           └── MessagingService.kt
│       └── ...
│
├── migration-tool/               # Python (импорт)
│   ├── telegram_importer.py
│   ├── whatsapp_importer.py
│   ├── ai_translator.py
│   └── requirements.txt
│
├── smart-contracts/              # Solidity
│   ├── P2PEscrow.sol
│   ├── FeeSplitter.sol
│   ├── package.json
│   ├── hardhat.config.ts
│   └── scripts/deploy.ts
│
├── cloudflare/                   # Cloudflare Workers
│   └── worker/
│       ├── wrangler.toml
│       └── src/
│           ├── worker.ts
│           ├── matrix.ts
│           └── webrtc.ts
│
├── self-hosting/                 # Docker
│   ├── docker-compose.yml
│   ├── Dockerfile
│   └── install.sh
│
├── uploads/                      # Файлы
│   ├── images/
│   ├── videos/
│   ├── audio/
│   └── files/
│
└── docs/
    ├── README.md
    ├── USER_GUIDE.md
    ├── SELF_HOSTING.md
    └── API.md
```

---

## 📦 Установка

### Требования

| Компонент | Версия | Ссылка |
|-----------|--------|--------|
| **Rust** | 1.75+ | [rust-lang.org](https://www.rust-lang.org) |
| **Node.js** | 18+ | [nodejs.org](https://nodejs.org) |
| **Android SDK** | 34+ | [developer.android.com](https://developer.android.com) |
| **Docker** | 20+ | [docker.com](https://www.docker.com) |

### Быстрый старт

#### 1. Backend сервер

```bash
cd server
cp .env.example .env
# Отредактируйте .env (JWT_SECRET, DATABASE_URL)

cargo run --release
# Сервер запустится на http://localhost:8008
```

#### 2. Frontend (Web)

```bash
cd frontend
npm install
npm run dev
# Откройте http://localhost:3000
```

#### 3. Android APK (Готовое подписанное)

**✅ Готовое APK уже подписано и доступно!**

```bash
# Установка на устройство
adb install mobile/android/app/build/outputs/apk/release/app-release.apk

# Или скопируйте APK на телефон и установите
```

**Сборка нового APK:**

```bash
cd mobile/android

# Debug APK (для тестирования)
./gradlew assembleDebug

# Release APK (подписанный)
./gradlew assembleRelease
```

**Keystore для подписи:**
- Файл: `mobile/android/liberty-reach.keystore`
- Валидность: 10 000 дней
- Ключ: RSA 2048 bit

📖 **Подробности:** [APK_INFO.md](APK_INFO.md)

#### 4. Desktop (Tauri)

**Desktop v1.0 (Базовая версия):**

```bash
cd messenger
cargo install tauri-cli
cargo tauri dev
# или
cargo tauri build
```

**Desktop v2.0 (Linux Mint — оптимизировано):**

```bash
# Установка зависимостей
sudo apt-get install -y libgtk-3-dev libwebkit2gtk-4.0-dev \
    libappindicator3-dev librsvg2-dev libnotify-dev libsecret-1-dev

# Сборка
cd desktop-v2
chmod +x scripts/build.sh
./scripts/build.sh

# Установка .deb пакета
sudo dpkg -i ../releases/secure-telegram-desktop_2.0.0_amd64.deb

# Запуск
secure-telegram-desktop
```

📖 **Документация:** [desktop-v2/README.md](desktop-v2/README.md)

#### 5. Enterprise версия (SSO, аудит, compliance)

```bash
# Установка зависимостей
sudo apt-get install -y libssl-dev pkg-config libpq-dev \
    libldap2-dev libsasl2-dev cmake build-essential

# Сборка
cd enterprise
chmod +x scripts/build.sh
./scripts/build.sh

# Установка .deb пакета
sudo dpkg -i ../releases/secure-telegram-enterprise_1.0.0_amd64.deb

# Настройка
sudo nano /etc/secure-telegram/config.toml
sudo nano /etc/secure-telegram/sso.toml

# Запуск
sudo systemctl start secure-telegram-enterprise
sudo systemctl enable secure-telegram-enterprise
```

📖 **Документация:** [enterprise/README.md](enterprise/README.md)

#### 6. Self-Hosting (Docker)

```bash
cd self-hosting
./install.sh
# или
docker-compose up -d
```

---

## 🔧 Конфигурация

### Переменные окружения (server/.env)

```bash
SERVER_ADDR=0.0.0.0:8008
DATABASE_URL=sqlite:./liberty_reach.db
JWT_SECRET=ваш-секретный-ключ
UPLOADS_DIR=./uploads
QWEN_API_KEY=ваш-Qwen-API-ключ
ADMIN_WALLET=0x...
```

### Android APK сборка

Создайте keystore:
```bash
keytool -genkey -v -keystore liberty-reach.keystore -alias liberty -keyalg RSA -keysize 2048 -validity 10000
```

Добавьте в `mobile/android/gradle.properties`:
```properties
LIBERTY_UPLOAD_STORE_FILE=liberty-reach.keystore
LIBERTY_UPLOAD_STORE_PASSWORD=ваш-пароль
LIBERTY_UPLOAD_KEY_ALIAS=liberty
LIBERTY_UPLOAD_KEY_PASSWORD=ваш-пароль
```

---

## 💰 Монетизация

```
┌─────────────────────────────────────────┐
│ ДОХОД ТОЛЬКО С ИНТЕГРАЦИЙ              │
├─────────────────────────────────────────┤
│ 0x Protocol:  0.5-3% с обмена          │
│ ABCEX API:    2-3% с покупки           │
│ Bitget API:   2-3% с покупки           │
│ P2P Escrow:   3% с сделки              │
├─────────────────────────────────────────┤
│ ВСЁ ОСТАЛЬНОЕ БЕСПЛАТНО:               │
│ ✓ Чаты, звонки, конференции            │
│ ✓ AI перевод, субтитры                 │
│ ✓ Рация, P2P сеть                      │
│ ✓ Self-hosting                         │
│ ✓ Миграция из Telegram/WhatsApp        │
└─────────────────────────────────────────┘
```

**Примерный доход:** $1,100-11,000/день (при объёме $10K-100K)

---

## 🗺️ RoadMap

```
2024 Q1                     2024 Q2                     2024 Q3-Q4
│                           │                           │
├── ✅ Базовая архитектура   ├── ✅ Web3 интеграции      ├── ✅ IPFS хранение
│   (январь-март)           │   (апрель-июнь)           │   (июль-сентябрь)
│                           │                           │
├── ✅ P2P сеть             ├── ✅ AI функции           ├── ✅ Мультиязычность
│   (январь-март)           │   (апрель-июнь)           │   (июль-сентябрь)
│                           │                           │
├── ✅ Android APK          ├── ✅ Smart contracts      ├── ✅ Desktop v2.0 (Linux Mint)
│   (январь-март)           │   (апрель-июнь)           │   (октябрь-декабрь)
│                           │                           │
├── ✅ Web frontend         ├── ✅ Миграция из TG/WA    ├── ✅ Enterprise версия
│   (январь-март)           │   (апрель-июнь)           │   (октябрь-декабрь)
│                           │                           │
└── ✅ Self-hosting         └── ✅ Монетизация          └── 🎯 iOS приложение
    (январь-март)               (апрель-июнь)               (октябрь-декабрь)
```

### Текущий статус

| Компонент | Статус | Готовность |
|-----------|--------|------------|
| **Backend** | ✅ Завершено | 100% |
| **Frontend Web** | ✅ Завершено | 100% |
| **Android APK** | ✅ Завершено | 100% |
| **Desktop (Tauri)** | ✅ Завершено | 100% |
| **Desktop v2.0 (Linux Mint)** | ✅ Завершено | 100% |
| **Enterprise (SSO, аудит)** | ✅ Завершено | 100% |
| **P2P сеть** | ✅ Завершено | 100% |
| **Web3 интеграции** | ✅ Завершено | 100% |
| **AI функции** | ✅ Завершено | 100% |
| **Cloudflare Worker** | ✅ Завершено | 100% |
| **Self-hosting** | ✅ Завершено | 100% |
| **IPFS хранение** | ✅ Завершено | 100% |
| **Мультиязычность** | ✅ Завершено | 100% |
| **iOS приложение** | 🚧 В разработке | 0% |

### Планы на будущее

- [ ] **iOS приложение** — нативное приложение для iPhone/iPad
- [ ] **AR/VR интеграция** — виртуальные комнаты для встреч
- [ ] **DAO управление** — голосование токенами за развитие

---

## ✅ Проверка всех функций

| Функция | Статус | Файл |
|---------|--------|------|
| Post-quantum шифрование | ✅ | `messenger/src/crypto/pqcrypto.rs` |
| Стеганография | ✅ | `messenger/src/crypto/steganography.rs` |
| P2P сеть | ✅ | `messenger/src/p2p/libp2p.rs` |
| 0x Integration | ✅ | `messenger/src/web3/0x_swap.rs` |
| AI Translator | ✅ | `messenger/src/ai/translator.rs` |
| Telegram Import | ✅ | `messenger/src/telegram/importer.rs` |
| Backend API | ✅ | `server/src/api/*` |
| WebSocket | ✅ | `server/src/websocket.rs` |
| База данных | ✅ | `server/src/db.rs` |
| Аутентификация | ✅ | `server/src/auth.rs` |
| Frontend Web | ✅ | `frontend/src/*` |
| Android APK | ✅ | `mobile/android/*` |
| Smart Contracts | ✅ | `smart-contracts/*` |
| Cloudflare Worker | ✅ | `cloudflare/worker/*` |
| Docker | ✅ | `self-hosting/*` |

---

## 📬 Поддержка

| Канал | Ссылка |
|-------|--------|
| **Email** | support@libertyreach.io |
| **GitHub** | [github.com/libertyreach/messenger](https://github.com/libertyreach/messenger) |
| **Документация** | [docs.libertyreach.io](https://docs.libertyreach.io) |
| **Telegram** | [@libertyreach](https://t.me/libertyreach) |

---

## 📄 Лицензия

[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

**MIT License** — свободное использование, модификация и распространение.

---

<div align="center">

**Secure Telegram Client** — приватность без компромиссов.

[Начать использование](#-быстрый-старт) • [Сообщить о проблеме](https://github.com/libertyreach/messenger/issues) • [Внести вклад](https://github.com/libertyreach/messenger/pulls)

</div>
