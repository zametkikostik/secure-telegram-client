#!/bin/bash
# Pre-install script

echo "📦 Подготовка к установке Secure Telegram Desktop..."

# Проверка архитектуры
ARCH=$(dpkg --print-architecture)
echo "Архитектура: $ARCH"

# Проверка минимальных требований
REQUIRED_MEM=512 # MB
AVAILABLE_MEM=$(free -m | awk 'NR==2{print $7}')

if [ "$AVAILABLE_MEM" -lt "$REQUIRED_MEM" ]; then
    echo "⚠️  Предупреждение: Мало свободной памяти ($AVAILABLE_MEM MB)"
fi

echo "✅ Готово к установке"
