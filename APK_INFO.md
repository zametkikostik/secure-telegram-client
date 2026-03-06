# 📱 Secure Telegram APK

## ✅ Готовое подписанное APK

**Версия:** v1.0.0  
**Дата:** 6 марта 2026  
**Статус:** Подписано release ключом

## 📦 Файлы

| Файл | Описание | Размер |
|------|----------|--------|
| `app-debug.apk` | Debug версия (для тестирования) | ~50 MB |
| `app-release.apk` | Release версия (подписано) | ~45 MB |

## 🔐 Подпись

APK подписано release ключом:

- **Keystore:** `mobile/android/liberty-reach.keystore`
- **Alias:** `liberty`
- **Валидность:** 10 000 дней (~27 лет)
- **Ключ:** RSA 2048 bit

## 📥 Установка

### На устройство

```bash
# Через ADB
adb install mobile/android/app/build/outputs/apk/release/app-release.apk

# Или скопируйте APK на устройство и установите
```

### На эмулятор

```bash
# Запуск эмулятора
emulator -avd Pixel_6_API_34

# Установка APK
adb install mobile/android/app/build/outputs/apk/release/app-release.apk
```

## 🔧 Сборка нового APK

### Debug APK

```bash
cd mobile/android
./gradlew assembleDebug

# APK будет в:
# app/build/outputs/apk/debug/app-debug.apk
```

### Release APK (подписанный)

```bash
cd mobile/android
./gradlew assembleRelease

# APK будет в:
# app/build/outputs/apk/release/app-release.apk
```

## 📋 Требования

- **Android:** 8.0+ (API 26)
- **Архитектура:** arm64-v8a, armeabi-v7a, x86_64
- **Разрешения:** Интернет, камера, микрофон, хранилище

## 🎯 Функции

- ✅ Приватные чаты
- ✅ Групповые чаты
- ✅ Аудио/Видео звонки (WebRTC)
- ✅ Push уведомления
- ✅ AI перевод
- ✅ P2P сеть

## 📊 Статус сборки

| Компонент | Статус |
|-----------|--------|
| React Native | ✅ 0.73 |
| WebRTC | ✅ 1.0.32006 |
| Firebase Messaging | ✅ 23.1.2 |
| Keystore | ✅ Создан |
| Подпись | ✅ Release |

## 🔗 Ссылки

- **Исходный код:** `mobile/src/App.tsx`
- **Конфигурация:** `mobile/android/app/build.gradle`
- **Manifest:** `mobile/android/app/src/main/AndroidManifest.xml`

---

**Secure Telegram Team © 2026**
