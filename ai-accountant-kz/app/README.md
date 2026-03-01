# AI Accountant KZ Tax API

Production Ready API сервис для расчета налогов ИП Казахстан 2026.

## Быстрый старт

### 1. Запуск через Docker Compose

```bash
# Копируем .env файл
cp .env.tax-api.example .env

# Редактируем .env (обязательно измените SECRET_KEY и API_KEY)

# Запускаем сервис
docker-compose -f docker-compose.tax-api.yml up -d

# Проверяем статус
docker-compose -f docker-compose.tax-api.yml ps

# Смотрим логи
docker-compose -f docker-compose.tax-api.yml logs -f tax_api
```

### 2. Локальный запуск (для разработки)

```bash
# Установка зависимостей
pip install -r requirements.tax-api.txt

# Копируем .env
cp .env.tax-api.example .env

# Запуск
uvicorn app.main:app --reload --host 0.0.0.0 --port 8000
```

## API Endpoints

### Health Check
```bash
curl http://localhost:8000/v1/health
```

### Расчет от оклада (gross)
```bash
curl -X POST "http://localhost:8000/v1/calculate/gross?amount=300000" \
  -H "X-API-Key: your-api-key"
```

### Расчет от суммы на руки (net)
```bash
curl -X POST "http://localhost:8000/v1/calculate/net?amount=250000" \
  -H "X-API-Key: your-api-key"
```

### Налоговые ставки
```bash
curl http://localhost:8000/v1/rates
```

## Документация

После запуска откройте:
- Swagger UI: http://localhost:8000/docs
- ReDoc: http://localhost:8000/redoc

## Константы 2026

| Параметр | Значение |
|----------|----------|
| МРП | 4 325 ₸ |
| МЗП | 85 000 ₸ |
| ОПВР | 2.5% |
| КБК ОПВР | 183110 |
| Вычет 14 МРП | 60 550 ₸ |

## Структура проекта

```
app/
├── __init__.py
├── main.py              # FastAPI приложение
├── core/
│   ├── __init__.py
│   ├── config.py        # Настройки через pydantic-settings
│   ├── logging_config.py # JSON логирование с Trace ID
│   └── tax_core.py      # Налоговый движок KZ 2026
├── api/
│   ├── __init__.py
│   └── tax.py           # API маршруты
└── schemas/
    ├── __init__.py
    └── tax.py           # Pydantic схемы
```

## Безопасность

### API Key Authentication

Все endpoints (кроме /v1/health) требуют заголовок `X-API-Key`:

```bash
curl -H "X-API-Key: your-secret-api-key" http://localhost:8000/v1/rates
```

### Environment Variables

Обязательные переменные в `.env`:

```env
# Безопасность (обязательно измените!)
SECRET_KEY=your-super-secret-key-min-32-chars
API_KEY=your-api-key-for-client-auth

# Окружение
ENVIRONMENT=production
DEBUG=false

# Логирование
LOG_LEVEL=INFO
LOG_FORMAT=json
```

## Логи

Логи выводятся в JSON формате (удобно для ELK/Loki):

```json
{
  "timestamp": "2026-03-01T12:00:00.000Z",
  "level": "INFO",
  "logger": "ai-accountant",
  "message": "Request: POST /v1/calculate/gross",
  "service": "AI Accountant KZ Tax API",
  "trace_id": "550e8400-e29b-41d4-a716-446655440000"
}
```

## Monitoring

### Health Check
```bash
curl http://localhost:8000/v1/health
```

### Performance Headers

Каждый ответ содержит заголовки:
- `X-Process-Time`: время обработки в мс
- `X-Trace-ID`: ID трассировки запроса

## Production Deployment

```bash
# Build
docker-compose -f docker-compose.tax-api.yml build

# Deploy
docker-compose -f docker-compose.tax-api.yml up -d

# Scale (если нужно)
docker-compose -f docker-compose.tax-api.yml up -d --scale tax_api=3
```

## License

Proprietary. All rights reserved.
