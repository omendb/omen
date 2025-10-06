# OmenDB Storage Strategy Assessment

**Date:** October 6, 2025
**Status:** Critical Strategic Analysis
**Purpose:** Determine optimal storage engine strategy for HTAP vision

---

## Executive Summary

**Question:** Is AlexStorage optimal for OmenDB's intended use cases? Should we support multiple storage engines?

**Answer:** AlexStorage alone is **insufficient**. OmenDB's HTAP vision requires **multi-engine architecture**:

✅ **Recommended:** Tiered storage with specialized engines
- **OLTP (Hot):** RocksDB with ALEX overlay (proven, fast writes)
- **OLAP (Cold):** Arrow/Parquet (columnar analytics)
- **Cache:** AlexStorage (ultra-fast reads for hot keys)
- **Learned Optimizer:** Routes queries to optimal engine

**Rationale:**
1. AlexStorage excels at reads (4.81x) but lags at writes (0.32x)
2. HTAP requires dual storage (row + columnar) per VLDB research
3. OmenDB already has 4+ storage implementations
4. Competitors all use multi-engine architectures

**Strategic Recommendation:** **Focus on learned optimization layer, not storage engine purity.**

---

## Current Storage Implementations

OmenDB already has **5 storage engines** implemented:

### 1. RocksStorage (RocksDB + ALEX)
**File:** `src/rocks_storage.rs`

**Architecture:**
- RocksDB: LSM-tree for transactional data
- ALEX: Learned index overlay for key tracking
- Use case: OLTP with learned optimization

**Performance:**
- Writes: ~1,565 ns (RocksDB baseline)
- Reads: Optimized via ALEX overlay
- Battle-tested at Facebook scale

**Assessment:** ✅ **Best choice for OLTP hot path**

### 2. ArrowStorage (Columnar)
**File:** `src/storage.rs`

**Architecture:**
- Arrow RecordBatch in-memory (hot batches)
- Parquet files on disk (cold storage)
- Learned index for timestamp column
- WAL for durability

**Use case:** Time-series analytics (OLAP)

**Assessment:** ✅ **Best choice for OLAP analytics**

### 3. AlexStorage (Pure Learned Index)
**File:** `src/alex_storage.rs`

**Architecture:**
- ALEX learned index: key → offset
- Mmap file: zero-copy value access
- Append-only writes
- WAL durability

**Performance (validated Phases 1-7):**
- Reads: 829ns (4.81x faster than RocksDB)
- Writes: 4,915ns (3.14x slower than RocksDB)
- Compaction: 2.85s for 1M keys

**Assessment:** ⚠️ **Specialized use case: read-heavy cache tier**

### 4. TableStorage (Schema-Agnostic Parquet)
**File:** `src/table_storage.rs`

**Architecture:**
- Arrow schema
- In-memory batches
- Parquet persistence
- Batch flushing

**Use case:** General-purpose table storage (OLAP)

**Assessment:** ✅ **Good for OLAP, overlaps with ArrowStorage**

### 5. RedbStorage (Embedded ACID)
**File:** `src/redb_storage.rs`

**Architecture:**
- Redb B-tree (ACID, MVCC)
- Embedded storage
- Crash recovery

**Assessment:** ⚠️ **Legacy, slow, replaced by RocksStorage**

---

## HTAP Requirements Analysis

### What is HTAP?

**Hybrid Transactional/Analytical Processing** (HTAP) enables real-time analytics on transactional data without ETL.

**Core Requirements** (per VLDB 2024 survey):

1. ✅ **Dual Storage Format**
   - Row-oriented for OLTP (fast point queries, updates)
   - Columnar for OLAP (fast scans, aggregations)

2. ✅ **Workload Isolation**
   - Separate execution paths for OLTP vs OLAP
   - Resource quotas to prevent OLAP from starving OLTP

3. ✅ **Data Freshness**
   - Async replication from row store to column store
   - Sub-second freshness guarantees

4. ⚠️ **Distributed Architecture** (Phase 2)
   - Horizontal scalability
   - Raft consensus for HA

### OmenDB's HTAP Vision

From `CLAUDE.md`:
```
Unified Engine:
├── OLTP Layer: Row-oriented transactions (PostgreSQL wire protocol)
├── OLAP Layer: Columnar analytics (DataFusion + Arrow/Parquet)
├── Learned Optimizer: Hot/cold placement, query routing
└── Storage: Tiered (memory → SSD → object storage)
```

**Performance Targets:**
- OLTP: 50K txn/sec, <10ms p99
- OLAP: 1M rows/sec scan, <1s queries
- Scale: 1TB databases, 100GB memory

