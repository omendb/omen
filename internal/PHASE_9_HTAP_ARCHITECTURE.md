# Phase 9: HTAP Architecture Implementation

**Date**: January 2025
**Status**: Planning
**Previous**: Phase 8 (SOTA improvements complete)

## Executive Summary

After analyzing the codebase, **OmenDB already has a unified HTAP architecture**. The Table system combines:
- **OLTP**: Row inserts via ALEX learned index
- **OLAP**: Arrow/Parquet columnar storage
- **Unified**: Same table serves both workloads

**Phase 9 Goal**: Add query routing and DataFusion integration to optimize query execution based on workload type.

---

## Current Architecture (What We Actually Have)

### Table System (`src/table.rs`)

```rust
pub struct Table {
    name: String,
    user_schema: SchemaRef,           // Arrow schema
    internal_schema: SchemaRef,        // With MVCC columns
    primary_key: String,

    storage: TableStorage,             // Arrow/Parquet columnar
    index: TableIndex,                 // ALEX learned index

    next_version: u64,                 // MVCC versioning
    current_txn_id: u64,
}
```

**What This Provides:**
- ✅ Columnar storage (Arrow → Parquet) for OLAP
- ✅ Learned index (ALEX) for fast OLTP lookups
- ✅ MVCC for transaction isolation
- ✅ Single table serves both workloads

### TableStorage (`src/table_storage.rs`)

```rust
pub struct TableStorage {
    schema: SchemaRef,
    batches: Vec<RecordBatch>,         // In-memory Arrow batches
    parquet_file: PathBuf,             // Persistent columnar storage
    batch_size: usize,
    pending_rows: Vec<Row>,
}
```

**Capabilities:**
- ✅ Insert rows (OLTP writes)
- ✅ RecordBatch storage (OLAP scans)
- ✅ Flush to Parquet (persistence)
- ✅ Scan operations

### TableIndex (`src/table_index.rs`)

Uses ALEX learned index for primary key lookups:
- ✅ 4.81x faster than RocksDB (10M keys, sequential)
- ✅ Trained on actual data distribution
- ✅ Updates incrementally

---

## What's Missing (Phase 9 Scope)

### 1. DataFusion Integration ❌

**Current State:**
- DataFusion exists (`src/datafusion/mod.rs`) but only for old redb storage
- Table system doesn't expose TableProvider interface
- No SQL query engine for Arrow/Parquet tables

**Phase 9.1 Goal:**
Implement `TableProvider` for Table system:

```rust
// New file: src/datafusion/table_provider.rs
impl TableProvider for Table {
    fn schema(&self) -> SchemaRef {
        self.user_schema.clone()
    }

    fn scan(&self, projection: &[usize], filters: &[Expr]) -> Result<ExecutionPlan> {
        // Use TableStorage's RecordBatches
        // Apply filters via DataFusion
        // Return optimized execution plan
    }
}
```

**Benefits:**
- SQL queries on Arrow tables
- Vectorized execution (DataFusion optimizer)
- Filter pushdown, projection pushdown

### 2. Query Router ❌

**Current State:**
- All queries go through same path
- No differentiation between OLTP (point) vs OLAP (scan/aggregate)

**Phase 9.2 Goal:**
Intelligent query routing based on query type:

```rust
// New file: src/query_router.rs
pub struct QueryRouter {
    temperature_tracker: TemperatureTracker,
}

impl QueryRouter {
    fn route_query(&self, query: &Query) -> ExecutionPath {
        match query {
            Query::PointLookup(key) => {
                // Use ALEX index (fast)
                ExecutionPath::LearnedIndex
            }
            Query::RangeScan { .. } | Query::Aggregate { .. } => {
                // Use DataFusion (vectorized)
                ExecutionPath::DataFusion
            }
            Query::Mixed { .. } => {
                // Use temperature model
                self.decide_hybrid(query)
            }
        }
    }
}
```

**Routing Logic:**
- Point queries (WHERE id = X) → ALEX index
- Range scans (WHERE id BETWEEN X AND Y) → DataFusion
- Aggregates (COUNT, SUM, AVG) → DataFusion
- Hot data (frequently accessed) → Keep in ALEX
- Cold data (rare access) → Parquet only

### 3. Temperature Tracking ❌

**Current State:**
- No access pattern monitoring
- All data treated equally

