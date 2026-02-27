# Dockerfile для Secure Telegram Client

FROM rust:1.75-slim as builder

# Установка зависимостей
RUN apt-get update && apt-get install -y \
    cmake \
    clang \
    libssl-dev \
    pkg-config \
    git \
    && rm -rf /var/lib/apt/lists/*

# Создание рабочего каталога
WORKDIR /app

# Копирование манифестов Cargo
COPY Cargo.toml Cargo.lock* ./

# Копирование исходного кода
COPY src ./src

# Сборка релизной версии
RUN cargo build --release

# Финальный образ
FROM debian:bookworm-slim

# Установка рантайм зависимостей
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Копирование бинарного файла из builder
COPY --from=builder /app/target/release/secure-tg /usr/local/bin/

# Создание пользователя для безопасности
RUN useradd -m -u 1000 securetg
USER securetg

# Рабочая директория
WORKDIR /home/securetg

# Точка входа
ENTRYPOINT ["secure-tg"]
CMD ["--help"]
