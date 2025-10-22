# Phase 2: Security Implementation Plan

**Date**: October 21, 2025
**Duration**: 1-2 weeks (10 days)
**Status**: IN PROGRESS - Days 1-4 COMPLETE ✅
**Priority**: HIGH - Required for production deployment

---

## Executive Summary

**Goal**: Implement complete authentication and SSL/TLS security for PostgreSQL wire protocol

**Current Status (Oct 21, 2025 Night)**:
- ✅ SCRAM-SHA-256 authentication working (src/postgres/auth.rs)
- ✅ OmenDbAuthSource with **persistent RocksDB user store** ⭐ NEW
- ✅ **32/32 security tests passing** ⭐ NEW
- ✅ Example server (postgres_server_auth.rs)
- ✅ Basic HTTP auth + TLS infrastructure (src/security.rs)
- ✅ **CREATE USER / DROP USER / ALTER USER SQL commands** ⭐ NEW
- ✅ **User persistence to disk (RocksDB)** ⭐ NEW

**What's Missing**:
- ❌ SSL/TLS for PostgreSQL wire protocol (Days 6-7)
- ❌ Integration with main catalog system (Day 5)
- ❌ Comprehensive security integration tests (Day 8)
- ❌ Security documentation (Day 9)

**Success Criteria**:
- [✅] SQL user management commands working (CREATE USER, DROP USER, ALTER USER)
- [✅] Users persist across database restarts
- [ ] SSL/TLS connections working for PostgreSQL protocol
- [✅] 32+ security tests passing (target: 50+)
- [✅] No hardcoded credentials in production mode
- [ ] Integration with existing postgres_server

---

## Architecture Overview

### Current Stack (Before Phase 2)
```
PostgreSQL Wire Protocol (Port 5433)
├── pgwire crate (with SCRAM feature)
├── OmenDbAuthSource (in-memory, working ✅)
└── SessionContext (no auth checks)
```

### Target Stack (After Phase 2)
```
PostgreSQL Wire Protocol (Port 5433) + SSL/TLS ⭐ NEW
├── pgwire crate (with SCRAM feature)
├── OmenDbAuthSource (disk-persisted ⭐ NEW)
├── UserStore (RocksDB-backed ⭐ NEW)
├── SQL Engine (CREATE USER / DROP USER support ⭐ NEW)
└── Catalog integration (user management ⭐ NEW)
```

---

## Week 1: User Management (Days 1-5)

### Day 1: User Persistence Layer

**Task**: Implement disk-backed user storage using RocksDB

**Files to Create/Modify**:
- `src/user_store.rs` (NEW - 300 lines)
  - UserStore struct with RocksDB backend
  - save_user() / load_user() / delete_user()
  - list_users()
  - Atomic operations for concurrent access

**Implementation**:
```rust
pub struct UserStore {
    db: Arc<rocksdb::DB>,
}

pub struct User {
    username: String,
    salted_password: Vec<u8>,
    salt: Vec<u8>,
    created_at: i64,
    roles: Vec<String>,  // For future RBAC
}

impl UserStore {
    pub fn new(path: impl AsRef<Path>) -> Result<Self> {
        // Open RocksDB with "users" column family
    }

    pub fn create_user(&self, user: &User) -> Result<()> {
        // Serialize and save to RocksDB
    }

    pub fn delete_user(&self, username: &str) -> Result<bool> {
        // Delete from RocksDB
    }

    pub fn get_user(&self, username: &str) -> Result<Option<User>> {
        // Load from RocksDB
    }

    pub fn list_users(&self) -> Result<Vec<String>> {
        // Iterate through RocksDB
    }
}
```

**Tests** (10 tests):
- test_create_user
- test_delete_user
- test_get_user
- test_list_users
- test_duplicate_user_error
- test_persistence_across_restart
- test_concurrent_user_creation
- test_user_serialization
- test_empty_store
- test_invalid_username

**Deliverable**: UserStore with 10/10 tests passing

---

### Day 2: Integrate UserStore with OmenDbAuthSource