### Gap Analysis

| HTAP Requirement | Current State | Gap |
|------------------|---------------|-----|
| Dual storage (row + columnar) | ✅ RocksStorage + ArrowStorage | ✅ Implemented |
| Workload isolation | ⚠️ Single DataFusion engine | ⚠️ Need routing layer |
| Data freshness | ❌ No replication | ❌ Need sync mechanism |
| Learned optimization | ⚠️ ALEX on RocksDB only | ⚠️ Need query router |
| Distributed | ❌ Single-node | ❌ Phase 2 |

**Assessment:** OmenDB has the storage engines but lacks the **orchestration layer**.

---

## AlexStorage: Strengths & Weaknesses

### Strengths ✅

**1. Ultra-Fast Reads**
- 829ns point queries (4.81x faster than RocksDB)
- O(log log n) ALEX lookups vs O(log n) B-tree
- Zero-copy mmap access

**2. Simple Implementation**
- 600 lines (vs 100K+ for RocksDB)
- Append-only writes
- Crash-safe (WAL + atomic compaction)

**3. Learned Index Validation**
- Proves learned indexes work in production
- Research-to-practice contribution
- IP differentiation

**4. Specialized Workloads**
- Read-heavy caches (90%+ reads)
- Sorted/sequential keys (time-series IDs)
- Hot tier for frequently-accessed data

### Weaknesses ❌

**1. Slow Writes**
- 4,915ns inserts (3.14x slower than RocksDB)
- No group commit yet (Phase 8 target)
- WAL fsync dominates latency

**2. Single-File Limitation**
- No LSM-tree levels
- Compaction blocks all operations (offline)
- Space amplification until compaction

**3. Not General-Purpose**
- Optimized for reads, not writes
- Not suitable for write-heavy OLTP
- Missing distributed capabilities

**4. Unproven at Scale**
- Tested to 1M keys only
- No 100M+ validation
- Compaction time grows linearly (2.85s for 1M)

### Honest Assessment

AlexStorage is a **specialized storage engine** for:
- ✅ Read-heavy cache tier (session stores, metadata)
- ✅ Hot key optimization (Zipfian workloads)
- ✅ Research validation (learned indexes work)

AlexStorage is **NOT suitable** for:
- ❌ General-purpose OLTP (slow writes)
- ❌ Write-heavy workloads (no group commit yet)
- ❌ Primary storage (not battle-tested at scale)

---

## Recommended Multi-Engine Architecture

### Tiered Storage Strategy

```
OmenDB Multi-Engine Architecture:
┌───────────────────────────────────────────────────┐
│              Query Router                         │
│  (Learned Optimizer - hot/cold placement)        │
└─────────┬─────────────────────┬──────────────────┘
          │                     │
    ┌─────▼──────────┐    ┌────▼──────────────────┐
    │ OLTP Engine    │    │ OLAP Engine           │
    │ (Transactional)│    │ (Analytical)          │
    └─────┬──────────┘    └────┬──────────────────┘
          │                     │
    ┌─────▼──────────┐    ┌────▼──────────────────┐
    │ Hot Tier       │    │ Cold Tier              │
    │ (L1 Cache)     │    │ (Columnar Storage)     │
    ├────────────────┤    ├────────────────────────┤
    │ AlexStorage    │    │ ArrowStorage           │
    │ - 4.81x reads  │    │ - Parquet files        │
    │ - <1µs latency │    │ - Compression          │
    │ - Hot keys     │    │ - OLAP scans           │
    └────────────────┘    └────────────────────────┘
          │                     │
    ┌─────▼──────────┐    ┌────▼──────────────────┐
    │ Warm Tier      │    │ Object Storage         │
    │ (L2 Cache)     │    │ (Archive)              │
    ├────────────────┤    ├────────────────────────┤
    │ RocksStorage   │    │ S3/MinIO               │
    │ - Fast writes  │    │ - Parquet files        │
    │ - ACID         │    │ - Cost-optimized       │
    │ - Battle-tested│    │ - Long-term retention  │
    └────────────────┘    └────────────────────────┘
          │                     │
          └──────┬──────────────┘
                 │
         ┌───────▼────────┐
         │ WAL Replication│
         │ (OLTP → OLAP)  │
         │ - Async        │
         │ - Sub-second   │
         └────────────────┘
```

### Storage Engine Roles

**1. AlexStorage (L1 Cache - Ultra-Hot Tier)**
- **Use case:** Top 1-5% hottest keys (Zipfian heavy hitters)
- **Workload:** 95%+ reads, <5% writes
- **Size:** 10-100K keys (fits in memory)
- **Performance:** <1µs reads, 4.81x speedup
- **Example:** Session cache, user profile cache, trending items

