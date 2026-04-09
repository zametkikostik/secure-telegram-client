# 🔐 Secure Messenger — Self-Hosted Deployment

**Enterprise-ready, one-click deployment for Ubuntu/Debian/Linux Mint**

---

## 🚀 Quick Start

### One-Click Install

```bash
# Download and run installer
curl -fsSL https://raw.githubusercontent.com/secure-telegram/client/main/self-hosting/scripts/install.sh | sudo bash

# OR clone and run manually
git clone https://github.com/secure-telegram/client.git
cd client/self-hosting/scripts
sudo ./install.sh
```

### Manual Deployment

```bash
# 1. Copy environment file
cp .env.example .env

# 2. Edit with your secrets
nano .env

# 3. Start all services
docker compose up -d

# 4. Check status
docker compose ps
docker compose logs -f backend
```

---

## 📁 Structure

```
self-hosting/
├── docker/
│   ├── Dockerfile              # Multi-stage build (Rust → minimal image)
│   └── config.example.toml     # Backend configuration template
├── nginx/
│   ├── nginx.conf              # Main Nginx config
│   └── conf.d/                 # Per-site configs (auto-created)
├── scripts/
│   └── install.sh              # One-click installer
├── docker-compose.yml          # Full stack: backend + postgres + redis + minio
├── .env.example                # Environment variables template
└── README.md                   # This file
```

---

## 🏗️ Architecture

```
                    ┌──────────────────────────────────────────┐
                    │                  Nginx                    │
                    │  Port 80/443 — Reverse Proxy + TLS       │
                    └─────────────────┬────────────────────────┘
                                      │
                    ┌─────────────────▼────────────────────────┐
                    │             Backend (Axum)               │
                    │  Port 3000 — Rust server                 │
                    │  • REST API + WebSocket                  │
                    │  • JWT Auth + Rate Limiting              │
                    │  • E2EE Message Relay                    │
                    └────┬──────────┬─────────────┬───────────┘
                         │          │             │
              ┌──────────▼┐  ┌─────▼─────┐  ┌───▼────────┐
              │ PostgreSQL │  │   Redis   │  │   MinIO    │
              │  Port 5432 │  │ Port 6379 │  │ Port 9000  │
              │           │  │           │  │            │
              │ • Users   │  │ • Cache   │  │ • Files    │
              │ • Chats   │  │ • Sessions│  │ • Images   │
              │ • Messages│  │ • Pub/Sub │  │ • Backups  │
              └───────────┘  └───────────┘  └────────────┘
```

---

## ⚙️ Configuration

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `POSTGRES_PASSWORD` | Database password | **CHANGE THIS** |
| `REDIS_PASSWORD` | Redis password | **CHANGE THIS** |
| `MINIO_SECRET_KEY` | MinIO secret key | **CHANGE THIS** |
| `JWT_SECRET` | JWT signing key (64+ chars) | **CHANGE THIS** |
| `RUST_LOG` | Log level | `info` |
| `RATE_LIMIT_PER_MINUTE` | API rate limit | `60` |
| `MAX_UPLOAD_SIZE_MB` | Max file upload | `50` |
| `CLOUDFLARE_WORKER_URL` | Push notifications URL | *(empty)* |

### Generate Secure Secrets

```bash
# PostgreSQL password
openssl rand -base64 32 | tr -d '/+=' | head -c 24

# Redis password
openssl rand -base64 32 | tr -d '/+=' | head -c 24

# MinIO secret key
openssl rand -base64 32 | tr -d '/+=' | head -c 32

# JWT secret (64 chars)
openssl rand -base64 64 | tr -d '/+=' | head -c 64
```

---

## 🐳 Docker Commands

###日常管理

```bash
# Start all services
docker compose up -d

# View logs
docker compose logs -f backend
docker compose logs -f postgres

# Stop all services
docker compose down

# Restart backend only
docker compose restart backend

# Update to latest version
docker compose pull
docker compose up -d --build
```

### Database

```bash
# Connect to PostgreSQL
docker compose exec postgres psql -U messenger -d messenger

# Backup database
docker compose exec postgres pg_dump -U messenger > backup.sql

# Restore database
cat backup.sql | docker compose exec -T postgres psql -U messenger
```

### Redis

```bash
# Connect to Redis
docker compose exec redis redis-cli -a "$REDIS_PASSWORD"

# Clear cache
docker compose exec redis redis-cli -a "$REDIS_PASSWORD" FLUSHDB
```

### MinIO

```bash
# Access MinIO console
open http://localhost:9001

# List buckets
docker compose exec minio mc ls myminio/

# Backup files
docker compose exec minio mc cp --recursive myminio/messenger-files /backup/
```

---

## 🔒 Security

### Production Checklist

- [ ] Change all passwords in `.env`
- [ ] Generate random JWT secret (64+ chars)
- [ ] Enable TLS (HTTPS) in Nginx
- [ ] Restrict database ports to localhost only
- [ ] Set up firewall (ufw/iptables)
- [ ] Enable automatic security updates
- [ ] Configure log rotation
- [ ] Set up monitoring (Prometheus + Grafana)
- [ ] Configure backups (daily database, weekly files)

### TLS/SSL with Let's Encrypt

```bash
# Install Certbot
apt install certbot python3-certbot-nginx

# Get certificate
certbot --nginx -d your-domain.com

# Auto-renewal (cron)
echo "0 3 * * * certbot renew --quiet" | crontab -
```

---

## 📊 Monitoring

### Health Checks

```bash
# Backend
curl http://localhost:3000/health

# PostgreSQL
docker compose exec postgres pg_isready -U messenger

# Redis
docker compose exec redis redis-cli -a "$REDIS_PASSWORD" ping

# MinIO
curl http://localhost:9000/minio/health/live
```

### Resource Usage

```bash
# Container stats
docker stats --format "table {{.Name}}\t{{.CPUPerc}}\t{{.MemUsage}}"

# Disk usage
docker system df
docker compose ps --format "table {{.Name}}\t{{.Status}}"
```

---

## 🔄 Updates

```bash
# Pull latest images
docker compose pull

# Rebuild backend
docker compose build backend

# Restart with new version
docker compose up -d

# Clean up old images
docker image prune -f
```

---

## 🆘 Troubleshooting

### Backend won't start

```bash
# Check logs
docker compose logs backend

# Check database connection
docker compose exec postgres pg_isready -U messenger

# Restart dependencies
docker compose restart postgres redis minio
```

### Database migration errors

```bash
# Run migrations manually
docker compose exec postgres psql -U messenger -d messenger -f /docker-entrypoint-initdb.d/01-init.sql
```

### High memory usage

```bash
# Reduce Redis maxmemory in .env
REDIS_MAX_MEMORY=128mb

# Reduce backend workers
# Edit Dockerfile: WORKERS=2
```

---

## 📝 License

MIT — Secure Telegram Team