**Task**: Replace in-memory HashMap with UserStore backend

**Files to Modify**:
- `src/postgres/auth.rs` (50 lines changed)
  - Replace HashMap with UserStore
  - Update add_user() / remove_user() to use UserStore
  - Maintain backward compatibility

**Changes**:
```rust
pub struct OmenDbAuthSource {
    // OLD: users: Arc<RwLock<HashMap<String, UserCredentials>>>,
    // NEW:
    user_store: Arc<UserStore>,
    iterations: usize,
}

impl OmenDbAuthSource {
    pub fn new(data_dir: impl AsRef<Path>) -> Result<Self> {
        Ok(Self {
            user_store: Arc::new(UserStore::new(data_dir)?),
            iterations: 4096,
        })
    }

    pub async fn add_user(&self, username: impl Into<String>, password: &str) -> Result<()> {
        let username = username.into();
        let salt: [u8; 16] = rand::random();
        let salted_password = gen_salted_password(password, &salt, self.iterations);

        let user = User {
            username: username.clone(),
            salted_password,
            salt: salt.to_vec(),
            created_at: chrono::Utc::now().timestamp(),
            roles: vec![],
        };

        self.user_store.create_user(&user)?;
        Ok(())
    }
}
```

**Tests** (4 existing tests should still pass):
- test_add_and_authenticate_user (modified to use data_dir)
- test_remove_user
- test_nonexistent_user
- test_no_username
- test_persistence_across_restarts (NEW)

**Deliverable**: OmenDbAuthSource with persistent storage

---

### Days 3-4: SQL User Management Commands

**Task**: Implement CREATE USER / DROP USER / ALTER USER

**Files to Modify**:
- `src/sql_engine.rs` (200 lines added)
  - parse_create_user()
  - parse_drop_user()
  - parse_alter_user()
  - execute_create_user()
  - execute_drop_user()
  - execute_alter_user()

**SQL Syntax**:
```sql
CREATE USER alice WITH PASSWORD 'secret123';
CREATE USER bob WITH PASSWORD 'pass456' ROLE admin;
DROP USER alice;
ALTER USER bob PASSWORD 'newpass789';
ALTER USER alice ROLE readonly;
```

**Implementation**:
```rust
// In sql_engine.rs
fn execute_statement(&self, stmt: &Statement) -> Result<Vec<Row>> {
    match stmt {
        Statement::CreateUser { username, password, role } => {
            self.execute_create_user(username, password, role.as_deref())
        }
        Statement::DropUser { username } => {
            self.execute_drop_user(username)
        }
        Statement::AlterUser { username, password, role } => {
            self.execute_alter_user(username, password.as_deref(), role.as_deref())
        }
        // ... existing cases
    }
}

fn execute_create_user(&self, username: &str, password: &str, role: Option<&str>) -> Result<Vec<Row>> {
    // Validate username (alphanumeric + underscore)
    if !username.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(anyhow!("Invalid username: must be alphanumeric"));
    }

    // Password strength validation
    if password.len() < 8 {
        return Err(anyhow!("Password must be at least 8 characters"));
    }

    // Add user via auth source
    self.auth_source.add_user(username, password).await?;

    Ok(vec![Row::new(vec![Value::Text(format!("CREATE USER {}", username))])])
}
```

**Parsing** (using sqlparser crate):
- Extend sqlparser::ast::Statement with CreateUser, DropUser, AlterUser variants
- Or: Parse as custom statement if not supported by sqlparser

**Tests** (15 tests):
- test_create_user_basic
- test_create_user_with_role
- test_create_user_duplicate_error
- test_create_user_invalid_name
- test_create_user_weak_password
- test_drop_user_basic
- test_drop_user_nonexistent
- test_alter_user_password
- test_alter_user_role
- test_alter_user_nonexistent
- test_user_management_integration
- test_create_user_sql_injection_prevention
- test_special_characters_in_password
- test_unicode_username
- test_case_sensitive_username

**Deliverable**: SQL user management working with 15/15 tests

---

### Day 5: Catalog Integration

