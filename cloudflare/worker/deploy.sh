#!/bin/bash
# Deploy script for Cloudflare Worker
# Usage: ./deploy.sh

set -e

echo "🚀 Deploying Secure Messenger Cloudflare Worker..."

# Check if wrangler is installed
if ! command -v wrangler &> /dev/null; then
    echo "❌ wrangler is not installed. Install with: npm install -g wrangler"
    exit 1
fi

# Check if logged in
echo "📡 Checking Cloudflare authentication..."
if ! wrangler whoami &> /dev/null; then
    echo "🔐 Not authenticated. Please login..."
    wrangler login
fi

# Install dependencies
echo "📦 Installing dependencies..."
npm install --production

# Deploy
echo "🚀 Deploying to Cloudflare..."
wrangler deploy --env production

echo ""
echo "✅ Deployment complete!"
echo "🌐 Worker URL: https://secure-messenger-push.kostik.workers.dev"
echo "🧪 Test with: curl https://secure-messenger-push.kostik.workers.dev/health"
echo ""
echo "📊 View logs: wrangler tail"
echo "📈 View metrics: wrangler metrics"
