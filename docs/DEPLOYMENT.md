# 🚀 Deployment Guide — Secure Messenger

**Version:** 1.0  
**Last Updated:** April 8, 2026  
**Target Audience:** DevOps engineers, system administrators

---

## 📋 Table of Contents

1. [Prerequisites](#prerequisites)
2. [Architecture Overview](#architecture-overview)
3. [Quick Start (One-Click)](#quick-start-one-click)
4. [Manual Deployment](#manual-deployment)
5. [Cloudflare Setup](#cloudflare-setup)
6. [SSL/TLS Configuration](#ssltls-configuration)
7. [Database Migration](#database-migration)
8. [Monitoring Setup](#monitoring-setup)
9. [CI/CD Pipeline](#cicd-pipeline)
10. [Backup & Recovery](#backup--recovery)
11. [Troubleshooting](#troubleshooting)
12. [Production Checklist](#production-checklist)

---

## Prerequisites

### **Minimum Requirements**

| Component | Specification | Notes |
|-----------|--------------|-------|
| **CPU** | 4 cores | 8+ cores recommended |
| **RAM** | 8 GB | 16 GB for production |
| **Disk** | 50 GB SSD | 100 GB+ for production |
| **Network** | 100 Mbps | 1 Gbps+ for high traffic |
| **OS** | Ubuntu 22.04+ / Debian 12+ | Other Linux with Docker |

### **Software**

| Software | Version | Purpose |
|----------|---------|---------|
| Docker | 24.0+ | Container runtime |
| Docker Compose | 2.20+ | Multi-container orchestration |
| Rust | 1.75+ | Backend compilation |
| Node.js | 20+ | Frontend build |
| Git | 2.40+ | Version control |

### **External Services**

| Service | Purpose | Required |
|---------|---------|----------|
| Cloudflare | CDN, WAF, DNS | ✅ Yes |
| Let's Encrypt | TLS certificates | ✅ Yes |
| GitHub | CI/CD, code hosting | ✅ Yes |
| SMTP Server | Email notifications | ⚠️ Optional |
| PagerDuty | Alerting | ⚠️ Optional |

---

## Architecture Overview

```
                              ┌─────────────────────────────────┐
                              │          Cloudflare CDN          │
                              │  • DNS • WAF • SSL • Rate Limit  │
                              └───────────────┬─────────────────┘
                                              │
                    ┌─────────────────────────▼─────────────────────────┐
                    │                    Nginx                           │
                    │          Reverse Proxy + TLS Termination           │
                    └──────────┬──────────────┬─────────────┬───────────┘
                               │              │             │
              ┌────────────────▼┐   ┌────────▼──────┐  ┌───▼────────────┐
              │   Backend API   │   │  WebSocket    │  │  Static Files  │
              │   (Axum/Rust)   │   │  Signaling    │  │   (MinIO)      │
              │   Port 3000     │   │  Port 3001    │  │   Port 9000    │
              └────┬──────┬─────┘   └───────┬───────┘  └────────────────┘
                   │      │                 │
          ┌────────▼┐  ┌─▼────────┐  ┌──────▼──────┐
          │PostgreSQL│  │  Redis   │  │ Prometheus  │
          │Port 5432 │  │Port 6379 │  │  Port 9090  │
          └──────────┘  └──────────┘  └─────────────┘
```

---

## Quick Start (One-Click)

### **Automated Installation**

```bash
# One-liner for Ubuntu/Debian
curl -fsSL https://raw.githubusercontent.com/secure-telegram/client/main/self-hosting/scripts/install.sh | sudo bash

# Or download and run manually
wget https://raw.githubusercontent.com/secure-telegram/client/main/self-hosting/scripts/install.sh
chmod +x install.sh
sudo ./install.sh
```

### **What the Installer Does:**

1. ✅ Checks system requirements (RAM, disk, architecture)
2. ✅ Installs Docker + Docker Compose
3. ✅ Generates secure passwords (openssl rand)
4. ✅ Creates `.env` file with secrets
5. ✅ Configures Nginx reverse proxy
6. ✅ Creates PostgreSQL init scripts
7. ✅ Builds and starts all containers
8. ✅ Verifies health checks

---

## Manual Deployment

### **Step 1: Clone Repository**

```bash
git clone https://github.com/secure-telegram/client.git
cd client
```

### **Step 2: Configure Environment**

```bash
cd self-hosting
cp .env.example .env

# Generate secrets
POSTGRES_PASSWORD=$(openssl rand -base64 32 | tr -d '/+=' | head -c 24)
REDIS_PASSWORD=$(openssl rand -base64 32 | tr -d '/+=' | head -c 24)
MINIO_SECRET=$(openssl rand -base64 32 | tr -d '/+=' | head -c 32)
JWT_SECRET=$(openssl rand -base64 64 | tr -d '/+=' | head -c 64)

# Edit .env with your values
nano .env
```

### **Step 3: Build and Start**

```bash
# Build all images
docker compose build

# Start services
docker compose up -d

# Check status
docker compose ps

# View logs
docker compose logs -f backend
```

### **Step 4: Verify Installation**

```bash
# Health check
curl http://localhost:3000/health

# Expected response:
# {"status":"ok","version":"0.1.0","uptime":123}

# Check database
docker compose exec postgres pg_isready -U messenger

# Check Redis
docker compose exec redis redis-cli -a "$REDIS_PASSWORD" ping
```

---

## Cloudflare Setup

### **1. Add Domain**

1. Log in to Cloudflare Dashboard
2. Add Site → Enter your domain
3. Cloudflare scans DNS records automatically

### **2. Configure DNS**

```
Type   Name     Content                Proxy   TTL
A      @        <your-server-ip>       Proxied Auto
CNAME  api      your-domain.com        Proxied Auto
CNAME  ws       your-domain.com        Proxied Auto
```

### **3. SSL/TLS Settings**

- **Mode**: Full (strict)
- **Minimum TLS Version**: 1.2
- **HSTS**: Enabled (max-age=31536000)
- **Always Use HTTPS**: On

### **4. WAF Rules**

See [Cloudflare Configuration](./cloudflare/CONFIGURATION.md) for detailed WAF rules.

### **5. Page Rules**

```
URL Pattern: your-domain.com/api/*
Settings:
  - Cache Level: Bypass
  - Security Level: High
```

---

## SSL/TLS Configuration

### **Let's Encrypt with Certbot**

```bash
# Install Certbot
sudo apt install certbot python3-certbot-nginx

# Get certificate
sudo certbot --nginx -d your-domain.com -d api.your-domain.com

# Auto-renewal
sudo systemctl enable certbot-renew.timer
sudo systemctl start certbot-renew.timer

# Test renewal
sudo certbot renew --dry-run
```

### **Manual Certificate (if not using Certbot)**

```bash
# Place certificates
sudo mkdir -p /etc/nginx/ssl
sudo cp fullchain.pem /etc/nginx/ssl/
sudo cp privkey.pem /etc/nginx/ssl/
sudo chmod 600 /etc/nginx/ssl/*

# Update Nginx config
# ssl_certificate /etc/nginx/ssl/fullchain.pem;
# ssl_certificate_key /etc/nginx/ssl/privkey.pem;
```

---

## Database Migration

### **Initial Schema**

The database schema is created automatically on first start via `init-scripts/01-init.sql`.

### **Manual Migration**

```bash
# Connect to database
docker compose exec postgres psql -U messenger -d messenger

# Run migration file
\i /docker-entrypoint-initdb.d/02-add-indexes.sql

# Verify
\dt  # List tables
\di  # List indexes
```

### **Backup**

```bash
# Full database backup
docker compose exec postgres pg_dump -U messenger > backup-$(date +%Y%m%d).sql

# Compress
gzip backup-$(date +%Y%m%d).sql

# Verify backup
gunzip -c backup-*.sql.gz | psql -U messenger -d messenger_test
```

### **Restore**

```bash
# Stop application
docker compose stop backend

# Restore
gunzip -c backup.sql.gz | docker compose exec -T postgres psql -U messenger

# Restart
docker compose start backend
```

---

## Monitoring Setup

### **Start Monitoring Stack**

```bash
cd self-hosting/monitoring
docker compose -f docker-compose.monitoring.yml up -d
```

### **Access Dashboards**

| Service | URL | Credentials |
|---------|-----|-------------|
| **Grafana** | http://localhost:3001 | admin / changeme |
| **Prometheus** | http://localhost:9090 | No auth |
| **Alertmanager** | http://localhost:9093 | No auth |

### **Configure Alerts**

1. Open Alertmanager: http://localhost:9093
2. Edit `alertmanager/alertmanager.yml`
3. Add your notification channels:
   - Email (SMTP credentials in .env)
   - Slack (webhook URL)
   - PagerDuty (service key)

```bash
# Reload Alertmanager config
curl -X POST http://localhost:9093/-/reload
```

### **Import Grafana Dashboard**

The dashboard is auto-provisioned. To customize:

1. Open Grafana → Dashboards → "Secure Messenger — Production"
2. Edit panels as needed
3. Save changes

---

## CI/CD Pipeline

### **GitHub Actions Workflow**

The pipeline is defined in `.github/workflows/ci-cd.yml`.

**Triggers:**
- Push to `main` → Test + Build + Deploy staging
- Tag (`v*`) → Test + Build + Sign + Release
- Pull Request → Test + Lint
- Schedule (weekly) → Security scan

**Jobs:**
1. **test** — Run all tests (Rust + Smart Contracts)
2. **lint** — Clippy + fmt + Solidity linter
3. **security** — cargo-audit + Slither + npm audit
4. **build** — Cross-compile for Linux, macOS, Windows
5. **sign** — Sign binaries with cosign
6. **release** — Create GitHub Release
7. **deploy-staging** — Deploy to staging server

### **Required Secrets**

| Secret | Description | Where to Set |
|--------|-------------|--------------|
| `STAGING_SSH_KEY` | SSH key for staging | GitHub → Settings → Secrets |
| `STAGING_HOST` | staging.messenger.app | GitHub → Settings → Secrets |
| `DOCKER_USERNAME` | Docker Hub username | GitHub → Settings → Secrets |
| `DOCKER_PASSWORD` | Docker Hub token | GitHub → Settings → Secrets |

### **Create a Release**

```bash
# Tag the release
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions will:
# 1. Run all tests
# 2. Build binaries
# 3. Sign with cosign
# 4. Create GitHub Release
# 5. Publish Docker image
```

---

## Backup & Recovery

### **Automated Backups**

```bash
#!/bin/bash
# backup.sh — Run daily via cron

DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_DIR="/backups/messenger"

# Database backup
docker compose exec postgres pg_dump -U messenger | gzip > "$BACKUP_DIR/db-$DATE.sql.gz"

# MinIO files backup
docker compose exec minio mc cp --recursive myminio/messenger-files "$BACKUP_DIR/minio-$DATE/"

# Config backup
cp .env "$BACKUP_DIR/env-$DATE.bak"

# Keep last 30 days
find "$BACKUP_DIR" -name "*.gz" -mtime +30 -delete
find "$BACKUP_DIR" -name "*.bak" -mtime +30 -delete
```

### **Cron Job**

```bash
# Add to crontab (daily at 3 AM)
0 3 * * * /opt/secure-messenger/scripts/backup.sh
```

### **Recovery Procedure**

1. **Stop services**: `docker compose down`
2. **Restore database**: `gunzip -c db-backup.sql.gz | docker compose exec -T postgres psql -U messenger`
3. **Restore files**: `docker compose exec minio mc cp --recursive backup/ myminio/messenger-files`
4. **Start services**: `docker compose up -d`
5. **Verify**: `curl http://localhost:3000/health`

---

## Troubleshooting

### **Backend Won't Start**

```bash
# Check logs
docker compose logs backend

# Common issues:
# - Database not ready: wait for postgres health check
# - Wrong credentials: verify .env values
# - Port already in use: lsof -i :3000

# Restart
docker compose restart backend
```

### **High Memory Usage**

```bash
# Check container memory
docker stats

# Reduce Redis maxmemory
# Edit .env: REDIS_MAX_MEMORY=128mb
docker compose up -d redis

# Restart backend with fewer workers
# Edit Dockerfile: WORKERS=2
docker compose build backend
docker compose up -d backend
```

### **SSL Certificate Issues**

```bash
# Check certificate
openssl s_client -connect your-domain.com:443 -servername your-domain.com

# Renew certificate
sudo certbot renew --force-renewal

# Check Nginx config
sudo nginx -t
```

### **Smart Contract Issues**

```bash
# Re-run tests
cd smart-contracts
npx hardhat test

# Check for vulnerabilities
npx hardhat run scripts/audit.js

# Redeploy contracts
npx hardhat run scripts/deploy.js --network sepolia
```

---

## Production Checklist

### **Before Going Live**

- [ ] **Security**
  - [ ] All passwords changed from defaults
  - [ ] TLS certificate valid and auto-renewing
  - [ ] WAF rules enabled and tested
  - [ ] Rate limiting configured
  - [ ] Dependencies audited (cargo-audit, npm audit)

- [ ] **Database**
  - [ ] Automated backups configured
  - [ ] Connection pooling enabled
  - [ ] Slow query logging enabled
  - [ ] Indexes verified

- [ ] **Monitoring**
  - [ ] Prometheus scraping all targets
  - [ ] Grafana dashboard imported
  - [ ] Alert rules active
  - [ ] Notification channels tested

- [ ] **CI/CD**
  - [ ] All secrets configured in GitHub
  - [ ] Staging environment working
  - [ ] Release process tested
  - [ ] Rollback procedure documented

- [ ] **Compliance**
  - [ ] GDPR checklist completed
  - [ ] 152-ФЗ checklist completed
  - [ ] Privacy policy published
  - [ ] Terms of service published

- [ ] **Documentation**
  - [ ] SECURITY.md reviewed
  - [ ] AUDIT.md reviewed
  - [ ] Runbooks created
  - [ ] Incident response plan

---

## 📞 Support

| Channel | Contact |
|---------|---------|
| GitHub Issues | https://github.com/secure-telegram/client/issues |
| Email | zametkikostik@gmail.com |
| Telegram | @kostik_support |

---

**This deployment guide should be reviewed before each production deployment.**