**Task**: Integrate UserStore with Catalog for unified management

**Files to Modify**:
- `src/catalog.rs` (100 lines added)
  - Add user_store field
  - create_user() / drop_user() / list_users()
  - Load users on catalog initialization

**Changes**:
```rust
pub struct Catalog {
    // ... existing fields
    user_store: Arc<UserStore>,
}

impl Catalog {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let user_store = Arc::new(UserStore::new(data_dir.join("users"))?);

        // Create default admin user if no users exist
        if user_store.list_users()?.is_empty() {
            println!("Creating default admin user (change password immediately!)");
            user_store.create_user(&User {
                username: "admin".to_string(),
                salted_password: gen_salted_password("changeme", &[0u8; 16], 4096),
                salt: vec![0u8; 16],
                created_at: chrono::Utc::now().timestamp(),
                roles: vec!["admin".to_string()],
            })?;
        }

        Ok(Self {
            // ... existing fields
            user_store,
        })
    }

    pub fn create_user(&self, username: &str, password: &str) -> Result<()> {
        self.user_store.create_user(&User {
            username: username.to_string(),
            // ... hash password
        })
    }
}
```

**Tests** (5 tests):
- test_catalog_user_management
- test_default_admin_user_creation
- test_catalog_restart_preserves_users
- test_user_isolation_per_catalog
- test_concurrent_catalog_user_ops

**Deliverable**: Catalog with integrated user management

---

## Week 2: SSL/TLS & Testing (Days 6-10)

### Day 6-7: SSL/TLS for PostgreSQL Wire Protocol

**Task**: Enable TLS encryption for PostgreSQL connections

**Files to Modify**:
- `src/postgres/server.rs` (150 lines added)
  - Add TLS configuration support
  - Load certificates from disk
  - Enable pgwire TLS mode

- `src/bin/postgres_server.rs` (50 lines modified)
  - Add --tls flag
  - Load TLS config from environment

**Implementation**:
```rust
// In postgres/server.rs
use tokio_rustls::{TlsAcceptor, rustls::ServerConfig};

pub struct PostgresServer {
    // ... existing fields
    tls_config: Option<Arc<ServerConfig>>,
}

impl PostgresServer {
    pub fn with_tls(addr: &str, ctx: Arc<RwLock<SessionContext>>,
                    auth_source: Arc<OmenDbAuthSource>,
                    tls_config: Arc<ServerConfig>) -> Self {
        Self {
            address: addr.to_string(),
            session_ctx: ctx,
            auth_source: Some(auth_source),
            tls_config: Some(tls_config),
        }
    }

    pub async fn serve(self) -> Result<()> {
        let listener = tokio::net::TcpListener::bind(&self.address).await?;

        if let Some(tls_config) = &self.tls_config {
            info!("TLS enabled for PostgreSQL connections");
            let tls_acceptor = TlsAcceptor::from(tls_config.clone());

            loop {
                let (socket, _) = listener.accept().await?;
                let tls_stream = tls_acceptor.accept(socket).await?;

                // Handle connection with TLS stream
                self.handle_connection(tls_stream).await?;
            }
        } else {
            // Existing non-TLS path
            // ...
        }
    }
}
```

**Certificate Management**:
- Support PEM format certificates
- Load from configurable paths
- Development: Self-signed cert generation script
- Production: User-provided certificates

**Environment Variables**:
```bash
export OMENDB_TLS_ENABLED=true
export OMENDB_TLS_CERT=/path/to/server.crt
export OMENDB_TLS_KEY=/path/to/server.key
export OMENDB_TLS_REQUIRE=true  # Reject non-TLS connections
```

**Tests** (10 tests):
- test_tls_connection_success
- test_tls_certificate_validation
- test_tls_connection_rejected_without_cert
- test_tls_and_auth_together
- test_tls_optional_mode
- test_tls_required_mode
- test_tls_config_from_env
- test_invalid_certificate_rejected
- test_expired_certificate_rejected
- test_tls_performance_overhead

**Deliverable**: TLS working with 10/10 tests

---

### Day 8: Integration Testing

