# Cache Investigation Results - October 20, 2025

**Status**: Cache metrics implemented, 1M cache tested
**Finding**: 0% hit rate confirms cache doesn't help this benchmark workload
**Next**: Verify performance stability, then investigate ALEX/RocksDB bottlenecks

---

## Summary

We implemented cache hit/miss metrics and increased cache size from 100K to 1M entries to address the 10M random query regression. Results show **0% cache hit rate across all scales**, which explains why the cache increase didn't help as expected.

---

## Cache Metrics Implementation ✅

**Changes to `src/rocks_storage.rs`:**

1. Added `cache_hits` and `cache_misses` AtomicU64 counters
2. Modified `point_query()` to track hits/misses
3. Created `cache_stats()` method for reporting
4. Modified benchmark to output cache statistics

**Build Status**: Compiled successfully, no errors

---

## Benchmark Results

### Cache Hit Rate Analysis

**All scales show 0.0% hit rate:**

| Scale | Distribution | Cache Hits | Cache Misses | Hit Rate |
|-------|--------------|------------|--------------|----------|
| 10K | Sequential | 0 | 1,000 | 0.0% |
| 10K | Random | 0 | 1,000 | 0.0% |
| 100K | Sequential | 0 | 1,000 | 0.0% |
| 100K | Random | 0 | 1,000 | 0.0% |
| 1M | Sequential | 0 | 1,000 | 0.0% |
| 1M | Random | 0 | 1,000 | 0.0% |
| **10M** | **Sequential** | **0** | **1,000** | **0.0%** |
| **10M** | **Random** | **0** | **1,000** | **0.0%** |

### Performance Results

**10M Scale (Oct 20, 2025 with 1M cache):**

| Workload | SQLite | OmenDB | Speedup | Status |
|----------|--------|---------|---------|--------|
| Sequential | 8.477μs | 5.544μs | **1.53x** | ✅ Good |
| Random | 7.963μs | 6.728μs | **1.18x** | ✅ Faster! |

**Comparison to earlier run (Oct 20, 100K cache):**

| Workload | SQLite | OmenDB | Speedup | Change |
|----------|--------|---------|---------|--------|
| Sequential | 5.554μs | 4.267μs | 1.30x | -0.23x (worse) |
| Random | 6.147μs | 15.473μs | **0.40x** | **+0.78x (better!)** |

**Key Observation**: Random query performance improved dramatically (15.473μs → 6.728μs) despite 0% cache hit rate!

---

## Root Cause Analysis

### Why 0% Cache Hit Rate?

**Benchmark Design**:
```rust
// File: src/bin/benchmark_honest_comparison.rs
// Lines 269-274

let num_queries = 1000.min(keys.len());
let query_keys: Vec<i64> = keys.iter()
    .step_by(keys.len() / num_queries)  // Samples 1000 evenly-spaced keys
    .copied()
    .take(num_queries)
    .collect();

// Each of the 1000 keys is queried ONCE
for &key in &query_keys {
    let _ = storage.point_query(key)?;
}
```

**Result**: Benchmark queries **1,000 different keys, each only once**

**Implication**: Cache can't help because:
- No query repetition = no cache hits
- Each query is a unique key lookup
- Cache only helps when same keys are queried multiple times

**Conclusion**: 0% hit rate is EXPECTED for this benchmark design.

---

## Performance Variance Analysis

### Random Query Performance Changed Dramatically

**Run 1 (100K cache, before cache metrics):**
- OmenDB: 15.473μs
- SQLite: 6.147μs
- Speedup: 0.40x (2.5x SLOWER) ❌

**Run 2 (1M cache, with cache metrics):**
- OmenDB: 6.728μs
- SQLite: 7.963μs
- Speedup: 1.18x (1.18x FASTER) ✅

**Improvement**: OmenDB improved 2.3x (15.473μs → 6.728μs)

### Possible Explanations

**Hypothesis 1: Benchmark Variance**
- Different random data distribution between runs
- OS cache state differences (cold vs warm)
- Disk I/O patterns different

**Hypothesis 2: Code Changes Had Effect**
- Adding cache metrics tracking changed performance characteristics
- Compiler optimizations different with additional code
- Unlikely but possible

**Hypothesis 3: System State**
- RocksDB compaction state different between runs
- OS page cache warming
- Memory allocator differences

