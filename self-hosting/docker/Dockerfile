# ============================================================================
# Secure Messenger Server — Multi-stage Docker Build
# ============================================================================
# Security-focused minimal image for production deployment
#
# Stages:
# 1. builder     — Compile Rust binary with optimizations
# 2. runner      — Minimal runtime image (Debian slim, ~50MB)
# ============================================================================

# ============================================================================
# Stage 1: Build
# ============================================================================
FROM rust:1.75-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy workspace Cargo.toml first (for dependency caching)
COPY Cargo.toml Cargo.lock ./
COPY server/Cargo.toml ./server/Cargo.toml

# Create dummy main.rs for dependency caching
RUN mkdir -p server/src && \
    echo "fn main() {}" > server/src/main.rs

# Build dependencies only (cached layer)
RUN cargo build --release --package secure-messenger-server --locked

# Remove dummy source
RUN rm -rf server/src

# Copy actual source code
COPY server/src ./server/src

# Rebuild with actual source (only this layer changes on code update)
RUN touch server/src/main.rs && \
    cargo build --release --package secure-messenger-server --locked

# Strip binary to reduce size
RUN strip target/release/secure-messenger-server

# ============================================================================
# Stage 2: Runtime
# ============================================================================
FROM debian:bookworm-slim AS runner

# Security: Create non-root user
RUN groupadd -r messenger && \
    useradd -r -g messenger -d /home/messenger -s /sbin/nologin messenger

# Install runtime dependencies only
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/* \
    && update-ca-certificates

# Create directories
RUN mkdir -p /app/data /app/logs /app/config && \
    chown -R messenger:messenger /app

WORKDIR /app

# Copy binary from builder
COPY --from=builder /app/target/release/secure-messenger-server /app/server

# Copy configuration templates
COPY self-hosting/config.example.toml /app/config/config.toml

# Set ownership
RUN chown -R messenger:messenger /app

# Switch to non-root user
USER messenger

# Expose ports
EXPOSE 3000 3001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Environment variables (override via docker-compose)
ENV RUST_LOG=info
ENV DATABASE_URL=postgres://messenger:password@postgres:5432/messenger
ENV REDIS_URL=redis://redis:6379
ENV MINIO_ENDPOINT=http://minio:9000
ENV MINIO_ACCESS_KEY=minioadmin
ENV MINIO_SECRET_KEY=minioadmin

# Run server
CMD ["/app/server"]
