# Backend Dockerfile
FROM rust:1.84-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config libssl-dev cmake && \
    rm -rf /var/lib/apt/lists/*

# Copy workspace
COPY Cargo.toml Cargo.lock ./
COPY messenger/Cargo.toml messenger/
COPY server/Cargo.toml server/

# Copy source
COPY messenger/src/ messenger/src/
COPY server/src/ server/src/

# Build server
RUN cargo build --release --bin secure-messenger-server

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    libssl3 ca-certificates && \
    rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/secure-messenger-server /app/

ENV DATABASE_URL=sqlite:/data/secure-messenger.db
ENV JWT_SECRET=change-me-in-production
ENV JWT_EXPIRY=86400
ENV RUST_LOG=info
ENV PORT=3000

VOLUME ["/data", "/app/uploads"]

EXPOSE 3000

CMD ["/app/secure-messenger-server"]
