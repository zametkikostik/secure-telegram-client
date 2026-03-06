# Secure Telegram Enterprise

> **Корпоративная версия с SSO, аудитом и compliance**

Secure Telegram Enterprise — это корпоративная версия приватного мессенджера с расширенными функциями для бизнеса.

## 🚀 Возможности

### 🔐 Single Sign-On (SSO)

- **OAuth2 / OpenID Connect** — интеграция с корпоративными IdP
- **SAML 2.0** — поддержка enterprise IdP (Okta, Azure AD, Ping)
- **LDAP / Active Directory** — аутентификация через AD
- **Kerberos** — бесшовная аутентификация в домене

### 📊 Централизованный аудит

- Логирование всех событий
- Поиск и фильтрация логов
- Экспорт в SIEM системы
- Отчёты для compliance

### 👥 Админ-панель

- Управление пользователями
- Управление группами и ролями
- Мониторинг активности
- Настройка SSO провайдеров

### 🛡️ Compliance

- **GDPR** — экспорт и удаление данных
- **DLP** — предотвращение утечек данных
- **Политики безопасности** — гибкие правила
- **Отчёты для регуляторов**

### 🔒 Безопасность

- Post-quantum шифрование (Kyber1024)
- AES-256-GCM / ChaCha20-Poly1305
- Ed25519 подписи
- MFA поддержка

## 📋 Требования

### Сервер

- **ОС:** Ubuntu 20.04+ / Debian 11+ / RHEL 8+
- **Процессор:** 4 ядра, 64-bit
- **ОЗУ:** 8 GB
- **Место на диске:** 10 GB

### Зависимости

- **PostgreSQL:** 13+
- **Redis:** 6+
- **OpenLDAP:** (опционально для LDAP)

## 🚀 Быстрый старт

### Установка из .deb пакета

```bash
# Скачать пакет
wget https://github.com/zametkikostik/secure-telegram-client/releases/latest/download/secure-telegram-enterprise_1.0.0_amd64.deb

# Установить
sudo dpkg -i secure-telegram-enterprise_1.0.0_amd64.deb

# Исправить зависимости
sudo apt-get install -f -y
```

### Настройка

```bash
# Редактирование конфигурации
sudo nano /etc/secure-telegram/config.toml

# Настройка SSO
sudo nano /etc/secure-telegram/sso.toml

# Запуск
sudo systemctl start secure-telegram-enterprise

# Автозапуск
sudo systemctl enable secure-telegram-enterprise

# Проверка статуса
sudo systemctl status secure-telegram-enterprise
```

### Сборка из исходников

```bash
# Установка зависимостей
sudo apt-get update
sudo apt-get install -y \
    libssl-dev pkg-config libpq-dev \
    libldap2-dev libsasl2-dev cmake build-essential

# Сборка
cd enterprise
chmod +x scripts/build.sh
./scripts/build.sh

# Установка
sudo dpkg -i ../releases/secure-telegram-enterprise_1.0.0_amd64.deb
```

## 📖 Конфигурация

### Основная конфигурация

`/etc/secure-telegram/config.toml`

```toml
[server]
listen_addr = "0.0.0.0:8080"
workers = 4

[database]
url = "postgresql://enterprise:password@localhost/secure_telegram"

[redis]
url = "redis://localhost:6379"

[jwt]
secret = "change-this-secret"
```

### SSO конфигурация

`/etc/secure-telegram/sso.toml`

```toml
# OAuth2 / OpenID Connect
[oauth2]
enabled = true
client_id = "your-client-id"
client_secret = "your-client-secret"
issuer_url = "https://auth.example.com"

# LDAP / Active Directory
[ldap]
enabled = true
url = "ldap://ad.example.com:389"
bind_dn = "CN=Service,DC=example,DC=com"
bind_password = "password"
base_dn = "DC=example,DC=com"
```

## 🔧 Админ-панель

### Dashboard

Откройте `https://your-server.com/admin`

- **Пользователи** — управление пользователями
- **Группы** — управление группами
- **Аудит** — просмотр логов
- **Compliance** — отчёты
- **Настройки** — конфигурация SSO

### API

```bash
# Получить список пользователей
curl -H "Authorization: Bearer <token>" \
  https://your-server.com/api/v1/users

# Получить аудит логи
curl -H "Authorization: Bearer <token>" \
  "https://your-server.com/admin/audit?limit=100"

# Экспорт compliance отчёта
curl -H "Authorization: Bearer <token>" \
  "https://your-server.com/admin/compliance/report?from=2024-01-01&to=2024-12-31"
```

## 🛡️ Политики безопасности

### Запрет номеров карт

```toml
[[policies]]
id = "policy_cc_001"
name = "Запрет номеров кредитных карт"
enabled = true

[[policies.rules]]
id = "rule_cc_001"
name = "Блокировка номеров карт"
condition = { type = "ContainsSensitiveData", patterns = ["\\b\\d{4}[- ]?\\d{4}[- ]?\\d{4}[- ]?\\d{4}\\b"] }
action = "Block"
```

### Рабочее время

```toml
[[policies]]
id = "policy_wh_001"
name = "Рабочее время"
enabled = true

[[policies.rules]]
id = "rule_wh_001"
name = "Только в рабочее время"
condition = { type = "OutsideWorkingHours", start = "09:00", end = "18:00", timezone = "Europe/Moscow" }
action = { type = "Warn", message = "Отправка сообщений разрешена только в рабочее время" }
```

## 📊 Мониторинг

### Prometheus метрики

Откройте `http://your-server:9090/metrics`

- `enterprise_users_total` — общее количество пользователей
- `enterprise_messages_total` — общее количество сообщений
- `enterprise_audit_events_total` — события аудита
- `enterprise_violations_total` — нарушения политик

### Grafana дашборды

Импорт дашборда из `docs/grafana-dashboard.json`

## 📄 Лицензии

- **Secure Telegram Enterprise** — Commercial License
- **Open Source компоненты** — их лицензии

## 📞 Поддержка

- **Email:** enterprise@secure-telegram.io
- **Документация:** https://docs.secure-telegram.io/enterprise
- **Portal:** https://support.secure-telegram.io

---

**Secure Telegram Team © 2026**
