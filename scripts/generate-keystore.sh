#!/bin/bash
# ============================================================================
# Secure Messenger - Android Keystore Generator
# ============================================================================
# Этот скрипт создаёт keystore для подписи Android APK
# и помогает настроить GitHub Secrets
# ============================================================================

set -e

# Цвета
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║   🔐  Secure Messenger - Android Keystore Generator   ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""

# Проверка наличия keytool
if ! command -v keytool &> /dev/null; then
    echo -e "${RED}❌ Error: keytool не найден!${NC}"
    echo -e "${YELLOW}Установи Java JDK: https://adoptium.net/${NC}"
    exit 1
fi

echo -e "${GREEN}✅ keytool найден${NC}"
echo ""

# Запрос данных для keystore
echo -e "${YELLOW}📝 Введи данные для keystore (или оставь по умолчанию):${NC}"
echo ""

read -p "  Keystore пароль (по умолчанию: android123): " KEYSTORE_PASSWORD
KEYSTORE_PASSWORD=${KEYSTORE_PASSWORD:-android123}

read -p "  Key alias (по умолчанию: secure-messenger): " KEY_ALIAS
KEY_ALIAS=${KEY_ALIAS:-secure-messenger}

read -p "  Key password (по умолчанию: android123): " KEY_PASSWORD
KEY_PASSWORD=${KEY_PASSWORD:-android123}

read -p "  validity лет (по умолчанию: 10000): " VALIDITY
VALIDITY=${VALIDITY:-10000}

echo ""
echo -e "${BLUE}🔨 Генерация keystore...${NC}"

# Создание keystore
keytool -genkey -v \
    -keystore secure-messenger.keystore \
    -alias "$KEY_ALIAS" \
    -keyalg RSA \
    -keysize 2048 \
    -validity "$VALIDITY" \
    -storepass "$KEYSTORE_PASSWORD" \
    -keypass "$KEY_PASSWORD" \
    -dname "CN=Secure Messenger, OU=Development, O=Secure Messenger, L=Sofia, C=BG" \
    -storetype PKCS12

echo ""
echo -e "${GREEN}✅ Keystore создан: secure-messenger.keystore${NC}"
echo ""

# Проверка keystore
echo -e "${BLUE}🔍 Проверка keystore...${NC}"
keytool -list -v \
    -keystore secure-messenger.keystore \
    -storepass "$KEYSTORE_PASSWORD" | head -20

echo ""
echo -e "${YELLOW}📦 Кодирование в Base64 для GitHub Secrets...${NC}"

# Кодирование в base64
KEYSTORE_BASE64=$(base64 -w 0 secure-messenger.keystore)

echo -e "${GREEN}✅ Keystore закодирован${NC}"
echo ""

# Создание файла с инструкциями
cat > GITHUB_SECRETS_SETUP.md << 'EOF'
# 🔐 Настройка GitHub Secrets для Android APK

## Добавь эти секреты в GitHub Repository

Перейди в: **Settings → Secrets and variables → Actions → New repository secret**

### 1. ANDROID_KEYSTORE_BASE64
```
Скопируй всё содержимое файла keystore-base64.txt
```

### 2. ANDROID_KEYSTORE_PASSWORD
```
Пароль от keystore (по умолчанию: android123)
```

### 3. ANDROID_KEY_ALIAS
```
alias ключа (по умолчанию: secure-messenger)
```

### 4. ANDROID_KEY_PASSWORD
```
Пароль от ключа (по умолчанию: android123)
```

## 🚀 Использование

После настройки secrets:
- **Push на main** → Собирается debug APK
- **Создание тега (v*)** → Собирается signed release APK

## ⚠️  Безопасность

- **НИКОГДА** не коммить `secure-messenger.keystore` в git!
- Храни backup keystore в безопасном месте
- Без keystore не сможешь обновлять приложение в Google Play
EOF

# Сохранение base64 в файл
echo "$KEYSTORE_BASE64" > keystore-base64.txt

echo -e "${GREEN}📄 Создан файл: keystore-base64.txt${NC}"
echo -e "${GREEN}📄 Создан файл: GITHUB_SECRETS_SETUP.md${NC}"
echo ""

echo -e "${BLUE}╔════════════════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║              🎉  ГОТОВО!  Следующие шаги:             ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════════════════════╝${NC}"
echo ""
echo -e "${YELLOW}1.${NC} Прочитай ${GREEN}GITHUB_SECRETS_SETUP.md${NC}"
echo -e "${YELLOW}2.${NC} Добавь секреты в GitHub repository"
echo -e "${YELLOW}3.${NC} Удали ${RED}secure-messenger.keystore${NC} после копирования base64"
echo -e "${YELLOW}4.${NC} Закоммить изменения и push на main"
echo ""
echo -e "${RED}⚠️  ВАЖНО: Удали secure-messenger.keystore после настройки!${NC}"
echo ""
