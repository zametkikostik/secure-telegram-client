#!/bin/bash
# Pre-remove script

echo "🗑️  Удаление Secure Telegram Desktop..."

# Сохранение данных пользователя
if [ "$1" = "remove" ]; then
    echo "⚠️  Данные пользователя будут сохранены в ~/.config/secure-telegram"
fi

if [ "$1" = "purge" ]; then
    echo "🗑️  Удаление данных пользователя..."
    rm -rf "$HOME/.config/secure-telegram"
fi

echo "✅ Удаление завершено"
