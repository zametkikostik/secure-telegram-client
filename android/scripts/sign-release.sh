#!/bin/bash
# Скрипт для подписи релизов Secure Messenger
# Использование: ./sign-release.sh <apk_path> [keystore_path]

set -e

# Цвета
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}🔐 Secure Messenger - Подпись релиза${NC}"
echo "========================================"

# Проверка аргументов
APK_PATH="${1:-app/build/outputs/apk/fdroid/release/app-fdroid-release-unsigned.apk}"
KEYSTORE_PATH="${2:-$KEYSTORE_PATH}"

if [ ! -f "$APK_PATH" ]; then
    echo -e "${RED}❌ APK файл не найден: $APK_PATH${NC}"
    exit 1
fi

# Генерация ключа если не существует
if [ -z "$KEYSTORE_PATH" ] || [ ! -f "$KEYSTORE_PATH" ]; then
    echo -e "${YELLOW}⚠️ Keystore не найден, генерация нового...${NC}"
    
    KEYSTORE_PATH="keystore/release.keystore"
    mkdir -p "$(dirname "$KEYSTORE_PATH")"
    
    # Генерация ключа
    keytool -genkey -v \
        -keystore "$KEYSTORE_PATH" \
        -alias "secure-messenger" \
        -keyalg RSA \
        -keysize 2048 \
        -validity 10000 \
        -storepass "android" \
        -keypass "android" \
        -dname "CN=Secure Messenger, OU=Development, O=Example, L=City, S=State, C=US"
    
    echo -e "${GREEN}✅ Keystore создан: $KEYSTORE_PATH${NC}"
    echo -e "${YELLOW}⚠️ Сохраните пароль: android${NC}"
fi

# Подпись APK
echo -e "${YELLOW}📝 Подпись APK...${NC}"

SIGNED_APK="${APK_PATH%.apk}-signed.apk"

# Использование apksigner
apksigner sign \
    --ks "$KEYSTORE_PATH" \
    --ks-key-alias "secure-messenger" \
    --ks-pass "pass:android" \
    --key-pass "pass:android" \
    --out "$SIGNED_APK" \
    "$APK_PATH"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ APK подписан: $SIGNED_APK${NC}"
else
    echo -e "${RED}❌ Ошибка подписи APK${NC}"
    exit 1
fi

# Верификация подписи
echo -e "${YELLOW}🔍 Верификация подписи...${NC}"

apksigner verify --verbose "$SIGNED_APK"

if [ $? -eq 0 ]; then
    echo -e "${GREEN}✅ Подпись верифицирована${NC}"
else
    echo -e "${RED}❌ Ошибка верификации подписи${NC}"
    exit 1
fi

# Вычисление хешей
echo -e "${YELLOW}📊 Вычисление хешей...${NC}"

SHA256=$(sha256sum "$SIGNED_APK" | cut -d' ' -f1)
MD5=$(md5sum "$SIGNED_APK" | cut -d' ' -f1)

echo "SHA256: $SHA256"
echo "MD5: $MD5"

# Сохранение хешей
echo "$SHA256  $(basename "$SIGNED_APK")" > "${SIGNED_APK}.sha256"
echo "$MD5  $(basename "$SIGNED_APK")" > "${SIGNED_APK}.md5"

echo -e "${GREEN}✅ Хешы сохранены${NC}"

# Информация о релизе
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}📦 Релиз готов к публикации${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "APK: $SIGNED_APK"
echo "Размер: $(du -h "$SIGNED_APK" | cut -f1)"
echo "SHA256: $SHA256"
echo ""
echo -e "${YELLOW}Следующие шаги:${NC}"
echo "1. Опубликуйте APK в IPFS"
echo "2. Создайте релиз на GitHub/Codeberg"
echo "3. Обновите manifest.json"
echo ""
