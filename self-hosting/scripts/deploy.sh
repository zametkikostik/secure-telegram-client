#!/bin/bash
# ============================================================================
# Secure Messenger — Production Deploy Script
# ============================================================================
# One-click production deployment!
#
# Usage:
#   ./deploy.sh          # Deploy everything
#   ./deploy.sh --dry    # Dry run (checks only)
#   ./deploy.sh --stop   # Stop all services
#   ./deploy.sh --status # Check status
# ============================================================================

set -euo pipefail

# Colors
RED='\033[0;31m'; GREEN='\033[0;32m'; YELLOW='\033[1;33m'
BLUE='\033[0;34m'; CYAN='\033[0;36m'; NC='\033[0m'; BOLD='\033[1m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_ok() { echo -e "${GREEN}[OK]${NC} $1"; }
log_warn() { echo -e "${YELLOW}[WARN]${NC} $1"; }
log_err() { echo -e "${RED}[ERROR]${NC} $1"; }
log_step() { echo -e "\n${BOLD}${CYAN}▸ $1${NC}"; }

# Config
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
ENV_FILE="$SCRIPT_DIR/.env"
COMPOSE_FILE="$SCRIPT_DIR/docker-compose.production.yml"

# ============================================================================
# Functions
# ============================================================================

check_prerequisites() {
    log_step "Checking prerequisites..."

    # Check .env exists
    if [ ! -f "$ENV_FILE" ]; then
        log_err ".env not found!"
        log_info "Run: cp .env.production .env && nano .env"
        exit 1
    fi

    # Check docker-compose
    if ! command -v docker-compose &> /dev/null; then
        log_err "docker-compose not found!"
        exit 1
    fi

    # Check domain (if TLS enabled)
    if grep -q "DOMAIN_NAME=messenger.your-domain.com" "$ENV_FILE"; then
        log_warn "Domain still set to placeholder. Edit .env!"
        if [ "$DRY_RUN" = "true" ]; then
            log_warn "Dry run — continuing anyway"
        else
            read -p "Continue with placeholder domain? (y/N) " -n 1 -r
            echo
            if [[ ! $REPLY =~ ^[Yy]$ ]]; then exit 1; fi
        fi
    fi

    log_ok "Prerequisites OK"
}

generate_secrets() {
    log_step "Checking secrets..."

    if grep -q "CHANGE_ME" "$ENV_FILE"; then
        log_warn "Found CHANGE_ME placeholders in .env"

        if [ "$DRY_RUN" = "true" ]; then
            log_info "Dry run — would generate secrets"
            echo ""
            echo "Run these commands to generate:"
            echo "  sed -i 's/POSTGRES_PASSWORD=CHANGE_ME.*/POSTGRES_PASSWORD=$(openssl rand -base64 32 | tr -d \\/+= | head -c 24)/' .env"
            echo "  sed -i 's/REDIS_PASSWORD=CHANGE_ME.*/REDIS_PASSWORD=$(openssl rand -base64 32 | tr -d \\/+= | head -c 24)/' .env"
            echo "  sed -i 's/MINIO_SECRET_KEY=CHANGE_ME.*/MINIO_SECRET_KEY=$(openssl rand -base64 32 | tr -d \\/+= | head -c 32)/' .env"
            echo "  sed -i 's/JWT_SECRET=CHANGE_ME.*/JWT_SECRET=$(openssl rand -base64 64 | tr -d \\/+= | head -c 64)/' .env"
            return
        fi

        read -p "Auto-generate secrets? (y/N) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            sed -i "s/POSTGRES_PASSWORD=CHANGE_ME.*/POSTGRES_PASSWORD=$(openssl rand -base64 32 | tr -d '\/+=' | head -c 24)/" "$ENV_FILE"
            sed -i "s/REDIS_PASSWORD=CHANGE_ME.*/REDIS_PASSWORD=$(openssl rand -base64 32 | tr -d '\/+=' | head -c 24)/" "$ENV_FILE"
            sed -i "s/MINIO_SECRET_KEY=CHANGE_ME.*/MINIO_SECRET_KEY=$(openssl rand -base64 32 | tr -d '\/+=' | head -c 32)/" "$ENV_FILE"
            sed -i "s/JWT_SECRET=CHANGE_ME.*/JWT_SECRET=$(openssl rand -base64 64 | tr -d '\/+=' | head -c 64)/" "$ENV_FILE"
            log_ok "Secrets generated and saved to .env"
        else
            log_info "Please edit .env manually"
            exit 1
        fi
    else
        log_ok "Secrets configured"
    fi
}

