# OmenDB Security Guide

**Last Updated**: October 22, 2025  
**Version**: 0.1.0-dev

---

## Overview

OmenDB implements production-grade security features including:

- **Authentication**: SCRAM-SHA-256 password authentication
- **Encryption**: TLS/SSL for PostgreSQL wire protocol
- **User Management**: Role-based access control (SQL commands)
- **Password Security**: Bcrypt hashing with salt
- **Connection Security**: TLS certificate validation

---

## Quick Start (Development)

### 1. Start Server (No Authentication)

```bash
cargo run --bin postgres_server
# Server starts on port 5433 without authentication
```

### 2. Start Server with Authentication

```bash
cargo run --bin postgres_server_auth
# Creates default admin user: admin/admin (change immediately!)
```

### 3. Start Server with TLS

```bash
# Generate self-signed certificate (development only)
openssl req -new -newkey rsa:2048 -days 365 -nodes -x509 \
  -keyout certs/key.pem -out certs/cert.pem \
  -subj "/C=US/ST=CA/L=SF/O=YourOrg/CN=localhost"

# Start server with TLS
cargo run --bin postgres_server -- --cert certs/cert.pem --key certs/key.pem
```

### 4. Connect with psql

```bash
# Without TLS
psql -h 127.0.0.1 -p 5433

# With TLS (self-signed cert)
psql "host=127.0.0.1 port=5433 sslmode=require"

# With authentication
psql -h 127.0.0.1 -p 5433 -U admin
# Password: admin (default, change immediately!)
```

---

## Production Deployment

### TLS Certificate Setup

**⚠️ WARNING**: Never use self-signed certificates in production!

#### Option 1: Let's Encrypt (Recommended)

```bash
# Install certbot
sudo apt-get install certbot

# Generate certificate (requires domain and port 80/443)
sudo certbot certonly --standalone -d your-domain.com

# Certificates will be in:
# /etc/letsencrypt/live/your-domain.com/fullchain.pem (cert)
# /etc/letsencrypt/live/your-domain.com/privkey.pem (key)

# Start OmenDB with production certificates
./postgres_server \
  --cert /etc/letsencrypt/live/your-domain.com/fullchain.pem \
  --key /etc/letsencrypt/live/your-domain.com/privkey.pem
```

#### Option 2: Corporate CA Certificate

```bash
# Obtain certificate from your organization's CA
# Ensure certificate includes:
# - Server authentication (Extended Key Usage)
# - Subject Alternative Name (SAN) matching your hostname

./postgres_server \
  --cert /path/to/server.crt \
  --key /path/to/server.key
```

### Production Checklist

- [x] **Change Default Password**: Never use admin/admin
- [x] **Enable TLS**: Always use `--cert` and `--key`
- [x] **Certificate Validation**: Use CA-signed certificates
- [x] **Key Permissions**: `chmod 600` on private keys
- [x] **Firewall Rules**: Restrict database port (5433) to authorized IPs
- [x] **Regular Updates**: Rotate certificates before expiry
- [x] **Monitoring**: Log failed authentication attempts
- [x] **Backup Security**: Encrypt backups, secure backup storage

---

## User Management

### Creating Users

```sql
-- Create user with strong password
CREATE USER alice WITH PASSWORD 'StrongP@ssw0rd123!';

-- Password requirements:
-- - Minimum 8 characters
-- - Mix of uppercase, lowercase, numbers, symbols recommended
```

### Changing Passwords

```sql
-- Change user password
ALTER USER alice WITH PASSWORD 'NewSecureP@ss456!';

-- Force admin password change on first login (recommended)
ALTER USER admin WITH PASSWORD 'ComplexAdminP@ss789!';
```

### Dropping Users

```sql
-- Remove user
DROP USER alice;
```

### Listing Users

```sql
-- View all users (admin only)
SELECT * FROM users;
```

---

## Authentication

