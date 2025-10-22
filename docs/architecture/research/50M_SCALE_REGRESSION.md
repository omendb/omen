# 50M Scale Performance Regression - Critical Issue

**Date**: January 2025
**Test**: ALEX vs SQLite at 50M scale
**Status**: ⚠️ **CRITICAL REGRESSION FOUND**
**Commit**: TBD

---

## Executive Summary

**Critical Finding**: Performance degrades significantly at 50M scale, especially for queries.

**Key Issues**:
- **Queries**: 2.6x slower than at 10M scale (6.5μs → 17μs)
- **Sequential inserts**: Now SLOWER than SQLite (0.83x)
- **Overall**: 1.32x faster (down from 2.11x at 10M)

**Root Cause Hypothesis**: ALEX nodes grow too large at 50M scale, making exponential search + linear scan inefficient.

---

## Detailed Results

### Performance Comparison: 10M vs 50M

| Metric | 10M Scale | 50M Scale | Degradation |
|--------|-----------|-----------|-------------|
| **Sequential Insert** | 1.50x faster | 0.83x (SLOWER) | **-45%** ⚠️ |
| **Sequential Query** | 1.06x faster | 0.43x (SLOWER) | **-59%** ⚠️ |
| **Random Insert** | 4.71x faster | 3.60x faster | **-24%** |
| **Random Query** | 1.17x faster | 0.44x (SLOWER) | **-62%** ⚠️ |
| **Overall** | 2.11x faster | 1.32x faster | **-37%** |

### Absolute Numbers

**50M Scale - Sequential Workload**:
```
SQLite Insert:   45,040 ms  (1,110K rows/sec)
OmenDB Insert:   54,493 ms  (  918K rows/sec) - 0.83x (SLOWER) ⚠️

SQLite Query:     7.31μs avg
OmenDB Query:    16.97μs avg - 0.43x (SLOWER) ⚠️
```

**50M Scale - Random Workload**:
```
SQLite Insert:  302,647 ms  (165K rows/sec)
OmenDB Insert:   84,146 ms  (594K rows/sec) - 3.60x faster ✅

SQLite Query:     7.48μs avg
OmenDB Query:    17.18μs avg - 0.44x (SLOWER) ⚠️
```

### Query Latency Trend (Critical)

| Scale | OmenDB Query | SQLite Query | Ratio |
|-------|--------------|--------------|-------|
| 1M | 2.5μs | 6.3μs | **2.5x faster** ✅ |
| 10M | 6.5μs | 6.9μs | **1.1x faster** ✅ |
| 50M | **17.0μs** | 7.4μs | **0.4x (SLOWER)** ⚠️ |

**Trend**: Query latency increasing faster than SQLite as scale grows.
- 1M → 10M: 2.5μs → 6.5μs (2.6x increase)
- 10M → 50M: 6.5μs → 17.0μs (2.6x increase again)

**Projection at 100M**: ~44μs (completely unacceptable)

---

## Root Cause Analysis

### Hypothesis 1: ALEX Node Growth

**Theory**: At 50M scale, ALEX nodes contain 10K-50K keys each, making linear scan within nodes slow.

**Evidence**:
- Query latency scales with node size
- Exponential search narrows to bounded range, then linear scan
- 50M rows / ~1000 leaves = ~50K keys per node
- Linear scan of 50K keys = ~50K comparisons worst case

**Math**:
```
10M scale:
- ~10K keys per node
- Linear scan: ~10K comparisons
- Query time: 6.5μs

50M scale:
- ~50K keys per node (5x larger)
- Linear scan: ~50K comparisons (5x more)
- Query time: 17μs (2.6x slower)

Expected if purely linear: 6.5μs × 5 = 32.5μs
Actual: 17μs (better than worst case, but still bad)
```

### Hypothesis 2: Model Prediction Degradation

**Theory**: Linear model becomes less accurate at 50M scale, causing larger exponential search radius.

