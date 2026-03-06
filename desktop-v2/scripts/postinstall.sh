#!/bin/bash
# Post-install script for Linux Mint

echo "🔧 Настройка Secure Telegram Desktop..."

# Обновление кэша иконок
if command -v update-icon-caches &> /dev/null; then
    update-icon-caches /usr/share/icons/* 2>/dev/null || true
fi

# Обновление базы desktop файлов
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database /usr/share/applications 2>/dev/null || true
fi

# Создание директории конфигурации
CONFIG_DIR="$HOME/.config/secure-telegram"
mkdir -p "$CONFIG_DIR"

echo "✅ Установка завершена!"
echo "Запустите Secure Telegram из меню приложений или командой:"
echo "  secure-telegram-desktop"