deploy() {
    log_step "Deploying production..."

    cd "$SCRIPT_DIR"

    # Pull images
    log_info "Pulling Docker images..."
    docker-compose -f "$COMPOSE_FILE" pull

    # Build backend
    log_info "Building backend..."
    docker-compose -f "$COMPOSE_FILE" build backend

    # Start services
    log_info "Starting services..."
    docker-compose -f "$COMPOSE_FILE" up -d

    log_ok "All services started!"
}

wait_for_healthy() {
    log_step "Waiting for services to be healthy..."

    local max_wait=120
    local waited=0

    while [ $waited -lt $max_wait ]; do
        local unhealthy=$(docker-compose -f "$COMPOSE_FILE" ps --format json 2>/dev/null | grep -c '"healthy":false' || echo "0")

        if [ "$unhealthy" = "0" ] || [ "$unhealthy" = "" ]; then
            log_ok "All services healthy!"
            return 0
        fi

        echo -ne "\r  Waiting... $waited/$max_wait seconds ($unhealthy unhealthy)"
        sleep 5
        waited=$((waited + 5))
    done

    echo ""
    log_warn "Some services may still be starting. Check with: docker-compose ps"
}

verify() {
    log_step "Verifying deployment..."

    # Health checks
    local backend_health=$(curl -sf http://localhost:3000/health 2>/dev/null || echo "FAIL")

    if [ "$backend_health" != "FAIL" ]; then
        log_ok "Backend: $backend_health"
    else
        log_err "Backend health check failed!"
        log_info "Check logs: docker-compose logs backend"
    fi

    # Show status
    echo ""
    docker-compose -f "$COMPOSE_FILE" ps
}

show_info() {
    echo ""
    echo -e "${BOLD}${GREEN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${GREEN}║${NC}  ${BOLD}🚀 SECURE MESSENGER — PRODUCTION DEPLOYED!${NC}           ${BOLD}${GREEN}║${NC}"
    echo -e "${BOLD}${GREEN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo ""

    local domain=$(grep "DOMAIN_NAME=" "$ENV_FILE" | cut -d= -f2)
    echo -e "${BLUE}URLs:${NC}"
    echo -e "  • Frontend:    https://$domain"
    echo -e "  • Backend API: https://$domain/api/v1/"
    echo -e "  • MinIO:       https://$domain/minio/"
    echo ""
    echo -e "${BLUE}Useful commands:${NC}"
    echo -e "  ${CYAN}docker-compose -f docker-compose.production.yml ps${NC}              — Status"
    echo -e "  ${CYAN}docker-compose -f docker-compose.production.yml logs -f backend${NC} — Logs"
    echo -e "  ${CYAN}docker-compose -f docker-compose.production.yml down${NC}            — Stop"
    echo -e "  ${CYAN}docker-compose -f docker-compose.production.yml restart backend${NC}  — Restart"
    echo ""
    echo -e "${YELLOW}⚠  IMPORTANT: Store .env securely!${NC}"
    echo -e "${YELLOW}   It contains all passwords and secrets.${NC}"
}

stop() {
    log_step "Stopping all services..."
    cd "$SCRIPT_DIR"
    docker-compose -f "$COMPOSE_FILE" down
    log_ok "All services stopped"
}

status() {
    log_step "Service status..."
    cd "$SCRIPT_DIR"
    docker-compose -f "$COMPOSE_FILE" ps
    echo ""
    log_step "Health checks..."
    curl -sf http://localhost:3000/health 2>/dev/null && echo "" || log_err "Backend unreachable"
}

# ============================================================================
# Main
# ============================================================================

DRY_RUN=false

case "${1:-deploy}" in
    --dry|-d)
        DRY_RUN=true
        check_prerequisites
        generate_secrets
        ;;
    --stop|stop)
        stop
        ;;
    --status|status)
        status
        ;;
    deploy|*)
        check_prerequisites
        generate_secrets
        deploy
        wait_for_healthy
        verify
        show_info
        ;;
esac
