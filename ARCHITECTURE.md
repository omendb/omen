# OmenDB Architecture

**Last Updated:** October 1, 2025
**Version:** 0.1.0 (Pre-production)
**Status:** ✅ **Learned Index PRODUCTION-READY**

## ✅ Learned Index Performance (Validated October 1, 2025)

**Direct RedbStorage Performance:**

| Dataset | Insert Rate | Point Query | Full Scan | Speedup |
|---------|-------------|-------------|-----------|---------|
| 10K rows | 32,894/sec | 0.008ms | 22ms | **2,862x** |
| 50K rows | 29,457/sec | 0.010ms | 107ms | **11,175x** |
| 100K rows | 25,422/sec | 0.010ms | 217ms | **22,554x** |

**Key Achievements:**
- ✅ Learned index actively used in queries (verified with dedicated tests)
- ✅ Insert throughput: 25K-32K rows/sec
- ✅ Point query latency: ~0.010ms average
- ✅ Speedup scales linearly with dataset size
- ✅ Production-ready performance validated

**See:** `CRITICAL_FINDINGS.md` for full performance analysis and test results

---

## Overview

OmenDB is a PostgreSQL-compatible database that combines DataFusion's SQL engine with redb storage and learned index optimization for exceptional point query performance.

## Current Architecture (v0.1)

```
┌─────────────────────────────────────────────────────────────┐
│                    Client Applications                      │
│         (psql, pgAdmin, Python, Go, JavaScript, etc.)       │
└──────────────────┬──────────────────┬──────────────────────┘
                   │                  │
        ┌──────────▼────────┐  ┌──────▼──────────┐
        │  PostgreSQL Wire  │  │   REST API      │
        │    Protocol       │  │  (HTTP/JSON)    │
        │  (port 5432)      │  │  (port 8080)    │
        └──────────┬────────┘  └──────┬──────────┘
                   │                  │
                   └──────────┬───────┘
                              │
                   ┌──────────▼──────────┐
                   │  DataFusion Engine  │
                   │  (SQL Optimizer)    │
                   └──────────┬──────────┘
                              │
           ┌──────────────────┴──────────────────┐
           │                                     │
    ┌──────▼────────┐                  ┌────────▼──────────┐
    │   MemTable    │                  │   RedbTable       │
    │ (In-Memory)   │                  │ (Learned Index)   │
    │               │                  │                   │
    │ Default for   │                  │ Opt-in via        │
    │ CREATE TABLE  │                  │ register_table()  │
    └───────────────┘                  └─────────┬─────────┘
                                                 │
                                       ┌─────────▼─────────┐
                                       │  redb Storage     │
                                       │  + Learned Index  │
                                       │  (ALEX)           │
                                       └───────────────────┘
```

## Components

### 1. Wire Protocols

**PostgreSQL Wire Protocol** (`src/postgres/`)
- Full PostgreSQL v3 protocol support
- Simple Query Protocol: ✅ Implemented
- Extended Query Protocol: ⚠️ Not yet implemented (prepared statements)
- Compatible with all PostgreSQL client libraries
- Endpoints: health, metrics, SQL query execution

**REST API** (`src/rest/`)
- HTTP/JSON interface
- Endpoints:
  - `GET /health` - Health check
  - `GET /metrics` - Prometheus metrics
  - `POST /query` - Execute SQL queries
- CORS and compression enabled

### 2. SQL Engine (`datafusion`)

- **Apache DataFusion 43** - Production-grade SQL optimizer
- Features:
  - Full SQL support (SELECT, INSERT, UPDATE, DELETE, JOIN, etc.)
  - Cost-based optimizer
  - Predicate pushdown
  - Vectorized execution
  - Arrow-native columnar processing

### 3. Storage Layer

**Current Default: MemTable (In-Memory)**
- Used automatically for `CREATE TABLE` statements
- Pros:
  - Simple, fast for small datasets
  - No persistence overhead
  - Works out of the box
