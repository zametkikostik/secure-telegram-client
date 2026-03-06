#!/bin/bash
# Скрипт сборки Enterprise сервера

set -e

echo "🚀 Сборка Secure Telegram Enterprise"

# Проверка зависимостей
echo "📦 Проверка зависимостей..."
REQUIRED_DEPS="cargo pkg-config"
for dep in $REQUIRED_DEPS; do
    if ! command -v $dep &> /dev/null; then
        echo "❌ Требуется $dep но не найден"
        exit 1
    fi
done

# Установка зависимостей для Enterprise
echo "📦 Установка системных зависимостей..."
sudo apt-get update
sudo apt-get install -y \
    libssl-dev \
    pkg-config \
    libpq-dev \
    libldap2-dev \
    libsasl2-dev \
    cmake \
    build-essential

# Сборка
echo "🦀 Сборка Rust..."
cargo build --release

# Создание пакета
echo "📦 Создание пакета..."
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
ARCH=$(dpkg --print-architecture)

mkdir -p ../releases
mkdir -p debian/DEBIAN
mkdir -p debian/usr/bin
mkdir -p debian/etc/secure-telegram
mkdir -p debian/var/log/secure-telegram
mkdir -p debian/lib/systemd/system

# Копирование бинарника
cp target/release/secure-telegram-enterprise debian/usr/bin/

# Копирование конфигов
cp config/*.toml debian/etc/secure-telegram/

# Создание systemd service
cat > debian/lib/systemd/system/secure-telegram-enterprise.service << EOF
[Unit]
Description=Secure Telegram Enterprise Server
After=network.target postgresql.service redis.service

[Service]
Type=notify
User=secure-telegram
Group=secure-telegram
WorkingDirectory=/usr/bin
ExecStart=/usr/bin/secure-telegram-enterprise
Restart=on-failure
RestartSec=5
Environment=RUST_LOG=info
Environment=DATABASE_URL=postgresql://enterprise:password@localhost/secure_telegram
Environment=REDIS_URL=redis://localhost:6379

# Security
NoNewPrivileges=true
ProtectSystem=strict
ProtectHome=true
ReadWritePaths=/var/log/secure-telegram

[Install]
WantedBy=multi-user.target
EOF

# Создание control файла
cat > debian/DEBIAN/control << EOF
Package: secure-telegram-enterprise
Version: ${VERSION}
Section: net
Priority: optional
Architecture: ${ARCH}
Depends: postgresql, redis-server, libssl3, libldap-2.5-0
Maintainer: Secure Telegram Team <enterprise@secure-telegram.io>
Description: Secure Telegram Enterprise Server
 Корпоративная версия мессенджера
 - SSO (OAuth2, SAML, LDAP)
 - Централизованный аудит
 - Админ-панель
 - Compliance (GDPR)
EOF

# Создание postinst скрипта
cat > debian/DEBIAN/postinst << EOF
#!/bin/bash
set -e

# Создание пользователя
if ! id -u secure-telegram &>/dev/null; then
    useradd -r -s /bin/false secure-telegram
fi

# Создание директорий
mkdir -p /var/log/secure-telegram
chown secure-telegram:secure-telegram /var/log/secure-telegram

mkdir -p /etc/secure-telegram
chown secure-telegram:secure-telegram /etc/secure-telegram

# Копирование конфигурации по умолчанию
if [ ! -f /etc/secure-telegram/config.toml ]; then
    cp /usr/share/doc/secure-telegram-enterprise/config.toml.example /etc/secure-telegram/config.toml
fi

# Reload systemd
systemctl daemon-reload

echo "✅ Установка завершена!"
echo "Отредактируйте /etc/secure-telegram/config.toml"
echo "Запуск: systemctl start secure-telegram-enterprise"
echo "Автозапуск: systemctl enable secure-telegram-enterprise"
EOF
chmod +x debian/DEBIAN/postinst

# Создание postrm скрипта
cat > debian/DEBIAN/postrm << EOF
#!/bin/bash
if [ "$1" = "purge" ]; then
    rm -rf /var/log/secure-telegram
    rm -rf /etc/secure-telegram
fi
EOF
chmod +x debian/DEBIAN/postrm

# Сборка .deb
fakeroot dpkg-deb --build debian ../releases/secure-telegram-enterprise_${VERSION}_${ARCH}.deb

echo ""
echo "✅ Сборка завершена!"
echo "📦 Пакет: ../releases/secure-telegram-enterprise_${VERSION}_${ARCH}.deb"
echo ""
echo "Установка:"
echo "  sudo dpkg -i ../releases/secure-telegram-enterprise_${VERSION}_${ARCH}.deb"
echo ""
echo "Настройка:"
echo "  sudo nano /etc/secure-telegram/config.toml"
echo ""
echo "Запуск:"
echo "  sudo systemctl start secure-telegram-enterprise"
echo "  sudo systemctl enable secure-telegram-enterprise"