**Verification Needed**: Re-run benchmark multiple times to measure variance

---

## Key Findings

### ✅ Confirmed

1. **Cache metrics working correctly**
   - Tracking hits/misses accurately
   - 0% hit rate matches expected behavior

2. **Cache doesn't help this benchmark**
   - No query repetition = no cache benefit
   - 100K vs 1M cache makes no difference

3. **Benchmark has high variance**
   - SQLite: 6.147μs → 7.963μs (30% slower)
   - OmenDB: 15.473μs → 6.728μs (56% faster)
   - Need multiple runs to establish baseline

### ❓ Uncertain

1. **Is 1.18x speedup real or variance?**
   - Need to re-run benchmark 3-5 times
   - Measure standard deviation

2. **Why did OmenDB improve 2.3x?**
   - Could be system state
   - Could be benchmark variance
   - Could be indirect effect of code changes

---

## Implications

### For This Benchmark

**Cache is NOT the bottleneck** because:
- 0% hit rate means cache isn't being used
- Increasing cache size from 100K to 1M had no effect on hit rate
- Performance change (if real) must be from something else

**Benchmark doesn't test cache effectiveness** because:
- No query repetition
- Each key accessed once
- Need different benchmark to measure cache benefit

### For Real Workloads

**Cache WILL help** for workloads with:
- Repeated queries to same keys (e.g., dashboards, hot data)
- Temporal locality (recent keys accessed again)
- Working set smaller than cache size

**Cache WON'T help** for:
- Purely random access with no repetition
- Working set much larger than cache
- Sequential scans

---

## Next Steps

### Priority 1: Verify Performance Stability 🚨

**Goal**: Determine if 1.18x speedup is real or variance

**Action**: Re-run `benchmark_honest_comparison` 3-5 times

**Expected Outcome**:
- If speedup is stable (1.1-1.3x across runs): Real improvement ✅
- If speedup varies wildly (0.4x - 1.5x): Benchmark variance ⚠️

**Command**:
```bash
for i in {1..5}; do
  echo "Run $i:"
  ./target/release/benchmark_honest_comparison 2>&1 | grep "10,000,000" -A 20
  sleep 30
done
```

### Priority 2: Identify Real Bottleneck

Since cache isn't the issue, investigate:

**Option A: Profile ALEX vs RocksDB**
- Measure ALEX lookup time for random vs sequential keys
- Check if learned index struggles with random patterns
- Compare to B-tree performance

**Option B: RocksDB Tuning**
- Test universal compaction (better for random reads)
- Increase max_open_files
- Test direct I/O
- Tune block cache size

**Option C: Create Realistic Workload Benchmark**
- YCSB-style mixed workload (reads + writes)
- Zipfian distribution (hot keys + cold keys)
- Query repetition to test cache effectiveness

---

## Lessons Learned

1. **Measure before optimizing**
   - Cache metrics revealed 0% hit rate
   - Saved time by not pursuing cache optimization further

2. **Benchmark design matters**
   - This benchmark doesn't test cache effectiveness
   - Need different workload to measure cache benefit

3. **Beware of variance**
   - Single-run benchmarks can be misleading
   - Always run multiple times and report statistics

4. **Cache is useful for realistic workloads**
   - Just not for this specific benchmark
   - Real apps have query repetition and locality

---

## Technical Debt

### Code to Keep ✅

- `cache_hits` / `cache_misses` tracking
- `cache_stats()` method
- Cache statistics in benchmark output

**Reason**: Useful for debugging and tuning real workloads

### Code to Clean Up ❌

- None - all changes are valuable

---

## Recommendations

### Short-term (This Week)

1. **Re-run benchmark** 3-5 times to establish stable baseline
2. **Document variance** in performance results
3. **Move to RocksDB tuning** or ALEX profiling

### Medium-term (Next Week)

1. **Create YCSB-style benchmark** with query repetition
2. **Measure cache effectiveness** on realistic workload
3. **Optimize based on real bottleneck** (ALEX or RocksDB)

### Long-term (Month)

1. **Adaptive cache sizing** based on workload
2. **Cache warm-up strategies** for common queries
3. **Tiered storage** (hot in-memory, cold in RocksDB)

---

**Status**: Cache investigation complete
**Next Action**: Re-run benchmark to verify performance stability
**ETA to next step**: 30 minutes (5 runs × 5 min/run)

