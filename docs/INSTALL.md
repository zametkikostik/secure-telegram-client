# 🚀 Инструкции по установке для Linux Mint/Ubuntu

## Системные зависимости

```bash
sudo apt update
sudo apt install -y \
  build-essential \
  cmake \
  pkg-config \
  libssl-dev \
  libgtk-3-dev \
  libwebkit2gtk-4.1-dev \
  libappindicator3-dev \
  librsvg2-dev \
  libclang-dev \
  protobuf-compiler \
  libsoup-3.0-dev \
  libjavascriptcoregtk-4.1-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev
```

## Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup install stable
```

## Node.js

```bash
curl -o- https://raw.githubusercontent.com/nvm-sh/nvm/v0.39.7/install.sh | bash
nvm install 18
```

## Python (для migration-tool)

```bash
sudo apt install python3.10 python3-pip python3-venv
```

## Сборка проекта

```bash
# 1. Rust workspace (без Tauri)
cargo check --workspace --exclude secure-messenger-tauri

# 2. Frontend
cd frontend
npm install
npm run build

# 3. Tauri app (требует системные библиотеки)
cargo tauri build
```

## Тестирование

```bash
# Crypto тесты
cargo test -p crypto

# Server тесты
cargo test -p secure-messenger-server

# Frontend тесты
cd frontend && npm test
```

## Деплой Cloudflare Workers

```bash
cd cloudflare
npm install
wrangler deploy
```
