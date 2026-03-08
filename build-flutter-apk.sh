#!/bin/bash

# Liberty Reach Messenger v2.0.0
# Скрипт сборки Flutter APK

set -e

echo "🦋 Liberty Reach Messenger - Сборка Flutter APK"
echo "================================================"
echo ""

# Проверка Flutter
echo "📱 Проверка Flutter..."
if ! command -v flutter &> /dev/null; then
    echo "❌ Flutter не найден!"
    echo ""
    echo "Установи Flutter:"
    echo "  sudo snap install flutter --classic"
    echo "  или"
    echo "  wget https://storage.googleapis.com/flutter_infra_release/releases/stable/linux/flutter_linux_3.16.0-stable.tar.xz"
    echo "  tar xf flutter_linux_*.tar.xz"
    echo "  export PATH=\"\$PATH:\$(pwd)/flutter/bin\""
    exit 1
fi

flutter --version
echo "✅ Flutter найден"
echo ""

# Переход в директорию Flutter
cd flutter_ui

# Установка зависимостей
echo "📦 Установка зависимостей..."
flutter pub get
echo "✅ Зависимости установлены"
echo ""

# Проверка Android SDK
echo "🤖 Проверка Android SDK..."
if [ -z "$ANDROID_HOME" ]; then
    echo "⚠️ ANDROID_HOME не установлен"
    echo "Установи:"
    echo "  export ANDROID_HOME=~/Android/Sdk"
    echo "  export PATH=\$PATH:\$ANDROID_HOME/tools:\$ANDROID_HOME/platform-tools"
fi
echo ""

# Сборка APK
echo "🔨 Сборка APK..."
flutter build apk --release

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ СБОРКА УСПЕШНА!"
    echo ""
    
    # Копирование APK в releases
    mkdir -p ../releases
    cp build/app/outputs/flutter-apk/*.apk ../releases/
    
    echo "📦 APK файлы:"
    ls -lh ../releases/*.apk
    echo ""
    
    echo "📊 Информация:"
    unzip -l build/app/outputs/flutter-apk/*.apk | grep -E "lib|classes.dex" | head -10
    echo ""
    
    echo "🚀 Установка на устройство:"
    echo "  adb install ../releases/LibertyReach-v2.0.0.apk"
    echo ""
else
    echo ""
    echo "❌ ОШИБКА СБОРКИ!"
    echo ""
    exit 1
fi
