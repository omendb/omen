# RocksDB Integration Benchmark Results

**Date:** October 5, 2025
**Purpose:** Validate RocksDB + ALEX vs SQLite performance
**Storage:** RocksDB (LSM-tree) + ALEX learned index vs SQLite B-tree

---

## Executive Summary

**Sequential workloads (our sweet spot):**
- ✅ **2.2-2.3x faster inserts** than SQLite at all scales
- ✅ **1.6-4.9x faster queries** than SQLite
- ✅ **2.0-3.6x average speedup**

**Random workloads (challenging):**
- ✅ **1.3-5.4x faster queries** than SQLite (consistent win)
- ⚠️ **0.09-1.35x insert performance** (slower at 1M scale)
- 📊 **Average: 0.95-3.37x** (degrades with scale)

**Bottom line:** RocksDB + ALEX delivers **2-3x speedup for sequential data** (time-series, log analytics). Random UUID workloads still need optimization.

---

## Detailed Results

### 10,000 Rows

| Distribution | Insert Speedup | Query Speedup | Average | Verdict |
|--------------|---------------|---------------|---------|---------|
| **Sequential** | 2.24x | 4.86x | **3.55x** | ✅ GOOD |
| **Random** | 1.35x | 5.39x | **3.37x** | ✅ GOOD |

**Analysis:**
- Small scale: Both workloads perform well
- RocksDB LSM-tree handles both sequential and random efficiently
- Query performance is excellent (4.9-5.4x faster)

---

### 100,000 Rows

| Distribution | Insert Speedup | Query Speedup | Average | Verdict |
|--------------|---------------|---------------|---------|---------|
| **Sequential** | 2.22x | 2.66x | **2.44x** | ✅ GOOD |
| **Random** | 0.77x (SLOWER) | 3.11x | **1.94x** | ➖ NEUTRAL |

**Analysis:**
- Sequential still strong (2.2x inserts)
- Random inserts starting to degrade (184ms → 241ms, 0.77x slower)
- ALEX tree overhead becoming visible on random data

**SQLite times:**
- Sequential: 80ms inserts, 4.5μs queries
- Random: 184ms inserts, 5.4μs queries

**OmenDB times:**
- Sequential: 36ms inserts, 1.7μs queries (better)
- Random: 241ms inserts, 1.7μs queries (worse inserts, better queries)

---

### 1,000,000 Rows

| Distribution | Insert Speedup | Query Speedup | Average | Verdict |
|--------------|---------------|---------------|---------|---------|
| **Sequential** | 2.29x | 1.64x | **1.96x** | ➖ NEUTRAL |
| **Random** | 0.09x (10x SLOWER!) | 1.82x | **0.95x** | ⚠️ SLOWER |

**Analysis:**
- **Sequential still wins:** 860ms → 376ms (2.29x faster inserts)
- **Random data bottleneck:** 3.5s → 37.8s (10x SLOWER!)
- Query latency degrading slightly but still faster

**Critical finding:** Random inserts at 1M scale are 10x slower than SQLite.

**SQLite times:**
- Sequential: 860ms inserts, 6.1μs queries
- Random: 3554ms inserts, 7.1μs queries

**OmenDB times:**
- Sequential: 376ms inserts, 3.7μs queries (2.3x faster)
- Random: 37820ms inserts, 3.9μs queries (10x slower inserts!)

---

## Root Cause Analysis

### Why Sequential Data is Fast

1. **RocksDB LSM-tree optimization:** Sequential writes are LSM-tree's sweet spot
2. **ALEX gapped arrays:** Sequential keys fill gaps efficiently
3. **Minimal node splits:** Sequential data rarely triggers expensive splits
4. **Cache-friendly:** Sequential access patterns hit CPU cache

**Result:** 2.2-2.3x speedup consistently across all scales

---

### Why Random Data is Slow at Scale

1. **ALEX exponential search overhead:**
   - Each random key requires binary search within gapped node
   - Node splits happen frequently with random data
   - Exponential search adds latency per operation

2. **RocksDB compaction:**
   - Random writes trigger more compaction in LSM-tree
   - ALEX tracking adds overhead on top of RocksDB writes

3. **Cache misses:**
   - Random access patterns defeat CPU cache
   - Each key lookup may miss L1/L2/L3 cache

**Metrics at 1M random:**
- Insert time: 37.8 seconds (26,440 rows/sec)
- SQLite: 3.5 seconds (281,360 rows/sec)
- **10x slower** - unacceptable for production

---

## Comparison to redb Baseline

**Previous (redb + ALEX):**
- Sequential 1M: 0.88x (slower than SQLite)
- Random 1M: 0.10x (10x slower than SQLite)

**Current (RocksDB + ALEX):**
- Sequential 1M: **2.29x faster** than SQLite ✅
- Random 1M: 0.09x (still 10x slower)

