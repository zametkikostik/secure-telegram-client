#!/bin/bash
# ============================================================================
# Secure Messenger — One-Click Self-Hosted Installer
# ============================================================================
# Supported OS: Ubuntu 20.04+, Debian 11+, Linux Mint 20+
#
# Usage:
#   curl -fsSL https://raw.githubusercontent.com/secure-telegram/client/main/self-hosting/install.sh | bash
#   OR
#   wget -qO- https://.../install.sh | bash
#   OR
#   chmod +x install.sh && ./install.sh
#
# What it does:
# 1. Checks system requirements
# 2. Installs Docker + Docker Compose
# 3. Configures environment
# 4. Starts all services
# 5. Verifies installation
# ============================================================================

set -euo pipefail

# ============================================================================
# Colors and formatting
# ============================================================================
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color
BOLD='\033[1m'

# ============================================================================
# Logging functions
# ============================================================================
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_error() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "\n${BOLD}${CYAN}▸ $1${NC}"; }

# ============================================================================
# Check if running as root
# ============================================================================
check_root() {
    if [ "$EUID" -ne 0 ]; then
        log_error "Please run as root (use sudo)"
        exit 1
    fi
}

# ============================================================================
# Detect OS and distribution
# ============================================================================
detect_os() {
    if [ -f /etc/os-release ]; then
        . /etc/os-release
        OS_NAME=$NAME
        OS_VERSION=$VERSION_ID
        OS_ID=$ID
    else
        log_error "Cannot detect OS"
        exit 1
    fi

    log_info "Detected: $OS_NAME $OS_VERSION ($OS_ID)"
}

# ============================================================================
# Check system requirements
# ============================================================================
check_requirements() {
    log_step "Checking system requirements..."

    # Check RAM (minimum 2GB)
    TOTAL_RAM_KB=$(grep MemTotal /proc/meminfo | awk '{print $2}')
    TOTAL_RAM_MB=$((TOTAL_RAM_KB / 1024))

    if [ "$TOTAL_RAM_MB" -lt 2048 ]; then
        log_warn "Low RAM detected: ${TOTAL_RAM_MB}MB (recommended: 4GB+)"
    else
        log_success "RAM: ${TOTAL_RAM_MB}MB"
    fi

    # Check disk space (minimum 10GB free)
    FREE_DISK_KB=$(df -k / | awk 'NR==2 {print $4}')
    FREE_DISK_GB=$((FREE_DISK_KB / 1024 / 1024))

    if [ "$FREE_DISK_GB" -lt 10 ]; then
        log_warn "Low disk space: ${FREE_DISK_GB}GB free (recommended: 20GB+)"
    else
        log_success "Disk: ${FREE_DISK_GB}GB free"
    fi

    # Check architecture
    ARCH=$(uname -m)
    if [[ "$ARCH" != "x86_64" && "$ARCH" != "aarch64" ]]; then
        log_error "Unsupported architecture: $ARCH (need x86_64 or aarch64)"
        exit 1
    fi
    log_success "Architecture: $ARCH"
}

# ============================================================================
# Install Docker
# ============================================================================
install_docker() {
    log_step "Installing Docker..."

    if command -v docker &> /dev/null && docker info &> /dev/null; then
        DOCKER_VERSION=$(docker --version)
        log_success "Docker already installed: $DOCKER_VERSION"
        return 0
    fi

    log_info "Installing Docker for $OS_ID..."

    case $OS_ID in
        ubuntu|debian|linuxmint)
            # Remove old versions
            apt-get remove -y docker docker-engine docker.io containerd runc 2>/dev/null || true

            # Install prerequisites
            apt-get update
            apt-get install -y \
                ca-certificates \
                curl \
                gnupg \
                lsb-release

            # Add Docker official GPG key
            install -m 0755 -d /etc/apt/keyrings
            curl -fsSL https://download.docker.com/linux/$OS_ID/gpg | \
                gpg --dearmor -o /etc/apt/keyrings/docker.gpg 2>/dev/null || \
                curl -fsSL https://download.docker.com/linux/debian/gpg | \
                gpg --dearmor -o /etc/apt/keyrings/docker.gpg
            chmod a+r /etc/apt/keyrings/docker.gpg

            # Set up repository
            echo \
                "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/docker.gpg] \
                https://download.docker.com/linux/$OS_ID \
                $(lsb_release -cs) stable" | \
                tee /etc/apt/sources.list.d/docker.list > /dev/null

            # Install Docker
            apt-get update
            apt-get install -y \
                docker-ce \
                docker-ce-cli \
                containerd.io \
                docker-buildx-plugin \
                docker-compose-plugin
            ;;
        *)
            log_error "Unsupported OS: $OS_ID"
            exit 1
            ;;
    esac

    # Start and enable Docker
    systemctl enable docker
    systemctl start docker

    DOCKER_VERSION=$(docker --version)
    log_success "Docker installed: $DOCKER_VERSION"
}

