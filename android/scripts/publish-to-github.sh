#!/bin/bash
# 🚀 Публикация релиза БЕЗ IPFS — только GitHub Releases
# Использование: ./publish-to-github.sh <apk_path> <version>

set -e

# Цвета
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${GREEN}🚀 Secure Messenger - Публикация на GitHub${NC}"
echo "========================================"

APK_PATH="$1"
VERSION="$2"

if [ -z "$APK_PATH" ] || [ -z "$VERSION" ]; then
    echo -e "${RED}Использование: $0 <apk_path> <version>${NC}"
    echo ""
    echo "Пример:"
    echo "  $0 app-fdroid-release-signed.apk 0.2.0"
    exit 1
fi

if [ ! -f "$APK_PATH" ]; then
    echo -e "${RED}❌ APK файл не найден: $APK_PATH${NC}"
    exit 1
fi

# Проверка Git
if ! command -v git &> /dev/null; then
    echo -e "${RED}❌ Git не установлен${NC}"
    exit 1
fi

# Проверка GitHub CLI (опционально)
HAS_GH=false
if command -v gh &> /dev/null; then
    HAS_GH=true
    echo -e "${BLUE}ℹ️  GitHub CLI найден${NC}"
fi

echo ""
echo -e "${YELLOW}📦 APK файл: $APK_PATH${NC}"
echo -e "${YELLOW}📝 Версия: $VERSION${NC}"
echo ""

# Вычисление хешей
echo -e "${BLUE}🔐 Вычисление хешей...${NC}"
SHA256=$(sha256sum "$APK_PATH" | awk '{print $1}')
MD5=$(md5sum "$APK_PATH" | awk '{print $1}')

echo "  SHA256: $SHA256"
echo "  MD5:    $MD5"
echo ""

# Создание CHANGELOG
CHANGELOG_FILE="CHANGELOG-v${VERSION}.md"
cat > "$CHANGELOG_FILE" << EOF
# 🔐 Secure Messenger v${VERSION}

## 📦 Что нового

- ✅ Постквантовое шифрование (Kyber-1024)
- ✅ Обфускация трафика (obfs4)
- ✅ Децентрализованные обновления
- ✅ P2P fallback режим
- ✅ Обход блокировок (DNS over HTTPS, TLS fingerprint)

## 📱 Технические детали

- **Package**: com.example.securemessenger.fdroid
- **Min SDK**: 26 (Android 8.0+)
- **Target SDK**: 35 (Android 15)
- **Размер**: $(du -h "$APK_PATH" | cut -f1)
- **Архитектуры**: arm64-v8a, armeabi-v7a, x86_64

## 🔐 Проверка целостности

\`\`\`bash
sha256sum app-fdroid-release-signed.apk
# Ожидаемый хеш: $SHA256
\`\`\`

## 📥 Установка

1. Скачайте APK
2. Разрешите установку из неизвестных источников
3. Установите APK

Или через ADB:
\`\`\`bash
adb install app-fdroid-release-signed.apk
\`\`\`

## ⚠️ Важно

Это исследовательский проект. Не используйте для критически важной коммуникации.

## 📚 Документация

- [README.md](README.md)
- [QUICKSTART.md](QUICKSTART.md)
- [DOWNLOAD.md](DOWNLOAD.md)
EOF

echo -e "${GREEN}✅ CHANGELOG создан: $CHANGELOG_FILE${NC}"
echo ""

# Инструкция для ручной публикации
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}📋 ИНСТРУКЦИЯ ПО ПУБЛИКАЦИИ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

if [ "$HAS_GH" = true ]; then
    echo -e "${GREEN}✅ GitHub CLI найден! Автоматическая публикация:${NC}"
    echo ""
    echo "Выполните команду:"
    echo ""
    echo -e "${YELLOW}gh release create v${VERSION} \\\\${NC}"
    echo -e "${YELLOW}    --title \"Secure Messenger v${VERSION}\" \\\\${NC}"
    echo -e "${YELLOW}    --notes-file ${CHANGELOG_FILE} \\\\${NC}"
    echo -e "${YELLOW}    ${APK_PATH}${NC}"
    echo ""
    read -p "Выполнить публикацию? (y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo -e "${YELLOW}📤 Публикация релиза...${NC}"
        gh release create v${VERSION} \
            --title "Secure Messenger v${VERSION}" \
            --notes-file "$CHANGELOG_FILE" \
            "$APK_PATH"
        echo -e "${GREEN}✅ Релиз опубликован!${NC}"
    fi
else
    echo -e "${YELLOW}⚠️  GitHub CLI не найден. Ручная публикация:${NC}"
    echo ""
    echo "1️⃣  Перейдите на:"
    echo -e "   ${BLUE}https://github.com/zametkikostik/secure-telegram-client/releases/new${NC}"
    echo ""
    echo "2️⃣  Заполните:"
    echo "   - Tag version: ${GREEN}v${VERSION}${NC}"
    echo "   - Release title: ${GREEN}Secure Messenger v${VERSION}${NC}"
    echo ""
    echo "3️⃣  Скопируйте CHANGELOG:"
    echo -e "   ${BLUE}cat ${CHANGELOG_FILE}${NC}"
    echo ""
    echo "4️⃣  Прикрепите файл:"
    echo -e "   ${GREEN}${APK_PATH}${NC}"
    echo ""
    echo "5️⃣  Нажмите ${GREEN}Publish release${NC}"
    echo ""
fi

echo ""
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}📊 ИНФОРМАЦИЯ О РЕЛИЗЕ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "Файл: $APK_PATH"
echo "Размер: $(du -h "$APK_PATH" | cut -f1)"
echo "Версия: $VERSION"
echo ""
echo "Хеши для проверки:"
echo "  SHA256: $SHA256"
echo "  MD5:    $MD5"
echo ""
echo "После публикации ссылка на релиз:"
echo "  https://github.com/zametkikostik/secure-telegram-client/releases/tag/v${VERSION}"
echo ""
echo "Прямая ссылка на APK:"
echo "  https://github.com/zametkikostik/secure-telegram-client/releases/download/v${VERSION}/${APK_PATH}"
echo ""

# Альтернативные варианты
echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}🔄 АЛЬТЕРНАТИВНЫЕ ВАРИАНТЫ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

echo "1️⃣  **Codeberg** (ЕС):"
echo "   https://codeberg.org/your-username/secure-telegram-client/releases"
echo ""

echo "2️⃣  **GitFlic** (РФ):"
echo "   https://gitflic.ru/project/your-username/secure-telegram-client/releases"
echo ""

echo "3️⃣  **IzzyOnDroid** (F-Droid репозиторий):"
echo "   https://gitlab.com/IzzyOnDroid/fdroiddata/-/merge_requests"
echo ""

echo "4️⃣  **F-Droid** (основной):"
echo "   https://gitlab.com/fdroid/fdroiddata/-/merge_requests"
echo ""

# Очистка
echo -e "${YELLOW}📝 Файлы для публикации:${NC}"
echo "   - $CHANGELOG_FILE (CHANGELOG)"
echo "   - $APK_PATH (APK)"
echo ""
echo -e "${GREEN}✅ Готово!${NC}"
