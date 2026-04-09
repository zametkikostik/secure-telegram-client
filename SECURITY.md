# 🔐 Security Policy — Secure Messenger

**Version:** 1.0.0  
**Last Updated:** April 8, 2026  
**Contact:** security@kostik.work

---

## 📋 Supported Versions

| Version | Supported | EOL Date |
|---------|-----------|----------|
| 1.x.x   | ✅ Yes    | -        |
| 0.9.x   | ✅ Yes    | Dec 2026 |
| < 0.9   | ❌ No     | -        |

---

## 🛡️ Security Architecture

### Encryption

| Component | Algorithm | Key Size | Notes |
|-----------|-----------|----------|-------|
| **Key Exchange** | X25519 + Kyber1024 | 256-bit + NIST Level 5 | Post-quantum hybrid |
| **Message Encryption** | ChaCha20-Poly1305 | 256-bit | AEAD authenticated encryption |
| **Digital Signatures** | Ed25519 | 256-bit | EdDSA signatures |
| **Key Derivation** | HKDF-SHA3-256 | 256-bit | Secure key expansion |
| **Password Hashing** | Argon2id | - | Memory-hard function |
| **Ad Bundle Encryption** | ChaCha20-Poly1305 | 256-bit | End-to-end encrypted ads |

### Network Security

- **Transport**: TLS 1.3 (minimum TLS 1.2)
- **Certificate**: Let's Encrypt (auto-renew, 90-day rotation)
- **HSTS**: Enabled with preload
- **WebSocket**: Secure (wss://)
- **DNSSEC**: Enabled via Cloudflare

### Data Protection

- **E2EE Messages**: Never stored on server in plaintext
- **Local Storage**: OS keychain (Keychain/Keyring)
- **File Storage**: Encrypted at rest (MinIO SSE)
- **Database**: PostgreSQL with encryption at rest
- **Backups**: Encrypted with separate key

---

## 🚨 Reporting a Vulnerability

We take security seriously. If you discover a vulnerability:

### **DO:**
- ✅ Email **security@kostik.work** with details
- ✅ Use our PGP key: [Download](https://kostik.work/pgp.asc)
- ✅ Include steps to reproduce
- ✅ Allow time for response (within 48 hours)
- ✅ Allow time for fix (within 30 days for critical)

### **DO NOT:**
- ❌ Open a public GitHub issue
- ❌ Exploit the vulnerability beyond verification
- ❌ Access other users' data
- ❌ Disrupt service availability

### **Response Timeline:**

| Severity | Acknowledgment | Fix Released | Disclosure |
|----------|----------------|--------------|------------|
| Critical | 24 hours | 7 days | 30 days |
| High | 48 hours | 14 days | 45 days |
| Medium | 72 hours | 30 days | 60 days |
| Low | 7 days | 90 days | 120 days |

### **Bug Bounty:**

| Severity | Reward |
|----------|--------|
| Critical | $5,000 - $10,000 |
| High | $2,000 - $5,000 |
| Medium | $500 - $2,000 |
| Low | $100 - $500 |

---

## 🔍 Security Audits

### Completed Audits

| Date | Auditor | Scope | Result | Report |
|------|---------|-------|--------|--------|
| Q2 2026 | *(Planned)* | Crypto + Network | - | - |

### Pending Audits

- [ ] **Cryptographic Implementation**
  - Auditor: *(TBD)*
  - Scope: X25519 + Kyber1024 + ChaCha20-Poly1305 implementation
  - Timeline: Q2 2026

- [ ] **Network Security**
  - Auditor: *(TBD)*
  - Scope: TLS, WebSocket, Cloudflare configuration
  - Timeline: Q2 2026

- [ ] **Smart Contracts**
  - Auditor: *(TBD)*
  - Scope: P2PEscrow.sol + FeeSplitter.sol
  - Timeline: Q3 2026

---

## 📜 Compliance

### GDPR (EU)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Data Minimization (Art. 5) | ✅ | E2EE, no server-side message storage |
| Lawful Processing (Art. 6) | ✅ | User consent on registration |
| Right to Access (Art. 15) | ✅ | Data export endpoint |
| Right to Erasure (Art. 17) | ✅ | Account deletion with cascade |
| Data Portability (Art. 20) | ✅ | JSON export of all user data |
| Privacy by Design (Art. 25) | ✅ | E2EE by default |
| Breach Notification (Art. 33) | ✅ | 72-hour notification process |

### 152-ФЗ (Russia)

| Requirement | Status | Evidence |
|-------------|--------|----------|
| Consent (Ст. 6) | ✅ | Explicit consent on registration |
| Access Rights (Ст. 14) | ✅ | User data export |
| Data Deletion (Ст. 17) | ✅ | Automated deletion policies |
| Data Localization (Ст. 18.1) | ⚠️ | Self-hosting option available |
| Security Measures (Ст. 19) | ✅ | Encryption, access controls |

---

## 🔧 Security Best Practices for Users

### **Client Security**
1. ✅ Enable 2FA (TOTP)
2. ✅ Use a strong, unique password
3. ✅ Keep the app updated
4. ✅ Verify contact's security keys
5. ✅ Enable screen lock on mobile

### **Self-Hosting Security**
1. ✅ Use strong passwords for all services
2. ✅ Enable firewall (UFW/iptables)
3. ✅ Configure TLS with HSTS
4. ✅ Enable automatic security updates
5. ✅ Restrict database ports to localhost
6. ✅ Use Cloudflare WAF rules
7. ✅ Monitor with Prometheus + alerts

---

## 🚫 Known Limitations

### **Not Yet Implemented**
- [ ] Forward secrecy for messages (future sessions)
- [ ] Deniable authentication
- [ ] Quantum-resistant signatures for messages (Kyber for KEM only)
- [ ] Hardware security key support (YubiKey, SoloKey)
- [ ] Secure enclave usage (TPM, Secure Enclave)

### **Workarounds**
- Messages are encrypted with recipient's public key — if key is compromised, past messages may be decryptable
- **Mitigation**: Users should rotate keys periodically

---

## 📚 Security Resources

- [NIST Post-Quantum Cryptography](https://csrc.nist.gov/projects/post-quantum-cryptography)
- [Cloudflare Learning Center — TLS](https://www.cloudflare.com/learning/ssl/transport-layer-security-tls/)
- [OWASP Top 10](https://owasp.org/www-project-top-ten/)
- [RFC 7748 — X25519](https://datatracker.ietf.org/doc/html/rfc7748)
- [FIPS 203 — ML-KEM (Kyber)](https://csrc.nist.gov/pubs/fips/203/final)

---

## 📞 Contact

| Channel | Contact |
|---------|---------|
| Security Email | security@kostik.work |
| PGP Key | https://kostik.work/pgp.asc |
| Telegram | @kostik_security |
| General Inquiries | zametkikostik@gmail.com |

---

**This security policy is a living document and will be updated as the project evolves.**
