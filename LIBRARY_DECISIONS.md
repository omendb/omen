# Library Decisions for OmenDB v1.0

**Date:** 2025-09-30
**Status:** Architecture review

---

## ✅ Already Decided

| Component | Library | Version | Justification |
|-----------|---------|---------|---------------|
| **Storage** | `redb` | 2.1 | Pure Rust, ACID, MVCC, 1.0 stable |
| **SQL Engine** | `datafusion` | 43 | Production SQL optimizer, Arrow-native |
| **Columnar** | `arrow` | 53 | Industry standard, DataFusion compatible |
| **Persistence** | `parquet` | 53 | Efficient columnar format |
| **Metrics** | `prometheus` | 0.13 | Industry standard monitoring |
| **Logging** | `tracing` | 0.1 | Structured logging standard |
| **TLS** | `rustls` | 0.21 | Modern, safe TLS |
| **Async Runtime** | `tokio` | 1.40 | De facto async standard |

---

## 🤔 Need to Decide

### 1. **PostgreSQL Wire Protocol** ⭐ IMPORTANT

**Options:**

| Library | Pros | Cons | Recommendation |
|---------|------|------|----------------|
| **pgwire** | ✅ Most mature<br>✅ Full protocol support<br>✅ Active maintenance | ⚠️ Async-first (good fit with tokio) | **USE THIS** |
| **pg_wire** | ✅ Simpler API | ❌ Less complete | Not recommended |
| **Custom** | ✅ Full control | ❌ Months of work<br>❌ Protocol complexity | Definitely not |

**Decision:** **Use `pgwire`**

```toml
pgwire = "0.27"
```

**Why:**
- PostgreSQL compatibility = massive ecosystem
- All language drivers work (Python, JS, Go, etc.)
- Tools: pgAdmin, DBeaver, Grafana
- COPY protocol for bulk loading

---

### 2. **HTTP/REST Server** (for management API)

**Options:**

| Library | Pros | Cons | Recommendation |
|---------|------|------|----------------|
| **axum** | ✅ Fast<br>✅ Type-safe<br>✅ Tokio-native<br>✅ Modern | Newer | **USE THIS** |
| **actix-web** | ✅ Very mature<br>✅ Fast | Different async runtime | Not with tokio |
| **warp** | ✅ Composable | ❌ Complex API | Overkill |
| **hyper** (current) | ✅ Low-level | ⚠️ Already using for metrics | Keep for metrics |

**Decision:** **Add `axum` for REST API, keep `hyper` for metrics endpoint**

```toml
axum = "0.7"
tower = "0.4"          # Middleware
tower-http = "0.5"     # HTTP middleware (CORS, compression, etc.)
```

**Why:**
- Management API: `/api/databases`, `/api/tables`, `/api/query`
- Health checks: `/health`, `/ready` (already have via hyper)
- Metrics: Keep hyper (already working)

---

### 3. **Caching Layer** ⭐ IMPORTANT for Performance

**Options:**

| Library | Type | Pros | Cons | Recommendation |
|---------|------|------|------|----------------|
| **moka** | In-memory LRU | ✅ Async-aware<br>✅ TTL support<br>✅ High performance | | **USE THIS** |
| **quick_cache** | In-memory | ✅ Very fast | ❌ No async | Second choice |
| **redis** | External | ✅ Distributed caching | ❌ External dependency | Not for embedded |

**Decision:** **Use `moka` for query result caching**

```toml
moka = "0.12"
```

**Use Cases:**
- Query plan caching (DataFusion)
- Learned index model caching
- Metadata caching (table schemas)
- Hot row caching

---

### 4. **Connection Pooling** (for server)

**Options:**

| Library | Pros | Cons | Recommendation |
|---------|------|------|----------------|
| **deadpool** | ✅ Generic pooling<br>✅ Tokio-native | Requires custom adapter | **USE THIS** |
| **bb8** | ✅ Simple API | ⚠️ Less flexible | Second choice |
| **Custom** | Already have! | ✅ Tailored to our needs | **KEEP CURRENT** |

**Decision:** **Keep our custom `ConnectionPool` (already implemented and tested)**

**Rationale:**
- Already have 19 passing tests
- Tailored to our architecture
- No external dependencies needed

---

