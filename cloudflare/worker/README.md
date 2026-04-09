# 🔐 Secure Messenger — Cloudflare Worker

Cloudflare Worker для relay сообщений и push-уведомлений когда P2P соединение недоступно.

## 📦 Bindings

| Binding | Тип | Назначение |
|---------|-----|------------|
| `MESSENGER_DB` | D1 Database | Пользователи, сообщения, логи |
| `PUSH_STORE` | KV Namespace | Push токены, session cache |
| `SESSION_CACHE` | KV Namespace | Временные сессии |
| `MESSAGE_BACKUP` | R2 Bucket | Бэкапы зашифрованных сообщений |
| `FILE_STORAGE` | R2 Bucket | Хранение файлов (стеганография) |

## 🚀 Быстрый старт

### 1. Установка зависимостей

```bash
cd cloudflare/worker
npm install
```

### 2. Создание ресурсов

```bash
# Создать D1 базу данных
npm run d1:create

# Создать KV namespaces
npm run kv:create
npm run kv:create:preview

# Создать R2 bucket
npm run r2:create
```

### 3. Обновить wrangler.toml

Заменить placeholder ID на реальные значения из вывода команд выше:

```toml
[[d1_databases]]
database_id = "реальный-d1-id"

[[kv_namespaces]]
id = "реальный-kv-id"
```

### 4. Применить миграции

```bash
# Локально
npm run db:migrate

# Production
npm run db:migrate:prod
```

### 5. Запуск

```bash
# Development
npm run dev

# Deploy
npm run deploy
```

## 📁 Структура

```
cloudflare/worker/
├── wrangler.toml              # Конфигурация Worker
├── package.json               # NPM зависимости
├── migrations/
│   └── 001_initial_schema.sql # D1 схема БД
├── src/
│   └── worker.js              # Основной код Worker
└── test/
    └── worker.test.js         # Тесты
```

## 🔧 API Endpoints

| Метод | Путь | Описание |
|-------|------|----------|
| `POST` | `/api/v1/msg` | Отправить зашифрованное сообщение |
| `GET` | `/api/v1/msg/{user_id}` | Получить pending сообщения |
| `POST` | `/api/v1/register` | Зарегистрировать push токен |
| `GET` | `/health` | Health check |

## 📊 Мониторинг

```bash
# Логи в реальном времени
wrangler tail

# Метрики
wrangler metrics
```

## 🔒 Безопасность

- Все сообщения зашифрованы E2EE (X25519 + Kyber1024)
- Worker не имеет доступа к содержимому сообщений
- Rate limiting на уровне IP
- CORS только для разрешённых origins
