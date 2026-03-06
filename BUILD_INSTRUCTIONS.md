# Liberty Reach — Инструкция по сборке

## 📖 Содержание

1. [Требования](#требования)
2. [Сборка messenger](#сборка-messenger)
3. [Сборка migration-tool](#сборка-migration-tool)
4. [Сборка смарт-контрактов](#сборка-смарт-контрактов)
5. [Сборка Docker образа](#сборка-docker-образа)

---

## 🛠 Требования

### Для messenger (Rust + Tauri)

- Rust 1.75+
- Node.js 18+
- npm/yarn
- Tauri CLI

### Для migration-tool (Python)

- Python 3.10+
- pip

### Для смарт-контрактов (Solidity)

- Node.js 18+
- Hardhat

### Для Docker

- Docker 20.10+
- Docker Compose 2.0+

---

## 🔨 Сборка messenger

### Установка зависимостей

```bash
cd messenger

# Установить Rust зависимости
cargo fetch

# Установить Node.js зависимости
npm install
```

### Debug сборка

```bash
cargo build
```

### Release сборка

```bash
cargo build --release
```

### Tauri приложение

```bash
# Установить Tauri CLI
cargo install tauri-cli

# Запустить в режиме разработки
cargo tauri dev

# Собрать приложение
cargo tauri build
```

### Результат

```
messenger/target/release/liberty_reach     # Linux
messenger/target/release/liberty_reach.exe # Windows
messenger/target/release/liberty_reach.app # macOS
```

---

## 🐍 Сборка migration-tool

### Установка зависимостей

```bash
cd migration-tool

# Создать виртуальное окружение
python -m venv venv
source venv/bin/activate  # Linux/Mac
venv\Scripts\activate     # Windows

# Установить зависимости
pip install -r requirements.txt
```

### Тест

```bash
python ai_translator.py
```

---

## 🔗 Сборка смарт-контрактов

### Установка зависимостей

```bash
cd smart-contracts

# Установить зависимости
npm install
```

### Компиляция

```bash
npx hardhat compile
```

### Тесты

```bash
npx hardhat test
```

### Деплой (testnet)

```bash
npx hardhat run scripts/deploy.ts --network goerli
```

### Деплой (mainnet)

```bash
npx hardhat run scripts/deploy.ts --network mainnet
```

---

## 🐳 Сборка Docker образа

### Локальная сборка

```bash
cd self-hosting

# Собрать образ
docker build -t libertyreach/messenger:latest ..

# Запустить
docker-compose up -d
```

### Production сборка

```bash
# Multi-arch сборка
docker buildx create --use
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t libertyreach/messenger:latest \
  --push \
  ..
```

---

## ✅ Проверка

### Тест messenger

```bash
cd messenger
cargo test
```

### Тест migration-tool

```bash
cd migration-tool
pytest
```

### Тест смарт-контрактов

```bash
cd smart-contracts
npx hardhat test
```

### Тест Docker

```bash
cd self-hosting
docker-compose up -d
docker-compose ps
curl http://localhost:8008/health
```

---

## 📦 Публикация

### Публикация Docker образа

```bash
docker login
docker push libertyreach/messenger:latest
```

### Публикация релиза

```bash
# Создать тег
git tag v1.0.0
git push origin v1.0.0

# Создать релиз на GitHub
# https://github.com/libertyreach/messenger/releases/new
```

---

## 📬 Поддержка

Email: dev@libertyreach.io  
GitHub: https://github.com/libertyreach/messenger