### 5. **Schema Management & Migrations**

**Options:**

| Library | Pros | Cons | Recommendation |
|---------|------|------|----------------|
| **refinery** | ✅ Embedded migrations<br>✅ Multiple DBs | Need adapter for redb | Could use |
| **Custom** | ✅ Full control | Need to build | **BUILD SIMPLE VERSION** |

**Decision:** **Build simple schema versioning** (not urgent for v1.0)

```rust
// Future: src/schema/migrations.rs
pub struct Migration {
    version: u32,
    up: fn(&Database) -> Result<()>,
    down: fn(&Database) -> Result<()>,
}
```

**Defer to v1.1** - Not critical for initial release

---

### 6. **Query Result Serialization**

**Current:** `serde_json` ✅

**Options for optimization:**

| Format | Library | Use Case | Add? |
|--------|---------|----------|------|
| JSON | `serde_json` ✅ | HTTP API | Keep |
| MessagePack | `rmp-serde` | Binary protocol | Maybe |
| Apache Arrow IPC | `arrow-ipc` ✅ | DataFusion native | Already have |
| CSV | `csv` | Data export | Add for COPY |

**Decision:** **Add `csv` for PostgreSQL COPY support**

```toml
csv = "1.3"
```

---

### 7. **Compression** (for network & storage)

**Current:** `flate2` (gzip) ✅

**Options:**

| Library | Algorithm | Speed | Ratio | Use Case |
|---------|-----------|-------|-------|----------|
| `flate2` ✅ | gzip/deflate | Medium | Good | Keep for backup |
| `zstd` | Zstandard | **Fast** | **Best** | **ADD for Parquet** |
| `lz4-flex` | LZ4 | Fastest | Lower | Maybe for network |

**Decision:** **Add `zstd` for Parquet compression**

```toml
zstd = "0.13"
```

**Why:** Parquet + zstd = industry standard for columnar data

---

### 8. **Rate Limiting** (for production safety)

**Options:**

| Library | Type | Recommendation |
|---------|------|----------------|
| **governor** | Token bucket | **USE THIS** |
| **tower-governor** | Tower middleware | Use with axum |
| Custom | Leaky bucket | Overkill |

**Decision:** **Add `governor` for API rate limiting**

```toml
governor = "0.6"
tower-governor = "0.4"
```

**Use Cases:**
- Limit queries per client
- Protect against abuse
- Fair resource allocation

---

### 9. **Configuration Management**

**Current:** Custom via `clap` + `env_logger`

**Better approach:**

| Library | Pros | Recommendation |
|---------|------|----------------|
| **figment** | ✅ Multiple sources (env, file, CLI)<br>✅ Type-safe<br>✅ Profiles | **USE THIS** |
| **config** | ✅ Simple | Less flexible |

**Decision:** **Add `figment` for configuration**

```toml
figment = { version = "0.10", features = ["toml", "env"] }
```

**Example:**
```toml
# omendb.toml
[server]
host = "0.0.0.0"
port = 5432

[storage]
path = "./data"
cache_size = "1GB"

[query]
timeout = "30s"
max_connections = 100
```

---

### 10. **CLI Framework** (for admin tools)

**Current:** `clap` ✅ (already using)

**Enhancement:**

```toml
# Already have
clap = { version = "4.0", features = ["derive"] }

# Add for better UX
clap_complete = "4.0"  # Shell completions
```

---

### 11. **Distributed Tracing** (for production observability)

**Current:** `tracing` ✅

**Add for full stack:**

```toml
# Already have
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["json", "env-filter"] }

# Add for distributed tracing
tracing-opentelemetry = "0.26"
opentelemetry = "0.26"
opentelemetry-jaeger = "0.26"  # Or use OTLP
```

**Use Cases:**
- Query execution tracing
- Cross-service requests
- Performance profiling
- Production debugging

**Decision:** **Add for v1.1** (not critical for v1.0)

---

### 12. **Backup & Point-in-Time Recovery**

**Current:** Custom backup in `src/backup.rs`

**Better approach:**

| Library | Pros | Recommendation |
|---------|------|----------------|
| **Custom** | ✅ Already have | **ENHANCE CURRENT** |
| `async-compression` | Streaming compression | Add this |
| `aws-sdk-s3` | S3 backup | Add for cloud |