**Evidence**:
- More keys = harder to fit linear model
- Larger search radius = more iterations before finding bounded range
- Then falls back to larger linear scan

**Test**: Check `max_error_bound` at different scales.

### Hypothesis 3: Retrain Overhead

**Theory**: Auto-retrain after every batch at 50M scale adds significant overhead.

**Evidence**:
- Sequential inserts now SLOWER (0.83x vs 1.50x at 10M)
- Retrain is O(n log n) per leaf
- 50M rows = more retrains = more overhead

**Calculation**:
```
50M sequential insert time: 54,493ms
10M sequential insert time: 5,900ms
Expected 50M time (linear): 5,900ms × 5 = 29,500ms
Actual: 54,493ms (1.85x slower than expected)

Extra overhead: 54,493 - 29,500 = 25,000ms
Likely cause: Excessive retraining
```

### Hypothesis 4: Memory Pressure

**Theory**: 50M rows exceed cache, causing more memory bandwidth bottlenecks.

**Evidence**:
- Working set at 50M: ~500MB (exceeds L3 cache)
- More cache misses = slower lookups
- SQLite also affected but less severely

---

## Scaling Analysis

### Linear Scaling Validation

**Expected (if perfect linear scaling)**:
- 50M = 5 × 10M
- Expected time: 10M_time × 5

**Actual**:

| Operation | 10M Time | Expected 50M | Actual 50M | Ratio |
|-----------|----------|--------------|------------|-------|
| Sequential Insert | 5.9s | 29.5s | 54.5s | **1.85x worse** ⚠️ |
| Random Insert | 10.6s | 53s | 84.1s | **1.59x worse** ⚠️ |

**Conclusion**: Non-linear scaling - performance degrades faster than data size.

### Query Scaling (Critical)

**Expected (if logarithmic scaling)**:
- Tree depth grows as O(log n)
- 50M = 5 × 10M → depth increases by log(5) ≈ 2.3 levels
- Expected query time: 6.5μs × 1.2 = ~8μs

**Actual**: 17μs (2.1x worse than projection)

**Conclusion**: Query scaling is NOT logarithmic - closer to linear or worse.

---

## Performance Breakdown

### What Still Works

✅ **Random inserts**: 3.60x faster
- Batch insert optimization still effective
- Pre-sorting helps significantly
- Better than SQLite at bulk random data

### What's Broken

⚠️ **Queries**: 0.43-0.44x (MUCH slower)
- Linear scan dominates at 50K keys/node
- Exponential search not helping enough
- Model accuracy degraded

⚠️ **Sequential inserts**: 0.83x (slower)
- Retrain overhead too high
- O(n log n) retrain on large nodes expensive
- Sequential data doesn't benefit from pre-sorting

---

## Implications

### For Competitive Claims

**Before 50M test**:
- Claimed: "2-3x faster at 1M-10M scale"
- Status: Validated ✅

**After 50M test**:
- Reality: "1.3x faster at 50M scale"
- Queries: **0.4x (SLOWER)** ⚠️
- **Cannot claim "faster" beyond 10M**

### For Production Use

**Safe Range**: 1M-10M rows
- 2-3x faster overall
- Queries 1.1-2.8x faster
- Validated and reliable

**Problematic Range**: 10M-50M rows
- Performance degrades
- Queries become slower than SQLite
- Not production-ready

**Unsafe Range**: 50M+ rows
- Much slower queries (2.3x worse)
- Sequential inserts slower
- **DO NOT USE** until fixed

### For Customers

**Write-Heavy Workloads**:
- Still 3.6x faster inserts at 50M ✅
- If queries are infrequent, still viable
- But overall worse than SQLite for mixed workloads

**Mixed Workloads**:
- Not competitive beyond 10M
- Queries too slow
- Recommend staying under 10M

---

## Next Steps (Critical Path)

### Immediate (This Week)

1. **Profile query path at 50M**
   - Measure exact time in exponential search vs linear scan
   - Check node sizes and tree depth
   - Identify bottleneck

