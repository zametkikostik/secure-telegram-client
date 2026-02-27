# 📱 Secure Messenger Android — Инструкция по сборке

## Требования

### Обязательные
- **Java 17+**: `sudo apt install openjdk-17-jdk`
- **Android SDK**: API 34, Build Tools 34
- **Android NDK**: 25.2.9519653
- **Rust 1.75+**: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
- **Gradle 8.5+**: `sdk install gradle 8.5`

### Опциональные (для полной сборки)
- **cargo-ndk**: `cargo install cargo-ndk`
- **Android Targets**: `rustup target add aarch64-linux-android armv7-linux-androideabi x86_64-linux-android`

---

## Быстрая сборка (без Rust JNI)

```bash
cd android

# Создание Gradle wrapper
gradle wrapper --gradle-version 8.5

# Сборка debug версии (без native библиотек)
./gradlew assembleFdroidDebug

# APK будет в:
# app/build/outputs/apk/fdroid/debug/app-fdroid-debug.apk
```

---

## Полная сборка (с Rust JNI)

### 1. Установка Android таргетов

```bash
# Это может занять 10-15 минут
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android
```

### 2. Установка NDK

```bash
~/Android/Sdk/cmdline-tools/latest/bin/sdkmanager --install "ndk;25.2.9519653"
```

### 3. Установка cargo-ndk

```bash
cargo install cargo-ndk
```

### 4. Сборка

```bash
# Запуск сборки APK
./build-apk.sh

# Или вручную:
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/25.2.9519653

# Сборка Rust библиотеки
cd core
cargo ndk -t arm64-v8a -t armeabi-v7a -t x86_64 \
    -o ../app/src/main/jniLibs build --release

# Сборка APK
cd ..
./gradlew assembleFdroidRelease
```

---

## Подпись релиза

```bash
# Автоматическая подпись
./scripts/sign-release.sh app/build/outputs/apk/fdroid/release/app-fdroid-release-unsigned.apk

# Ручная подпись
apksigner sign \
    --ks keystore/release.keystore \
    --ks-key-alias secure-messenger \
    --ks-pass pass:android \
    --key-pass pass:android \
    --out app-fdroid-release-signed.apk \
    app/build/outputs/apk/fdroid/release/app-fdroid-release-unsigned.apk
```

---

## Публикация в IPFS

```bash
# Публикация APK
./scripts/ipfs-publish.sh app-fdroid-release-signed.apk 0.2.0

# Выведет:
# - APK CID
# - Manifest CID
# - Ссылки на IPFS шлюзы
```

---

## Установка на устройство

### Через ADB
```bash
adb install app/build/outputs/apk/fdroid/debug/app-fdroid-debug.apk
```

### Через IPFS
1. Откройте https://ipfs.io/ipfs/<APK_CID>
2. Скачайте APK
3. Установите на устройство

---

## Решение проблем

### Ошибка: "SDK location not found"
```bash
# Создайте local.properties
echo "sdk.dir=$HOME/Android/Sdk" > local.properties
```

### Ошибка: "NDK not found"
```bash
# Установите NDK
~/Android/Sdk/cmdline-tools/latest/bin/sdkmanager --install "ndk;25.2.9519653"
```

### Ошибка: "target not found"
```bash
# Установите Android таргеты
rustup target add aarch64-linux-android
```

### Ошибка: "cargo-ndk: command not found"
```bash
# Установите cargo-ndk
cargo install cargo-ndk
```

---

## Проверка сборки

```bash
# Проверка APK
apksigner verify --verbose app-fdroid-release-signed.apk

# Проверка хешей
sha256sum app-fdroid-release-signed.apk
```

---

## F-Droid сборка

F-Droid использует свою сборочную среду. Для проверки совместимости:

```bash
# Используйте fdroid build command
fdroid build --verbose --test com.example.securemessenger.fdroid
```

---

## Структура APK

```
app-fdroid-release.apk
├── AndroidManifest.xml
├── classes.dex          # Kotlin код
├── lib/
│   ├── arm64-v8a/
│   │   └── libsecure_messenger_core.so  # Rust библиотека
│   ├── armeabi-v7a/
│   └── x86_64/
└── res/                 # Ресурсы
```

---

## Размер APK

- **Debug**: ~5-8 MB (без Rust)
- **Release**: ~3-5 MB (сжатый, без Rust)
- **Full с Rust**: ~8-12 MB

---

## Воспроизводимость

Для воспроизводимой сборки:

1. Используйте фиксированные версии зависимостей
2. Отключите incremental компиляцию
3. Используйте одинаковый NDK и SDK
4. Собирайте в чистой среде (Docker)

```bash
# Пример Docker сборки
docker run -it --rm \
    -v $(pwd):/app \
    -v $HOME/.cargo:/root/.cargo \
    rust:1.75 \
    bash -c "cd /app/android && ./build-apk.sh"
```

---

**Версия**: 0.2.0  
**Дата**: 2024-02-27
