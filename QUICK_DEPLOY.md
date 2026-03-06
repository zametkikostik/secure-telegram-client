# 🚀 Инструкция по развёртыванию

## Быстрый старт (5 минут)

### 1. Деплой Backend в Cloudflare

```bash
# Перейдите в папку Cloudflare Worker
cd /home/kostik/secure-telegram-client/cloudflare/worker

# Установите Wrangler (если не установлен)
npm install -g wrangler

# Логин в Cloudflare
wrangler login

# Проверьте wrangler.toml
cat wrangler.toml

# Деплой
wrangler deploy
```

**Запомните URL:** `https://secure-telegram-worker.<subdomain>.workers.dev`

### 2. Настройка переменных в Cloudflare

1. Откройте [Cloudflare Dashboard](https://dash.cloudflare.com/)
2. Workers & Pages → secure-telegram-worker
3. Settings → Variables and Secrets → Add variable

Добавьте:
```
DATABASE_URL=sqlite:./secure-telegram.db
JWT_SECRET=your-super-secret-jwt-key-change-this
UPLOADS_DIR=/tmp/uploads
```

### 3. Установка APK на телефон

```bash
# Перейдите в папку mobile
cd /home/kostik/secure-telegram-client/mobile/android/app/build/outputs/apk/release/

# Подключите телефон по USB (включите отладку)
# Проверьте подключение
adb devices

# Установите APK
adb install app-release.apk
```

### 4. Настройка приложения на телефоне

1. Откройте Secure Telegram
2. Введите Server URL: `https://secure-telegram-worker.<subdomain>.workers.dev`
3. Зарегистрируйтесь
4. Готово!

---

## 🔧 Детальная инструкция

### Cloudflare Worker

#### 1. Создание аккаунта Cloudflare

1. Перейдите на https://dash.cloudflare.com/sign-up
2. Зарегистрируйтесь (бесплатно)
3. Подтвердите email

#### 2. Создание Worker

```bash
cd cloudflare/worker

# Логин
wrangler login

# Проверка аккаунта
wrangler whoami

# Деплой
wrangler deploy
```

#### 3. Настройка KV Storage

```bash
# Создайте KV namespace
wrangler kv:namespace create PUSH_STORE

# Скопируйте ID из вывода
# Добавьте в wrangler.toml:
# [[kv_namespaces]]
# binding = "PUSH_STORE"
# id = "your-namespace-id"
```

#### 4. Проверка работы

```bash
# Проверка health endpoint
curl https://secure-telegram-worker.<subdomain>.workers.dev/health

# Ответ: OK
```

### APK Установка

#### Через USB

```bash
# Включите Developer Options на телефоне
# Settings → About Phone → Tap "Build Number" 7 times

# Включите USB Debugging
# Settings → Developer Options → USB Debugging

# Подключите телефон
adb devices

# Установите
adb install mobile/android/app/build/outputs/apk/release/app-release.apk
```

#### Через WiFi

```bash
# Сначала подключите по USB
adb tcpip 5555
adb connect <PHONE_IP>:5555

# Теперь можно по WiFi
adb install mobile/android/app/build/outputs/apk/release/app-release.apk
```

---

## 📊 Мониторинг

### Cloudflare Dashboard

1. Workers & Pages → secure-telegram-worker
2. Analytics → Просмотр статистики
3. Logs → Просмотр логов

### Логи в реальном времени

```bash
wrangler tail --name secure-telegram-worker
```

### Логи на телефоне

```bash
adb logcat | grep -i "secure-telegram"
```

---

## 🛡️ Безопасность

### Смените секреты

```bash
# Cloudflare Dashboard → Workers → Variables
# Измените:
JWT_SECRET=новый-секретный-ключ
```

### Rate Limiting

В `wrangler.toml`:
```toml
[vars]
RATE_LIMIT_REQUESTS = 100
RATE_LIMIT_WINDOW = 60
```

---

## ❓ Troubleshooting

### Ошибка: "Worker not found"

```bash
# Проверьте имя worker в wrangler.toml
name = "secure-telegram-worker"

# Пересоздайте
wrangler delete
wrangler deploy
```

### Ошибка: "APK not installed"

```bash
# Проверьте подключение
adb devices

# Если телефон не виден:
# 1. Включите USB Debugging
# 2. Разрешите отладку на телефоне
# 3. Переподключите кабель
```

### Ошибка: "Cannot connect to server"

1. Проверьте URL сервера
2. Проверьте, что Worker задеплоен
3. Проверьте логи: `wrangler tail`

---

## 📞 Поддержка

- GitHub Issues: https://github.com/zametkikostik/secure-telegram-client/issues
- Документация: https://github.com/zametkikostik/secure-telegram-client/tree/main/docs

---

**Secure Telegram Team © 2026**
