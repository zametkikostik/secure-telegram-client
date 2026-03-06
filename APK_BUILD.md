# Liberty Reach — Инструкция по сборке APK

## 📱 Сборка Android APK

### Требования

- Node.js 18+
- Android Studio
- JDK 17
- Android SDK 34

### Шаг 1: Установка зависимостей

```bash
cd mobile
npm install
```

### Шаг 2: Настройка Android SDK

Убедитесь, что установлены:
- Android SDK Platform 34
- Android SDK Build-Tools 34.0.0
- Android NDK 25.1.8937393

### Шаг 3: Создание keystore (для release)

```bash
cd mobile/android
keytool -genkey -v \
  -keystore liberty-reach.keystore \
  -alias liberty \
  -keyalg RSA \
  -keysize 2048 \
  -validity 10000
```

### Шаг 4: Настройка подписи

Создайте `mobile/android/gradle.properties`:

```properties
LIBERTY_UPLOAD_STORE_FILE=liberty-reach.keystore
LIBERTY_UPLOAD_STORE_PASSWORD=ваш-пароль
LIBERTY_UPLOAD_KEY_ALIAS=liberty
LIBERTY_UPLOAD_KEY_PASSWORD=ваш-пароль
```

### Шаг 5: Сборка Debug APK

```bash
cd mobile
npm run build:apk
```

**Результат:**
```
mobile/android/app/build/outputs/apk/debug/app-debug.apk
```

### Шаг 6: Сборка Release APK

```bash
cd mobile
npm run build:android
```

**Результат:**
```
mobile/android/app/build/outputs/apk/release/app-release.apk
```

### Шаг 7: Установка на устройство

```bash
# Через ADB
adb install android/app/build/outputs/apk/debug/app-debug.apk

# Или скопируйте APK на устройство и установите вручную
```

---

## 🔧 Troubleshooting

### Ошибка: SDK not found

```bash
export ANDROID_HOME=$HOME/Android/Sdk
export PATH=$PATH:$ANDROID_HOME/tools:$ANDROID_HOME/platform-tools
```

### Ошибка: Keystore not found

Убедитесь, что keystore находится в `mobile/android/` и пути в `gradle.properties` правильные.

### Ошибка: Build failed

```bash
cd mobile/android
./gradlew clean
cd ../..
npm run build:apk
```

---

## 📊 Характеристики APK

| Параметр | Значение |
|----------|----------|
| Мин. версия Android | 8.0 (API 24) |
| Целевая версия | Android 14 (API 34) |
| Размер (debug) | ~50-80 MB |
| Размер (release) | ~30-50 MB |
| Архитектуры | arm64-v8a, armeabi-v7a, x86_64 |

---

## 🚀 Публикация

### Google Play Store

1. Соберите release APK
2. Создайте аккаунт разработчика ($25)
3. Загрузите APK в Google Play Console
4. Заполните описание и скриншоты
5. Отправьте на модерацию

### Прямая дистрибуция

Разместите APK на:
- GitHub Releases
- Вашем сайте
- F-Droid (open source)

---

## 📬 Поддержка

Email: support@libertyreach.io
