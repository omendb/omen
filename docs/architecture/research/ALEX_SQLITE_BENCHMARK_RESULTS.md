# ALEX vs SQLite Benchmark Results - January 2025

**Date**: January 2025
**Benchmark**: Table system (ALEX learned index) vs SQLite (B-tree)
**Status**: Honest competitive assessment
**Commit**: TBD

---

## Executive Summary

**Results at 1M scale:**
- **Sequential (time-series)**: 2.37x average speedup ✅
- **Random (UUID-like)**: 2.25x average speedup, BUT 10x SLOWER inserts ⚠️

**Key Finding**: ALEX delivers excellent query performance (2.79-4.40x faster) across all workloads, but random inserts are significantly slower than SQLite due to learned index restructuring.

**Validated Claims:**
- ✅ Faster queries on all workloads (2.79-4.40x)
- ✅ Faster sequential inserts (1.95x)
- ⚠️ Slower random inserts (0.10x - major issue)
- ⚠️ Overall speedup (2.31x) below 5-15x projection

**Recommendation**: Position OmenDB for **time-series and sequential workloads**, not random UUID workloads.

---

## Detailed Results

### Test Configuration

**Systems:**
- SQLite 3.x: B-tree indexes, full ACID, WAL mode
- OmenDB: Table with ALEX learned index, Arrow/Parquet storage, MVCC

**Workloads:**
- Sequential: 0, 1, 2, 3, ... (time-series pattern)
- Random: Random i64 values (UUID-like pattern)

**Metrics:**
- Bulk insert: Time to insert all rows
- Point queries: Average latency over 1000 queries

### 10K Scale Results

| Workload | Insert Speedup | Query Speedup | Overall |
|----------|----------------|---------------|---------|
| Sequential | 1.39x | 17.19x | 9.29x ✅ |
| Random | 1.62x | 22.72x | 12.17x ✅ |
| **Average** | **1.51x** | **19.96x** | **10.73x** |

**Analysis**:
- Excellent performance at small scale
- Query speedup is exceptional (17-22x)
- Insert performance modest but positive

### 1M Scale Results

| Workload | Insert Speedup | Query Speedup | Overall |
|----------|----------------|---------------|---------|
| Sequential | 1.95x ✅ | 2.79x ✅ | 2.37x ✅ |
| Random | 0.10x ⚠️ | 4.40x ✅ | 2.25x ✅ |
| **Average** | **1.03x** | **3.60x** | **2.31x** |

**Analysis**:
- Sequential performance solid (2.37x overall)
- Random insert performance TERRIBLE (10x slower)
- Query performance still good (2.79-4.40x)
- Overall below projection (2.31x vs 5-15x target)

### Detailed Breakdown (1M Scale)

#### Sequential (Time-Series) Workload

```
BULK INSERT:
  SQLite:   835.5 ms  (1,196,835 rows/sec)
  OmenDB:   427.5 ms  (2,339,247 rows/sec)
  Speedup:  1.95x ✅

POINT QUERIES (1000 queries):
  SQLite:   5.789 μs avg
  OmenDB:   2.074 μs avg
  Speedup:  2.79x ✅

AVERAGE SPEEDUP: 2.37x ✅
```

**Verdict**: Good performance, nearly 2x faster on inserts, 2.79x on queries.

#### Random (UUID-Like) Workload

```
BULK INSERT:
  SQLite:   3,330.7 ms  (300,238 rows/sec)
  OmenDB:  34,112.2 ms  (29,315 rows/sec)
  Speedup:  0.10x ⚠️ MUCH SLOWER

POINT QUERIES (1000 queries):
  SQLite:   6.334 μs avg
  OmenDB:   1.438 μs avg
  Speedup:  4.40x ✅

AVERAGE SPEEDUP: 2.25x
```

**Verdict**: Terrible insert performance (10x slower), but excellent query performance (4.40x faster). Overall still positive due to query gains.

---

## Root Cause Analysis

### Why Random Inserts Are Slow

**Problem**: 34 seconds vs 3.3 seconds (10x slower)

**Root Causes:**
1. **ALEX restructuring**: Random inserts cause frequent node splits and rebalancing
2. **Cache misses**: Random access pattern defeats CPU cache
3. **Non-sequential I/O**: Arrow/Parquet storage optimized for sequential writes
4. **Index updates**: Each insert updates ALEX which isn't optimized for random data

**Evidence**:
- Sequential inserts: 1.95x faster (ALEX works well)
- Random inserts: 0.10x faster (ALEX struggles)
- Difference: 19.5x performance gap between workloads

### Why Queries Are Fast

**Positive**: 2.79-4.40x faster queries across all workloads

**Reasons:**
1. **ALEX prediction**: Even with random data, ALEX provides faster lookups than B-tree
2. **Cache-friendly**: Read patterns benefit from ALEX's compact structure
3. **Lower tree height**: ALEX's learned model reduces tree traversal depth

**Evidence**:
- Sequential queries: 2.79x faster
- Random queries: 4.40x faster
- Consistent query advantage regardless of insert pattern

---

## Competitive Assessment

### vs SQLite (1M Scale)

| Metric | Sequential | Random | Overall |
|--------|------------|--------|---------|
| Insert | 1.95x faster | 0.10x faster (10x slower) | 1.03x faster |
| Query | 2.79x faster | 4.40x faster | 3.60x faster |
| **Average** | **2.37x** | **2.25x** | **2.31x** |

