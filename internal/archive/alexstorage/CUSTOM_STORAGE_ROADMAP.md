# Custom AlexStorage Implementation Roadmap

**Timeline:** 8 weeks
**Goal:** 10-50x speedup vs SQLite for sequential workloads, 2-5x for random
**Current baseline:** RocksDB + ALEX (2-3x sequential, 0.1x random)

---

## Executive Summary

**Why custom storage?**
- RocksDB baseline: 2-3x speedup for sequential, but 10x slower for random
- ALEX has overhead per-key insertion that kills random performance
- Need batch-optimized, zero-copy, state-of-the-art storage for competitive moat

**Target performance:**
- **Sequential inserts:** 10-20x faster than SQLite (from 2.3x with RocksDB)
- **Random inserts:** 2-5x faster than SQLite (from 0.09x with RocksDB)
- **Query latency:** <1μs average (from 1.7-3.9μs with RocksDB)
- **Scale:** Linear to 100M+ keys

**Key innovations:**
1. **Batch-mode ALEX:** Amortize training/split overhead across batches
2. **Memory-mapped storage:** Zero-copy reads, OS-managed caching
3. **Lazy node splitting:** Defer expensive splits until flush
4. **Adaptive layout:** Hot/cold data separation for cache efficiency

---

## Architecture Overview

```
AlexStorage (custom state-of-the-art):
├── ALEX Tree Layer
│   ├── Batch training (amortize model building)
│   ├── Lazy splits (defer until flush)
│   └── Gapped arrays (50% capacity, O(1) inserts)
│
├── Storage Layer
│   ├── Memory-mapped files (zero-copy reads)
│   ├── Append-only log (sequential writes)
│   ├── Compaction (background merge)
│   └── Bloom filters (skip non-existent keys)
│
└── Optimization Layer
    ├── LRU cache (hot values in memory)
    ├── Predictive prefetch (learned patterns)
    ├── Adaptive layout (hot/cold separation)
    └── SIMD vectorization (bulk operations)
```

---

## 8-Week Timeline

### Week 1: Foundation & Design

**Goal:** Design document + basic memory-mapped storage

**Tasks:**
1. Detailed architecture document
   - Data structures (ALEX tree, mmap layout, log format)
   - Interface design (Storage trait, batch API)
   - Performance targets per component

2. Memory-mapped storage prototype
   - Basic mmap file management
   - Append-only log format
   - Simple value read/write

3. Benchmark harness
   - Microbenchmarks for mmap vs RocksDB
   - Test data generators (sequential, random, Zipfian)

**Deliverables:**
- `internal/ALEX_STORAGE_DESIGN.md` (architecture)
- `src/alex_storage/mmap.rs` (mmap prototype)
- `benches/alex_storage_micro.rs` (microbenchmarks)

**Success criteria:**
- Mmap read latency: <500ns (vs 1-4μs RocksDB)
- Append-only write: >5M writes/sec

---

### Week 2: Batch-Mode ALEX

**Goal:** ALEX tree with batch training and lazy splits

**Tasks:**
1. Batch insertion API
   ```rust
   pub fn insert_batch(&mut self, entries: Vec<(i64, usize)>) -> Result<()> {
       // Sort keys for sequential insertion
       // Bulk train models (amortize overhead)
       // Defer node splits until batch complete
   }
   ```

2. Lazy split implementation
   - Accumulate inserts in buffer
   - Split only when buffer full or flush called
   - Minimize per-key overhead

3. Benchmarks vs current ALEX
   - Measure batch overhead reduction
   - Target: 10-100x fewer splits for random data

**Deliverables:**
- `src/alex_storage/batch_alex.rs` (batch ALEX)
- `benches/batch_alex.rs` (batch vs single-key)

**Success criteria:**
- Batch mode: 10-50x faster than single-key for random data
- Sequential: Maintain 2-3x RocksDB baseline

---

### Week 3: Integrated Storage Layer

**Goal:** Combine batch ALEX + mmap storage

**Tasks:**
1. Value storage in mmap files
   - Layout: [ALEX metadata | value log]
   - ALEX stores file offset, not value
   - Zero-copy reads via mmap pointers

2. Write path
   - Batch ALEX insert (keys → offsets)
   - Append values to mmap log
   - Sync to disk (durability)

3. Read path
   - ALEX point query (key → offset)
   - Mmap pointer dereference (offset → value)
   - LRU cache for hot values

**Deliverables:**
- `src/alex_storage/mod.rs` (integrated storage)
- `src/alex_storage/tests.rs` (unit tests)

**Success criteria:**
- Point query: <1μs average
- Insert throughput: >2M/sec sequential

---

### Week 4: Compaction & Durability

**Goal:** Production-ready storage with compaction

**Tasks:**
1. Compaction strategy
   - Background thread for merging segments
   - Garbage collection of overwritten values
   - ALEX tree rebuild (amortized)

