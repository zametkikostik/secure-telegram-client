# 🔐 Инструкция по анонимизации GitHub аккаунта

## ⚠️ ВАЖНО

Эта инструкция поможет защитить вашу приватность при публикации проекта.

---

## 📋 Чеклист анонимизации

### 1️⃣ Профиль аккаунта

**URL**: https://github.com/settings/profile

**Измените:**

| Поле | Действие |
|------|----------|
| **Name** | Замените реальное имя на `Secure Telegram Team` |
| **Company** | Удалите или укажите `Educational Project` |
| **Location** | Удалите (не указывайте город/страну) |
| **Email** | Удалите публичный email |

---

### 2️⃣ Настройки репозитория

**URL**: https://github.com/secure-telegram-team/secure-telegram-client/settings

**Измените:**

1. **Repository name** (опционально):
   - Было: `secure-telegram-client`
   - Стало: `secure-messenger` (более анонимно)

2. **Description**:
   ```
   Educational project. No personal data.
   ```

3. **Website** (опционально):
   ```
   https://secure-telegram-team.github.io
   ```

---

### 3️⃣ Трансфер репозитория (опционально)

Если хотите **полностью отделить** от личного аккаунта:

#### Шаг 1: Создайте новый аккаунт

1. Выйдите из текущего аккаунта
2. Создайте новый аккаунт на **другой email** (ProtonMail/Tutanota)
3. Username: `secure-telegram-team`
4. Name: `Secure Telegram Team`

#### Шаг 2: Трансфер репозитория

1. В настройках репозитория: **Settings → Transfer**
2. Укажите нового владельца: `secure-telegram-team`
3. Подтвердите трансфер

#### Шаг 3: Обновите remote

```bash
cd secure-telegram-client
git remote set-url origin git@github.com:secure-telegram-team/secure-telegram-client.git
git push --force
```

---

### 4️⃣ Очистка истории коммитов (если нужно)

Если в истории есть коммиты с реальным email:

```bash
# Перепишите историю с новым email
git filter-branch --env-filter '
OLD_EMAIL="your-real-email@example.com"
CORRECT_NAME="Secure Telegram Team"
CORRECT_EMAIL="secure-tg@users.noreply.github.com"

if [ "$GIT_COMMITTER_EMAIL" = "$OLD_EMAIL" ]
then
    export GIT_COMMITTER_NAME="$CORRECT_NAME"
    export GIT_COMMITTER_EMAIL="$CORRECT_EMAIL"
fi

if [ "$GIT_AUTHOR_EMAIL" = "$OLD_EMAIL" ]
then
    export GIT_AUTHOR_NAME="$CORRECT_NAME"
    export GIT_AUTHOR_EMAIL="$CORRECT_EMAIL"
fi
' --tag-name-filter cat -- --branches --tags

# Отправьте изменения
git push --force --tags origin 'refs/heads/*'
```

⚠️ **Внимание**: Это изменит все хеши коммитов!

---

### 5️⃣ Скрытие email в коммитах

**URL**: https://github.com/settings/emails

**Установите:**
- ✅ **Keep my email address private**
- ✅ **Block command line pushes that expose my email**

---

### 6️⃣ Двухфакторная аутентификация

**URL**: https://github.com/settings/security

**Включите:**
- ✅ **Two-factor authentication**
- ✅ **Passkeys** (рекомендуется)

---

## ✅ Проверка результатов

После всех изменений проверьте:

1. **Профиль**: https://github.com/secure-telegram-team
   - ❌ Нет реального имени
   - ❌ Нет локации
   - ❌ Нет email

2. **Репозиторий**: https://github.com/secure-telegram-team/secure-telegram-client
   - ✅ В описании нет личных данных
   - ✅ В коде нет личных данных

3. **Коммиты**: https://github.com/secure-telegram-team/secure-telegram-client/commits/master
   - ✅ Author: `Secure Telegram Team`
   - ✅ Email: `secure-tg@users.noreply.github.com`

---

## 🎯 Минимальные действия (если не хотите создавать новый аккаунт)

**Сделайте хотя бы это:**

1. ✅ Измените **Name** в профиле на `Secure Telegram Team`
2. ✅ Удалите **Location** и **Email** из профиля
3. ✅ Включите **Keep my email address private**
4. ✅ Включите **Two-factor authentication**

**Это уже достаточно для базовой защиты!**

---

## 📚 Дополнительные ресурсы

- [GitHub Privacy Settings](https://docs.github.com/en/account-and-profile/setting-up-and-managing-your-personal-account-on-github/managing-your-personal-account)
- [Keeping your email address private](https://docs.github.com/en/account-and-profile/setting-up-and-managing-your-personal-account-on-github/managing-email-preferences/keeping-your-email-address-private)
- [Transferring a repository](https://docs.github.com/en/repositories/creating-and-managing-repositories/transferring-a-repository)

---

**⚠️ ЭТО НЕ ЗАМЕНЯЕТ ЮРИДИЧЕСКУЮ КОНСУЛЬТАЦИЮ!**

**Проконсультируйтесь с юристом перед публикацией!**
