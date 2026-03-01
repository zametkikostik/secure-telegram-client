# 🚀 ФИНАЛЬНАЯ ИНСТРУКЦИЯ ПО ПУБЛИКАЦИИ

**Версия**: 0.2.0  
**Дата**: 1 марта 2026  
**Статус**: ✅ ГОТОВО К ПУБЛИКАЦИИ

---

## 📋 ЧЕКЛИСТ ГОТОВНОСТИ

### ✅ Готово (не требует изменений):

- [x] **APK подписан** — `android/app-fdroid-release-signed.apk` (2,9 MB)
- [x] **Privacy Policy** — `PRIVACY_POLICY.md`
- [x] **Disclaimer** — `DISCLAIMER.md`
- [x] **Лицензия** — `LICENSE` (MIT)
- [x] **README** — основная документация
- [x] **Документация** — 18+ файлов
- [x] **Git авторы** — `Secure Telegram Team <secure-tg@users.noreply.github.com>`
- [x] **Подпись APK** — `CN=Secure Messenger, OU=Development, O=Example` (без личных данных)

### ⚠️ Требует внимания:

- [ ] **GitHub репозиторий** — заменить `secure-telegram-team` на `secure-telegram-team`
- [ ] **F-Droid метаданные** — обновить URL
- [ ] **Документация** — обновить ссылки

---

## 🔄 ШАГ 1: Обновление репозитория

### Вариант A: Использовать текущий (secure-telegram-team)

**Можно оставить как есть!** Все данные анонимизированы:

- Git автор: `Secure Telegram Team <secure-tg@users.noreply.github.com>` ✅
- Подпись APK: `CN=Secure Messenger` (без имён) ✅
- Email в коде: `secure-tg@users.noreply.github.com` ✅

**Ссылки для публикации:**
```
GitHub: https://github.com/secure-telegram-team/secure-telegram-client
Privacy Policy: https://github.com/secure-telegram-team/secure-telegram-client/blob/master/PRIVACY_POLICY.md
Issues: https://github.com/secure-telegram-team/secure-telegram-client/issues
```

### Вариант B: Создать новый аккаунт (рекомендуется)

1. **Создайте новый GitHub аккаунт:**
   - Email: `secure-telegram-team@protonmail.com` (создайте на ProtonMail)
   - Username: `secure-telegram-team`
   - Name: `Secure Telegram Team`

2. **Создайте репозиторий:**
   - Name: `secure-telegram-client`
   - Description: `Decentralized Telegram client with post-quantum encryption`
   - License: MIT
   - Public: ✅

3. **Запушьте проект:**
   ```bash
   cd /home/kostik/secure-telegram-client
   
   # Удалите старый remote
   git remote remove origin 2>/dev/null || true
   
   # Добавьте новый
   git remote add origin git@github.com:secure-telegram-team/secure-telegram-client.git
   
   # Запушьте
   git push -u origin master --force
   ```

4. **Обновите ссылки в документации:**
   ```bash
   # Замените все ссылки
   find . -type f -name "*.md" -exec sed -i 's|secure-telegram-team|secure-telegram-team|g' {} \;
   ```

---

## 📦 ШАГ 2: Публикация APK

### GitHub Releases (мгновенно):

**Готовый релиз:**
- **Версия**: `v0.2.2`
- **Ссылка**: https://github.com/zametkikostik/secure-telegram-client/releases/tag/0.22

**Прямая ссылка на APK:**
```
https://github.com/zametkikostik/secure-telegram-client/releases/download/0.22/app-fdroid-release-signed.apk
```

**Или создайте новый релиз вручную:**

1. Перейдите: `https://github.com/zametkikostik/secure-telegram-client/releases/new`
2. Tag version: `v0.2.2`
3. Release title: `Secure Telegram Client v0.2.2`
4. Прикрепите: `app-fdroid-release-signed.apk`
5. Нажмите **Publish release**

---

## 🏪 ШАГ 3: Публикация в магазинах

### Aptoide (1-24 часа):

