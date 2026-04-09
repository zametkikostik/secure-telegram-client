# 🔧 Инструкция по деплою Cloudflare Worker

**Дата:** 7 апреля 2026 г.
**Статус:** ⚠️ wrangler upload зависает (проблемы с сетью)

---

## Проблема

`wrangler deploy` зависает на этапе "Total Upload" - это проблема с сетью или Cloudflare API.

**Решение:** Ручной деплой через Cloudflare Dashboard (займёт 2 минуты)

---

## 📋 Инструкция: Ручной деплой через Dashboard

### Шаг 1: Откройте Cloudflare Dashboard

```
https://dash.cloudflare.com/
```

### Шаг 2: Перейдите к Worker

```
Workers & Pages → secure-messenger-push
```

Или напрямую:
```
https://dash.cloudflare.com/zametkikostik@gmail.com/workers/services/view/secure-messenger-push/production
```

### Шаг 3: Edit Code

1. Нажмите кнопку **"Edit Code"** (в правом верхнем углу)
2. Откроется редактор кода

### Шаг 4: Вставьте код Worker

1. Откройте файл:
   ```
   /home/kostik/secure-messenger/secure-telegram-client/cloudflare/worker/src/worker.js
   ```

2. Скопируйте **ВСЁ** содержимое файла (Ctrl+A, Ctrl+C)

3. Вставьте в онлайн-редактор Cloudflare (Ctrl+V)

4. Нажмите **"Save"** (Ctrl+S)

### Шаг 5: Настройте Bindings (если ещё не настроены)

1. Перейдите в: **Settings → Variables → Bindings**

2. Нажмите **"Add Binding"** → **KV Namespace**

3. Добавьте:
   - **Variable name:** `PUSH_STORE`
   - **KV Namespace:** выберите `PUSH_STORE` (id: `9c0651f314b64b49bc215fc5f56163f4`)

4. Нажмите **"Save"**

5. Перейдите в: **Settings → Variables**

6. Добавьте Environment Variables:
   - `ENVIRONMENT` = `production`
   - `LOG_LEVEL` = `info`
   - `MAX_MESSAGE_SIZE` = `1048576`
   - `MESSAGE_TTL` = `604800000`

7. Нажмите **"Save and Deploy"**

---

## ✅ Проверка деплоя

После деплоя протестируйте:

```bash
# Health check
curl https://secure-messenger-push.kostik.workers.dev/health

# Register device
curl -X POST https://secure-messenger-push.kostik.workers.dev/p2p/register \
  -H "Content-Type: application/json" \
  -d '{"deviceId":"test-001","userId":"user-123","publicKey":"test-key"}'

# Send message
curl -X POST https://secure-messenger-push.kostik.workers.dev/p2p/send \
  -H "Content-Type: application/json" \
  -d '{"fromDeviceId":"test-001","toDeviceId":"test-002","encryptedMessage":"encrypted-data","messageType":"text"}'

# Get messages
curl "https://secure-messenger-push.kostik.workers.dev/p2p/messages?deviceId=test-002"
```

---

## 🔍 Мониторинг

```bash
# Логи в реальном времени
wrangler tail

# История деплоев
wrangler deployments list

# Метрики
wrangler metrics
```

---

## 📁 Файлы

- **Worker код:** `/home/kostik/secure-messenger/secure-telegram-client/cloudflare/worker/src/worker.js`
- **Конфиг:** `/home/kostik/secure-messenger/secure-telegram-client/cloudflare/worker/wrangler.toml`
- **KV Namespace ID:** `9c0651f314b64b49bc215fc5f56163f4`
- **Account ID:** `9d3f70325c3f26a70c09c2d13b981f3c`

---

## 🚀 Альтернатива: Wrangler CLI (если заработает)

Если сеть станет стабильнее:

```bash
cd /home/kostik/secure-messenger/secure-telegram-client/cloudflare/worker
wrangler deploy
```

Или используйте deploy скрипт:

```bash
./deploy.sh
```

---

## ⚠️ Важно

- Worker уже был задеплоен 8 марта 2026 (версия: `81cbfdd1-7551-4f2d-8a7d-35b130d5c458`)
- Текущий код может отличаться от того, что в production
- После ручного деплоя проверьте что все endpoints работают

---

> **Примечание:** wrangler v4.70.0 имеет проблемы с upload в вашей сети. Можно попробовать обновить до v4.81.0 или использовать Dashboard.
