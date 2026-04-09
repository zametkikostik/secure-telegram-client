# ============================================================================
# Cloudflare Configuration — Secure Messenger
# ============================================================================
# Custom Domain: messenger.your-domain.com
# SSL: Full (strict)
# WAF Rules: OWASP + custom rules
# ============================================================================

## 1. DNS Records
## ============================================================================
# Type  Name                    Content                     Proxy   TTL
# A     messenger               <backend-server-ip>         DNS Only  Auto
# CNAME api                     messenger.your-domain.com   Proxied   Auto
# CNAME ws                      messenger.your-domain.com   Proxied   Auto
# TXT     @                     "v=spf1 include:_spf.google.com ~all"
# MX      @                     mail.your-domain.com        10

## 2. SSL/TLS Settings
## ============================================================================
# Mode: Full (strict)
# Minimum TLS Version: TLS 1.2
# Certificate: Let's Encrypt (auto-renew via Cloudflare)
# HSTS: Enabled (max-age=31536000, includeSubDomains, preload)
# Opportunistic Encryption: On
# TLS 1.3: Enabled
# Always Use HTTPS: On

## 3. WAF Rules
## ============================================================================

### Rule 1: Block SQL Injection
# Expression:
contains(http.request.uri, "union") or
contains(http.request.uri, "select") or
contains(http.request.uri, "insert") or
contains(http.request.uri, "delete") or
contains(http.request.uri, "drop") or
contains(http.request.uri, "--") or
contains(http.request.uri, ";")
# Action: Block

### Rule 2: Block XSS
# Expression:
contains(http.request.uri, "<script") or
contains(http.request.uri, "javascript:") or
contains(http.request.uri, "onerror=") or
contains(http.request.uri, "onload=")
# Action: Block

### Rule 3: Rate Limiting (API)
# Expression:
(http.request.uri.path matches "^/api/") and
(ip.geoip.country ne "US" and ip.geoip.country ne "DE" and ip.geoip.country ne "RU")
# Action: Challenge (CAPTCHA)
# Rate: 100 requests per 10 minutes per IP

### Rule 4: Block Bad Bots
# Expression:
(http.user_agent contains "bot" and http.user_agent does not contain "Googlebot") or
(http.user_agent contains "crawler") or
(http.user_agent contains "scraper")
# Action: Block

### Rule 5: WebSocket Protection
# Expression:
(http.request.uri.path matches "^/ws" and http.request.method ne "GET")
# Action: Block

### Rule 6: Cloudflare Managed Rules
# Enable OWASP ModSecurity Core Rule Set
# Enable Cloudflare Managed Ruleset
# Enable Cloudflare Exposed Credentials Check

## 4. Caching Rules
## ============================================================================
# Bypass cache for:
# - /api/* (dynamic API responses)
# - /ws/* (WebSocket connections)
# - /health (health checks)

# Cache static assets:
# - /static/* (images, JS, CSS) — Cache TTL: 1 year
# - /assets/* — Cache TTL: 1 month

## 5. Page Rules
## ============================================================================
# Rule 1: messenger.your-domain.com/api/*
#   - Cache Level: Bypass
#   - Security Level: High
#   - Disable Apps

# Rule 2: messenger.your-domain.com/health
#   - Cache Level: Cache Everything
#   - Edge Cache TTL: 10 seconds

# Rule 3: messenger.your-domain.com/ws/*
#   - Cache Level: Bypass
#   - Disable Apps
#   - Disable Performance (WebSocket optimization)

## 6. Edge Certificates
## ============================================================================
# Certificate Authority: Let's Encrypt
# Validity: 90 days (auto-renew)
# Minimum TLS Version: 1.2
# TLS 1.3: Enabled
# HSTS:
#   - Max-Age: 31536000 (1 year)
#   - Include SubDomains: Yes
#   - Preload: Yes
#   - No-Sniff: Yes

## 7. Network Settings
## ============================================================================
# HTTP/2: Enabled
# HTTP/3 (with QUIC): Enabled
# gRPC: Enabled (for API)
# WebSockets: Enabled
# Onion Routing: Enabled
# Zero RTT: Disabled (security)
# IP Geolocation: Enabled
# Pseudo IPv4: Off
# Maximum Upload Size: 100 MB

## 8. Bot Management
## ============================================================================
# Enable Bot Fight Mode
# Block known bad bots
# Allow verified bots (Google, Bing, etc.)
# Rate limit unknown bots

## 9. Access (Zero Trust)
## ============================================================================
# Admin Panel Protection:
#   - URL: messenger.your-domain.com/admin/*
#   - Policy: Require SSO (SAML/OAuth2)
#   - Allowed: Company email domain only
#   - MFA: Required

# API Rate Limiting:
#   - URL: messenger.your-domain.com/api/*
#   - Limit: 1000 requests per hour per user
#   - Action: 429 Too Many Requests

## 10. Monitoring & Alerts
## ============================================================================
# Alert 1: 5xx error rate > 1%
#   - Notification: Email + Slack
# Alert 2: Origin server unreachable
#   - Notification: Email + PagerDuty
# Alert 3: WAF triggered > 100 times/hour
#   - Notification: Email + Slack
# Alert 4: SSL certificate expiry < 7 days
#   - Notification: Email

## 11. Cloudflare Worker (Edge Logic)
## ============================================================================
# Deploy worker for:
# - Custom rate limiting (per-user)
# - Request logging to external SIEM
# - Custom authentication header injection
# - Geographic blocking (if needed)