2. Durability guarantees
   - Write-ahead log (WAL) for crash recovery
   - Fsync on commit
   - Checkpoint/snapshot mechanism

3. Bloom filters
   - Skip non-existent keys (avoid ALEX overhead)
   - Tunable false-positive rate

**Deliverables:**
- `src/alex_storage/compaction.rs`
- `src/alex_storage/wal.rs`
- `src/alex_storage/bloom.rs`

**Success criteria:**
- Compaction: <10% overhead on throughput
- Recovery: <1s for 1M keys
- Bloom filter: 99%+ skip rate for non-existent keys

---

### Week 5: Optimization & SIMD

**Goal:** Squeeze every bit of performance

**Tasks:**
1. SIMD vectorization
   - Batch key comparisons (AVX2/AVX-512)
   - Parallel model predictions
   - Vectorized searches in gapped nodes

2. Adaptive layout
   - Hot/cold data separation (learned access patterns)
   - Tiered storage (memory → SSD → cold storage)
   - Predictive prefetching

3. Cache optimization
   - LRU eviction tuning
   - Cache-aligned data structures
   - Reduce cache misses in ALEX nodes

**Deliverables:**
- `src/alex_storage/simd.rs` (SIMD ops)
- `src/alex_storage/adaptive.rs` (hot/cold)
- `benches/alex_storage_simd.rs`

**Success criteria:**
- SIMD: 2-4x faster bulk operations
- Adaptive layout: 50%+ fewer cache misses
- Overall: 5-10x faster than RocksDB baseline

---

### Week 6: Full System Integration

**Goal:** Integrate AlexStorage into OmenDB engine

**Tasks:**
1. Replace RocksStorage in DataFusion layer
   - Update `redb_table.rs` → `alex_table.rs`
   - Batch query optimization
   - Range query via ALEX

2. Update SQL engine
   - Batch insert support
   - Transaction integration
   - Learned query optimization

3. End-to-end testing
   - All 249 existing tests pass
   - New AlexStorage-specific tests
   - Stress testing (100M+ keys)

**Deliverables:**
- `src/datafusion/alex_table.rs`
- Updated `src/sql_engine.rs`
- Integration tests

**Success criteria:**
- All tests pass (249/249)
- 100M keys: <30s insert, <2μs query

---

### Week 7: Benchmarking & Validation

**Goal:** Validate 10-50x claims with honest benchmarks

**Tasks:**
1. Re-run honest comparison vs SQLite
   - Sequential: Target 10-20x at 10M+ scale
   - Random: Target 2-5x at 10M+ scale
   - Queries: Target <1μs average

2. Competitive benchmarks
   - vs CockroachDB (single-node OLTP)
   - vs TiDB (single-node OLTP)
   - vs DuckDB (OLAP queries)

3. Real-world workloads
   - Time-series (sensor data)
   - Log aggregation (application logs)
   - IoT ingestion (high-frequency writes)

**Deliverables:**
- `src/bin/benchmark_alex_vs_sqlite.rs`
- `src/bin/benchmark_alex_vs_cockroachdb.rs`
- `internal/ALEX_BENCHMARK_RESULTS.md`

**Success criteria:**
- Sequential: 10-20x faster than SQLite ✅
- Random: 2-5x faster than SQLite ✅
- CockroachDB: 10-50x faster single-node writes

---

### Week 8: Documentation & Customer Validation

**Goal:** Production-ready release with customer validation

**Tasks:**
1. Documentation
   - Architecture guide (`docs/ALEX_STORAGE.md`)
   - Performance tuning guide
   - API documentation (Rust docs)
   - Migration guide (RocksDB → AlexStorage)

2. Customer validation
   - 3-5 LOIs from time-series/IoT companies
   - Pilot deployment setup
   - Performance validation on real workloads

3. Fundraising materials
   - Updated pitch deck (10-50x validated)
   - Demo application (real-time analytics)
   - Technical whitepaper

**Deliverables:**
- Complete documentation
- 3-5 customer LOIs
- YC S25 application materials

**Success criteria:**
- 10-50x claims validated with benchmarks
- Customer LOIs secured
- Production-ready AlexStorage

---

## Expected Performance (Week 8)

### vs SQLite (Validated Targets)

| Scale | Workload | SQLite | AlexStorage | Speedup |
|-------|----------|--------|-------------|---------|
| 1M | Sequential insert | 860ms | **40-80ms** | **10-20x** ✅ |
| 1M | Random insert | 3.5s | **700ms-1.7s** | **2-5x** ✅ |
| 10M | Sequential insert | 8.6s | **430-860ms** | **10-20x** ✅ |
| 10M | Random insert | 35s | **7-17s** | **2-5x** ✅ |
| 10M | Point query | 6-7μs | **<1μs** | **6-7x** ✅ |
| 10M | Range query (1K) | 6ms | **<1ms** | **6x+** ✅ |

### vs RocksDB Baseline (Our Current)