**Phase 9.3 Goal:**
Track data "temperature" for placement decisions:

```rust
// New file: src/temperature.rs
pub struct TemperatureTracker {
    access_counts: HashMap<KeyRange, u64>,
    last_access: HashMap<KeyRange, Instant>,
    window: Duration,  // 5 minutes
}

impl TemperatureTracker {
    fn get_temperature(&self, key_range: &KeyRange) -> f64 {
        let freq = self.access_counts.get(key_range).unwrap_or(&0);
        let recency = self.last_access.get(key_range)
            .map(|t| t.elapsed().as_secs())
            .unwrap_or(u64::MAX);

        // Temperature formula (α=0.6, β=0.4)
        let freq_score = (*freq as f64 / 1000.0).min(1.0);
        let recency_score = (1.0 - (recency as f64 / 300.0)).max(0.0);

        0.6 * freq_score + 0.4 * recency_score
    }

    fn classify(&self, temp: f64) -> DataTier {
        if temp > 0.8 {
            DataTier::Hot    // Keep in ALEX + memory
        } else if temp > 0.3 {
            DataTier::Warm   // ALEX + Parquet
        } else {
            DataTier::Cold   // Parquet only
        }
    }
}
```

**Use Cases:**
- Promote hot data to ALEX index
- Demote cold data (remove from ALEX, keep in Parquet)
- Pre-fetch warm data before queries

---

## Architecture Comparison

### Original Research Plan (WRONG) ❌

```
AlexStorage (OLTP)  →  WAL Replication  →  ArrowStorage (OLAP)
     ↓                                            ↓
  Key-Value                              Time-Series Columnar
  (Separate systems, needs sync)
```

**Problems:**
- Assumes two separate storage engines
- Requires replication layer
- Schema mismatch (key-value vs time-series)
- Complexity of CDC/WAL streaming

### Actual Architecture (CORRECT) ✅

```
Table (Unified HTAP)
  ├── TableIndex (ALEX) ────→ Fast point queries (OLTP)
  ├── TableStorage (Arrow) ──→ Columnar analytics (OLAP)
  └── QueryRouter ──────────→ Route by workload type
           ↓
    [Point Query] → ALEX index (389ns)
    [Range/Agg]   → DataFusion (vectorized)
```

**Benefits:**
- Single source of truth (no sync needed)
- No schema mismatch (same Arrow schema)
- Simpler architecture (no replication)
- Already production-ready storage

---

## Phase 9 Implementation Plan

### Week 1: DataFusion TableProvider

**Files to Create:**
- `src/datafusion/table_provider.rs` - TableProvider impl for Table
- `src/datafusion/exec_plan.rs` - Custom execution plan for learned index

**Implementation:**
1. Implement TableProvider trait for Table
2. Expose RecordBatches from TableStorage
3. Add filter/projection pushdown
4. Integrate with DataFusion SessionContext

**Test:**
```rust
#[test]
fn test_datafusion_sql_on_table() {
    let table = Table::new(...);
    table.insert(row!(id: 1, name: "Alice"));

    let ctx = SessionContext::new();
    ctx.register_table("users", Arc::new(table));

    let result = ctx.sql("SELECT * FROM users WHERE id > 10").await?;
    assert_eq!(result.count(), ...);
}
```

### Week 2: Query Router

**Files to Create:**
- `src/query_router.rs` - Query routing logic
- `src/query_classifier.rs` - Classify query types

**Implementation:**
1. Parse SQL to identify query type (point/range/aggregate)
2. Route point queries → ALEX index
3. Route range/aggregate → DataFusion
4. Benchmark both paths

**Decision Tree:**
```
Query
  ├─ Point (WHERE id = X)
  │   └─→ ALEX index (389ns)
  │
  ├─ Range (WHERE id BETWEEN X AND Y)
  │   ├─ Small range (<100 rows) → ALEX
  │   └─ Large range → DataFusion
  │
  └─ Aggregate (COUNT, SUM, AVG)
      └─→ DataFusion (vectorized)
```

### Week 3: Temperature Tracking

**Files to Create:**
- `src/temperature.rs` - Temperature model
- `src/table.rs` - Add temperature tracking to Table

**Implementation:**
1. Track access patterns per key range
2. Calculate temperature score (frequency + recency)
3. Classify hot/warm/cold data
4. Influence query routing decisions