**Conclusion**:
- ✅ Faster for time-series/sequential workloads (2.37x)
- ⚠️ Problematic for random/UUID workloads (slow inserts)
- ✅ Always faster queries (2.79-4.40x)

### Comparison to Projections

**Projected (from STATUS_REPORT_JAN_2025.md)**:
- 1M scale: 3-5x average speedup
- 10M scale: 5-15x average speedup

**Actual (1M scale)**:
- Sequential: 2.37x (below projection)
- Random: 2.25x (below projection)
- Overall: 2.31x (below projection)

**Status**: ⚠️ Below projections, but still positive

---

## Market Positioning

### ✅ Strong Use Cases (Time-Series)

**Where OmenDB Excels:**
1. **IoT sensor data**: Sequential timestamps, append-only
2. **Metrics/logs**: Time-ordered events
3. **Financial tick data**: Ordered market data
4. **Analytics on sequential data**: 2.37x speedup validated

**Validated Claims (1M scale):**
- "2.37x faster for time-series workloads" ✅
- "2.79x faster queries on sequential data" ✅
- "1.95x faster bulk inserts on time-series" ✅

### ⚠️ Weak Use Cases (Random Data)

**Where OmenDB Struggles:**
1. **UUID primary keys**: Random inserts 10x slower
2. **User-generated IDs**: Non-sequential patterns
3. **Random access patterns**: Poor insert performance

**Honest Assessment:**
- "10x slower random inserts vs SQLite" ⚠️
- "Still 4.40x faster queries, but inserts are a bottleneck"
- "Not recommended for random UUID workloads"

### Competitive Positioning

**Recommended Positioning:**
> "OmenDB: 2-3x faster database for time-series and sequential workloads. Optimized for IoT, metrics, logs, and analytics. Not recommended for random UUID primary keys."

**What NOT to claim:**
- ❌ "5-15x faster than SQLite" (only 2.31x at 1M)
- ❌ "Faster for all workloads" (random inserts are slow)
- ❌ "Drop-in SQLite replacement" (workload-dependent)

**What TO claim:**
- ✅ "2.37x faster for time-series workloads"
- ✅ "2.79-4.40x faster queries across all workloads"
- ✅ "Optimized for sequential data patterns"

---

## 10M Scale Projection

**Skip 10M testing** due to random insert performance issues.

**Projected 10M results** (if we tested):
- Sequential: 2-3x speedup (similar to 1M)
- Random: 0.05-0.10x speedup (even worse, 10-20x slower inserts)
- Overall: 1-2x speedup (not compelling)

**Reason to skip:**
- Random insert bottleneck won't improve at scale
- ALEX restructuring overhead increases with dataset size
- Better to fix root cause before scale testing

---

## Next Steps

### Short-Term (1-2 Weeks)

**1. Fix Random Insert Performance**
- Investigate ALEX restructuring overhead
- Consider batch insertion optimization
- Profile random insert codepath
- **Target**: Get random inserts to at least 0.5x SQLite (2x slower acceptable)

**2. Focus on Sequential Workloads**
- Market OmenDB for time-series use cases
- Update README to emphasize time-series optimization
- Remove claims about general-purpose database

**3. Update Competitive Assessment**
- Document honest 2.31x result (not 5-15x)
- Clearly state random insert limitation
- Position for time-series market

### Medium-Term (1-2 Months)

**1. Optimize for Mixed Workloads**
- Investigate hybrid index strategies
- Consider sorting batches before insertion
- Add buffer pool for random inserts

**2. Customer Validation**
- Find time-series customers (IoT, metrics, logs)
- Avoid UUID-based systems
- Validate 2-3x speedup in production

**3. Fundraising Strategy**
- Focus on time-series market ($1.45B TAM)
- Honest claims: "2-3x faster for time-series"
- Avoid overstating performance

---

## Lessons Learned

### What Worked

1. **Sequential performance**: 1.95-2.79x faster (validated)
2. **Query performance**: Always faster (2.79-4.40x)
3. **ALEX advantage**: Clear benefit for sorted data

### What Didn't Work

1. **Random inserts**: 10x slower (dealbreaker for UUID workloads)
2. **10x projection**: Actual 2.31x (overoptimistic)
3. **General-purpose positioning**: Too broad, need niche focus

### Honest Takeaways

1. **Learned indexes have tradeoffs**: Great for sorted, terrible for random
2. **Niche focus required**: Time-series, not general-purpose
3. **Transparency matters**: Honest benchmarks build trust
4. **Projections were wrong**: 2.31x not 5-15x

---

## Conclusion

**Current Status:**
- ✅ Validated: 2.37x faster for time-series workloads
- ⚠️ Issue: 10x slower random inserts
- ⚠️ Result: 2.31x overall (below 5-15x projection)

**Market Position:**
- **Viable**: Time-series, IoT, metrics, logs
- **Not viable**: Random UUIDs, general-purpose database

**Next Action:**
1. Fix random insert performance (target 0.5x SQLite)
2. Reposition for time-series market
3. Update all claims to reflect 2-3x reality

**Fundraising Impact:**
- Seed still viable with honest positioning
- Focus on $1.45B time-series market
- Claim: "2-3x faster for time-series workloads"

---

**Last Updated:** January 2025
**Status:** Honest competitive assessment complete
**Action Required:** Fix random inserts, update positioning