**2. RocksStorage (L2 Cache - Hot/Warm Tier)**
- **Use case:** General-purpose OLTP (80% of transactions)
- **Workload:** Mixed read/write, transactional
- **Size:** 1-10M keys (SSD)
- **Performance:** 1-2µs reads, 1.5µs writes
- **Example:** User accounts, orders, inventory, transactions

**3. ArrowStorage (Cold Tier - OLAP)**
- **Use case:** Analytical queries, aggregations
- **Workload:** Large scans, time-series analytics
- **Size:** 100M+ rows (SSD/object storage)
- **Performance:** 1M rows/sec scans, columnar compression
- **Example:** Historical data, reporting, dashboards

**4. Object Storage (Archive Tier)**
- **Use case:** Long-term retention, compliance
- **Workload:** Rare access, bulk exports
- **Size:** Unlimited (S3, MinIO, etc.)
- **Performance:** Seconds to minutes
- **Example:** 90+ day old data, backups, audit logs

### Learned Optimizer (Query Router)

**Role:** Route queries to optimal storage engine based on:
1. Access frequency (hot/cold detection)
2. Workload type (OLTP vs OLAP)
3. Data age (recency)
4. Query pattern (point vs range vs scan)

**Example Decision Logic:**
```
Query: SELECT * FROM users WHERE user_id = 123
├─> Check AlexStorage (L1 cache)
│   └─> Hit: Return in <1µs ✅
│   └─> Miss: Check RocksStorage (L2)
│       └─> Hit: Return in ~2µs, promote to L1 cache
│       └─> Miss: Return NULL

Query: SELECT COUNT(*) FROM orders WHERE date > '2025-01-01'
└─> Route to ArrowStorage (OLAP tier)
    └─> Columnar scan: 1M rows/sec

Query: INSERT INTO orders (user_id, amount) VALUES (123, 50.00)
└─> Write to RocksStorage (OLTP tier)
    └─> Async replicate to ArrowStorage (OLAP tier)
    └─> Update AlexStorage (L1 cache) if hot key
```

**Implementation:** Learned model trained on query patterns:
- Input: Query features (key, workload type, access frequency)
- Output: Optimal storage tier + confidence score
- Training: Online learning from query logs

---

## Competitive Benchmark

### SingleStore (MySQL-Compatible HTAP)

**Architecture:**
- **Rowstore:** MemSQL (in-memory row-oriented) for OLTP
- **Columnstore:** Distributed columnar storage for OLAP
- **Universal Storage:** Hybrid row/column in single table

**Performance:**
- OLTP: 1M txn/sec
- OLAP: 10M rows/sec scan
- Freshness: Real-time (no ETL)

**Lessons:**
- Dual storage is essential for HTAP
- Workload isolation prevents contention
- Learned query optimizer routes efficiently

### CockroachDB (PostgreSQL-Compatible, Not HTAP)

**Architecture:**
- **LSM-tree:** RocksDB for all storage
- **No columnar store:** OLAP queries slow
- **Workaround:** Columnstore via external integration

**Performance:**
- OLTP: 50K txn/sec
- OLAP: Slow (no columnar optimization)

**Lessons:**
- Pure row storage insufficient for analytics
- CockroachDB users export to ClickHouse/Snowflake for OLAP
- OmenDB's dual storage is a competitive advantage

### TiDB (MySQL-Compatible HTAP)

**Architecture:**
- **TiKV:** Row-oriented LSM-tree (OLTP)
- **TiFlash:** Columnar storage (OLAP)
- **Raft consensus:** Replication between TiKV and TiFlash

**Performance:**
- OLTP: 100K txn/sec
- OLAP: 1M rows/sec scan
- Freshness: <1 second

**Lessons:**
- Async replication from row to column store
- Learned query optimizer routes workloads
- Multi-Raft ensures consistency

**OmenDB Positioning vs Competitors:**

| Feature | SingleStore | CockroachDB | TiDB | **OmenDB (Proposed)** |
|---------|-------------|-------------|------|----------------------|
| Dual storage (row+column) | ✅ Yes | ❌ No | ✅ Yes | ✅ Yes (RocksDB + Arrow) |
| Learned optimization | ✅ Yes | ⚠️ Partial | ⚠️ Partial | ✅ **Yes (ALEX + query router)** |
| PostgreSQL compatible | ❌ No (MySQL) | ✅ Yes | ❌ No (MySQL) | ✅ **Yes** |
| Distributed | ✅ Yes | ✅ Yes | ✅ Yes | ⚠️ Phase 2 |
| Learned cache tier | ❌ No | ❌ No | ❌ No | ✅ **Yes (AlexStorage)** |