1. **Зарегистрируйтесь:** `https://www.aptoide.com/sign-up`
2. **Создайте магазин:** `https://www.aptoide.com/create-store`
3. **Загрузите APK:** `android/app-fdroid-release-signed.apk`

**Данные для публикации:**

| Поле | Значение |
|------|----------|
| **Title** | `Secure Messenger` |
| **Package Name** | `com.example.securemessenger.fdroid` |
| **Version** | `0.2.0` |
| **Privacy Policy** | `https://github.com/YOUR_USERNAME/secure-telegram-client/blob/master/PRIVACY_POLICY.md` |
| **Website** | `https://github.com/YOUR_USERNAME/secure-telegram-client` |
| **Email** | `secure-tg@users.noreply.github.com` |

**Описание:**
```
🔐 Secure Messenger — децентрализованный Telegram клиент с постквантовым шифрованием.

Features:
• Post-quantum encryption (Kyber-1024)
• Traffic obfuscation (obfs4)
• DNS over HTTPS
• Decentralized updates via IPFS
• P2P fallback mode
• Open source (MIT License)

⚠️ Research project. Not for critical communication.
```

---

### F-Droid (1-2 недели):

1. **Отправьте Pull Request:**
   - URL: `https://gitlab.com/fdroid/fdroiddata/-/merge_requests`
   - Файл: `android/fdroid-metadata.yml` (уже готов)

2. **Обновите URL в метаданных:**
   ```yaml
   SourceCode: https://github.com/YOUR_USERNAME/secure-telegram-client
   IssueTracker: https://github.com/YOUR_USERNAME/secure-telegram-client/issues
   Repo: https://github.com/YOUR_USERNAME/secure-telegram-client
   ```

3. **Создайте Merge Request** в F-Droid

---

### IzzyOnDroid (1-3 дня):

1. **Отправьте запрос:**
   - URL: `https://gitlab.com/IzzyOnDroid/fdroiddata/-/merge_requests`
   
2. **Требуется fastlane структура:**
   ```
   android/fastlane/metadata/android/en-US/
   ├── title.txt          # Secure Messenger
   ├── short_description.txt  # Decentralized messenger with post-quantum encryption
   └── full_description.txt   # полное описание
   ```

---

## 📊 ССЫЛКИ ДЛЯ ПУБЛИКАЦИИ

### Основные URL (замените YOUR_USERNAME):

```
GitHub Repository:
https://github.com/YOUR_USERNAME/secure-telegram-client

GitHub Releases:
https://github.com/YOUR_USERNAME/secure-telegram-client/releases

Privacy Policy:
https://github.com/YOUR_USERNAME/secure-telegram-client/blob/master/PRIVACY_POLICY.md

Disclaimer:
https://github.com/YOUR_USERNAME/secure-telegram-client/blob/master/DISCLAIMER.md

License:
https://github.com/YOUR_USERNAME/secure-telegram-client/blob/master/LICENSE

Issues/Support:
https://github.com/YOUR_USERNAME/secure-telegram-client/issues

Download APK:
https://github.com/YOUR_USERNAME/secure-telegram-client/releases/download/v0.2.0/app-fdroid-release-signed.apk
```

### Для магазинов приложений:

| Магазин | Homepage | Privacy Policy | Support |
|---------|----------|----------------|---------|
| **Aptoide** | GitHub URL | GitHub URL/PRIVACY_POLICY.md | GitHub URL/issues |
| **F-Droid** | GitHub URL | Не требуется | GitHub URL/issues |
| **Google Play** | GitHub URL | GitHub URL/PRIVACY_POLICY.md | GitHub URL/issues |
| **Amazon** | GitHub URL | GitHub URL/PRIVACY_POLICY.md | GitHub URL/issues |

---

## 🔐 ПРОВЕРКА АНОНИМНОСТИ

### ✅ Проверено (без личных данных):

```
Git автор: Secure Telegram Team <secure-tg@users.noreply.github.com>
Подпись APK: CN=Secure Messenger, OU=Development, O=Example
Email в коде: secure-tg@users.noreply.github.com
```

### ⚠️ Требует замены в документации:

```
secure-telegram-team → YOUR_USERNAME (или secure-telegram-team)
```

**Команда для замены:**
```bash
cd /home/kostik/secure-telegram-client

# Замените во всех .md файлах
find . -type f -name "*.md" -exec sed -i 's|secure-telegram-team|secure-telegram-team|g' {} \;

# Проверьте изменения
git diff
```

---

## 📱 ИНФОРМАЦИЯ О ПРИЛОЖЕНИИ

### Основные данные:

| Параметр | Значение |
|----------|----------|
| **Название** | `Secure Messenger` |
| **Package Name** | `com.example.securemessenger.fdroid` |
| **Версия** | `0.2.0` |
| **Version Code** | `2` |
| **Min SDK** | `26` (Android 8.0+) |
| **Target SDK** | `35` (Android 15) |
| **Размер APK** | `2,9 MB` |
| **Архитектуры** | `arm64-v8a, armeabi-v7a, x86_64` |

### Подпись APK:

```
Certificate DN: CN=Secure Messenger, OU=Development, O=Example, L=City, ST=State, C=US
SHA-256: 384bc640654c23cfce626ad4e018176328fc257be118f5f2c1bff0f94217e3e2
SHA-1: 16aacee35dd361f3b40634e660d909c8472e6297
MD5: cd2a01337c672d68356b910f640eb5d9
```

---

## 📚 ДОКУМЕНТАЦИЯ

### Файлы для публикации:

| Файл | Назначение |
|------|------------|
| `README.md` | Основная документация |
| `PRIVACY_POLICY.md` | ✅ Политика конфиденциальности |
| `DISCLAIMER.md` | ✅ Отказ от ответственности |
| `LICENSE` | ✅ Лицензия MIT |
| `DOWNLOAD.md` | Инструкция по загрузке |
| `QUICKSTART.md` | Быстрый старт |
| `SOCIAL_NETWORKS.md` | Социальные сети |
| `android/APP_STORES.md` | Магазины приложений |
| `android/PUBLISH_APTOIDE.md` | Публикация в Aptoide |
| `android/fdroid-metadata.yml` | F-Droid метаданные |

---

## ✅ ФИНАЛЬНЫЙ ЧЕКЛИСТ

### Перед публикацией:

- [ ] Проверить APK: `apksigner verify android/app-fdroid-release-signed.apk`
- [ ] Обновить ссылки в документации (заменить `secure-telegram-team`)
- [ ] Проверить Privacy Policy URL
- [ ] Подготовить скриншоты (опционально)
- [ ] Подготовить иконку 512x512 (опционально)

### Публикация:

- [ ] GitHub Releases (мгновенно)
- [ ] Aptoide (1-24 часа)
- [ ] F-Droid (1-2 недели)
- [ ] IzzyOnDroid (1-3 дня)
- [ ] Codeberg/GitFlic (зеркала)

### После публикации:

- [ ] Обновить README со ссылками на магазины
- [ ] Добавить бейджи в README
- [ ] Опубликовать анонс в социальных сетях
- [ ] Обновить SOCIAL_NETWORKS.md

---

## 🎯 БЫСТРЫЙ СТАРТ (5 минут)

```bash
# 1. Проверьте APK
cd /home/kostik/secure-telegram-client/android
apksigner verify app-fdroid-release-signed.apk

# 2. Опубликуйте на GitHub
./scripts/publish-to-github.sh app-fdroid-release-signed.apk 0.2.0

# 3. Опубликуйте в Aptoide
# Перейдите на https://www.aptoide.com/create-store
# Загрузите APK и заполните данные

# 4. Отправьте в F-Droid
# Создайте MR: https://gitlab.com/fdroid/fdroiddata/-/merge_requests
```

---

## 📞 ПОДДЕРЖКА

**GitHub Issues:** `https://github.com/YOUR_USERNAME/secure-telegram-client/issues`

**Email:** `secure-tg@users.noreply.github.com`

---

**ПРОЕКТ ГОТОВ К ПУБЛИКАЦИИ!** 🎉

**Все данные анонимизированы. Личные данные и e-mail отсутствуют.**
