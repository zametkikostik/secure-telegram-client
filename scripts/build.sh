#!/bin/bash
# Скрипт сборки Secure Telegram Client

set -e

echo "🔐 Secure Telegram Client - Сборка"
echo "=================================="

# Цвета для вывода
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Проверка зависимостей
check_dependencies() {
    echo -e "${YELLOW}Проверка зависимостей...${NC}"
    
    # Rust
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Rust не установлен${NC}"
        echo "Установите Rust: https://rustup.rs/"
        exit 1
    fi
    echo "✅ Rust: $(rustc --version)"
    
    # CMake
    if ! command -v cmake &> /dev/null; then
        echo -e "${RED}❌ CMake не установлен${NC}"
        exit 1
    fi
    echo "✅ CMake: $(cmake --version | head -1)"
    
    # Clang
    if ! command -v clang &> /dev/null; then
        echo -e "${YELLOW}⚠️ Clang не найден (может потребоваться для некоторых крипто-библиотек)${NC}"
    else
        echo "✅ Clang: $(clang --version | head -1)"
    fi
    
    # OpenSSL
    if ! pkg-config --exists openssl 2>/dev/null; then
        echo -e "${YELLOW}⚠️ OpenSSL dev пакеты не найдены${NC}"
    else
        echo "✅ OpenSSL: $(pkg-config --modversion openssl)"
    fi
}

# Сборка debug версии
build_debug() {
    echo -e "${YELLOW}Сборка debug версии...${NC}"
    cargo build
    echo -e "${GREEN}✅ Debug сборка завершена${NC}"
    echo "Бинарный файл: ./target/debug/secure-tg"
}

# Сборка release версии
build_release() {
    echo -e "${YELLOW}Сборка release версии...${NC}"
    cargo build --release
    echo -e "${GREEN}✅ Release сборка завершена${NC}"
    echo "Бинарный файл: ./target/release/secure-tg"
    
    # Strip бинарника
    if command -v strip &> /dev/null; then
        echo "Strip бинарного файла..."
        strip target/release/secure-tg 2>/dev/null || true
    fi
}

# Запуск тестов
run_tests() {
    echo -e "${YELLOW}Запуск тестов...${NC}"
    cargo test --verbose
    echo -e "${GREEN}✅ Тесты завершены${NC}"
}

# Проверка кода
lint() {
    echo -e "${YELLOW}Проверка кода (Clippy)...${NC}"
    cargo clippy -- -D warnings
    echo -e "${GREEN}✅ Проверка завершена${NC}"
}

# Форматирование
format() {
    echo -e "${YELLOW}Форматирование кода...${NC}"
    cargo fmt
    echo -e "${GREEN}✅ Форматирование завершено${NC}"
}

# Очистка
clean() {
    echo -e "${YELLOW}Очистка...${NC}"
    cargo clean
    echo -e "${GREEN}✅ Очистка завершена${NC}"
}

# Создание конфига по умолчанию
init_config() {
    echo -e "${YELLOW}Создание конфигурации...${NC}"
    
    if [ ! -f config.json ]; then
        cat > config.json << EOF
{
  "api_id": 0,
  "api_hash": "YOUR_API_HASH_HERE",
  "encryption": {
    "kyber_enabled": true,
    "steganography_enabled": true,
    "obfuscation_enabled": true,
    "auto_steganography": true
  },
  "proxy": {
    "enabled": false,
    "host": "127.0.0.1",
    "port": 1080,
    "proxy_type": "socks5"
  },
  "auto_update": true
}
EOF
        echo -e "${GREEN}✅ Конфигурация создана: config.json${NC}"
        echo -e "${YELLOW}⚠️ Не забудьте установить api_id и api_hash!${NC}"
    else
        echo -e "${YELLOW}⚠️ config.json уже существует${NC}"
    fi
}

# Помощь
show_help() {
    echo "Использование: $0 [команда]"
    echo ""
    echo "Команды:"
    echo "  debug       Сборка debug версии"
    echo "  release     Сборка release версии"
    echo "  test        Запуск тестов"
    echo "  lint        Проверка кода (Clippy)"
    echo "  format      Форматирование кода"
    echo "  clean       Очистка"
    echo "  init        Инициализация конфигурации"
    echo "  all         Полная сборка (lint + test + release)"
    echo "  help        Эта справка"
    echo ""
    echo "По умолчанию: release"
}

# Основная логика
main() {
    check_dependencies
    echo ""
    
    case "${1:-release}" in
        debug)
            build_debug
            ;;
        release)
            build_release
            ;;
        test)
            run_tests
            ;;
        lint)
            lint
            ;;
        format)
            format
            ;;
        clean)
            clean
            ;;
        init)
            init_config
            ;;
        all)
            format
            lint
            run_tests
            build_release
            ;;
        help|--help|-h)
            show_help
            ;;
        *)
            echo -e "${RED}❌ Неизвестная команда: $1${NC}"
            show_help
            exit 1
            ;;
    esac
}

main "$@"