**Task**: Comprehensive security integration tests

**Files to Create**:
- `tests/security_integration_tests.rs` (NEW - 500 lines)
  - 20+ comprehensive integration tests
  - Auth + TLS + SQL user management together
  - Real psql client connections
  - Concurrent security scenarios

**Test Categories**:

**Authentication Tests** (10):
- test_scram_sha256_auth_success
- test_scram_sha256_auth_failure
- test_no_auth_when_disabled
- test_concurrent_auth_requests
- test_auth_timeout
- test_max_auth_retries
- test_auth_with_special_chars_password
- test_auth_unicode_username
- test_auth_case_sensitivity
- test_auth_after_user_deletion

**TLS Tests** (5):
- test_tls_required_rejects_plain
- test_tls_optional_accepts_both
- test_tls_cipher_suite_negotiation
- test_tls_protocol_version
- test_tls_certificate_chain

**SQL User Management Tests** (10):
- test_create_user_via_sql
- test_drop_user_via_sql
- test_alter_user_password_via_sql
- test_user_persistence_after_restart
- test_sql_injection_in_create_user
- test_concurrent_user_creation
- test_duplicate_user_error
- test_drop_nonexistent_user
- test_alter_nonexistent_user
- test_admin_user_cannot_be_deleted

**End-to-End Tests** (5):
- test_full_auth_flow_with_psql
- test_tls_auth_together
- test_multiple_users_concurrent_access
- test_user_management_across_restart
- test_security_config_from_env

**Performance Tests** (3):
- test_auth_latency_overhead
- test_tls_throughput_impact
- test_1000_concurrent_auth_requests

**Deliverable**: 33+ security integration tests passing

---

### Day 9: Documentation & Examples

**Task**: Complete security documentation

**Files to Create**:
- `docs/SECURITY.md` (NEW)
  - Authentication setup guide
  - TLS certificate generation
  - User management best practices
  - Security configuration reference

- `examples/secure_deployment.rs` (NEW)
  - Full example with auth + TLS
  - Production-ready configuration
  - Monitoring and logging

- `scripts/generate_certs.sh` (NEW)
  - Self-signed cert generation for development
  - Instructions for production certificates

**Content**:

**SECURITY.md Structure**:
1. Overview (authentication, TLS, user management)
2. Quick Start (development mode)
3. Production Deployment
4. User Management (CREATE USER, DROP USER, ALTER USER)
5. TLS Configuration (certificates, cipher suites)
6. Security Best Practices
7. Troubleshooting
8. Security Audit Checklist

**Example Configuration**:
```toml
# config/production.toml
[security.auth]
enabled = true
session_timeout = 3600
iterations = 4096

[security.tls]
enabled = true
cert_file = "/etc/omendb/certs/server.crt"
key_file = "/etc/omendb/certs/server.key"
require = true  # Reject non-TLS connections

[security.password_policy]
min_length = 12
require_uppercase = true
require_lowercase = true
require_digit = true
require_special = true
```

**Deliverable**: Complete security documentation

---

### Day 10: Final Validation & Cleanup

**Task**: Final testing, performance validation, security audit

**Activities**:

1. **Run Full Test Suite**
   - 429 library tests ✓
   - 7 cache tests ✓
   - 14 MVCC tests ✓
   - 10 user store tests ✓
   - 33+ security integration tests ✓
   - **Target: 493+ tests passing**

2. **Performance Validation**
   - Benchmark auth overhead (<10ms per connection)
   - Benchmark TLS overhead (<20% throughput reduction)
   - Validate no regression in query performance

3. **Security Audit**
   - Run `cargo audit` (no vulnerabilities)
   - Check for hardcoded credentials (none in production mode)
   - Verify password hashing (SCRAM-SHA-256 with 4096 iterations)
   - Test TLS cipher suites (strong ciphers only)

4. **Code Cleanup**
   - Remove debug logging with passwords
   - Ensure no TODO/FIXME related to security
   - Run `cargo clippy -- -D warnings`
   - Update ARCHITECTURE.md with security layer

