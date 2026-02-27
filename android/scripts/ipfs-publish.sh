#!/bin/bash
# Скрипт для публикации релиза в IPFS
# Использование: ./ipfs-publish.sh <apk_path> <version>

set -e

# Цвета
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${GREEN}🌐 Secure Messenger - Публикация в IPFS${NC}"
echo "========================================"

APK_PATH="$1"
VERSION="$2"

if [ -z "$APK_PATH" ] || [ -z "$VERSION" ]; then
    echo -e "${RED}Использование: $0 <apk_path> <version>${NC}"
    exit 1
fi

if [ ! -f "$APK_PATH" ]; then
    echo -e "${RED}❌ APK файл не найден: $APK_PATH${NC}"
    exit 1
fi

# Проверка IPFS
if ! command -v ipfs &> /dev/null; then
    echo -e "${RED}❌ IPFS не установлен${NC}"
    echo "Установите IPFS: https://docs.ipfs.io/install/"
    exit 1
fi

echo -e "${YELLOW}📤 Добавление APK в IPFS...${NC}"

# Добавление файла в IPFS
APK_CID=$(ipfs add -Q "$APK_PATH")

echo -e "${GREEN}✅ APK добавлен в IPFS${NC}"
echo "CID: $APK_CID"

# Создание manifest.json
echo -e "${YELLOW}📝 Создание manifest.json...${NC}"

cat > manifest.json << EOF
{
  "latest_version": "$VERSION",
  "latest_version_code": $(echo $VERSION | cut -d'.' -f3),
  "apk_cid": "$APK_CID",
  "public_key": "${IPFS_PUBLIC_KEY:-YOUR_PUBLIC_KEY_HERE}",
  "changelog": "Release $VERSION",
  "published_at": "$(date -u +%Y-%m-%dT%H:%M:%SZ)"
}
EOF

# Добавление manifest в IPFS
MANIFEST_CID=$(ipfs add -Q manifest.json)

echo -e "${GREEN}✅ Manifest добавлен в IPFS${NC}"
echo "Manifest CID: $MANIFEST_CID"

# Пиннинг на Pinata (опционально)
if [ -n "$PINATA_API_KEY" ] && [ -n "$PINATA_SECRET_KEY" ]; then
    echo -e "${YELLOW}📌 Пиннинг на Pinata...${NC}"
    
    curl -X POST "https://api.pinata.cloud/pinning/pinByHash" \
        -H "Content-Type: application/json" \
        -H "pinata_api_key: $PINATA_API_KEY" \
        -H "pinata_secret_api_key: $PINATA_SECRET_KEY" \
        -d "{\"hashToPin\": \"$APK_CID\"}"
    
    curl -X POST "https://api.pinata.cloud/pinning/pinByHash" \
        -H "Content-Type: application/json" \
        -H "pinata_api_key: $PINATA_API_KEY" \
        -H "pinata_secret_api_key: $PINATA_SECRET_KEY" \
        -d "{\"hashToPin\": \"$MANIFEST_CID\"}"
    
    echo -e "${GREEN}✅ Пиннинг выполнен${NC}"
fi

# Вывод информации
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}📦 Релиз опубликован в IPFS${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo "APK CID: $APK_CID"
echo "Manifest CID: $MANIFEST_CID"
echo ""
echo "IPFS шлюзы для загрузки:"
echo "  https://ipfs.io/ipfs/$APK_CID"
echo "  https://cloudflare-ipfs.com/ipfs/$APK_CID"
echo "  https://dweb.link/ipfs/$APK_CID"
echo ""
echo "Manifest:"
echo "  https://ipfs.io/ipfs/$MANIFEST_CID"
echo ""

# Очистка
rm -f manifest.json

echo -e "${YELLOW}Следующие шаги:${NC}"
echo "1. Обновите manifest CID в приложении"
echo "2. Опубликуйте релиз на Codeberg/GitFlic"
echo "3. Отправьте manifest в рассылку"
echo ""
