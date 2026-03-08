# 🦋 УСТАНОВКА FLUTTER И СБОРКА APK

## 📋 ТРЕБОВАНИЯ

- **ОС:** Linux (Ubuntu/Debian/Mint)
- **RAM:** 4 GB+
- **Disk:** 10 GB+
- **Android SDK:** Да

---

## 🚀 БЫСТРАЯ УСТАНОВКА

### 1. Установи Flutter

```bash
# Скачай Flutter
cd /opt
sudo wget https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_3.16.0-stable.tar.xz

# Распакуй
sudo tar xf flutter_linux_*.tar.xz
sudo chown -R $USER:$USER flutter

# Добавь в PATH
export PATH="$PATH:/opt/flutter/bin"
echo 'export PATH="$PATH:/opt/flutter/bin"' >> ~/.bashrc
source ~/.bashrc

# Проверь
flutter --version
```

### 2. Настрой Android SDK

```bash
# Скачай Android command-line tools
mkdir -p ~/Android/Sdk
cd ~/Android/Sdk
wget https://dl.google.com/android/repository/commandlinetools-linux-9477386_latest.zip
unzip commandlinetools-*.zip

# Установи SDK
export ANDROID_HOME=~/Android/Sdk
export PATH=$PATH:$ANDROID_HOME/cmdline-tools/bin:$ANDROID_HOME/platform-tools
echo 'export ANDROID_HOME=~/Android/Sdk' >> ~/.bashrc
echo 'export PATH=$PATH:$ANDROID_HOME/cmdline-tools/bin:$ANDROID_HOME/platform-tools' >> ~/.bashrc

# Приними лицензии
yes | sdkmanager --licenses

# Установи компоненты
sdkmanager "platform-tools"
sdkmanager "platforms;android-34"
sdkmanager "build-tools;34.0.0"
```

### 3. Проверь установку

```bash
flutter doctor -v
```

Должно быть:
```
✅ Android toolchain - develop for Android devices
✅ Android Studio (optional)
✅ Connected device
```

---

## 🔨 СБОРКА APK

### Автоматически (скрипт):

```bash
cd /home/kostik/secure-telegram-client
./build-flutter-apk.sh
```

### Вручную:

```bash
cd flutter_ui

# Установи зависимости
flutter pub get

# Собери APK
flutter build apk --release

# APK будет в:
# build/app/outputs/flutter-apk/app-release.apk
```

---

## 📱 УСТАНОВКА НА ТЕЛЕФОН

### Через USB:

```bash
# Включи отладку по USB на телефоне
# Подключи телефон

# Проверь подключение
adb devices

# Установи APK
adb install flutter_ui/build/app/outputs/flutter-apk/app-release.apk
```

### Или скопируй APK на телефон:

```bash
# Скопируй файл
cp flutter_ui/build/app/outputs/flutter-apk/app-release.apk /sdcard/Download/

# На телефоне: Файловый менеджер → Download → Установить
```

---

## 🐛 ТРАБЛШУТИНГ

### Ошибка: "No Android SDK found"

**Решение:**
```bash
export ANDROID_HOME=~/Android/Sdk
export PATH=$PATH:$ANDROID_HOME/cmdline-tools/bin:$ANDROID_HOME/platform-tools
```

### Ошибка: "License not accepted"

**Решение:**
```bash
yes | sdkmanager --licenses
```

### Ошибка: "No devices found"

**Решение:**
- Включи отладку по USB на телефоне
- Разреши доступ на телефоне
- Проверь кабель

### Ошибка: "Gradle build failed"

**Решение:**
```bash
cd flutter_ui/android
./gradlew clean
cd ..
flutter clean
flutter pub get
flutter build apk
```

---

## 📊 РАЗМЕР APK

| Тип | Размер |
|-----|--------|
| **Debug** | ~45 MB |
| **Release** | ~25 MB |
| **Split per ABI** | ~15 MB (arm64) |

---

## 🎯 ССЫЛКИ

- **Flutter:** https://flutter.dev
- **Android SDK:** https://developer.android.com/studio
- **Документация:** flutter_ui/README.md

---

*Инструкция по установке Flutter*  
*Март 2026*
