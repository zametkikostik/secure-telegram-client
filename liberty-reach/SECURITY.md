# Security Policy - Liberty Reach

## 🔒 Vulnerability Disclosure

At Liberty Reach, we take security seriously. If you discover a security vulnerability, we appreciate your help in disclosing it responsibly.

### Reporting a Vulnerability

**Please do NOT report security vulnerabilities through public GitHub issues.**

Instead, please report them via email to:

📧 **Security Team**: [zametkikostik@gmail.com](mailto:zametkikostik@gmail.com)

### What to Include

When reporting a vulnerability, please include:

1. **Description**: Clear description of the vulnerability
2. **Impact**: Potential impact if exploited
3. **Reproduction Steps**: Detailed steps to reproduce the issue
4. **Affected Versions**: Which versions are affected
5. **Suggested Fix**: If you have suggestions for fixing the issue

### Response Time

- **Initial Response**: Within 48 hours
- **Status Update**: Within 5 business days
- **Resolution Timeline**: Depends on severity (critical issues prioritized)

### Security Best Practices

#### For Users

1. **Never share your identity.key** - This is your cryptographic identity
2. **Keep .env.local secure** - Contains API keys and secrets
3. **Verify HMAC signatures** - When using Cloudflare Worker
4. **Update regularly** - Stay on the latest version

#### For Developers

1. **No hardcoded secrets** - Use environment variables
2. **Zeroize sensitive data** - Clear keys from memory after use
3. **Validate all input** - Never trust external data
4. **Use constant-time comparisons** - For cryptographic operations

### Security Features

Liberty Reach implements the following security measures:

| Feature | Description |
|---------|-------------|
| **E2EE** | AES-256-GCM end-to-end encryption |
| **Double Ratchet** | Forward secrecy for messages |
| **Noise Protocol** | Transport encryption (DPI protection) |
| **Ed25519 Signatures** | Message authentication |
| **Zeroize** | Memory cleanup for sensitive data |
| **HMAC Verification** | API request authentication |
| **No Central Servers** | Zero-knowledge architecture |

### Known Limitations

1. **WebRTC**: Requires STUN/TURN servers (potential metadata leak)
2. **Cloudflare Worker**: Metadata stored with 24h TTL
3. **AI Integration**: External API calls (Ollama recommended for privacy)

### Security Audit History

| Date | Version | Auditor | Status |
|------|---------|---------|--------|
| 2026-03-12 | v0.4.0-fortress-stable | Internal | ✅ Passed |

### PGP Key

For encrypted communications, use our PGP key:

```
TODO: Add PGP public key
```

### Bug Bounty

Currently, we do not offer a bug bounty program. However, we:

- ✅ Credit reporters (if desired)
- ✅ Provide detailed status updates
- ✅ Appreciate responsible disclosure

### Legal

We do not take legal action against security researchers who:

- Follow responsible disclosure
- Do not access user data without permission
- Do not disrupt service for other users

---

**Thank you for helping keep Liberty Reach secure!** 🛡️

**Last Updated**: March 12, 2026