**Security Checklist**:
- [ ] No hardcoded passwords in source
- [ ] No passwords in logs (even at debug level)
- [ ] SCRAM-SHA-256 authentication working
- [ ] TLS 1.2+ only (no SSL, no TLS 1.0/1.1)
- [ ] Strong cipher suites configured
- [ ] User passwords hashed with PBKDF2 (4096 iterations)
- [ ] Default admin user warns to change password
- [ ] SQL injection tests passing
- [ ] Rate limiting for auth attempts (optional, nice to have)
- [ ] Audit logging for user management (optional, nice to have)

**Deliverable**: Production-ready security implementation

---

## Success Criteria (Phase 2 Complete)

**Must Have** (required for completion):
- [ ] CREATE USER / DROP USER / ALTER USER SQL commands working
- [ ] Users persist to disk (RocksDB-backed)
- [ ] SCRAM-SHA-256 authentication working
- [ ] TLS/SSL for PostgreSQL connections
- [ ] 50+ security tests passing (currently 33+ planned, add more as needed)
- [ ] Zero cargo audit vulnerabilities
- [ ] No hardcoded credentials in production mode
- [ ] Complete security documentation

**Should Have** (target goals):
- [ ] Performance overhead: <10ms auth, <20% TLS throughput impact
- [ ] Example secure deployment configuration
- [ ] Certificate generation scripts
- [ ] Security best practices guide

**Nice to Have** (stretch goals):
- [ ] Rate limiting for auth attempts
- [ ] Audit logging for user operations
- [ ] Role-based access control (RBAC) basic support
- [ ] Table-level permissions (GRANT/REVOKE)
- [ ] Password complexity policies

---

## Files Summary

**New Files** (5):
- `src/user_store.rs` (300 lines)
- `tests/security_integration_tests.rs` (500 lines)
- `docs/SECURITY.md` (documentation)
- `examples/secure_deployment.rs` (150 lines)
- `scripts/generate_certs.sh` (script)

**Modified Files** (5):
- `src/postgres/auth.rs` (50 lines changed)
- `src/sql_engine.rs` (200 lines added)
- `src/catalog.rs` (100 lines added)
- `src/postgres/server.rs` (150 lines added)
- `src/bin/postgres_server.rs` (50 lines modified)

**Total New/Modified Lines**: ~1,500 lines

---

## Risk Mitigation

**Risk**: TLS integration with pgwire is complex
**Mitigation**: pgwire crate already supports TLS, follow their examples. Start with non-TLS and add TLS incrementally.

**Risk**: User persistence causes performance regression
**Mitigation**: RocksDB is already used for storage, user operations are infrequent. Benchmark early.

**Risk**: SQL parsing for user commands is tricky
**Mitigation**: Start with simple regex-based parsing if sqlparser doesn't support. Can refine later.

**Risk**: Timeline slip (10 days might not be enough)
**Mitigation**:
- Must-have: User management + TLS (Days 1-7)
- Nice-to-have: RBAC, permissions (defer to Phase 3 or later)
- Cut scope if needed, document deferred features

---

## Next Steps After Phase 2

**Phase 3 Week 3-4**: SQL Features (2 weeks)
- Aggregations with JOINs (GROUP BY, HAVING)
- Subqueries
- Multi-way joins
- ORDER BY for JOINs

**Phase 4**: Observability (1 week)
- EXPLAIN query plans
- Query logging
- Metrics export

**Phase 5**: Backup/Restore (1 week)
- Online backup
- Point-in-time recovery

---

## Ready to Start

Phase 2 is ready to begin. All prerequisites are in place:
- ✅ SCRAM-SHA-256 authentication working
- ✅ Basic TLS infrastructure in place
- ✅ PostgreSQL wire protocol stable
- ✅ RocksDB storage available
- ✅ Test infrastructure ready

**First Task**: Day 1 - Implement UserStore with RocksDB persistence

---

**Date**: October 21, 2025
**Status**: READY TO START
**Timeline**: 10 days to completion
**Next**: Begin Day 1 implementation
