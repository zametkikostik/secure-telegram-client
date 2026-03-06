# 📝 ИНСТРУКЦИЯ ПО КОММИТУ И ОТПРАВКЕ НА GITHUB

## 🚀 Быстрый старт

### 1. Настройка Git (если не настроена)

```bash
# Установите ваше имя и email
git config --global user.name "zametkikostik"
git config --global user.email "zametkikostik@gmail.com"

# Настройка SSH ключа (если не настроен)
ssh-keygen -t ed25519 -C "zametkikostik@gmail.com"
cat ~/.ssh/id_ed25519.pub
# Скопируйте вывод и добавьте в GitHub: Settings → SSH and GPG keys
```

### 2. Проверка репозитория

```bash
cd /home/kostik/secure-telegram-client

# Проверка статуса
git status

# Проверка remote
git remote -v
# Должно быть: origin https://github.com/zametkikostik/secure-telegram-client.git
```

### 3. Добавление файлов

```bash
# Добавить все файлы проекта
git add .

# ИЛИ добавить только конкретные файлы
git add README.md
git add messenger/
git add server/
git add frontend/
git add mobile/
```

### 4. Создание коммита

```bash
git commit -m "feat: полный аудит и исправление критических ошибок

- Исправлен Dockerfile для server/
- Реализован get_current_user() API
- Добавлена WebSocket авторизация
- Добавлены все файлы Android проекта
- Исправлена паника в group.rs
- Обновлён P2PEscrow.sol
- Добавлены .env.local.example и обновлён .gitignore
- Обновлена документация

Готовность проекта: 88%"
```

### 5. Отправка на GitHub

```bash
# Отправить основную ветку
git push -u origin main

# Если ветка называется master
git push -u origin master
```

---

## 📋 ПОДРОБНАЯ ИНСТРУКЦИЯ

### Создание репозитория на GitHub

1. Перейдите на https://github.com/new
2. Введите имя: `secure-telegram-client`
3. Описание: "Децентрализованный мессенджер с AI-переводом и Web3"
4. Public или Private (на ваше усмотрение)
5. **НЕ** ставьте галочку "Initialize with README"
6. Нажмите "Create repository"

### Привязка к удалённому репозиторию

```bash
cd /home/kostik/secure-telegram-client

# Если remote ещё не настроен
git remote add origin https://github.com/zametkikostik/secure-telegram-client.git

# Или через SSH (рекомендуется)
git remote add origin git@github.com:zametkikostik/secure-telegram-client.git

# Проверка
git remote -v
```

### Первый коммит

```bash
# Добавить все файлы
git add .

# Проверить что будет закоммичено
git status

# Создать коммит
git commit -m "Initial commit: Liberty Reach v1.0.0

Полная реализация мессенджера:
- Backend (Rust + Axum)
- Frontend (React + TypeScript)
- Mobile (React Native Android)
- Desktop (Tauri)
- Smart Contracts (Solidity)
- AI интеграции (Qwen)
- Web3 (0x, ABCEX, Bitget)
- P2P (libp2p)
- Self-hosting (Docker)

Готовность: 88%"

# Отправить на GitHub
git push -u origin main
```

---

## 🔐 БЕЗОПАСНОСТЬ

### Что НЕ коммитить:

```bash
# Никогда не добавляйте в git:
.env.local              # Содержит секреты
google-services.json    # Firebase ключи
*.keystore, *.jks       # Ключи подписи
uploads/                # Файлы пользователей
*.db, *.sqlite          # Базы данных
```

### Проверка перед коммитом:

```bash
# Проверить что будет закоммичено
git status

# Посмотреть изменения
git diff --cached

# Если есть лишние файлы - убрать
git reset HEAD <файл>
rm <файл>  # если нужно удалить
```

---

## 🛠️ ПОЛЕЗНЫЕ КОМАНДЫ

### Просмотр истории

```bash
# Последние коммиты
git log --oneline -10

# Красивый лог
git log --graph --oneline --all
```

### Ветки

```bash
# Создать новую ветку
git checkout -b feature/new-feature

# Переключиться на ветку
git checkout main

# Слить ветки
git merge feature/new-feature
```

### Отмена изменений

```bash
# Отменить изменения в файле
git checkout -- <файл>

# Отменить коммит (сохранить изменения)
git reset HEAD~1

# Отменить всё
git reset --hard HEAD
```

---

## 📊 ЧЕКЛИСТ ПЕРЕД ОТПРАВКОЙ

- [ ] Проверить `.gitignore` (нет ли секретов)
- [ ] Проверить `.env.local` (не закоммичен)
- [ ] Проверить `Cargo.toml` и `package.json` (актуальные версии)
- [ ] Запустить тесты (если есть)
- [ ] Проверить сборку (`cargo check`, `npm run build`)
- [ ] Написать понятное сообщение коммита

---

## 🎯 РЕКОМЕНДАЦИИ ПО ВЕТКАМ

### Основная структура:

```
main          # Стабильная версия (production)
develop       # Ветка разработки
feature/*     # Новые функции
bugfix/*      # Исправления ошибок
release/*     # Подготовка релиза
```

### Пример workflow:

```bash
# Создать ветку для новой функции
git checkout develop
git checkout -b feature/websocket-auth

# Работа над функцией...
git add .
git commit -m "feat: добавить WebSocket авторизацию"

# Отправить ветку
git push -u origin feature/websocket-auth

# Создать Pull Request на GitHub
# После мержа - удалить ветку
```

---

## 📬 ПОМОЩЬ

### Проблемы и решения:

**Проблема:** `git push` требует пароль  
**Решение:** Настройте SSH ключ или используйте Personal Access Token

**Проблема:** Конфликты слияния  
**Решение:** `git merge --abort` и решите конфликты вручную

**Проблема:** Случайно закоммитил секреты  
**Решение:** 
```bash
git reset HEAD~1
git rm --cached <файл>
echo "<файл>" >> .gitignore
git commit -m "Remove secrets"
```

---

## 🔗 ССЫЛКИ

- GitHub: https://github.com/zametkikostik/secure-telegram-client
- Документация: https://docs.github.com/en/get-started
- Git Book: https://git-scm.com/book/ru/v2

---

**Создано:** 6 марта 2026 г.  
**Для:** Liberty Reach Project
