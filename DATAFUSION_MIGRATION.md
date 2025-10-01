# DataFusion + RocksDB Migration Plan

**Date:** 2025-09-30
**Status:** 🚀 APPROVED - Migration in progress

## Executive Summary

Migrating from custom SQL engine to **Apache DataFusion** + **RocksDB** for production-grade architecture.

**Time Savings:** 6-12 months of development
**Risk Reduction:** Using battle-tested components instead of custom implementations

---

## Architecture Decision

### ❌ What We're Replacing

```
Custom Components (High Risk):
├── sql_engine.rs - Custom SQL execution
├── mvcc.rs - Custom MVCC versioning
├── table.rs - Custom table management
└── wal.rs - Custom write-ahead log

Problems:
- Months to reach DataFusion's maturity
- High bug risk (data corruption, edge cases)
- Missing features (JOINs, window functions, CTEs)
```

### ✅ New Architecture

```
Proven Components (Low Risk):
├── DataFusion - SQL engine & optimizer
├── RocksDB - Transactional storage + MVCC
├── Delta Lake - OLAP layer (optional for now)
└── Learned Indexes - Our innovation

Benefits:
- Production-ready from day 1
- Full SQL support (JOINs, subqueries, etc.)
- Focus on our differentiator (learned indexes)
```

---

## Hybrid OLTP/OLAP Architecture

```
┌──────────────────────────────────────────────┐
│     PostgreSQL Wire Protocol (pgwire)        │
│     - PostgreSQL client compatibility        │
└──────────────────────────────────────────────┘
                     │
┌──────────────────────────────────────────────┐
│        SQL Engine (Apache DataFusion)        │
│  - Query parsing & optimization              │
│  - Physical plan generation                  │
│  - JOIN/aggregate execution                  │
└──────────────────────────────────────────────┘
                     │
          ┌──────────┴──────────┐
          │                     │
┌─────────▼──────────┐  ┌──────▼────────────┐
│   OLTP Layer       │  │  OLAP Layer       │
│   (RocksDB)        │  │  (Parquet/Arrow)  │
│                    │  │                   │
│ ✅ Transactions    │  │ ✅ Analytics      │
│ ✅ MVCC (built-in) │  │ ✅ Scans          │
│ ✅ Point queries   │  │ ✅ Aggregates     │
│ ✅ WAL (built-in)  │  │ ✅ Time travel    │
│                    │  │                   │
│ 🎯 Learned Index   │  │ 🎯 Columnar      │
│    - RMI over keys │  │    - DataFusion   │
│    - 4x speedup    │  │    - Parquet      │
└────────────────────┘  └───────────────────┘
```

---

## Component Breakdown

### 1. DataFusion (SQL Engine)

**What it provides:**
- ✅ SQL parsing (SELECT, INSERT, UPDATE, DELETE, JOINs, CTEs, window functions)
- ✅ Query optimizer (cost-based, predicate pushdown, partition pruning)
- ✅ Physical execution (multi-threaded, vectorized)
- ✅ Arrow integration (zero-copy operations)

**What we implement:**
- `TableProvider` trait for our learned index tables
- Query routing (OLTP vs OLAP)
- Custom optimizations (learned index pushdown)

**Example:**
```rust
use datafusion::prelude::*;

// Create context
let ctx = SessionContext::new();

// Register our learned index table
ctx.register_table("users", Arc::new(LearnedIndexTable::new(rocksdb)))?;

// Execute any SQL
let df = ctx.sql("
    SELECT u.name, COUNT(o.id) as order_count
    FROM users u
    LEFT JOIN orders o ON u.id = o.user_id
    WHERE u.age > 18
    GROUP BY u.name
    ORDER BY order_count DESC
    LIMIT 10
").await?;

let results = df.collect().await?;
```

### 2. RocksDB (OLTP Storage)