### SCRAM-SHA-256

OmenDB uses SCRAM-SHA-256 (Salted Challenge Response Authentication Mechanism) for secure password authentication:

1. **Password Storage**: Passwords are hashed with bcrypt (cost factor 12)
2. **Never Plaintext**: Passwords never stored or transmitted in plaintext
3. **Salt**: Each password has unique random salt
4. **PostgreSQL Compatible**: Standard PostgreSQL authentication protocol

### Connection Strings

```bash
# Basic authentication
psql "host=127.0.0.1 port=5433 user=alice password=SecurePass123!"

# With TLS (recommended)
psql "host=127.0.0.1 port=5433 user=alice password=SecurePass123! sslmode=require"

# Environment variable (more secure)
export PGPASSWORD='SecurePass123!'
psql -h 127.0.0.1 -p 5433 -U alice

# pgpass file (~/.pgpass) - most secure
# Format: hostname:port:database:username:password
127.0.0.1:5433:*:alice:SecurePass123!
chmod 600 ~/.pgpass
psql -h 127.0.0.1 -p 5433 -U alice
```

---

## TLS/SSL Configuration

### Server-Side TLS

```rust
use omendb::postgres::PostgresServer;
use datafusion::prelude::SessionContext;

let ctx = SessionContext::new();
let server = PostgresServer::with_addr("0.0.0.0:5433", ctx)
    .with_tls("certs/server.crt", "certs/server.key")?;

server.serve().await?;
```

### Client-Side TLS

```bash
# Require TLS (fail if not available)
psql "host=db.example.com port=5433 sslmode=require"

# Verify certificate (production)
psql "host=db.example.com port=5433 sslmode=verify-full sslrootcert=/path/to/ca.crt"

# Disable TLS (development only, NOT recommended)
psql "host=127.0.0.1 port=5433 sslmode=disable"
```

### SSL Modes

| Mode | Encryption | Certificate Validation | Use Case |
|------|-----------|----------------------|----------|
| `disable` | ❌ No | ❌ No | Development only |
| `allow` | ⚠️ Opportunistic | ❌ No | Not recommended |
| `prefer` | ⚠️ Opportunistic | ❌ No | Not recommended |
| `require` | ✅ Yes | ⚠️ No (MITM risk) | Basic production |
| `verify-ca` | ✅ Yes | ✅ CA only | Better production |
| `verify-full` | ✅ Yes | ✅ CA + hostname | **Recommended production** |

---

## Security Best Practices

### 1. Password Policy

```
✅ DO:
- Use unique passwords per database
- Minimum 12 characters for production
- Mix of uppercase, lowercase, numbers, symbols
- Rotate passwords every 90 days
- Use password managers (1Password, Bitwarden)

❌ DON'T:
- Reuse passwords across systems
- Use dictionary words
- Share passwords via email/Slack
- Store passwords in code or config files
- Use default passwords (admin/admin)
```

### 2. TLS Configuration

```
✅ DO:
- Always enable TLS in production
- Use CA-signed certificates (Let's Encrypt)
- Set sslmode=verify-full for clients
- Renew certificates before expiry (30 days)
- Use TLS 1.2+ only

❌ DON'T:
- Use self-signed certs in production
- Disable certificate validation
- Expose unencrypted port publicly
- Use expired certificates
- Skip hostname verification
```

### 3. Network Security

```bash
# Firewall: Allow only authorized IPs
sudo ufw allow from 10.0.0.0/8 to any port 5433 proto tcp

# Bind to specific interface (not 0.0.0.0 if possible)
./postgres_server --addr 10.0.1.5:5433

# Use VPN or private network for database access
```

### 4. Monitoring & Auditing

```bash
# Log failed authentication attempts
grep "authentication failed" /var/log/omendb/server.log

# Monitor connection patterns
tail -f /var/log/omendb/server.log | grep "Connection"

# Alert on suspicious activity
# - Multiple failed logins from same IP
# - Connections from unexpected IP ranges
# - Unusual query patterns
```