| Metric | RocksDB+ALEX | AlexStorage | Improvement |
|--------|--------------|-------------|-------------|
| Sequential insert (1M) | 376ms | **40-80ms** | **5-10x** |
| Random insert (1M) | 37.8s | **700ms-1.7s** | **20-50x** |
| Point query (1M) | 3.7μs | **<1μs** | **4-7x** |
| Memory overhead | 100% (RocksDB + ALEX) | **20-30%** | **3-5x less** |

---

## Risk Mitigation

### Technical Risks

**Risk 1: Mmap performance on macOS**
- Issue: macOS mmap slower than Linux
- Mitigation: Test on Linux, provide optimized paths per OS
- Fallback: Direct I/O if mmap underperforms

**Risk 2: Random data still slow**
- Issue: Batch mode may not fully solve random overhead
- Mitigation: Hybrid approach (B-tree for random, ALEX for sequential)
- Fallback: Document "sequential-optimized" positioning

**Risk 3: Compaction overhead**
- Issue: Background compaction may impact write throughput
- Mitigation: Tunable compaction policy, rate limiting
- Fallback: Disable compaction for write-only workloads

### Timeline Risks

**Risk: 8 weeks is aggressive**
- Mitigation: Parallel development (SIMD while building compaction)
- Fallback: MVP in 6 weeks, optimizations in weeks 7-8
- De-scope: SIMD and adaptive layout if time-constrained

---

## Success Metrics

### Week 4 (MVP Checkpoint)
- ✅ Basic AlexStorage working (insert, query, durability)
- ✅ 5-10x faster than RocksDB for sequential
- ✅ Parity or 2x faster for random (vs RocksDB)

### Week 8 (Production Release)
- ✅ 10-20x faster than SQLite for sequential
- ✅ 2-5x faster than SQLite for random
- ✅ <1μs query latency at 10M+ scale
- ✅ 249/249 tests passing
- ✅ 3-5 customer LOIs

### Fundraising (Post-Week 8)
- ✅ Validated 10-50x claims with honest benchmarks
- ✅ Customer validation on real workloads
- ✅ Clear competitive advantage vs SQLite/CockroachDB/TiDB
- ✅ Technical moat (state-of-the-art learned storage)

---

## Comparable Projects

### LearnedKV (SOSP 2024)
- **Speedup:** 4.32x at 10M+ keys, 1KB values, Zipfian
- **Approach:** LSM-tree + learned index
- **Our advantage:** Batch ALEX + mmap (targeting 10-50x)

### ALEX Paper (SIGMOD 2020)
- **Speedup:** 2-3x vs B-tree in-memory
- **Limitation:** No production storage layer
- **Our advantage:** Full storage system with durability

### QuestDB (Time-series DB, $15M Series A)
- **Speedup:** 10x faster ingestion than TimescaleDB
- **Approach:** Columnar storage, mmap, SIMD
- **Our advantage:** Learned indexes + similar optimizations

**Positioning:** We combine ALEX learned indexes (algorithmic advantage) with QuestDB-style optimizations (engineering advantage) for 10-50x total speedup.

---

## Funding Story

### Seed Round ($1-3M) Narrative

**Technical differentiation:**
- ✅ State-of-the-art learned indexes (ALEX + batch mode)
- ✅ Zero-copy storage (mmap + SIMD)
- ✅ 10-50x validated speedup for sequential workloads
- ✅ Production-ready system (249 tests, customer validation)

**Market opportunity:**
- $22.8B ETL market (companies needing real-time analytics)
- Target: Companies using PostgreSQL + Snowflake + Fivetran
- Pitch: "Eliminate ETL pipelines with learned index optimization"

**Traction milestones:**
- Week 8: 3-5 customer LOIs (time-series, IoT, monitoring)
- Month 6: $50K-$100K ARR from pilots
- Month 12: $1M ARR target (10-20 customers)

**Comparable funding:**
- QuestDB: $15M Series A (time-series focus, 10x speedup)
- DuckDB: $52.5M (100x analytics speedup)
- Our ask: $1-3M seed (10-50x OLTP+OLAP speedup)

---

## Next Actions

### Immediate (This Week)
1. Create `internal/ALEX_STORAGE_DESIGN.md` (detailed architecture)
2. Prototype mmap storage (`src/alex_storage/mmap.rs`)
3. Microbenchmarks (mmap read latency, append throughput)

### Week 2-3
1. Implement batch-mode ALEX
2. Integrate with mmap storage
3. Benchmark vs RocksDB baseline

### Week 4-5
1. Add compaction, durability, Bloom filters
2. SIMD optimizations
3. Hit 5-10x RocksDB baseline

### Week 6-8
1. Full system integration
2. Competitive benchmarks (SQLite, CockroachDB)
3. Customer validation, fundraising prep

---

**Last Updated:** October 5, 2025
**Status:** RocksDB baseline complete (2-3x), AlexStorage roadmap defined
**Timeline:** 8 weeks to production-ready 10-50x system
**Target:** Seed fundraising Q1 2026 with validated claims
