#!/bin/bash
# Manual deploy using Cloudflare API v4
# This bypasses wrangler upload issues

ACCOUNT_ID="9d3f70325c3f26a70c09c2d13b981f3c"
WORKER_NAME="secure-messenger-push"

echo "🚀 Starting manual deploy via API..."

# First, get API token info
echo "📡 Checking authentication..."
TOKEN_INFO=$(wrangler whoami 2>&1)
echo "$TOKEN_INFO" | head -10

echo ""
echo "⚠️  Wrangler upload seems to hang. Let's try alternative approaches:"
echo ""
echo "Option 1: Use wrangler deploy --keep-vars"
echo "Option 2: Deploy via Cloudflare Dashboard"
echo "Option 3: Use curl with API token"
echo ""

# Try with verbose logging
echo "📦 Attempting upload with verbose logging..."
cd /home/kostik/secure-messenger/secure-telegram-client/cloudflare/worker
timeout 60 wrangler deploy --verbose 2>&1 | tail -30

if [ $? -eq 124 ]; then
    echo ""
    echo "❌ Upload timed out."
    echo ""
    echo "📋 MANUAL DEPLOY INSTRUCTIONS:"
    echo "==============================="
    echo "1. Open: https://dash.cloudflare.com/"
    echo "2. Go to: Workers & Pages → secure-messenger-push"
    echo "3. Click: 'Edit Code'"
    echo "4. Copy content from: src/worker.js"
    echo "5. Paste and click: 'Save and Deploy'"
    echo ""
    echo "Worker file location:"
    echo "/home/kostik/secure-messenger/secure-telegram-client/cloudflare/worker/src/worker.js"
    echo ""
    echo "After deploy, test with:"
    echo "curl https://secure-messenger-push.kostik.workers.dev/health"
fi
