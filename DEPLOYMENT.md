# 🚀 Развёртывание Secure Telegram

## 24/7 в облаке через Cloudflare

### 1. Backend на Cloudflare Workers

```bash
cd cloudflare/worker

# Установите wrangler
npm install -g wrangler

# Логин в Cloudflare
wrangler login

# Настройка wrangler.toml
cp wrangler.toml.example wrangler.toml
# Отредактируйте account_id и zone_id

# Деплой
wrangler deploy
```

**URL после деплоя:** `https://secure-telegram-worker.<your-subdomain>.workers.dev`

### 2. Настройка переменных окружения

```bash
# Cloudflare Dashboard → Workers → secure-telegram → Settings → Variables
DATABASE_URL=your-database-url
JWT_SECRET=your-jwt-secret
UPLOADS_DIR=/tmp/uploads
```

### 3. Подключение домена (опционально)

```bash
# В Cloudflare Dashboard:
# Workers → secure-telegram → Triggers → Add Custom Domain
secure-messenger.yourdomain.com
```

---

## 📱 Установка APK на телефон

### Способ 1: Через ADB (USB)

```bash
# Включите отладку по USB на телефоне
# Подключите телефон к компьютеру

# Проверка подключения
adb devices

# Установка APK
adb install mobile/android/app/build/outputs/apk/release/app-release.apk

# Запуск приложения
adb shell am start -n io.libertyreach.messenger/.MainActivity
```

### Способ 2: Через WiFi (Airdroid/adb wireless)

```bash
# Подключите телефон по USB сначала
adb tcpip 5555

# Отключите USB
adb connect <PHONE_IP>:5555

# Установка по WiFi
adb install mobile/android/app/build/outputs/apk/release/app-release.apk
```

### Способ 3: Прямая установка

1. Скопируйте APK на телефон
2. Откройте файл APK на телефоне
3. Разрешите установку из неизвестных источников
4. Установите приложение

---

## 🔧 Настройка приложения

### 1. Откройте приложение

### 2. Введите данные сервера

```
Server URL: https://secure-telegram-worker.<your-subdomain>.workers.dev
```

### 3. Зарегистрируйтесь или войдите

---

## ✅ Проверка работы

### 1. Проверка Backend

```bash
curl https://secure-telegram-worker.<your-subdomain>.workers.dev/health
# Ответ: OK
```

### 2. Проверка WebSocket

```javascript
const ws = new WebSocket('wss://secure-telegram-worker.<your-subdomain>.workers.dev/ws');
ws.onopen = () => console.log('Connected!');
```

### 3. Проверка APK

- Откройте приложение на телефоне
- Попробуйте зарегистрироваться
- Отправьте тестовое сообщение

---

## 🔄 CI/CD для автоматического деплоя

### GitHub Actions

```yaml
# .github/workflows/deploy.yml
name: Deploy to Cloudflare

on:
  push:
    branches: [main]

jobs:
  deploy:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      
      - name: Install Wrangler
        run: npm install -g wrangler
      
      - name: Deploy to Cloudflare
        run: wrangler deploy
        working-directory: ./cloudflare/worker
        env:
          CLOUDFLARE_API_TOKEN: ${{ secrets.CLOUDFLARE_API_TOKEN }}
          CLOUDFLARE_ACCOUNT_ID: ${{ secrets.CLOUDFLARE_ACCOUNT_ID }}
```

---

## 📊 Мониторинг

### Cloudflare Analytics

- Cloudflare Dashboard → Workers → Analytics
- Запросы, ошибки, задержки

### Логи

```bash
wrangler tail --name secure-telegram-worker
```

---

## 🛡️ Безопасность

### Rate Limiting

```toml
# wrangler.toml
[vars]
RATE_LIMIT_REQUESTS = 100
RATE_LIMIT_WINDOW = 60
```

### CORS

```typescript
// worker/src/worker.ts
const corsHeaders = {
  'Access-Control-Allow-Origin': '*',
  'Access-Control-Allow-Methods': 'GET, POST, PUT, DELETE, OPTIONS',
  'Access-Control-Allow-Headers': 'Content-Type, Authorization',
};
```

---

## 📞 Поддержка

При проблемах:
1. Проверьте логи Cloudflare
2. Проверьте логи приложения (adb logcat)
3. Проверьте переменные окружения

---

**Secure Telegram Team © 2026**