# ============================================================================
# Install Docker Compose (if not plugin)
# ============================================================================
install_compose() {
    log_step "Checking Docker Compose..."

    if docker compose version &> /dev/null; then
        COMPOSE_VERSION=$(docker compose version)
        log_success "Docker Compose installed: $COMPOSE_VERSION"
        return 0
    fi

    if command -v docker-compose &> /dev/null; then
        log_success "docker-compose (v1) found"
        return 0
    fi

    log_info "Installing Docker Compose..."
    DOCKER_CONFIG=${DOCKER_CONFIG:-/usr/lib/docker/cli-plugins}
    mkdir -p $DOCKER_CONFIG

    COMPOSE_VERSION=$(curl -s https://api.github.com/repos/docker/compose/releases/latest | grep '"tag_name"' | sed -E 's/.*"([^"]+)".*/\1/')
    curl -SL "https://github.com/docker/compose/releases/download/${COMPOSE_VERSION}/docker-compose-linux-$(uname -m)" \
        -o $DOCKER_CONFIG/docker-compose
    chmod +x $DOCKER_CONFIG/docker-compose

    log_success "Docker Compose installed: $COMPOSE_VERSION"
}

# ============================================================================
# Generate secure passwords
# ============================================================================
generate_secrets() {
    log_step "Generating secure secrets..."

    ENV_FILE="$INSTALL_DIR/.env"

    if [ -f "$ENV_FILE" ]; then
        log_warn ".env file already exists, skipping secret generation"
        return 0
    fi

    POSTGRES_PASSWORD=$(openssl rand -base64 32 | tr -d '/+=' | head -c 24)
    REDIS_PASSWORD=$(openssl rand -base64 32 | tr -d '/+=' | head -c 24)
    MINIO_SECRET=$(openssl rand -base64 32 | tr -d '/+=' | head -c 32)
    JWT_SECRET=$(openssl rand -base64 64 | tr -d '/+=' | head -c 64)

    cat > "$ENV_FILE" << EOF
# ============================================================================
# Secure Messenger — Environment Configuration
# Generated: $(date -u +"%Y-%m-%d %H:%M:%S UTC")
# ============================================================================

# Application
RUST_LOG=info
BACKEND_PORT=3000

# PostgreSQL
POSTGRES_USER=messenger
POSTGRES_PASSWORD=${POSTGRES_PASSWORD}
POSTGRES_DB=messenger
POSTGRES_EXTERNAL_PORT=127.0.0.1:5432

# Redis
REDIS_PASSWORD=${REDIS_PASSWORD}
REDIS_MAX_MEMORY=256mb
REDIS_EXTERNAL_PORT=127.0.0.1:6379

# MinIO (S3 Storage)
MINIO_ACCESS_KEY=minioadmin
MINIO_SECRET_KEY=${MINIO_SECRET}
MINIO_BUCKET=messenger-files
MINIO_API_PORT=9000
MINIO_CONSOLE_PORT=127.0.0.1:9001

# JWT Authentication
JWT_SECRET=${JWT_SECRET}
JWT_EXPIRY=86400

# Rate Limiting
RATE_LIMIT_PER_MINUTE=60
MAX_UPLOAD_SIZE_MB=50

# Nginx
NGINX_HTTP_PORT=80
NGINX_HTTPS_PORT=443

# Optional: Cloudflare Worker URL for push notifications
CLOUDFLARE_WORKER_URL=
EOF

    chmod 600 "$ENV_FILE"
    log_success "Secrets generated and saved to .env"
}

# ============================================================================
# Configure Nginx
# ============================================================================
configure_nginx() {
    log_step "Configuring Nginx..."

    NGINX_DIR="$INSTALL_DIR/nginx/conf.d"
    mkdir -p "$NGINX_DIR"

    cat > "$NGINX_DIR/messenger.conf" << 'EOF'
upstream backend {
    server backend:3000;
}

server {
    listen 80;
    server_name _;

    # Security headers
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;
    add_header Referrer-Policy "strict-origin-when-cross-origin" always;
    add_header Content-Security-Policy "default-src 'self'; script-src 'self'; style-src 'self' 'unsafe-inline'; img-src 'self' data: blob:; font-src 'self'; connect-src 'self' ws: wss:;" always;

    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;

    # Health check
    location /health {
        proxy_pass http://backend/health;
        proxy_set_header Host $host;
        access_log off;
    }

    # API routes
    location /api/ {
        limit_req zone=api burst=20 nodelay;

        proxy_pass http://backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # WebSocket
    location /ws {
        proxy_pass http://backend;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket timeouts (longer)
        proxy_connect_timeout 60s;
        proxy_send_timeout 3600s;
        proxy_read_timeout 3600s;
    }

    # MinIO console (optional, restrict by IP in production)
    location /minio/ {
        proxy_pass http://minio:9000/;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Deny access to hidden files
    location ~ /\. {
        deny all;
        access_log off;
        log_not_found off;
    }
}
EOF

    log_success "Nginx configured"
}

# ============================================================================
# Create init scripts for PostgreSQL
# ============================================================================
create_init_scripts() {
    log_step "Creating database initialization scripts..."

    INIT_DIR="$INSTALL_DIR/init-scripts"
    mkdir -p "$INIT_DIR"

    cat > "$INIT_DIR/01-init.sql" << 'EOF'
-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Users table
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR(50) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    public_key_x25519 BYTEA,
    public_key_kyber BYTEA,
    public_key_ed25519 BYTEA,
    display_name VARCHAR(100),
    avatar_url TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Chats table
CREATE TABLE IF NOT EXISTS chats (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    chat_type VARCHAR(20) NOT NULL CHECK (chat_type IN ('private', 'group', 'channel')),
    name VARCHAR(255),
    description TEXT,
    avatar_url TEXT,
    created_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Chat members
CREATE TABLE IF NOT EXISTS chat_members (
    chat_id UUID REFERENCES chats(id) ON DELETE CASCADE,
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    role VARCHAR(20) DEFAULT 'member',
    joined_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (chat_id, user_id)
);

-- Messages table (encrypted payloads only)
CREATE TABLE IF NOT EXISTS messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    chat_id UUID REFERENCES chats(id) ON DELETE CASCADE,
    sender_id UUID REFERENCES users(id),
    ciphertext BYTEA NOT NULL,
    nonce BYTEA NOT NULL,
    signature BYTEA,
    message_type VARCHAR(20) DEFAULT 'text',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Files table (MinIO references)
CREATE TABLE IF NOT EXISTS files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    owner_id UUID REFERENCES users(id),
    minio_key VARCHAR(500) NOT NULL,
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type VARCHAR(100),
    checksum_sha256 VARCHAR(64),
    is_public BOOLEAN DEFAULT false,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Sessions table
CREATE TABLE IF NOT EXISTS sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    token_hash VARCHAR(255) NOT NULL,
    device_info TEXT,
    ip_address INET,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_messages_chat_id ON messages(chat_id);
CREATE INDEX IF NOT EXISTS idx_messages_created_at ON messages(created_at);
CREATE INDEX IF NOT EXISTS idx_files_owner_id ON files(owner_id);
CREATE INDEX IF NOT EXISTS idx_sessions_user_id ON sessions(user_id);
CREATE INDEX IF NOT EXISTS idx_sessions_token_hash ON sessions(token_hash);
CREATE INDEX IF NOT EXISTS idx_users_username ON users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users(email);

-- Full-text search on users (trigram)
CREATE INDEX IF NOT EXISTS idx_users_username_trgm ON users USING gin (username gin_trgm_ops);

-- Grants
GRANT ALL PRIVILEGES ON ALL TABLES IN SCHEMA public TO messenger;
GRANT ALL PRIVILEGES ON ALL SEQUENCES IN SCHEMA public TO messenger;
EOF

    log_success "Database init scripts created"
}

# ============================================================================
# Start services
# ============================================================================
start_services() {
    log_step "Starting services..."

    cd "$INSTALL_DIR"

    # Pull latest images
    log_info "Pulling Docker images..."
    docker compose pull

    # Build backend
    log_info "Building backend image..."
    docker compose build backend

    # Start all services
    log_info "Starting containers..."
    docker compose up -d

    log_success "All services started"
}

# ============================================================================
# Verify installation
# ============================================================================
verify_installation() {
    log_step "Verifying installation..."

    sleep 10  # Wait for services to start

    # Check containers
    RUNNING=$(docker compose ps --format json 2>/dev/null | grep -c '"running"' || docker compose ps --status running --format "table" 2>/dev/null | grep -c "up" || echo "0")

    if [ "$RUNNING" -ge 4 ]; then
        log_success "All containers running"
    else
        log_warn "Some containers may still be starting. Check with: docker compose ps"
    fi

    # Check backend health
    sleep 5
    BACKEND_HEALTH=$(curl -sf http://localhost:3000/health 2>/dev/null || echo "")

    if [ -n "$BACKEND_HEALTH" ]; then
        log_success "Backend healthy: $BACKEND_HEALTH"
    else
        log_warn "Backend health check failed (may need more time to start)"
    fi

    # Check PostgreSQL
    PG_STATUS=$(docker compose exec -T postgres pg_isready -U messenger 2>/dev/null || echo "not ready")
    if echo "$PG_STATUS" | grep -q "accepting"; then
        log_success "PostgreSQL ready"
    else
        log_warn "PostgreSQL not ready yet"
    fi

    # Check Redis
    REDIS_PASSWORD=$(grep REDIS_PASSWORD .env | cut -d= -f2)
    REDIS_STATUS=$(docker compose exec -T redis redis-cli -a "$REDIS_PASSWORD" ping 2>/dev/null || echo "")
    if [ "$REDIS_STATUS" = "PONG" ]; then
        log_success "Redis ready"
    else
        log_warn "Redis not ready yet"
    fi
}

# ============================================================================
# Print summary
# ============================================================================
print_summary() {
    echo ""
    echo -e "${BOLD}${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${GREEN}║${NC}  ${BOLD}Secure Messenger — Installation Complete!${NC}             ${BOLD}${GREEN}║${NC}"
    echo -e "${BOLD}${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BLUE}Services:${NC}"
    echo -e "  • Backend:     http://localhost:3000"
    echo -e "  • Nginx:       http://localhost:80"
    echo -e "  • MinIO:       http://localhost:9000 (console: http://localhost:9001)"
    echo -e "  • PostgreSQL:  localhost:5432"
    echo -e "  • Redis:       localhost:6379"
    echo ""
    echo -e "${BLUE}Useful commands:${NC}"
    echo -e "  ${CYAN}docker compose ps${NC}              — Show running containers"
    echo -e "  ${CYAN}docker compose logs -f backend${NC}  — View backend logs"
    echo -e "  ${CYAN}docker compose down${NC}             — Stop all services"
    echo -e "  ${CYAN}docker compose restart backend${NC}  — Restart backend"
    echo -e "  ${CYAN}docker compose exec backend sh${NC}  — Shell inside backend"
    echo ""
    echo -e "${BLUE}MinIO credentials:${NC}"
    echo -e "  Access Key: ${YELLOW}minioadmin${NC}"
    echo -e "  Secret Key: ${YELLOW}(see .env file)${NC}"
    echo ""
    echo -e "${YELLOW}⚠  IMPORTANT: Store the .env file securely!${NC}"
    echo -e "${YELLOW}   It contains all passwords and secrets.${NC}"
    echo ""
}

# ============================================================================
# Main
# ============================================================================
main() {
    echo -e "${BOLD}${BLUE}"
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║           Secure Messenger — Self-Hosted Setup          ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"

    check_root
    detect_os
    check_requirements
    install_docker
    install_compose

    # Determine install directory
    INSTALL_DIR="${INSTALL_DIR:-/opt/secure-messenger}"

    if [ ! -d "$INSTALL_DIR" ]; then
        log_step "Creating installation directory: $INSTALL_DIR"
        mkdir -p "$INSTALL_DIR"
    fi

    # Copy project files (if running from repo)
    SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
    PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

    if [ -f "$PROJECT_ROOT/server/Cargo.toml" ]; then
        log_info "Copying project files..."
        cp -r "$PROJECT_ROOT/server" "$INSTALL_DIR/server"
        cp "$PROJECT_ROOT/Cargo.toml" "$INSTALL_DIR/Cargo.toml"
        cp "$PROJECT_ROOT/Cargo.lock" "$INSTALL_DIR/Cargo.lock" 2>/dev/null || true
        cp -r "$SCRIPT_DIR/docker" "$INSTALL_DIR/self-hosting/docker"
        cp -r "$SCRIPT_DIR/docker-compose.yml" "$INSTALL_DIR/"
    fi

    cd "$INSTALL_DIR"

    # Configure
    create_init_scripts
    configure_nginx
    generate_secrets

    # Start
    start_services

    # Verify
    verify_installation

    # Summary
    print_summary
}

# Run
main "$@"