**Decision:** **Enhance current backup with streaming**

```toml
async-compression = { version = "0.4", features = ["tokio", "zstd"] }
aws-sdk-s3 = "1.0"  # Optional feature
```

---

### 13. **Testing Libraries**

**Current:**
- `criterion` ✅ (benchmarks)
- `proptest` ✅ (property testing)
- `quickcheck` ✅ (property testing)
- `tempfile` ✅ (temp directories)

**Add:**

```toml
[dev-dependencies]
# Already have
criterion = "0.5"
proptest = "1.4"
tempfile = "3.8"

# Add for integration testing
testcontainers = "0.23"  # For PostgreSQL compatibility tests
wiremock = "0.6"         # For HTTP testing
assert_cmd = "2.0"       # For CLI testing
predicates = "3.0"       # For assertions
```

---

### 14. **Error Handling Enhancement**

**Current:** `anyhow` + `thiserror` ✅

**Add for better errors:**

```toml
# Already have
anyhow = "1.0"
thiserror = "1.0"

# Add for rich error context
miette = { version = "7.0", features = ["fancy"] }  # Beautiful error reports
```

**Why:** Better error messages for users (SQL syntax errors, etc.)

---

### 15. **Async Utilities**

**Current:** `tokio` + `async-trait` ✅

**Add:**

```toml
# Already have
tokio = { version = "1.40", features = ["full"] }
async-trait = "0.1"

# Add for utilities
futures = "0.3"          # Async combinators
pin-project-lite = "0.2" # Pin projections
```

---

## 📋 Final Recommended Additions

### **Critical for v1.0:**

```toml
# PostgreSQL protocol (CRITICAL)
pgwire = "0.27"

# HTTP REST API
axum = "0.7"
tower = "0.4"
tower-http = "0.5"

# Caching
moka = "0.12"

# Configuration
figment = { version = "0.10", features = ["toml", "env"] }

# Compression
zstd = "0.13"

# CSV export (PostgreSQL COPY)
csv = "1.3"

# Rate limiting
governor = "0.6"
tower-governor = "0.4"

# Async utilities
futures = "0.3"
```

### **Nice-to-Have for v1.0:**

```toml
# Better error messages
miette = { version = "7.0", features = ["fancy"] }

# Shell completions
clap_complete = "4.0"

# Streaming compression
async-compression = { version = "0.4", features = ["tokio", "zstd"] }
```

### **Defer to v1.1:**

```toml
# Distributed tracing
tracing-opentelemetry = "0.26"
opentelemetry-jaeger = "0.26"

# Cloud backup
aws-sdk-s3 = "1.0"

# Schema migrations
refinery = "0.8"
```

---

## 🎯 Updated Architecture

```
┌─────────────────────────────────────────┐
│     PostgreSQL Wire Protocol (pgwire)   │ ← NEW
│     REST API (axum + tower)             │ ← NEW
└─────────────────────────────────────────┘
                    │
┌─────────────────────────────────────────┐
│   Query Cache (moka)                    │ ← NEW
│   Rate Limiting (governor)              │ ← NEW
└─────────────────────────────────────────┘
                    │
┌─────────────────────────────────────────┐
│     SQL Engine (DataFusion)             │ ✅
└─────────────────────────────────────────┘
                    │
        ┌───────────┴───────────┐
        │                       │
┌───────▼────────┐    ┌────────▼────────┐
│  OLTP (redb)   │    │  OLAP (Parquet) │
│  + Learned     │    │  + zstd         │ ← NEW
│    Index       │    │    compression  │
└────────────────┘    └─────────────────┘
```

---

## Summary

**Add these 10 libraries for production-grade v1.0:**

1. ✅ `pgwire` - PostgreSQL protocol
2. ✅ `axum` - REST API
3. ✅ `moka` - Query caching
4. ✅ `figment` - Configuration
5. ✅ `zstd` - Compression
6. ✅ `csv` - Data export
7. ✅ `governor` - Rate limiting
8. ✅ `futures` - Async utilities
9. ✅ `miette` - Error messages
10. ✅ `async-compression` - Streaming

**Total new dependencies: 10**
**Time to integrate: 2-3 weeks**

All are mature, production-ready libraries used in major Rust projects.
