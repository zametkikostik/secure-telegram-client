#!/bin/bash
# Build script for Secure Telegram Client
# Usage: ./scripts/build.sh [--release]

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "🔨 Building Secure Telegram Client..."

# Build Rust workspace
echo "📦 Building Rust workspace..."
cd "$PROJECT_DIR"
if [[ "$1" == "--release" ]]; then
    cargo build --release
else
    cargo build
fi

# Build frontend
echo "🌐 Building frontend..."
cd "$PROJECT_DIR/frontend"
npm install
npm run build

# Build Tauri app
echo "🖥️ Building Tauri app..."
cd "$PROJECT_DIR/messenger"
if [[ "$1" == "--release" ]]; then
    cargo tauri build
else
    cargo tauri dev
fi

echo "✅ Build complete!"
