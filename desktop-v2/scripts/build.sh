#!/bin/bash
# Скрипт сборки Desktop v2.0 для Linux Mint

set -e

echo "🚀 Сборка Secure Telegram Desktop v2.0 для Linux Mint"

# Проверка зависимостей
echo "📦 Проверка зависимостей..."
REQUIRED_DEPS="cargo npm dpkg-deb fakeroot"
for dep in $REQUIRED_DEPS; do
    if ! command -v $dep &> /dev/null; then
        echo "❌ Требуется $dep но не найден"
        exit 1
    fi
done

# Установка зависимостей для Linux Mint
echo "📦 Установка зависимостей..."
sudo apt-get update
sudo apt-get install -y \
    libgtk-3-dev \
    libwebkit2gtk-4.0-dev \
    libappindicator3-dev \
    librsvg2-dev \
    libnotify-dev \
    libsecret-1-dev \
    libssl-dev \
    pkg-config \
    build-essential

# Сборка frontend
echo "🔨 Сборка frontend..."
cd ../frontend
npm install
npm run build
cd ../desktop-v2

# Сборка Rust
echo "🦀 Сборка Rust..."
cargo build --release

# Создание .deb пакета
echo "📦 Создание .deb пакета..."
VERSION=$(grep '^version' Cargo.toml | head -1 | cut -d'"' -f2)
ARCH=$(dpkg --print-architecture)
DEB_FILE="../releases/secure-telegram-desktop_${VERSION}_${ARCH}.deb"

mkdir -p ../releases
mkdir -p debian/DEBIAN
mkdir -p debian/usr/bin
mkdir -p debian/usr/share/applications
mkdir -p debian/usr/share/icons/hicolor/256x256/apps

# Копирование бинарника
cp target/release/secure-telegram-desktop debian/usr/bin/

# Копирование .desktop файла
cp scripts/io.secure-telegram.desktop debian/usr/share/applications/

# Копирование иконки
cp icons/256x256.png debian/usr/share/icons/hicolor/256x256/apps/io.secure-telegram.png

# Создание control файла
cat > debian/DEBIAN/control << EOF
Package: secure-telegram-desktop
Version: ${VERSION}
Section: net
Priority: optional
Architecture: ${ARCH}
Depends: libgtk-3-0, libwebkit2gtk-4.0-37, libappindicator3-1, librsvg2-common, libnotify4, libsecret-1-0
Maintainer: Secure Telegram Team <support@secure-telegram.io>
Description: Secure Telegram Client Desktop v2.0
 Приватный мессенджер с post-quantum шифрованием
 Оптимизирован для Linux Mint
EOF

# Создание postinst скрипта
cat > debian/DEBIAN/postinst << EOF
#!/bin/bash
update-desktop-database
update-icon-caches
EOF
chmod +x debian/DEBIAN/postinst

# Создание postrm скрипта
cat > debian/DEBIAN/postrm << EOF
#!/bin/bash
update-desktop-database
update-icon-caches
EOF
chmod +x debian/DEBIAN/postrm

# Сборка .deb
fakeroot dpkg-deb --build debian ../releases/secure-telegram-desktop_${VERSION}_${ARCH}.deb

echo "✅ Сборка завершена!"
echo "📦 Пакет: $DEB_FILE"
echo ""
echo "Установка:"
echo "  sudo dpkg -i $DEB_FILE"
echo ""
echo "Запуск:"
echo "  secure-telegram-desktop"