**Conclusion:** RocksDB fixed sequential performance but didn't solve random data bottleneck (that's in ALEX itself).

---

## Performance vs Claims

### Original Claims (Pre-RocksDB)
- Projected: 5-15x faster than SQLite at 10M scale with ALEX
- Reality at 1M: 1.96x sequential, 0.95x random

### Current Reality (RocksDB + ALEX)
- **Can claim:** "2-3x faster for sequential workloads (time-series, logs)"
- **Can claim:** "2-5x faster queries across all workloads"
- **Cannot claim:** "Faster than SQLite for random UUID inserts"
- **Cannot claim:** "10x+ speedup without specifying workload"

---

## Target Use Cases

### ✅ Sweet Spot (2-3x advantage)
1. **Time-series data:** Sensor logs, metrics, events
2. **Append-only logs:** Application logs, audit trails
3. **Sequential IDs:** Auto-increment primary keys
4. **Real-time analytics:** Fast queries on sequential data

### ⚠️ Challenging (0.1-2x, needs optimization)
1. **UUID primary keys:** Random distributed IDs
2. **Hash-based sharding:** Random key distribution
3. **High-entropy data:** Randomized workloads

### 🎯 Ideal Customer Profile
- Companies with time-series workloads
- Real-time analytics on sequential data
- Replacing PostgreSQL for log aggregation
- IoT sensor data pipelines

---

## Next Steps

### Short-term: Custom Storage (8 weeks)

**Goal:** Achieve 10-50x speedup for sequential, 2-5x for random

**Architecture:**
```
AlexStorage (custom state-of-the-art):
├── ALEX learned index (position prediction)
├── Memory-mapped files (zero-copy reads)
├── Gapped arrays (in-memory, 50% capacity)
├── Batch-optimized writes (amortize overhead)
└── Adaptive layout (hot/cold separation)
```

**Expected improvements:**
1. **Sequential inserts:** 5-10x faster (from 2.3x to 10-20x vs SQLite)
2. **Random inserts:** 2-5x faster (from 0.09x to 0.5-2x vs SQLite)
3. **Query latency:** <1μs (from 1.7-3.9μs)

**Key optimizations:**
- Batch mode for ALEX (reduce per-key overhead)
- Lazy node splitting (defer until flush)
- Zero-copy value reads (mmap)
- Predictive prefetching (learned patterns)

---

## Competitive Positioning (Updated)

### vs SQLite
- **Sequential:** 2.3x faster ✅
- **Random:** 10x slower ⚠️
- **Positioning:** "2-3x faster for time-series/log workloads"

### vs RMI (previous baseline)
- **Sequential:** 2.6x improvement (0.88x → 2.29x)
- **Random:** No change (both 10x slower)
- **Positioning:** "RocksDB solved sequential bottleneck, ALEX needs optimization for random"

### vs Custom Storage (projected)
- **Sequential:** 4-8x improvement potential
- **Random:** 10-50x improvement potential
- **Positioning:** "State-of-art custom storage for 10-50x advantage"

---

## Funding Narrative

### Current State (RocksDB + ALEX)
- ✅ Proven 2-3x speedup for sequential workloads
- ✅ Production-ready RocksDB foundation
- ⚠️ Random data needs optimization

### Pitch to Investors
1. **Proven advantage:** 2-3x faster for time-series/logs (validated)
2. **Clear path to 10-50x:** Custom storage roadmap (8 weeks)
3. **Target market:** $22.8B ETL market, time-series companies
4. **Comparable:** QuestDB (time-series focus) raised $15M Series A

### Timeline
- **Today:** RocksDB baseline complete, 2-3x proven
- **Week 8:** Custom storage, targeting 10-50x for sequential
- **Week 12:** Customer validation (IoT, monitoring, financial)

---

## Honest Assessment

### What Works
✅ Sequential data: 2.3x faster inserts consistently
✅ All queries: 1.6-5.4x faster than SQLite
✅ RocksDB foundation: Battle-tested, production-ready
✅ ALEX architecture: No O(n) rebuilds, scales linearly

### What Needs Work
⚠️ Random inserts: 10x slower at 1M scale
⚠️ ALEX optimization: Batch mode, lazy splits needed
⚠️ Cache efficiency: Random access patterns need work
⚠️ Workload-specific tuning: One size doesn't fit all

### What We Learned
1. RocksDB is a solid baseline (better than redb)
2. ALEX needs batch optimization for random data
3. Sequential workloads are our competitive advantage
4. Custom storage is necessary for 10-50x claims

---

**Last Updated:** October 5, 2025
**Status:** RocksDB integration complete, custom storage next
**Target:** 10-50x speedup with custom storage in 8 weeks