**Integration:**
```rust
impl Table {
    pub fn get(&self, key: &Value) -> Result<Row> {
        // Update temperature
        self.temperature.record_access(key);

        // Route based on temperature
        if self.temperature.is_hot(key) {
            self.index.search(key)  // ALEX
        } else {
            self.storage.scan_with_filter(key)  // Arrow
        }
    }
}
```

### Week 4: Benchmarks & Validation

**Benchmarks to Add:**
- OLTP: Point queries (measure ALEX speedup)
- OLAP: Range scans, aggregates (measure DataFusion speedup)
- Mixed: 80% point + 20% analytics (measure router effectiveness)
- Temperature: Hot/cold data placement impact

**Success Metrics:**
- Point queries: <1µs (ALEX) vs >10µs (DataFusion scan)
- Range scans: DataFusion matches or beats ALEX
- Aggregates: 10x+ faster with DataFusion vectorization
- Router overhead: <50ns decision time

---

## Corrected Research Findings

### What We DON'T Need (From Original Plan)

❌ **WAL Replication Between Systems** - Already unified
❌ **Schema Conversion (Row → Columnar)** - Already columnar
❌ **CDC Pipeline (Debezium + Kafka)** - No separate systems
❌ **Separate OLTP/OLAP Storage** - Already combined in Table

### What We DO Need (Revised Plan)

✅ **DataFusion Integration** - SQL query engine for Arrow tables
✅ **Query Routing** - OLTP vs OLAP execution path
✅ **Temperature Model** - Hot/cold data classification
✅ **Hybrid Execution** - Combine ALEX + DataFusion for mixed queries

---

## Competitive Position (Honest)

### vs TiDB (Separate TiKV + TiFlash)

**TiDB Approach:**
- Separate OLTP (TiKV) and OLAP (TiFlash) stores
- Raft replication between them (~2-5 sec lag)
- Complex distributed coordination

**OmenDB Approach:**
- Single table with dual access paths (ALEX + DataFusion)
- No replication lag (same data)
- Simpler architecture (single-node initially)

**Trade-off:**
- ✅ OmenDB: No replication lag, simpler
- ❌ TiDB: Proven at scale, distributed

### vs CockroachDB (Row-based HTAP)

**CockroachDB Approach:**
- Row-based storage for everything
- Vectorized execution (ColFlow) on row data
- MVCC + follower reads for analytics

**OmenDB Approach:**
- Columnar storage (Arrow) native for OLAP
- Learned index for OLTP
- Temperature-based routing

**Trade-off:**
- ✅ OmenDB: Better OLAP (columnar), faster OLTP (learned index)
- ❌ CockroachDB: Distributed, battle-tested

---

## Next Steps

**Immediate (This Week):**
1. ✅ Document actual architecture
2. 🔨 Implement DataFusion TableProvider for Table
3. 📐 Design query router interface

**Short-Term (Weeks 1-2):**
1. Complete DataFusion integration
2. Implement basic query router (point vs scan)
3. Add temperature tracking skeleton

**Medium-Term (Weeks 3-4):**
1. Full temperature model with hot/cold placement
2. Comprehensive benchmarks (OLTP/OLAP/Mixed)
3. Update honest competitive assessment with real data

---

## Files Modified

### New Files (Phase 9)
- `src/datafusion/table_provider.rs` - TableProvider for Table
- `src/query_router.rs` - Query routing logic
- `src/temperature.rs` - Temperature tracking model
- `src/bin/benchmark_htap.rs` - HTAP workload benchmarks

### Modified Files
- `src/table.rs` - Add temperature tracking, query routing
- `src/datafusion/mod.rs` - Export TableProvider
- `src/sql_engine.rs` - Integrate query router

---

## Testing Strategy

### Unit Tests
- DataFusion TableProvider correctness
- Query router decision logic
- Temperature model accuracy

### Integration Tests
- OLTP workload (100% point queries)
- OLAP workload (100% scans/aggregates)
- Mixed workload (80/20, 50/50, 20/80)
- Hot/cold data placement

### Performance Tests
- ALEX vs DataFusion for point queries
- DataFusion speedup for aggregates
- Router overhead measurement
- Temperature model impact

---

**Document Status**: Complete
**Architecture**: Unified HTAP (Table + ALEX + Arrow)
**Next Action**: Implement DataFusion TableProvider
**Est. Timeline**: 4 weeks (Phases 9.1-9.4)