---

## Security Testing

### Running Security Tests

```bash
# All security tests (57 total)
cargo test --test security_integration_tests
cargo test --test tls_integration_tests
cargo test user_store_tests
cargo test auth_tests
cargo test user_management_sql_tests
cargo test catalog_user_management_tests

# Specific test categories
cargo test test_password_hashing_security
cargo test test_tls_certificate_validation
cargo test test_user_isolation
cargo test test_permission_boundary
```

### Manual Security Validation

```bash
# Test 1: Invalid credentials fail
psql -h 127.0.0.1 -p 5433 -U fake_user
# Expected: "authentication failed"

# Test 2: TLS enforced
psql "host=127.0.0.1 port=5433 sslmode=require"
# Expected: Connection succeeds with encryption

# Test 3: Weak password rejected
psql -h 127.0.0.1 -p 5433 -U admin -c "CREATE USER weak WITH PASSWORD '123'"
# Expected: Error (password too short)

# Test 4: User isolation
psql -h 127.0.0.1 -p 5433 -U alice -c "DROP USER bob"
# Expected: Permission denied (if implemented)
```

---

## Threat Model & Mitigations

### Threats Mitigated

| Threat | Mitigation | Status |
|--------|-----------|--------|
| **Man-in-the-Middle** | TLS encryption | ✅ Implemented |
| **Password Sniffing** | TLS + SCRAM-SHA-256 | ✅ Implemented |
| **Brute Force** | Rate limiting (future) | ⚠️ Roadmap |
| **SQL Injection** | Parameterized queries | ✅ DataFusion |
| **Unauthorized Access** | Authentication required | ✅ Implemented |
| **Weak Passwords** | Password validation | ✅ Implemented |
| **Credential Reuse** | Per-user salted hashes | ✅ Implemented |

### Known Limitations (v0.1.0)

- **No Rate Limiting**: Brute force attacks possible (roadmap: v0.2.0)
- **No IP Whitelisting**: Use firewall rules externally
- **No Audit Logging**: Query logging planned (v0.2.0)
- **No Row-Level Security**: Table-level only (v1.0.0+)
- **No Multi-Factor Auth**: Password-only (future consideration)

---

## Reporting Security Issues

**🔒 Responsible Disclosure**

If you discover a security vulnerability in OmenDB:

1. **DO NOT** create a public GitHub issue
2. Email security@omendb.com with:
   - Description of vulnerability
   - Steps to reproduce
   - Potential impact
   - Suggested fix (if available)
3. Allow 90 days for patching before public disclosure
4. We will credit researchers in security advisories

---

## Compliance & Standards

### Industry Standards

- ✅ **OWASP**: Password storage (bcrypt, salted hashes)
- ✅ **NIST 800-63B**: Authentication guidelines
- ✅ **PostgreSQL Security**: Wire protocol compatibility
- ✅ **TLS Best Practices**: CA-signed certificates, verify-full mode

### Future Compliance (Roadmap)

- **SOC 2 Type II**: Audit logging, access controls
- **GDPR**: Data encryption at rest, right to deletion
- **HIPAA**: Encryption, audit trails, access logs
- **PCI DSS**: Network segmentation, encryption

---

## Additional Resources

- **PostgreSQL Security**: https://www.postgresql.org/docs/current/client-authentication.html
- **TLS Best Practices**: https://wiki.mozilla.org/Security/Server_Side_TLS
- **Let's Encrypt**: https://letsencrypt.org/getting-started/
- **OWASP Password Storage**: https://cheatsheetseries.owasp.org/cheatsheets/Password_Storage_Cheat_Sheet.html

---

**Version History:**
- 2025-10-22: Initial security documentation (v0.1.0-dev)
- Phase 2 Security (Days 1-10) implementation complete