2. **Test node size hypothesis**
   - Implement node size logging
   - Compare 1M vs 10M vs 50M node statistics
   - Validate if nodes are 5x larger at 50M

3. **Measure retrain overhead**
   - Add instrumentation to retrain()
   - Count retrain calls at 50M
   - Measure cumulative retrain time

### Short-Term (1-2 Weeks)

**Option 1: Optimize Within-Node Search**
- Implement true binary search (handle gaps better)
- Expected: 2-5x query improvement
- Complexity: High (gaps make binary search tricky)

**Option 2: Limit Node Size**
- Force splits at smaller node sizes (e.g., 10K keys max)
- Expected: Faster queries, more memory usage
- Complexity: Medium

**Option 3: Reduce Retrain Frequency**
- Only retrain when model error exceeds threshold
- Expected: Faster inserts, slightly slower queries
- Complexity: Low

**Option 4: Hybrid Approach**
- Pack keys after retrain (remove gaps for queries)
- Keep gaps for inserts
- Complexity: High

### Long-Term (1-2 Months)

1. **Multi-level ALEX tree**
   - Inner nodes for routing
   - Leaf nodes stay small
   - Better scaling to 100M+

2. **Cache-aware data layout**
   - Pack hot keys together
   - Better cache locality

3. **Adaptive node sizing**
   - Smaller nodes for large datasets
   - Dynamic splitting thresholds

---

## Comparison to Projections

**Original Projection** (from STATUS_REPORT):
- 10M: 5-15x faster
- 50M: 5-15x faster (assumed linear scaling)

**Actual**:
- 10M: 2.11x faster ✅
- 50M: 1.32x faster ⚠️

**Status**: Projections invalidated beyond 10M scale.

---

## Honest Assessment

### What We Learned

1. **ALEX doesn't scale linearly to 50M+** - Node size growth dominates
2. **Auto-retrain has limits** - Helps at 10M, hurts at 50M
3. **Linear scan is the bottleneck** - Need better within-node search
4. **SQLite scales better for queries** - B-tree depth grows slower

### What This Means

**For Seed Fundraising**:
- Can claim "2-3x faster at 1M-10M scale" ✅
- Cannot claim "scales to 50M+" ❌
- Must acknowledge "query optimization needed beyond 10M"

**For Production**:
- Safe for datasets under 10M rows
- Risky for 10M-50M rows
- Not recommended for 50M+ rows (until fixed)

**For Technical Roadmap**:
- Critical: Fix query performance at scale
- Priority: Implement multi-level ALEX or better within-node search
- Timeline: 2-4 weeks minimum

---

## Recommendations

### Ship or Fix?

**Option 1: Ship as-is (1M-10M niche)**
- Target: Small-medium datasets only
- Positioning: "2-3x faster for 1M-10M row datasets"
- Risk: Limited TAM, customers may hit limits

**Option 2: Fix before shipping (recommended)**
- Target: Fix query performance at 50M+
- Timeline: 2-4 weeks
- Outcome: "2-3x faster at all scales" (validated)
- Risk: Time investment, may not succeed

**Recommendation**: Fix before major customers.
- 50M is a realistic production size
- Can't claim "production-ready" with this regression
- Better to fix now than face customer complaints later

### Immediate Action

1. Profile query path (find exact bottleneck)
2. Implement fastest fix (likely node size limit)
3. Re-test at 50M
4. If successful, continue to 100M
5. If not, consider fundamental architecture changes

---

## Test Data

**Command Run**:
```bash
./target/release/benchmark_table_vs_sqlite 50000000
```

**Results**:
- Sequential: 0.63x overall (SLOWER)
- Random: 2.02x overall
- Average: 1.32x overall

**Time**: ~12 minutes total
- SQLite: ~6 minutes
- OmenDB: ~2.5 minutes inserts, but queries offset gains

---

**Last Updated**: January 2025
**Status**: Critical regression found, investigation in progress
**Next Action**: Profile query path at 50M scale to find exact bottleneck
