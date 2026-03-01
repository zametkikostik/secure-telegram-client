# 🇰🇿 AI-Бухгалтер для ИП (Казахстан) - Production

**Полноценная система автоматизации бухгалтерии для ИП на упрощёнке**

## 🎯 Возможности

### 📊 Бухгалтерский учёт
- ✅ Учёт доходов и расходов
- ✅ AI-классификация транзакций (Qwen 3.5 Plus)
- ✅ Расчёт налога (4% с 2026 года)
- ✅ Контроль лимита дохода (94.3 млн ₸)
- ✅ Экспорт в CSV/Excel

### 🏦 Интеграция с банками
- ✅ **Kaspi Bank** - автоматическая синхронизация транзакций
- ✅ **Halyk Bank** - получение счетов и транзакций
- ✅ Webhook уведомления от банков
- ✅ Проверка баланса в реальном времени

### 📑 Налоговая отчётность
- ✅ Генерация декларации 101.02 (XML/PDF)
- ✅ Отправка в СНИС (налоговую)
- ✅ Проверка контрагентов по ИНН
- ✅ Получение ЭСФ (счета-фактуры)
- ✅ Статус деклараций

### 🔔 Уведомления
- ✅ Telegram бот для уведомлений
- ✅ Напоминания об уплате налога
- ✅ Напоминания о сдаче декларации
- ✅ Предупреждения о приближении к лимиту

### 🛡️ Безопасность
- ✅ JWT аутентификация
- ✅ Хранение паролей (bcrypt)
- ✅ Rate limiting
- ✅ Security headers
- ✅ Audit log всех действий

## 🏗️ Архитектура

```
┌─────────────┐     ┌──────────────┐     ┌─────────────┐
│   Kaspi     │     │    Halyk     │     │  Налоговая  │
│    Bank     │     │    Bank      │     │   (СНИС)    │
└──────┬──────┘     └──────┬───────┘     └──────┬──────┘
       │                   │                     │
       │    ┌──────────────┴─────────────────────┤
       │    │                                     │
       ▼    ▼                                     ▼
┌─────────────────────────────────────────────────────────┐
│                  AI-Бухгалтер                           │
│  ┌─────────┐  ┌─────────┐  ┌─────────┐  ┌─────────┐   │
│  │  FastAPI│  │ Celery  │  │   AI    │  │  Redis  │   │
│  │   API   │  │ Worker  │  │Classifier│  │  Cache  │   │
│  └────┬────┘  └────┬────┘  └────┬────┘  └────┬────┘   │
│       │           │           │           │           │
│       └───────────┴───────────┴───────────┘           │
│                       │                                │
│              ┌────────┴────────┐                       │
│              │   PostgreSQL    │                       │
│              │    Database     │                       │
│              └─────────────────┘                       │
└─────────────────────────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────┐
│                    Frontend                             │
│         Web UI + Telegram Bot + Mobile (future)         │
└─────────────────────────────────────────────────────────┘
```

## 📦 Быстрый старт

### 1. Через Docker (рекомендуется)

```bash
# Клонировать репозиторий
git clone <repo-url>
cd ai-accountant-kz

# Скопировать конфиг
cp .env.production .env

# Отредактировать .env (добавить API ключи)
nano .env

# Запустить всё
docker-compose up -d

# Проверить статус
docker-compose ps
```

### 2. Локальная разработка

```bash
# Создать venv
python -m venv venv
source venv/bin/activate

# Установить зависимости
pip install -r requirements.txt

# Запустить Redis (требуется для Celery)
redis-server

# Запустить приложение
uvicorn src.main:app --reload

# Запустить worker (в отдельном терминале)
celery -A src.tasks worker --loglevel=info

# Запустить scheduler (в отдельном терминале)
celery -A src.tasks beat --loglevel=info
```

## 🌐Endpoints

### Веб-интерфейс
- `http://localhost:8000` - Главная страница
- `http://localhost:8000/docs` - Swagger API документация

### API

#### Аутентификация
```bash
# Регистрация
POST /api/v1/auth/register
{
  "email": "user@example.kz",
  "password": "secure_password",
  "full_name": "Иванов Иван",
  "inn": "123456789012"
}

# Вход
POST /api/v1/auth/login
# Формат: username/password
# Ответ: { "access_token": "...", "token_type": "bearer" }
```

#### Транзакции
```bash
# Список
GET /api/v1/transactions?limit=50&tx_type=income

# Создать
POST /api/v1/transactions?date=2026-02-28&amount=100000&type=income&description=Оплата

# Удалить
DELETE /api/v1/transactions/{id}

# AI классификация
POST /api/v1/ai/classify-all
```

