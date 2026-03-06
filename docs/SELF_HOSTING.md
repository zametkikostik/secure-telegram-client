# Liberty Reach — Self-Hosting Руководство

## 📖 Содержание

1. [Требования](#требования)
2. [Быстрый старт](#быстрый-старт)
3. [Ручная установка](#ручная-установка)
4. [Конфигурация](#конфигурация)
5. [Обновление](#обновление)
6. [Troubleshooting](#troubleshooting)

---

## 🛠 Требования

- Docker 20.10+
- Docker Compose 2.0+
- 2 GB RAM
- 10 GB свободного места
- Открытые порты: 8008

---

## 🚀 Быстрый старт

### One-click install

```bash
# Клонировать репозиторий
git clone https://github.com/libertyreach/messenger.git
cd messenger/self-hosting

# Запустить установщик
chmod +x install.sh
./install.sh
```

### Ручной запуск

```bash
# Создать .env
cat > .env << EOF
SERVER_NAME=localhost
ADMIN_WALLET=0x0000000000000000000000000000000000000000
EOF

# Запустить
docker-compose up -d
```

---

## 📦 Ручная установка

### Шаг 1: Подготовка

```bash
mkdir -p liberty-reach/data
cd liberty-reach
```

### Шаг 2: Docker Compose

Создайте `docker-compose.yml`:

```yaml
version: '3.8'

services:
  liberty-reach:
    image: libertyreach/messenger:latest
    ports:
      - "8008:8008"
    volumes:
      - ./data:/data
    environment:
      - SERVER_NAME=${SERVER_NAME}
      - ADMIN_WALLET=${ADMIN_WALLET}
    restart: unless-stopped
```

### Шаг 3: Запуск

```bash
docker-compose up -d
```

### Шаг 4: Проверка

```bash
docker-compose ps
docker-compose logs -f
```

---

## ⚙️ Конфигурация

### Переменные окружения

| Переменная | Описание | По умолчанию |
|------------|----------|--------------|
| `SERVER_NAME` | Домен сервера | `localhost` |
| `ADMIN_WALLET` | Кошелёк администратора | `0x000...000` |
| `MAX_CONNECTIONS` | Макс. подключений | `1000` |
| `LOG_LEVEL` | Уровень логов | `info` |

### Порты

| Порт | Описание |
|------|----------|
| 8008 | HTTP API |
| 8009 | WebSocket |
| 9009 | P2P (TCP) |
| 9010 | P2P (QUIC) |

---

## 🔄 Обновление

```bash
# Обновить образ
docker-compose pull

# Перезапустить
docker-compose down
docker-compose up -d
```

---

## 🔧 Troubleshooting

### Сервер не запускается

```bash
# Проверить логи
docker-compose logs

# Проверить порты
netstat -tlnp | grep 8008
```

### Ошибка подключения к базе данных

```bash
# Проверить volume
docker volume ls
docker-compose exec liberty-reach ls /data
```

### Высокая нагрузка

```bash
# Проверить ресурсы
docker stats

# Увеличить лимиты в docker-compose.yml
deploy:
  resources:
    limits:
      cpus: '2'
      memory: 2G
```

---

## 📊 Мониторинг

### Prometheus + Grafana

```yaml
# docker-compose.yml
services:
  prometheus:
    image: prom/prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml

  grafana:
    image: grafana/grafana
    ports:
      - "3000:3000"
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin
```

---

## 📬 Поддержка

Email: support@libertyreach.io  
GitHub Issues: https://github.com/libertyreach/messenger/issues
