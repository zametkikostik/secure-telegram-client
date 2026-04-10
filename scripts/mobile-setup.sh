#!/bin/bash
# ============================================================================
# Secure Messenger - Mobile Setup Script
# ============================================================================
# Этот скрипт инициализирует Android проект для React Native
# ============================================================================

set -e

# Цвета
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   📱  Secure Messenger - Mobile Setup Wizard         ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Проверка Node.js
if ! command -v node &> /dev/null; then
    echo -e "${RED}❌ Node.js не найден!${NC}"
    echo -e "${YELLOW}Установи: https://nodejs.org/${NC}"
    exit 1
fi

NODE_VERSION=$(node -v | cut -d'v' -f2 | cut -d'.' -f1)
if [ "$NODE_VERSION" -lt 18 ]; then
    echo -e "${RED}❌ Node.js должен быть версии 18 или выше${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Node.js v$(node -v) найден${NC}"

# Проверка Java
if ! command -v java &> /dev/null; then
    echo -e "${RED}❌ Java не найдена!${NC}"
    echo -e "${YELLOW}Установи: https://adoptium.net/${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Java $(java -version 2>&1 | head -1) найдена${NC}"

# Проверка Android SDK
if [ -z "$ANDROID_HOME" ]; then
    echo -e "${YELLOW}⚠️  ANDROID_HOME не установлен${NC}"
    echo ""
    echo -e "${YELLOW}Установи Android SDK:${NC}"
    echo "  1. Скачай Android Studio: https://developer.android.com/studio"
    echo "  2. Установи Android SDK (API 33+)"
    echo "  3. Добавь в ~/.bashrc или ~/.zshrc:"
    echo ""
    echo "     export ANDROID_HOME=\$HOME/Android/Sdk"
    echo "     export PATH=\$PATH:\$ANDROID_HOME/platform-tools"
    echo "     export PATH=\$PATH:\$ANDROID_HOME/cmdline-tools/latest/bin"
    echo ""
    read -p "Продолжить без ANDROID_HOME? (y/n) " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        exit 1
    fi
else
    echo -e "${GREEN}✅ ANDROID_HOME=$ANDROID_HOME${NC}"
fi

echo ""
echo -e "${BLUE}📦 Установка зависимостей...${NC}"

# Установка зависимостей
npm install

echo ""
echo -e "${GREEN}✅ Зависимости установлены${NC}"
echo ""

# Проверка наличия Android проекта
if [ ! -d "android" ] || [ -z "$(ls -A android)" ]; then
    echo -e "${YELLOW}⚠️  Android проект не найден${NC}"
    echo ""
    echo -e "${BLUE}Варианты:${NC}"
    echo ""
    echo "  1) Создать новый Android проект (рекомендуется)"
    echo "  2) Использовать существующий android/ директорию"
    echo ""
    read -p "Выбери вариант (1/2): " -n 1 -r
    echo
    
    if [[ $REPLY =~ ^[1]$ ]]; then
        echo ""
        echo -e "${BLUE}🔨 Создание Android проекта...${NC}"
        echo ""
        echo -e "${YELLOW}Выполни вручную:${NC}"
        echo ""
        echo "  cd mobile"
        echo "  npx react-native init SecureMessenger --template react-native-template-typescript"
        echo ""
        echo "Или следуй: https://reactnative.dev/docs/environment-setup"
    fi
fi

echo ""
echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              🎉  ГОТОВО!  Следующие шаги:             ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}1.${NC} Запусти Metro bundler: ${GREEN}npm start${NC}"
echo -e "${YELLOW}2.${NC} В другом терминале: ${GREEN}npm run android${NC}"
echo -e "${YELLOW}3.${NC} Для сборки APK: ${GREEN}cd scripts && ./generate-keystore.sh${NC}"
echo ""
