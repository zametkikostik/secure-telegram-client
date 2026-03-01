# 📥 Скачать Secure Telegram Client

**Версия**: 0.2.2  
**Дата**: 1 марта 2026  
**Релиз**: https://github.com/secure-telegram-team/secure-telegram-client/releases/tag/0.22

---

## 🚀 Быстрая загрузка

### Android APK

| Файл | Размер | Статус | Ссылка |
|------|--------|--------|--------|
| **GitHub Releases** | 2,9 MB | ✅ Готов | [Скачать](https://github.com/secure-telegram-team/secure-telegram-client/releases/download/0.22/app-fdroid-release-signed.apk) |

### Desktop (Linux)

| Файл | Размер | Статус | Ссылка |
|------|--------|--------|--------|
| **secure-tg** | 8,3 MB | ✅ Готов | `target/release/secure-tg` |

---

## 📱 Android

### Способ 1: Скачать готовый APK

```bash
# Перейдите в директорию проекта
cd /home/kostik/secure-telegram-client/android

# APK файл находится здесь:
ls -lh app-fdroid-release-signed.apk
```

**Установка на устройство:**

```bash
# Через ADB
adb install app-fdroid-release-signed.apk

# Или скопируйте файл на устройство и откройте вручную
```

### Способ 2: Собрать самостоятельно

```bash
cd /home/kostik/secure-telegram-client/android

# Debug версия
./gradlew assembleFdroidDebug

# Release версия
./gradlew assembleFdroidRelease

# APK появится в:
# app/build/outputs/apk/fdroid/debug/app-fdroid-debug.apk
# app/build/outputs/apk/fdroid/release/app-fdroid-release-signed.apk
```

### Требования Android

- **Минимальная версия**: Android 8.0 (API 26)
- **Целевая версия**: Android 15 (API 35)
- **Архитектуры**: arm64-v8a, armeabi-v7a, x86_64

---

## 🖥️ Desktop (Linux)

### Способ 1: Скачать готовый бинарник

```bash
cd /home/kostik/secure-telegram-client

# Запуск
./target/release/secure-tg --help

# Инициализация конфигурации
./target/release/secure-tg --init-config
```

### Способ 2: Собрать самостоятельно

```bash
# Установка зависимостей (Ubuntu/Debian)
sudo apt-get update
sudo apt-get install -y cmake clang libssl-dev pkg-config git libsqlite3-dev

# Сборка
cd /home/kostik/secure-telegram-client
cargo build --release

# Запуск
./target/release/secure-tg
```

---

## 📚 Документация

| Файл | Описание |
|------|----------|
| [README.md](README.md) | Основная документация |
| [QUICKSTART.md](QUICKSTART.md) | Быстрый старт |
| [ARCHITECTURE.md](ARCHITECTURE.md) | Архитектура проекта |
| [STATUS_100_PERCENT.md](STATUS_100_PERCENT.md) | Статус реализации |
| [DISCLAIMER.md](DISCLAIMER.md) | Предупреждение |
| [HOW_TO_RUN_CI.md](HOW_TO_RUN_CI.md) | Использование CI/CD |

---

## 🔐 Проверка целостности

### APK подпись

```bash
# Проверка подписи APK
apksigner verify --verbose android/app-fdroid-release-signed.apk
```

### Хеш-суммы

```bash
# Вычисление хешей
sha256sum android/app-fdroid-release-signed.apk
sha256sum target/release/secure-tg
```

---

## 🌐 Git репозиторий

### Клонирование

```bash
# HTTPS
git clone https://github.com/secure-telegram-team/secure-telegram-client.git

# SSH
git clone git@github.com:secure-telegram-team/secure-telegram-client.git
```

### Текущая версия

```bash
cd secure-telegram-client
git log -1 --oneline
# Коммит: [hash] Версия 0.2.0
```

---

## 📦 F-Droid

Проект доступен в каталоге F-Droid.

**Package ID**: `com.example.securemessenger.fdroid`

**Метаданные**: `android/fdroid-metadata.yml`

---

## ⚠️ Важно

1. **Исследовательский проект** — не используйте для критически важной коммуникации
2. **Debug APK** — подписан debug ключом, не для production
3. **Release APK** — подписан release ключом, готов к использованию
4. **Desktop версия** — требует настройки конфигурации перед запуском

---

## 📞 Поддержка

- **GitHub Issues**: https://github.com/secure-telegram-team/secure-telegram-client/issues
- **Лицензия**: MIT

---

**Проект готов к загрузке и использованию!** ✅