**Differentiation:** OmenDB's learned optimization goes **deeper** than competitors (learned cache tier + query routing).

---

## Strategic Recommendations

### Option 1: Multi-Engine HTAP (Recommended)

**Approach:** Build tiered storage with learned optimization layer

**Components:**
1. ✅ AlexStorage: L1 cache (ultra-hot tier) - **Keep, optimize**
2. ✅ RocksStorage: L2 cache (OLTP tier) - **Already implemented**
3. ✅ ArrowStorage: OLAP tier - **Already implemented**
4. ⚠️ Query Router: Learned optimizer - **Build Phase 8**
5. ⚠️ WAL Replication: OLTP → OLAP sync - **Build Phase 9**

**Timeline:** 6-8 weeks to MVP
- Week 1-2: Query router (learned model + routing logic)
- Week 3-4: WAL replication (async OLTP → OLAP)
- Week 5-6: Hot/cold promotion policies
- Week 7-8: Testing, benchmarks, documentation

**Benefits:**
- ✅ True HTAP (dual storage)
- ✅ Differentiated (learned optimization at multiple levels)
- ✅ PostgreSQL compatible
- ✅ Leverages existing engines (RocksDB + Arrow + ALEX)

**Investment:**
- AlexStorage: Keep as L1 cache (no wasted work)
- RocksDB: Proven, battle-tested foundation
- Arrow/Parquet: Industry standard for OLAP
- Learned optimizer: Novel differentiation

### Option 2: AlexStorage-Only (Not Recommended)

**Approach:** Improve AlexStorage to be general-purpose

**Tasks:**
- Group commit (Phase 8) - improve writes 5-10x
- Range queries (Phase 9) - OLAP support
- Background compaction (Phase 10) - online operation
- Distributed clustering (Phase 11+) - horizontal scaling

**Timeline:** 6-12 months

**Problems:**
- ❌ Still slower writes than RocksDB (even with group commit)
- ❌ No columnar format (poor OLAP performance)
- ❌ Reinventing the wheel (RocksDB already solves this)
- ❌ Unproven at scale (risk)

**Assessment:** Not competitive with established HTAP systems

### Option 3: RocksDB-Only, Drop AlexStorage (Not Recommended)

**Approach:** Use RocksDB for everything, add ALEX overlay

**Benefits:**
- ✅ Fast writes (proven)
- ✅ Battle-tested
- ✅ ACID, HA, distribution available

**Problems:**
- ❌ Wastes Phases 1-7 work (6 weeks of development)
- ❌ Less differentiated (just RocksDB + learned layer)
- ❌ No L1 cache tier (miss 4.81x speedup opportunity)
- ❌ Still need ArrowStorage for OLAP

**Assessment:** Safe but less differentiated

---

## Implementation Roadmap

### Phase 8: Query Router (2 weeks)

**Goal:** Route queries to optimal storage engine

**Tasks:**
1. Design routing API
   ```rust
   trait QueryRouter {
       fn route(&self, query: &Query) -> StorageEngine;
       fn should_cache(&self, key: i64) -> bool;
       fn promote_to_l1(&mut self, key: i64);
   }
   ```

2. Implement simple heuristics:
   - Read-heavy keys (>90% reads) → AlexStorage (L1)
   - Transactional keys → RocksStorage (L2)
   - Range queries → ArrowStorage (OLAP)

3. Track access patterns:
   - LRU cache for hot keys
   - Access frequency counters
   - Workload classification (OLTP vs OLAP)

4. Benchmark routing overhead:
   - Target: <50ns routing decision
   - Measure cache hit rates

**Deliverable:** Working query router with 80%+ optimal routing

### Phase 9: WAL Replication (2 weeks)

**Goal:** Async sync from OLTP to OLAP

**Tasks:**
1. WAL consumer for RocksStorage:
   ```rust
   impl WalConsumer for ArrowStorage {
       fn consume(&mut self, entry: WalEntry) -> Result<()>;
   }
   ```

2. Async replication thread:
   - Read RocksStorage WAL
   - Transform to Arrow RecordBatch
   - Append to ArrowStorage

3. Freshness guarantees:
   - Target: <1 second lag
   - Configurable batching (100ms - 1s)

4. Consistency:
   - Version tracking (OLTP watermark)
   - OLAP queries see consistent snapshots

**Deliverable:** Sub-second OLTP → OLAP replication

### Phase 10: Learned Optimizer (3 weeks)

