#!/bin/bash
# One-click install script for Liberty Reach Self-Hosting

set -e

echo "🚀 Liberty Reach Self-Hosting Installer"
echo "======================================="

# Проверка Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker не найден. Установите Docker сначала."
    exit 1
fi

# Проверка docker compose (новая версия) или docker-compose (старая)
if command -v docker compose &> /dev/null; then
    DOCKER_COMPOSE="docker compose"
    echo "✓ Docker Compose найден (docker compose)"
elif command -v docker-compose &> /dev/null; then
    DOCKER_COMPOSE="docker-compose"
    echo "✓ Docker Compose найден (docker-compose)"
else
    echo "❌ Docker Compose не найден. Установите Docker Compose сначала."
    exit 1
fi

echo "✓ Docker найден"

# Создание директорий
mkdir -p ./data
mkdir -p ./uploads

# Запрос конфигурации
echo ""
echo "=== Конфигурация ==="
read -p "Введите SERVER_ADDR (по умолчанию: 0.0.0.0:8008): " SERVER_ADDR
SERVER_ADDR=${SERVER_ADDR:-0.0.0.0:8008}

read -p "Введите ADMIN_WALLET (по умолчанию: 0x000...000): " ADMIN_WALLET
ADMIN_WALLET=${ADMIN_WALLET:-0x0000000000000000000000000000000000000000}

read -p "Введите JWT_SECRET (по умолчанию: сгенерировать случайный): " JWT_SECRET
if [ -z "$JWT_SECRET" ]; then
    JWT_SECRET=$(openssl rand -hex 32)
    echo "✓ Сгенерирован случайный JWT_SECRET"
fi

# Создание .env файла
cat > .env << EOF
SERVER_ADDR=$SERVER_ADDR
ADMIN_WALLET=$ADMIN_WALLET
JWT_SECRET=$JWT_SECRET
RUST_LOG=info
EOF

echo "✓ .env файл создан"
echo ""

# Запуск Docker Compose
echo "🔄 Запуск Liberty Reach сервера..."
$DOCKER_COMPOSE up -d

echo ""
echo "✅ Liberty Reach запущен!"
echo ""
echo "📬 API доступен:"
echo "   http://localhost:8008"
echo "   http://localhost:8008/health - проверка здоровья"
echo ""
echo "📖 Логи:"
echo "   $DOCKER_COMPOSE logs -f"
echo ""
echo "🛑 Остановка:"
echo "   $DOCKER_COMPOSE down"
echo ""
