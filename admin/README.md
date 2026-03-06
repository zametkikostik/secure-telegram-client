# 🛡️ Secure Telegram Admin Panel

> **Админ-панель с верификацией и бейджами**

## 🚀 Возможности

### 👥 Управление пользователями
- Просмотр списка пользователей
- Бан/Разбан пользователей
- Поиск пользователей

### ✅ Верификация
- Заявки на верификацию
- Выдача бейджа верификации (✓)
- Отзыв верификации
- Типы верификации: личность, организация, бот

### 🎖️ Бейджи
- Создание кастомных бейджей
- Выдача бейджей пользователям
- Предустановленные бейджи:
  - ✓ **Verified** — верифицированный пользователь
  - ★ **Premium** — premium подписчик
  - 🤖 **Bot** — официальный бот
  - 🎧 **Support** — поддержка
  - 🛡️ **Admin** — администратор

### 📊 Модерация
- Просмотр жалоб
- Обработка жалоб
- Статистика платформы

### 🤖 Управление ботами
- Список всех ботов
- Верификация ботов (✓)
- Статистика ботов

### 📈 Dashboard
- Всего пользователей
- Всего ботов
- Верифицированные пользователи
- Ожидающие верификации
- Активные жалобы
- Забаненные пользователи
- Сообщения за 24 часа

## 📋 Быстрый старт

### 1. Установка зависимостей

```bash
cd admin
cargo build --release
```

### 2. Настройка переменных окружения

```bash
export ADMIN_ADDR="0.0.0.0:8082"
export ADMIN_SECRET="your-admin-secret-change-me"
export JWT_SECRET="your-jwt-secret"
```

### 3. Запуск

```bash
cargo run --release
```

### 4. Вход в админ-панель

```bash
# Логин по умолчанию создаётся в БД
# Или создайте через SQL:
INSERT INTO admins (id, username, password_hash, email, role)
VALUES ('admin_1', 'admin', '$argon2id$v=19$m=19456,t=2,p=1$...', 'admin@secure-telegram.io', 'superadmin');
```

## 🔗 API Endpoints

### Auth

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `POST` | `/admin/login` | Вход админа |
| `POST` | `/admin/logout` | Выход |
| `GET` | `/admin/me` | Текущий админ |

### Dashboard

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `GET` | `/admin/dashboard` | Статистика платформы |

### Users

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `GET` | `/admin/users` | Список пользователей |
| `GET` | `/admin/users/:user_id` | Информация о пользователе |
| `POST` | `/admin/users/:user_id/ban` | Забанить |
| `POST` | `/admin/users/:user_id/unban` | Разбанить |

### Verification

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `GET` | `/admin/verification/requests` | Заявки на верификацию |
| `POST` | `/admin/verification/:user_id` | Верифицировать |
| `POST` | `/admin/verification/:user_id/revoke` | Отозвать |

### Badges

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `GET` | `/admin/badges` | Список бейджей |
| `POST` | `/admin/badges` | Создать бейдж |
| `DELETE` | `/admin/badges/:badge_id` | Удалить бейдж |
| `POST` | `/admin/users/:user_id/badges` | Выдать бейдж |

### Moderation

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `GET` | `/admin/reports` | Список жалоб |
| `POST` | `/admin/reports/:report_id` | Обработать жалобу |

### Bots

| Метод | Endpoint | Описание |
|-------|----------|----------|
| `GET` | `/admin/bots` | Список ботов |
| `POST` | `/admin/bots/:bot_id/verify` | Верифицировать бота |

## 📖 Примеры использования

### Верификация пользователя

```bash
# Получить заявки на верификацию
curl http://localhost:8082/admin/verification/requests

# Верифицировать пользователя
curl -X POST http://localhost:8082/admin/verification/user_123 \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN"
```

### Выдача бейджа

```bash
# Создать бейдж
curl -X POST http://localhost:8082/admin/badges \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN" \
  -d '{
    "name": "VIP",
    "description": "VIP пользователь",
    "icon_url": "/badges/vip.svg",
    "color": "#FFD700"
  }'

# Выдать бейдж пользователю
curl -X POST http://localhost:8082/admin/users/user_123/badges/badge_456 \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN"
```

### Бан пользователя

```bash
curl -X POST http://localhost:8082/admin/users/user_123/ban \
  -H "Authorization: Bearer YOUR_ADMIN_TOKEN"
```

## 🎨 Предустановленные бейджи

| Бейдж | ID | Описание | Цвет |
|-------|-----|----------|------|
| ✓ Verified | `verified` | Верифицированный пользователь | `#3390EC` |
| ★ Premium | `premium` | Premium подписчик | `#FFD700` |
| 🤖 Bot | `bot` | Официальный бот | `#FF6B6B` |
| 🎧 Support | `support` | Поддержка | `#4ECDC4` |
| 🛡️ Admin | `admin` | Администратор | `#FF4757` |

## 🔐 Безопасность

- ✅ Argon2 хэширование паролей
- ✅ JWT токены с expiration
- ✅ Role-based access control
- ✅ Audit logging всех действий

## 📊 Dashboard

Dashboard показывает:
- Всего пользователей
- Всего ботов
- Верифицированные пользователи
- Ожидающие верификации
- Активные жалобы
- Забаненные пользователи
- Сообщения за 24 часа

---

**Secure Telegram Team © 2026**
