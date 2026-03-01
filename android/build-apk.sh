#!/bin/bash
# Скрипт для сборки Secure Messenger Android APK

set -e

echo "🔐 Secure Messenger Android - Сборка APK"
echo "========================================"

cd /home/kostik/secure-telegram-client/android

# Проверка окружения
echo "📋 Проверка окружения..."
export ANDROID_HOME=$HOME/Android/Sdk
export NDK_HOME=$ANDROID_HOME/ndk/25.2.9519653

echo "  - Android SDK: $ANDROID_HOME"
echo "  - NDK: $NDK_HOME"
echo "  - Java: $(java -version 2>&1 | head -1)"
echo "  - Rust: $(rustc --version)"

# Создание keystore для подписи
if [ ! -f keystore/release.keystore ]; then
    echo "📝 Создание keystore..."
    mkdir -p keystore
    keytool -genkey -v \
        -keystore keystore/release.keystore \
        -alias secure-messenger \
        -keyalg RSA \
        -keysize 2048 \
        -validity 10000 \
        -storepass android \
        -keypass android \
        -dname "CN=Secure Messenger, OU=Development, O=Example, L=City, S=State, C=US"
    echo "✅ Keystore создан"
fi

# Сборка Rust библиотеки (если cargo-ndk доступен)
if command -v cargo-ndk &> /dev/null; then
    echo "🦀 Сборка Rust библиотеки..."
    cd core
    export ANDROID_NDK_HOME=$NDK_HOME
    
    # Попытка сборки
    if cargo ndk -t arm64-v8a -o ../app/src/main/jniLibs build --release 2>/dev/null; then
        echo "✅ Rust библиотека собрана"
    else
        echo "⚠️ Rust библиотека не собрана (будет заглушка)"
        # Создаём пустую директорию для .so
        mkdir -p ../app/src/main/jniLibs/arm64-v8a
    fi
    cd ..
fi

# Сборка APK через Gradle
echo "📱 Сборка APK..."
./gradlew assembleFdroidRelease --no-daemon || {
    echo "⚠️ Gradle сборка не удалась, создаём debug APK"
    ./gradlew assembleFdroidDebug --no-daemon
    APK_PATH="app/build/outputs/apk/fdroid/debug/app-fdroid-debug-unsigned.apk"
}

APK_PATH="app/build/outputs/apk/fdroid/release/app-fdroid-release-unsigned.apk"

if [ -f "$APK_PATH" ]; then
    echo "✅ APK создан: $APK_PATH"
    echo "📊 Размер: $(du -h "$APK_PATH" | cut -f1)"
else
    echo "❌ APK не найден"
    exit 1
fi

# Подпись APK
echo "🔐 Подпись APK..."
./scripts/sign-release.sh "$APK_PATH"

echo ""
echo "✅ Сборка завершена!"
echo ""
