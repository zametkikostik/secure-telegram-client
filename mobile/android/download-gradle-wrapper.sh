#!/bin/bash
# Скрипт для загрузки gradle-wrapper.jar

GRADLE_VERSION=8.2
WRAPPER_JAR="/home/kostik/secure-telegram-client/mobile/android/gradle/wrapper/gradle-wrapper.jar"
WRAPPER_URL="https://raw.githubusercontent.com/gradle/gradle/v${GRADLE_VERSION}.0/gradle/wrapper/gradle-wrapper.jar"

echo "Загрузка gradle-wrapper.jar..."

# Пробуем скачать через curl
if command -v curl &> /dev/null; then
    curl -L -o "$WRAPPER_JAR" "$WRAPPER_URL"
# Или через wget
elif command -v wget &> /dev/null; then
    wget -O "$WRAPPER_JAR" "$WRAPPER_URL"
else
    echo "Ошибка: curl или wget не найдены"
    echo "Пожалуйста, установите curl или wget и запустите скрипт снова"
    exit 1
fi

# Проверка успешности загрузки
if [ -f "$WRAPPER_JAR" ] && [ -s "$WRAPPER_JAR" ]; then
    echo "✓ gradle-wrapper.jar успешно загружен"
    echo "  Размер: $(ls -lh "$WRAPPER_JAR" | awk '{print $5}')"
else
    echo "Ошибка при загрузке gradle-wrapper.jar"
    echo "Вы можете скачать его вручную:"
    echo "  $WRAPPER_URL"
    exit 1
fi
