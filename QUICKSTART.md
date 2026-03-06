# Liberty Reach — Быстрый старт

## 🚀 5 минут до первого запуска

### Вариант 1: Docker (самый быстрый)

```bash
cd self-hosting
./install.sh
```

**Готово!** Откройте http://localhost:8008

---

### Вариант 2: Ручная сборка

#### 1. Backend

```bash
cd server
cp .env.example .env
cargo run --release
```

#### 2. Frontend

```bash
cd frontend
npm install
npm run dev
```

Откройте http://localhost:3000

---

### Вариант 3: Android APK

```bash
cd mobile
npm install
npm run build:apk
```

Установите `app-debug.apk` на устройство.

---

## 📋 Тестирование

### 1. Регистрация

1. Откройте http://localhost:3000/register
2. Введите имя пользователя и пароль
3. Нажмите "Зарегистрироваться"

### 2. Создание чата

1. Нажмите "+" в списке чатов
2. Выберите тип чата
3. Пригласите участников

### 3. Отправка сообщения

1. Откройте чат
2. Введите сообщение
3. Нажмите "➤"

### 4. AI Перевод

1. Откройте настройки чата
2. Включите "Авто-перевод"
3. Выберите язык

---

## 🎯 Следующие шаги

- [Полная документация](docs/)
- [API документация](docs/API.md)
- [Self-hosting](docs/SELF_HOSTING.md)
- [Сборка APK](APK_BUILD.md)

---

## ❓ Проблемы?

```bash
# Проверьте логи сервера
docker-compose logs -f

# Или
cd server
RUST_LOG=debug cargo run
```

**Email:** support@libertyreach.io
