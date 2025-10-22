# OmenDB Security Guide

**Last Updated**: October 21, 2025

This guide covers security features and best practices for deploying OmenDB in production environments.

## Table of Contents

1. [Authentication](#authentication)
2. [TLS/SSL Encryption](#tlsssl-encryption)
3. [User Management](#user-management)
4. [Network Security](#network-security)
5. [Security Best Practices](#security-best-practices)
6. [Deployment Examples](#deployment-examples)

---

## Authentication

OmenDB supports SCRAM-SHA-256 authentication for PostgreSQL connections.

### Enabling Authentication

```bash
# Create users in the OmenDB catalog
psql -h localhost -p 5433 -U admin -d postgres

# In psql:
CREATE USER alice WITH PASSWORD 'secure_password';
CREATE USER bob WITH PASSWORD 'another_password';
```

### User Management

```sql
-- Create user
CREATE USER username WITH PASSWORD 'password';

-- Change password
ALTER USER username WITH PASSWORD 'new_password';

-- Drop user
DROP USER username;

-- List users
SELECT * FROM system.users;
```

### Authentication Configuration

Users are stored in the system catalog (`system.users` table) with SCRAM-SHA-256 hashed passwords.

---

## TLS/SSL Encryption

### Current Status

**Direct TLS support is planned but not yet implemented** due to limitations in the current pgwire library version. PostgreSQL's wire protocol requires specific SSLRequest message handling that needs additional implementation work.

### Recommended Approach: TLS Termination at Reverse Proxy

For production deployments, use TLS termination at a reverse proxy or connection pooler. This is the **industry standard approach** used by most database deployments and provides several benefits:

- **Proven security**: Battle-tested TLS implementations
- **Performance**: Optimized TLS handling
- **Flexibility**: Easy certificate management and renewal
- **Monitoring**: Centralized connection monitoring
- **Load balancing**: Built-in connection pooling

---

## TLS Deployment Options

### Option 1: PgBouncer (Recommended)

PgBouncer is a lightweight PostgreSQL connection pooler with TLS support.

**Setup:**

```bash
# Install pgbouncer
apt-get install pgbouncer  # Ubuntu/Debian
brew install pgbouncer     # macOS

# Configure TLS
cat > /etc/pgbouncer/pgbouncer.ini << EOF
[databases]
mydb = host=127.0.0.1 port=5433 dbname=postgres

[pgbouncer]
listen_addr = 0.0.0.0
listen_port = 5432
auth_type = scram-sha-256
auth_file = /etc/pgbouncer/userlist.txt

# TLS Configuration
client_tls_sslmode = require
client_tls_cert_file = /etc/pgbouncer/server.crt
client_tls_key_file = /etc/pgbouncer/server.key
client_tls_ca_file = /etc/pgbouncer/ca.crt

# Connection pooling
max_client_conn = 1000
default_pool_size = 25
EOF

# Start pgbouncer
pgbouncer -d /etc/pgbouncer/pgbouncer.ini
```

**Client connection:**

```bash
psql "postgresql://username@hostname:5432/mydb?sslmode=require"
```

### Option 2: HAProxy

HAProxy provides high-performance load balancing and TLS termination.

**Setup:**

```bash
# Install HAProxy
apt-get install haproxy

# Configure TLS
cat > /etc/haproxy/haproxy.cfg << EOF
global
    log /dev/log local0
    chroot /var/lib/haproxy
    user haproxy
    group haproxy
    daemon

defaults
    log global
    mode tcp
    timeout connect 5000ms
    timeout client 50000ms
    timeout server 50000ms

frontend postgresql
    bind *:5432 ssl crt /etc/haproxy/postgres.pem
    mode tcp
    default_backend omendb

backend omendb
    mode tcp
    server omen1 127.0.0.1:5433 check
EOF

# Combine cert and key for HAProxy
cat /path/to/server.crt /path/to/server.key > /etc/haproxy/postgres.pem

# Restart HAProxy
systemctl restart haproxy
```

### Option 3: Nginx (TCP/UDP Load Balancing)

Nginx can provide TLS termination with the stream module.

**Setup:**

```nginx
stream {
    upstream omendb {
        server 127.0.0.1:5433;
    }

    server {
        listen 5432 ssl;
        proxy_pass omendb;

        ssl_certificate     /etc/nginx/certs/server.crt;
        ssl_certificate_key /etc/nginx/certs/server.key;
        ssl_protocols       TLSv1.2 TLSv1.3;
        ssl_ciphers         HIGH:!aNULL:!MD5;
    }
}
```

### Option 4: Stunnel

Stunnel is a simple TLS wrapper for TCP connections.

**Setup:**

```bash
# Install stunnel
apt-get install stunnel4

# Configure
cat > /etc/stunnel/postgres.conf << EOF
[postgres]
accept = 0.0.0.0:5432
connect = 127.0.0.1:5433
cert = /etc/stunnel/server.crt
key = /etc/stunnel/server.key
EOF

# Start stunnel
stunnel /etc/stunnel/postgres.conf
```

---

## Certificate Management

### Generating Self-Signed Certificates (Development Only)

For development and testing, use the provided script:

```bash
# Generate certificates
./scripts/generate_test_certs.sh /path/to/cert/dir

# Certificates will be created:
# - server.crt (certificate)
# - server.key (private key)
```

**⚠️ Warning**: Self-signed certificates are for testing only. Do not use in production.

### Production Certificates

For production, obtain certificates from a trusted Certificate Authority (CA):

**Option 1: Let's Encrypt (Free)**

```bash
# Install certbot
apt-get install certbot

# Generate certificate
certbot certonly --standalone -d database.example.com

# Certificates will be in:
# /etc/letsencrypt/live/database.example.com/fullchain.pem
# /etc/letsencrypt/live/database.example.com/privkey.pem
```

**Option 2: Commercial CA**

Purchase certificates from providers like:
- DigiCert
- GlobalSign
- Sectigo

**Option 3: Internal CA**

For internal networks, use your organization's Certificate Authority.

---

## Network Security

### Firewall Configuration

```bash
# Allow PostgreSQL port only from specific IPs
ufw allow from 10.0.0.0/8 to any port 5432 proto tcp
ufw enable

# Or using iptables
iptables -A INPUT -p tcp -s 10.0.0.0/8 --dport 5432 -j ACCEPT
iptables -A INPUT -p tcp --dport 5432 -j DROP
```

### Bind Address

By default, OmenDB listens on `127.0.0.1:5433` (localhost only). For production:

```bash
# Listen on specific interface
./postgres_server --addr 10.0.1.100:5433

# Listen on all interfaces (use with firewall!)
./postgres_server --addr 0.0.0.0:5433
```

### Docker Networking

When running in Docker, use internal networks:

```yaml
version: '3.8'
services:
  omendb:
    image: omendb:latest
    networks:
      - internal
    ports:
      - "127.0.0.1:5433:5433"  # Bind to localhost only

  pgbouncer:
    image: pgbouncer:latest
    networks:
      - internal
      - public
    ports:
      - "5432:5432"  # Public TLS endpoint

networks:
  internal:
    internal: true
  public:
```

---

## Security Best Practices

### 1. Authentication

- ✅ Always enable authentication in production
- ✅ Use strong passwords (minimum 12 characters)
- ✅ Rotate passwords regularly
- ✅ Use different passwords for each user
- ❌ Never use default passwords
- ❌ Never commit passwords to version control

### 2. Network Security

- ✅ Use TLS for all connections over untrusted networks
- ✅ Configure firewall rules to limit access
- ✅ Bind to specific interfaces when possible
- ✅ Use VPN for remote database access
- ❌ Never expose database directly to internet without TLS
- ❌ Never use `0.0.0.0` binding without firewall

### 3. Certificate Management

- ✅ Use certificates from trusted CAs in production
- ✅ Automate certificate renewal (e.g., with certbot)
- ✅ Use strong key sizes (2048-bit minimum, 4096-bit recommended)
- ✅ Monitor certificate expiration
- ❌ Never use self-signed certificates in production
- ❌ Never commit private keys to version control

### 4. Access Control

- ✅ Follow principle of least privilege
- ✅ Create separate users for different applications
- ✅ Audit user access regularly
- ✅ Remove unused accounts
- ❌ Never use admin accounts for applications
- ❌ Never share credentials between users

### 5. Monitoring

- ✅ Enable connection logging
- ✅ Monitor failed authentication attempts
- ✅ Set up alerts for unusual activity
- ✅ Review logs regularly
- ✅ Use metrics for connection monitoring

### 6. Updates

- ✅ Keep OmenDB updated to latest version
- ✅ Update TLS libraries regularly
- ✅ Monitor security advisories
- ✅ Test updates in staging first

---

## Deployment Examples

### Example 1: Single Server with PgBouncer

```
Internet → [Load Balancer:443] → [PgBouncer:5432 TLS] → [OmenDB:5433]
```

**Configuration:**
- Load balancer: HTTPS to TCP forwarding
- PgBouncer: TLS termination, SCRAM-SHA-256 auth
- OmenDB: Plain TCP, SCRAM-SHA-256 auth
- Network: Firewall rules limiting access

### Example 2: High Availability Setup

```
Internet → [HAProxy:5432 TLS] → [OmenDB-1:5433]
                               → [OmenDB-2:5433]
                               → [OmenDB-3:5433]
```

**Configuration:**
- HAProxy: TLS termination, health checks, load balancing
- OmenDB instances: Plain TCP, SCRAM-SHA-256 auth
- Network: Internal network, firewall protection

### Example 3: Development Setup

```
Developer → [Stunnel:5432 TLS] → [OmenDB:5433]
```

**Configuration:**
- Stunnel: TLS wrapper with self-signed cert
- OmenDB: Plain TCP, auth optional for dev
- Network: Localhost only

---

## Client Connection Examples

### psql

```bash
# Without TLS (development)
psql -h localhost -p 5433 -U alice -d postgres

# With TLS via PgBouncer
psql "postgresql://alice@hostname:5432/postgres?sslmode=require"

# With TLS and certificate verification
psql "postgresql://alice@hostname:5432/postgres?sslmode=verify-full&sslrootcert=/path/to/ca.crt"
```

### Python (psycopg2)

```python
import psycopg2

# Without TLS (development)
conn = psycopg2.connect(
    host="localhost",
    port=5433,
    user="alice",
    password="secure_password",
    dbname="postgres"
)

# With TLS via PgBouncer
conn = psycopg2.connect(
    host="hostname",
    port=5432,
    user="alice",
    password="secure_password",
    dbname="postgres",
    sslmode="require"
)
```

### Go (pgx)

```go
import "github.com/jackc/pgx/v5"

// With TLS
config, _ := pgx.ParseConfig(
    "postgresql://alice:password@hostname:5432/postgres?sslmode=require"
)
conn, _ := pgx.ConnectConfig(context.Background(), config)
```

### Rust (tokio-postgres)

```rust
use tokio_postgres::{NoTls, Config};

// Without TLS (development)
let (client, connection) = tokio_postgres::connect(
    "host=localhost port=5433 user=alice password=password",
    NoTls
).await?;

// With TLS (via reverse proxy)
let (client, connection) = tokio_postgres::connect(
    "host=hostname port=5432 user=alice password=password sslmode=require",
    TlsConnector::new()
).await?;
```

---

## Troubleshooting

### Connection Refused

```
Error: connection refused
```

**Solutions:**
- Check OmenDB is running: `ps aux | grep postgres_server`
- Verify listen address: Check server logs
- Check firewall: `sudo ufw status` or `sudo iptables -L`
- Verify port: `netstat -tlnp | grep 5433`

### Authentication Failed

```
Error: password authentication failed
```

**Solutions:**
- Verify user exists: `SELECT * FROM system.users;`
- Check password is correct
- Verify SCRAM-SHA-256 is enabled
- Check client authentication method

### TLS Connection Issues

```
Error: SSL connection has been closed unexpectedly
```

**Solutions:**
- Verify certificate is valid: `openssl x509 -in server.crt -text -noout`
- Check certificate matches hostname
- Verify TLS version compatibility
- Check reverse proxy logs

### Certificate Verification Failed

```
Error: certificate verify failed
```

**Solutions:**
- Install CA certificate on client
- Use `sslmode=require` instead of `verify-full` for self-signed certs
- Check certificate expiration date
- Verify certificate chain is complete

---

## Future Enhancements

### Planned for Future Releases

1. **Native TLS Support**: Direct PostgreSQL wire protocol TLS (SSLRequest handling)
2. **Row-Level Security**: Fine-grained access control
3. **Audit Logging**: Comprehensive query and access logging
4. **LDAP Integration**: Enterprise directory service integration
5. **OAuth 2.0 Support**: Modern authentication flows
6. **Certificate Authentication**: Client certificate support
7. **IP Allowlisting**: Built-in IP-based access control

---

## Security Contact

For security issues, please email: security@omendb.com

**Do not** file public GitHub issues for security vulnerabilities.

---

## References

- [PostgreSQL Security](https://www.postgresql.org/docs/current/security.html)
- [PgBouncer Documentation](https://www.pgbouncer.org/)
- [HAProxy Documentation](http://www.haproxy.org/)
- [Let's Encrypt](https://letsencrypt.org/)
- [SCRAM-SHA-256](https://tools.ietf.org/html/rfc7677)

---

*Last Updated: October 21, 2025*