#### Банки
```bash
# Синхронизация Kaspi
POST /api/v1/integrations/kaspi/sync?days=30

# Синхронизация Halyk
POST /api/v1/integrations/halyk/sync?days=30

# Баланс Kaspi
GET /api/v1/integrations/kaspi/balance

# Счета Halyk
GET /api/v1/integrations/halyk/accounts
```

#### Налоговая
```bash
# Отправка декларации
POST /api/v1/integrations/tax/submit-declaration?period=2026

# Список деклараций
GET /api/v1/integrations/tax/declarations

# Скачать декларацию
POST /api/v1/integrations/tax/declaration/{id}/download

# Проверка контрагента
GET /api/v1/integrations/counterparty/{inn}
```

#### Отчёты
```bash
# Сводка
GET /api/v1/summary

# Расчёт налога
GET /api/v1/tax/calculate?period=2026

# Экспорт CSV
GET /api/v1/export/csv

# Месячная статистика
GET /api/v1/stats/monthly?year=2026
```

## 🔑 Настройка

### Переменные окружения (.env)

```ini
# Приложение
ENVIRONMENT=production
DEBUG=false
SECRET_KEY=your-super-secret-key-min-32-chars

# База данных
DATABASE_URL=postgresql://user:pass@db:5432/accountant

# Redis (Celery)
CELERY_BROKER_URL=redis://redis:6379/0

# AI (DashScope/Qwen)
OPENAI_API_KEY=your_dashscope_api_key
OPENAI_MODEL=qwen-plus

# Telegram
TELEGRAM_BOT_TOKEN=bot_token_from_botfather
TELEGRAM_CHAT_ID=your_chat_id
TELEGRAM_ENABLED=true

# Kaspi Bank
KASPI_API_KEY=your_kaspi_api_key
KASPI_MERCHANT_ID=your_merchant_id

# Halyk Bank
HALYK_API_KEY=your_halyk_api_key
HALYK_CLIENT_ID=your_client_id

# Логирование
LOG_LEVEL=INFO
```

### Получение API ключей

#### Kaspi Pay
1. Войти в Kaspi Business
2. Настройки → API
3. Создать API ключ

#### Halyk Open Banking
1. Войти в Halyk Business
2. Developer Portal
3. Register application

#### DashScope (Qwen AI)
1. https://dashscope.aliyun.com/
2. Register → API Keys
3. Create new key

#### Telegram Bot
1. Найти @BotFather
2. /newbot
3. Сохранить токен

## 📅 Налоговые сроки

| Событие | Срок |
|---------|------|
| Декларация за полугодие | 20 августа |
| Декларация за год | 20 февраля |
| Уплата налога (квартал) | 20 января, апреля, июля, октября |

## 🧪 Тесты

```bash
# Запустить тесты
pytest tests/ -v

# С покрытием
pytest tests/ --cov=src --cov-report=html
```

## 📊 Мониторинг

```bash
# Логи приложения
docker-compose logs -f app

# Логи worker
docker-compose logs -f worker

# Статус Celery задач
celery -A src.tasks inspect active

# Health check
curl http://localhost:8000/health
```

## 🚀 Production деплой

```bash
# На сервере
git pull
docker-compose pull
docker-compose up -d

# Миграции БД
docker-compose exec app alembic upgrade head
```

## 📁 Структура проекта

```
ai-accountant-kz/
├── src/
│   ├── api/              # API endpoints
│   │   ├── auth.py       # Аутентификация
│   │   ├── transactions.py
│   │   ├── reports.py
│   │   ├── ai_api.py
│   │   └── integrations.py  # Банки + Налоговая
│   ├── ai/               # AI модули
│   │   └── classifier.py
│   ├── core/             # Ядро
│   │   ├── config.py
│   │   ├── database.py
│   │   ├── security.py
│   │   └── tax_rules.py
│   ├── integrations/     # Внешние сервисы
│   │   ├── kaspi.py
│   │   ├── halyk.py
│   │   └── tax_service.py
│   ├── models/           # Pydantic модели
│   ├── utils/            # Утилиты
│   │   ├── telegram.py
│   │   ├── logging.py
│   │   └── middleware.py
│   ├── tasks.py          # Celery задачи
│   └── main.py           # Точка входа
├── migrations/           # Alembic миграции
├── tests/                # Тесты
├── static/               # Frontend
├── templates/            # HTML шаблоны
├── docker-compose.yml
├── Dockerfile
└── requirements.txt
```

## 🤝 Поддержка

- Telegram: @ai_accountant_kz_support
- Email: support@ai-accountant.kz
- Документация: https://docs.ai-accountant.kz

## 📄 Лицензия

MIT License

---

**AI-Бухгалтер © 2026** - Автоматизация бухгалтерии для ИП Казахстана 🇰🇿
