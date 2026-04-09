#!/bin/bash
# Test script for Secure Telegram Client
# Usage: ./scripts/test.sh

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

echo "🧪 Running tests..."

# Rust tests
echo "📦 Running Rust tests..."
cd "$PROJECT_DIR"
cargo test --workspace

# Frontend tests
echo "🌐 Running frontend tests..."
cd "$PROJECT_DIR/frontend"
npm test -- --run

echo "✅ All tests passed!"