**Goal:** Learned model for hot/cold placement

**Tasks:**
1. Collect training data:
   - Query logs (key, access frequency, latency)
   - Cache hit/miss statistics
   - Workload classification labels

2. Train learned model:
   - Features: Key, access frequency, recency, query type
   - Labels: Optimal tier (L1 cache, L2 cache, OLAP)
   - Model: Lightweight decision tree or linear model
   - Target: >90% accuracy

3. Online learning:
   - Update model periodically (every 1 hour)
   - A/B test learned vs heuristic routing
   - Measure improvement in cache hit rate

4. Promotion policies:
   - Hot key promotion (L2 → L1)
   - Cold key demotion (L1 → L2 → OLAP)
   - Adaptive thresholds based on workload

**Deliverable:** Learned query optimizer with measurable improvement

### Phase 11: Benchmarking (1 week)

**Goal:** Validate HTAP performance

**Tasks:**
1. TPC-H (OLAP):
   - Load 10GB dataset
   - Run all 22 queries
   - Compare vs PostgreSQL, DuckDB

2. TPC-C (OLTP):
   - 100 warehouses
   - Mixed transaction workload
   - Measure tpmC throughput

3. CH-benCHmark (HTAP):
   - Combined OLTP + OLAP
   - Measure throughput with concurrent workloads

4. Publish results:
   - Honest performance report
   - Identify strengths/weaknesses
   - Set performance goals

**Deliverable:** Public benchmark report

---

## Success Metrics

### Technical Metrics

**Performance:**
- ✅ OLTP: 50K txn/sec (target from CLAUDE.md)
- ✅ OLAP: 1M rows/sec scan
- ✅ L1 cache hit rate: >80%
- ✅ L2 cache hit rate: >95%
- ✅ OLTP → OLAP lag: <1 second

**Scalability:**
- ✅ 1M keys in AlexStorage (L1)
- ✅ 10M keys in RocksStorage (L2)
- ✅ 100M rows in ArrowStorage (OLAP)
- ✅ 1TB total dataset

**Reliability:**
- ✅ Crash recovery (WAL replay)
- ✅ Atomic compaction (AlexStorage)
- ✅ ACID transactions (RocksStorage)

### Business Metrics

**Differentiation:**
- ✅ Only PostgreSQL-compatible HTAP with learned cache tier
- ✅ 4.81x read speedup for hot keys
- ✅ Real-time analytics without ETL
- ✅ Lower latency than competitors (L1 cache advantage)

**Market Positioning:**
- Target: $22.8B ETL market (companies needing real-time analytics)
- Competition: CockroachDB ($5B), SingleStore ($1.3B), TiDB ($270M raised)
- Advantage: Learned optimization + PostgreSQL compatibility + HTAP

---

## Conclusion

**Answer to Original Question:**

**Is AlexStorage optimal for intended use cases?**
- **No** - AlexStorage alone is insufficient for HTAP workloads
- **Yes** - AlexStorage is optimal for **L1 cache tier** (ultra-hot keys)

**Should we support multiple storage engines?**
- **Yes, absolutely** - Multi-engine architecture is required for HTAP

**Recommended Strategy:**

✅ **Multi-Engine Tiered Storage:**
1. AlexStorage (L1): Ultra-hot cache (4.81x reads)
2. RocksStorage (L2): General OLTP (proven, fast writes)
3. ArrowStorage (Cold): OLAP analytics (columnar)
4. Query Router: Learned optimizer (hot/cold placement)

**Why This Works:**
- ✅ Plays to AlexStorage's strengths (ultra-fast reads)
- ✅ Avoids AlexStorage's weaknesses (slow writes)
- ✅ Leverages proven tech (RocksDB, Arrow)
- ✅ Differentiated (learned cache tier + query routing)
- ✅ Meets HTAP requirements (dual storage, workload isolation)

**Investment:**
- AlexStorage development (Phases 1-7): **Not wasted** - becomes L1 cache
- RocksDB: Already battle-tested, production-ready
- Arrow/Parquet: Industry standard for OLAP
- Learned optimizer: Novel IP, competitive advantage

**Timeline to HTAP MVP:** 6-8 weeks
- Week 1-2: Query router
- Week 3-4: WAL replication (OLTP → OLAP)
- Week 5-6: Hot/cold promotion
- Week 7-8: Benchmarking

**Confidence:** 95% that multi-engine architecture is optimal for OmenDB's HTAP vision

---

**Last Updated:** October 6, 2025
**Status:** Strategic recommendation - multi-engine tiered storage
**Next Action:** Implement query router (Phase 8)
