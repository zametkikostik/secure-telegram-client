#!/bin/bash

# Liberty Reach Messenger v2.0.0
# Скрипт автоматической установки APK на телефон

set -e

echo "🚀 Liberty Reach Messenger v2.0.0 - Установка APK"
echo "=================================================="
echo ""

# Пути
APK_PATH="releases/LibertyReach-v2.0.0.apk"
ALTERNATIVE_APK="mobile/android/app/build/outputs/apk/release/app-release.apk"
PACKAGE_NAME="io.libertyreach"

# Проверка ADB
echo "📱 Проверка ADB..."
if ! command -v adb &> /dev/null; then
    echo "❌ ADB не найден!"
    echo ""
    echo "Установи ADB:"
    echo "  sudo apt install adb"
    echo "  или"
    echo "  sudo dnf install adb"
    exit 1
fi
echo "✅ ADB найден"
echo ""

# Проверка APK
echo "📦 Проверка APK файла..."
if [ -f "$APK_PATH" ]; then
    echo "✅ APK найден: $APK_PATH"
elif [ -f "$ALTERNATIVE_APK" ]; then
    APK_PATH="$ALTERNATIVE_APK"
    echo "✅ APK найден: $APK_PATH"
else
    echo "❌ APK не найден!"
    echo ""
    echo "Собери APK:"
    echo "  cd mobile/android && ./gradlew assembleRelease"
    exit 1
fi

# Размер APK
APK_SIZE=$(du -h "$APK_PATH" | cut -f1)
echo "📊 Размер APK: $APK_SIZE"
echo ""

# Подключение устройства
echo "🔌 Проверка подключения устройств..."
DEVICES=$(adb devices | grep -v "List" | grep "device$" | wc -l)

if [ "$DEVICES" -eq 0 ]; then
    echo "❌ Устройства не найдены!"
    echo ""
    echo "Что делать:"
    echo "  1. Подключи телефон USB кабелем"
    echo "  2. Включи отладку по USB на телефоне:"
    echo "     Настройки → О телефоне → 7 раз 'Номер сборки'"
    echo "     Настройки → Для разработчиков → Отладка по USB ✅"
    echo "  3. Разреши доступ на телефоне"
    echo "  4. Запусти скрипт снова"
    echo ""
    echo "Или используй беспроводное подключение:"
    echo "  adb connect 192.168.1.XXX:5555"
    exit 1
fi

echo "✅ Найдено устройств: $DEVICES"
adb devices
echo ""

# Удаление старой версии
echo "🗑️ Удаление старой версии (если есть)..."
adb uninstall $PACKAGE_NAME 2>/dev/null || echo "Старая версия не найдена"
echo ""

# Установка
echo "📲 Установка APK..."
echo "   Файл: $APK_PATH"
echo "   Пакет: $PACKAGE_NAME"
echo ""

adb install -r "$APK_PATH"

if [ $? -eq 0 ]; then
    echo ""
    echo "✅ УСПЕШНО!"
    echo ""
    echo "🎉 Liberty Reach Messenger v2.0.0 установлен!"
    echo ""
    
    # Запуск приложения
    echo "🚀 Запуск приложения..."
    adb shell am start -n $PACKAGE_NAME/.MainActivity
    
    echo ""
    echo "📱 Приложение запущено на твоём телефоне!"
    echo ""
    echo "📊 Информация:"
    echo "   Версия: 2.0.0"
    echo "   Cloudflare: https://secure-messenger-push.zametkikostik.workers.dev"
    echo "   Репозиторий: https://github.com/zametkikostik/secure-telegram-client"
    echo ""
else
    echo ""
    echo "❌ Ошибка установки!"
    echo ""
    echo "Возможные причины:"
    echo "  - Недостаточно места на телефоне"
    echo "  - Несовместимая версия Android (нужно 8.0+)"
    echo "  - Повреждённый APK файл"
    echo ""
    exit 1
fi