**What it provides:**
- ✅ ACID transactions (optimistic concurrency control)
- ✅ MVCC (snapshot isolation built-in)
- ✅ Write-Ahead Log (durability guaranteed)
- ✅ LSM tree (write-optimized)
- ✅ Compaction (automatic background cleanup)

**How we use it:**
```rust
use rocksdb::{TransactionDB, Transaction, Options};

// Open transactional database
let db = TransactionDB::open(&Options::default(), "./data/oltp")?;

// Begin transaction (MVCC snapshot created automatically)
let txn = db.transaction();

// Point query via learned index
let predicted_key = learned_index.predict(user_id);
let user_data = txn.get(predicted_key)?;

// Update (MVCC versioning automatic)
txn.put(predicted_key, new_user_data)?;

// Commit (WAL ensures durability)
txn.commit()?;
```

### 3. Learned Indexes (Our Innovation)

**Integration with RocksDB:**
```rust
struct LearnedIndexTable {
    rocksdb: Arc<TransactionDB>,
    learned_index: RecursiveModelIndex,
    schema: SchemaRef,
}

#[async_trait]
impl TableProvider for LearnedIndexTable {
    async fn scan(&self, ctx: &SessionState, projection: Option<&Vec<usize>>,
                  filters: &[Expr], limit: Option<usize>)
        -> Result<Arc<dyn ExecutionPlan>> {

        // Detect point query: WHERE id = 123
        if let Some(point_value) = extract_point_query(filters) {
            // Use learned index for O(1) lookup
            let rocksdb_key = self.learned_index.predict(point_value);
            return Ok(Arc::new(PointQueryPlan {
                rocksdb: self.rocksdb.clone(),
                key: rocksdb_key,
                schema: self.schema.clone(),
            }));
        }

        // Range query or scan - use RocksDB iterator
        Ok(Arc::new(ScanPlan {
            rocksdb: self.rocksdb.clone(),
            schema: self.schema.clone(),
            filters: filters.to_vec(),
        }))
    }
}
```

---

## Migration Steps

### Phase 1: Add Dependencies (Week 1)

```toml
[dependencies]
# SQL engine
datafusion = "43"

# OLTP storage
rocksdb = "0.22"

# PostgreSQL wire protocol
pgwire = "0.27"

# Existing (keep)
arrow = "53"
parquet = "53"
prometheus = "0.13"
tracing = "0.1"
```

### Phase 2: Implement RocksDB Layer (Week 1-2)

```rust
// src/storage/rocksdb_storage.rs
pub struct RocksDBStorage {
    db: Arc<TransactionDB>,
    learned_index: RecursiveModelIndex,
}

impl RocksDBStorage {
    pub fn new(path: &Path) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_max_background_jobs(4);

        let db = TransactionDB::open(&opts, path)?;
        let learned_index = RecursiveModelIndex::new(1_000_000);

        Ok(Self {
            db: Arc::new(db),
            learned_index,
        })
    }

    pub fn begin_transaction(&self) -> Transaction {
        self.db.transaction()
    }

    pub fn point_query(&self, key: i64) -> Result<Option<Vec<u8>>> {
        let predicted_key = self.learned_index.predict(key);
        let txn = self.db.transaction();
        Ok(txn.get(predicted_key)?)
    }
}
```

### Phase 3: Implement TableProvider (Week 2)

```rust
// src/datafusion/learned_table.rs
pub struct LearnedIndexTable {
    storage: Arc<RocksDBStorage>,
    schema: SchemaRef,
}

#[async_trait]
impl TableProvider for LearnedIndexTable {
    fn schema(&self) -> SchemaRef {
        self.schema.clone()
    }

    async fn scan(...) -> Result<Arc<dyn ExecutionPlan>> {
        // Detect query pattern
        // Route to learned index or full scan
        // Return optimized execution plan
    }
}
```

### Phase 4: Replace sql_engine.rs (Week 2-3)

