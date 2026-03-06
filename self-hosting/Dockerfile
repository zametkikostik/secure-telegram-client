FROM rust:1.75-slim as builder

WORKDIR /app

# Копирование Cargo.toml и загрузка зависимостей
COPY server/Cargo.toml ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Копирование исходного кода
COPY server/src ./src
COPY server/Cargo.toml ./

# Сборка
RUN cargo build --release

# Финальный образ
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/liberty-reach-server /app/server

EXPOSE 8008

ENV SERVER_ADDR=0.0.0.0:8008
ENV DATABASE_URL=sqlite:./liberty_reach.db
ENV JWT_SECRET=change-me-in-production
ENV UPLOADS_DIR=/app/uploads

VOLUME ["/app/uploads"]

CMD ["./server"]
