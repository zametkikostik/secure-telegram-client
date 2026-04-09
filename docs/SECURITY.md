# 🔐 Security Guide

> Критические требования к безопасности Secure Telegram Client

---

## 🚨 КРИТИЧЕСКИЕ ПРАВИЛА

### 1. Приватные ключи

- **НИКОГДА** не покидают устройство пользователя
- Хранятся только в зашифрованном виде (Argon2id + ChaCha20-Poly1305)
- Zeroize после использования
- **SECURITY**: требует аудита перед production

### 2. Шифрование данных

| Тип | Алгоритм |
|-----|----------|
| E2EE | X25519 + Kyber1024 + ChaCha20-Poly1305 + HMAC-SHA3 |
| Password | Argon2id (64MB, 3 iterations, 4 parallelism) |
| Signatures | Ed25519 + ML-DSA (post-quantum) |
| Random | OsRng (cryptographically secure) |

### 3. Логи

- Только зашифрованные
- Локальные (никакой отправки на сервер)
- С ротацией (max 7 дней)
- **НЕ** содержат чувствительных данных

### 4. Сетевая безопасность

- Все соединения: TLS 1.3+
- P2P: Noise protocol handshake
- Cloudflare: mTLS для Workers
- **SECURITY**: проверить certificate pinning

---

## 📋 ЧЕКЛИСТ АУДИТА

### Криптография

- [ ] Проверить X25519 key exchange
- [ ] Проверить Kyber1024 implementation (liboqs)
- [ ] Проверить ChaCha20-Poly1305 nonce generation
- [ ] Проверить HMAC-SHA3 verification
- [ ] Проверить Argon2id parameters
- [ ] Проверить Ed25519 signature verification
- [ ] Проверить ML-DSA implementation
- [ ] Проверить zeroize после использования ключей

### Хранение ключей

- [ ] Ключи шифруются перед записью на диск
- [ ] Пароль для расшифровки ключей не хранится в памяти
- [ ] Key derivation function использует достаточно памяти
- [ ] Secure enclave/TPM используется если доступен

### Сетевая безопасность

- [ ] Certificate pinning для всех соединений
- [ ] Noise protocol handshake корректен
- [ ] Нет утечек метаданных
- [ ] Rate limiting на все endpoint'ы
- [ ] DDoS protection

### P2P безопасность

- [ ] Peer verification через DHT
- [ ] Message authentication через HMAC
- [ ] Replay attack protection
- [ ] Sybil attack resistance

### Web3 безопасность

- [ ] Smart contracts аудированы
- [ ] No reentrancy vulnerabilities
- [ ] Access control корректен
- [ ] Fee calculation точна

### Приложение

- [ ] No hardcoded secrets
- [ ] Environment variables для secrets
- [ ] CSP headers корректны
- [ ] XSS protection
- [ ] CSRF protection
- [ ] Input validation
- [ ] Output encoding

---

## 🧪 ПЕНТЕСТ ПЛАН

### 1. Статический анализ

```bash
# Rust
cargo audit
cargo clippy -- -D warnings

# Frontend
npm run lint
npm audit

# Smart contracts
slither smart-contracts/
mythril analyze smart-contracts/contracts/
```

### 2. Динамический анализ

```bash
# Fuzzing
cargo fuzz run crypto_fuzz

# Memory safety
valgrind --leak-check=full target/release/secure-messenger

# Network analysis
wireshark -i any -f "port 3000 or port 1420"
```

### 3. Ручной аудит

- [ ] Code review всех crypto модулей
- [ ] Проверка всех TODO и SECURITY комментариев
- [ ] Review access control logic
- [ ] Review error handling

---

## 🚨 ИЗВЕСТНЫЕ УЯЗВИМОСТИ (TODO)

| Модуль | Уязвимость | Статус |
|--------|------------|--------|
| hybrid_encrypt.rs | Nonce не случайный (placeholder) | 🔴 Критично |
| keypair.rs | Kyber1024 не интегрирован | 🟡 В процессе |
| steganography | LSB легко обнаружить | 🟡 В процессе |
| server/auth | JWT не реализован | ⬜ Запланировано |
| p2p.rs | libp2p не интегрирован | ⬜ Запланировано |

---

## 📜 COMPLIANCE

### GDPR (EU)

- Право на удаление: ✅ (реализовано в storage.rs)
- Право на доступ: ✅ (API endpoint'ы)
- Право на перенос: ⬜ (TODO)
- Минимизация данных: ✅ (только зашифрованные)

### 152-ФЗ (Russia)

- Локализация данных: ✅ (SQLite локальный)
- Шифрование: ✅ (post-quantum E2EE)
- Согласие на обработку: ⬜ (TODO)

---

## 🔐 KEY MANAGEMENT

### Генерация ключей

```
1. User enters password
2. Argon2id(password) → key_encryption_key
3. Generate X25519 keypair
4. Generate Kyber1024 keypair (TODO)
5. Generate Ed25519 keypair
6. Encrypt all private keys with key_encryption_key
7. Store encrypted keys locally
8. Zeroize password from memory
```

### Использование ключей

```
1. Load encrypted keys from disk
2. User enters password
3. Argon2id(password) → key_encryption_key
4. Decrypt keys
5. Use keys for encryption/signing
6. Zeroize decrypted keys after use
```

---

## 📞 INCIDENT RESPONSE

Если обнаружена уязвимость:

1. **НЕ** публикуй публично
2. Напиши на zametkikostik@gmail.com
3. Опиши уязвимость и шаги для воспроизведения
4. Дождись ответа (max 48 часов)

---

> **Помни**: Безопасность — это процесс, а не состояние. 🔐