```rust
// src/sql_engine_v2.rs (new file)
pub struct SqlEngine {
    ctx: SessionContext,
    rocksdb: Arc<RocksDBStorage>,
}

impl SqlEngine {
    pub fn new(data_dir: &Path) -> Result<Self> {
        let ctx = SessionContext::new();
        let rocksdb = Arc::new(RocksDBStorage::new(data_dir)?);

        Ok(Self { ctx, rocksdb })
    }

    pub async fn execute(&self, sql: &str) -> Result<Vec<RecordBatch>> {
        // DataFusion handles everything
        let df = self.ctx.sql(sql).await?;
        Ok(df.collect().await?)
    }

    pub async fn execute_with_transaction(&self, sql: &str) -> Result<Vec<RecordBatch>> {
        let txn = self.rocksdb.begin_transaction();

        // Execute query within transaction context
        let result = self.execute(sql).await?;

        txn.commit()?;
        Ok(result)
    }
}
```

### Phase 5: Add PostgreSQL Wire Protocol (Week 3-4)

```rust
// src/server/postgres_server.rs
use pgwire::api::*;

pub struct PostgresServer {
    engine: Arc<SqlEngine>,
}

#[async_trait]
impl SimpleQueryHandler for PostgresServer {
    async fn do_query(&self, sql: &str) -> PgWireResult<Response> {
        let results = self.engine.execute(sql).await?;
        Ok(Response::Query(results))
    }
}
```

---

## Verification Plan

### Tests to Migrate

1. ✅ **Keep existing tests** - Ensure backward compatibility
2. ✅ **Add DataFusion tests** - Verify SQL correctness
3. ✅ **Add transaction tests** - RocksDB ACID properties
4. ✅ **Add learned index tests** - Integration with DataFusion

### Performance Benchmarks

```rust
#[bench]
fn bench_point_query_learned_index(b: &mut Bencher) {
    // Compare: Learned index vs B-tree vs full scan
    // Expected: Learned index 4x faster
}

#[bench]
fn bench_analytical_query(b: &mut Bencher) {
    // Compare: DataFusion vs custom engine
    // Expected: DataFusion faster (vectorized execution)
}

#[bench]
fn bench_transaction_throughput(b: &mut Bencher) {
    // Measure: Transactions per second
    // Expected: 10K+ txn/sec (RocksDB baseline)
}
```

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| DataFusion learning curve | Excellent docs, active community |
| RocksDB complexity | Well-documented, many examples |
| Integration bugs | Comprehensive test suite |
| Performance regression | Benchmarks before/after migration |
| Data migration | Keep old code until verified |

---

## Success Metrics

**Before Migration:**
- ❌ No JOINs
- ❌ No UPDATE/DELETE (incomplete)
- ❌ Custom MVCC (untested)
- ❌ No transaction tests
- ⚠️ Limited SQL support

**After Migration:**
- ✅ Full SQL (JOINs, CTEs, window functions)
- ✅ ACID transactions (RocksDB proven)
- ✅ Production-grade MVCC
- ✅ PostgreSQL compatibility
- ✅ Learned index integration
- ✅ 6-12 months saved

---

## Timeline

| Week | Milestone | Status |
|------|-----------|--------|
| 1 | Add dependencies, RocksDB integration | 🔄 In Progress |
| 2 | TableProvider implementation | 📅 Planned |
| 3 | DataFusion SQL engine replacement | 📅 Planned |
| 4 | PostgreSQL wire protocol | 📅 Planned |
| 5 | Full test suite + benchmarks | 📅 Planned |
| 6 | Production validation | 📅 Planned |

**Total: 6 weeks to production-grade architecture**

---

## Decision: APPROVED ✅

**Rationale:**
- No major drawbacks to DataFusion for our use case
- 6-12 months development time saved
- Production-grade from day 1
- Focus on our differentiator (learned indexes)

**Next Steps:**
1. Add DataFusion + RocksDB dependencies
2. Implement RocksDB storage layer
3. Create TableProvider for learned indexes
4. Migrate SQL execution to DataFusion