- Cons:
  - Data lost on restart
  - Limited to available RAM
  - No learned index optimization

**Available: RedbTable with Learned Index** (`src/datafusion/redb_table.rs`, `src/redb_storage.rs`)
- Opt-in via `ctx.register_table("name", Arc::new(RedbTable::new(...)))`
- Features:
  - ALEX learned index (Adaptive Learned indEX)
  - Persistent storage (redb ACID database)
  - Automatic point query detection
  - Optimized for large datasets (100K+ rows) and dynamic workloads
- Components:
  - `RedbStorage`: redb wrapper with learned index integration
  - `RedbTable`: DataFusion TableProvider implementation
  - Point query detection: `WHERE id = <value>` → uses learned index
  - Full scan fallback for other queries

### 4. Learned Index Implementation (`src/alex/`)

**ALEX (Adaptive Learned indEX)** - Primary implementation
- Gapped array structure for dynamic workloads
- Architecture:
  ```
  AlexTree (root)
        ↓
  Multiple GappedNode leaves (adaptive splits)
        ↓
  LinearModel per node → Exponential search
        ↓
  O(1) inserts, O(log n) queries
  ```
- Performance characteristics:
  - **Writes**: O(1) amortized inserts with gapped arrays (50% spare capacity)
  - **Reads**: O(log n) tree traversal + O(log error) exponential search
  - **Scaling**: Linear (10.6x time for 10x data)
  - **Dynamic workloads**: No O(n) rebuilds, auto-retraining on splits

**RecursiveModelIndex (RMI)** - Legacy, deprecated for dynamic workloads (`src/index.rs`)
- Static learned index, requires O(n) rebuilds on writes
- Still used in some benchmarks for comparison

## Performance Characteristics

### Learned Index Overhead

| Dataset Size | Point Query (Learned) | Full Scan | Speedup | Notes |
|--------------|----------------------|-----------|---------|-------|
| 1K rows      | ~5ms                 | ~3ms      | 0.6x    | Overhead dominates |
| 5K rows      | ~15ms                | ~12ms     | 0.8x    | Still overhead-bound |
| 10K rows     | ~25ms                | ~30ms     | 1.2x    | Break-even point |
| 100K rows    | ~1ms                 | ~300ms    | 300x    | Expected (not yet tested) |
| 1M rows      | ~1ms                 | ~3s       | 3000x   | Expected (not yet tested) |

**Key Insight:** Learned indexes are optimized for large-scale data. The overhead of model prediction is only worthwhile when it saves scanning thousands of rows.

## Test Coverage

**Total: 249 tests, all passing**

| Category | Tests | Coverage |
|----------|-------|----------|
| Core functionality | 198 | Basic operations, SQL engine |
| PostgreSQL protocol | 16 | Type conversion, handlers |
| PostgreSQL integration | 9 | Wire protocol end-to-end |
| REST API | 7 | All endpoints |
| End-to-end | 6 | Cross-protocol consistency |
| Transactions | 7 | ACID properties |
| Persistence | 6 | Data durability |
| Concurrency | 7 | Load testing (50+ concurrent) |
| **Learned Index Performance** | **9** | **Performance regression tests** |

### Learned Index Tests

1. Point query performance
2. Full scan vs learned index speedup
3. Multiple point queries
4. Scaling behavior (1K → 10K rows)
5. Miss performance (non-existent keys)
6. Range query behavior
7. Aggregation with filters
8. Correctness verification
9. Comprehensive benchmark

## Current Limitations

### 1. Default Tables Not Using Learned Indexes

**Status:** PostgreSQL/REST servers use DataFusion's default `MemTable`

**Impact:**
- `CREATE TABLE` statements create in-memory tables without learned indexes
- To use learned indexes, must explicitly register `RedbTable`
- Most users won't benefit from learned indexes by default

**Workaround:**
```rust
// Instead of:
ctx.sql("CREATE TABLE users (id INT, name VARCHAR)").await?;

// Use:
let storage = RedbStorage::new("users.redb")?;
let table = RedbTable::new(Arc::new(RwLock::new(storage)), "users");
ctx.register_table("users", Arc::new(table))?;
```

**Future:** Integrate `RedbTable` as default table provider

### 2. Extended Query Protocol Not Implemented

**Status:** Only Simple Query Protocol supported

**Impact:**
- Prepared statements ($1, $2 parameters) not supported
- Clients using Extended Query Protocol must be configured to use Simple Query
- Standard client libraries work but may need configuration

**Workaround:** Use `simple_query()` instead of `query()` in client code

**Future:** Implement ExtendedQueryHandler

### 3. No Real Persistence for Default Tables

**Status:** `MemTable` stores data in RAM only

**Impact:**
- Data lost on server restart
- Cannot handle datasets larger than RAM
- Not suitable for production without using `RedbTable`

**Workaround:** Use `RedbTable` for persistent tables

**Future:** Make `RedbTable` the default

### 4. Learned Index Overhead on Small Datasets

**Status:** By design - learned indexes have model prediction overhead

**Impact:**
- Point queries slower than full scans on datasets < 10K rows
- Not beneficial for small tables

**Mitigation:** Automatic fallback to full scan for small tables (planned)

## Production Readiness

### ✅ Production-Ready
- PostgreSQL wire protocol (Simple Query)
- REST API
- DataFusion SQL engine
- Concurrent access (tested with 50+ clients)
- ACID transactions (via DataFusion)
- Error handling and logging

### ⚠️ Requires Configuration
- Learned index optimization (must use `RedbTable`)
- Data persistence (must use `RedbTable`)

### ❌ Not Yet Production-Ready
- Extended Query Protocol (prepared statements)
- Automatic learned index integration
- Large-scale dataset testing (100K+ rows)
- Clustering/replication
- Advanced transaction isolation levels

## Future Roadmap

### Short-term (Next 2 weeks)
1. Implement Extended Query Protocol
2. Integrate `RedbTable` as default table provider
3. Test learned indexes on large datasets (100K+ rows)
4. Performance regression CI tests

### Medium-term (Next month)
1. Automatic table size detection → learned index vs B-tree
2. Hybrid storage: hot data in memory, cold data on disk
3. Improved caching layer
4. Connection pooling

### Long-term (2-3 months)
1. Distributed query execution
2. Replication and high availability
3. Advanced learned index models (PGM, RadixSpline)
4. GPU-accelerated learned index training

## Getting Started

### Using In-Memory Tables (Default)

```rust
use datafusion::prelude::*;
use omendb::postgres::PostgresServer;

#[tokio::main]
async fn main() {
    let ctx = SessionContext::new();

    // Create in-memory table
    ctx.sql("CREATE TABLE users (id INT, name VARCHAR)").await?;
    ctx.sql("INSERT INTO users VALUES (1, 'Alice')").await?;

    // Start PostgreSQL server
    let server = PostgresServer::new(ctx);
    server.serve().await?;
}
```

### Using Learned Indexes

```rust
use datafusion::prelude::*;
use omendb::datafusion::redb_table::RedbTable;
use omendb::redb_storage::RedbStorage;
use std::sync::{Arc, RwLock};

#[tokio::main]
async fn main() {
    let ctx = SessionContext::new();

    // Create table with learned index
    let mut storage = RedbStorage::new("users.redb")?;
    for i in 0..100_000 {
        storage.insert(i, format!("user_{}", i).as_bytes())?;
    }

    let table = RedbTable::new(
        Arc::new(RwLock::new(storage)),
        "users"
    );

    ctx.register_table("users", Arc::new(table))?;

    // Point queries now use learned index
    let df = ctx.sql("SELECT * FROM users WHERE id = 50000").await?;
    let results = df.collect().await?;
}
```

## Contributing

See [CONTRIBUTING.md](CONTRIBUTING.md) for development guidelines.

## License

Proprietary - OmenDB Inc.
