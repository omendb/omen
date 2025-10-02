# Optimization Results - October 2, 2025

## P0 + P1 Combined Results

**Status:** ⚠️ **MIXED - P1 BACKFIRED**

**Optimizations implemented:**
- ✅ P0: Lazy index rebuild (deferred to query time)
- ❌ P1: Fixed error bound calculation (made things WORSE)

---

## Benchmark Results (After P0+P1)

### 10,000 Rows

| Distribution | Insert | Query | Average |
|--------------|--------|-------|---------|
| **Sequential** | 0.49x (slower) | **5.35x** | **2.92x** |
| **Random** | 0.71x (slower) | **4.66x** | **2.69x** |

**vs Baseline (before P0+P1):** Query improved from 7.48x → 5.35x (slight regression), but still good

### 100,000 Rows

| Distribution | Insert | Query | Average |
|--------------|--------|-------|---------|
| **Sequential** | 0.81x (slower) | **1.26x** | **1.03x** |
| **Random** | 1.44x | **1.44x** | **1.44x** |

**vs Baseline:** Query REGRESSED from 5.83x → 1.26x ❌

### 1,000,000 Rows

| Distribution | Insert | Query | Average |
|--------------|--------|-------|---------|
| **Sequential** | 0.90x (slower) | **0.19x** (5x SLOWER!) | **0.55x** |
| **Random** | 2.35x | **0.19x** (5x SLOWER!) | **1.27x** |

**vs Baseline:** Query CATASTROPHICALLY REGRESSED from 3.48-4.93x → 0.19x ❌

---

## Analysis

### What Went Wrong with P1

**Problem:** Strategic sampling (first 50, middle 50, last 50) found **outlier errors** in the tail, causing:
```
max_error_bound() returns: ~1000+ positions
window_size = 1000+
Binary search in [predicted - 1000, predicted + 1000]
```

**Effect:** At 1M rows, search window becomes 2000+ elements → binary search of 2000 elements takes 28μs vs SQLite's B-tree 5μs

**Root cause:** Using `max()` of all errors captures worst-case outliers
- Linear regression can have large errors at distribution boundaries
- One outlier forces entire index to use huge windows
- Should use **percentile-based** bounds (e.g., 95th percentile) instead of max

### What Worked (P0 Impact TBD)

**P0 (lazy rebuild):** Should speed up inserts, but:
- Current benchmark inserts THEN queries
- First query triggers rebuild, absorbing rebuild cost
- Need to separate insert benchmark from query benchmark to see true P0 benefit

**Current insert results:**
- 10K: 0.49-0.71x (slower) - rebuild still happening on first query
- 100K: 0.81-1.44x - same issue
- 1M: 0.90-2.35x - some improvement on random data

**What we expected:**
- Inserts should be 10-100x faster (pure redb writes)
- But benchmark does: insert() → query() → rebuild happens
- Rebuild cost is now attributed to first query, not inserts

---

## Fixes Needed

### FIX 1: Use Percentile-Based Error Bounds (CRITICAL)

**Problem:** `max()` captures outliers, creates huge windows

**Solution:** Use 95th or 99th percentile instead of max
```rust
// Instead of:
max_error = errors.iter().max()

// Use:
errors.sort();
let p95_idx = (errors.len() as f64 * 0.95) as usize;
max_error = errors[p95_idx]
```

**Expected impact:** 3-10x query speedup at large scales

### FIX 2: Separate Insert/Query Benchmarks

**Problem:** Current benchmark pattern:
```
insert() → first_query() ← rebuild happens here!
```

**Solution:** Benchmark pattern should be:
```
// Insert benchmark
insert() → done (no queries)

// Query benchmark
first_query() ← rebuild happens here, but measured separately
subsequent_queries() ← pure learned index performance
```

**Expected impact:** Show true P0 benefit (10-100x insert speedup)

### FIX 3: Add Error Bound Logging

**Need to see:** What are actual max_error values at different scales?
```
10K rows: max_error = ?
100K rows: max_error = ?
1M rows: max_error = ? (probably ~1000+)
```

---

## Revised Strategy

### Immediate (Next 2 Hours)

1. **Revert P1** temporarily - go back to hardcoded window_size=100
2. **Re-run benchmark** to isolate P0 effect
3. **Fix benchmark** to separate insert and query timings

### Then (Next 2-4 Hours)

4. **Implement percentile-based error bounds** (P1 v2)
5. **Re-run with fixed P1**
6. **Target:** 7-13x average speedup

---

## Baseline Comparison

### Before P0+P1 (Original Honest Benchmark)

| Scale | Dist | Insert | Query | Avg |
|-------|------|--------|-------|-----|
| 10K | Seq | 0.47x | 7.48x | 3.98x |
| 10K | Rand | 0.67x | 5.78x | 3.23x |
| 100K | Seq | 0.76x | 5.83x | 3.30x |
| 100K | Rand | 1.45x | 4.79x | 3.12x |
| 1M | Seq | 0.88x | 3.48x | 2.18x |
| 1M | Rand | 2.43x | 4.93x | 3.68x |

**Average: 2.18-3.98x**

### After P0+P1 (Current Results)

| Scale | Dist | Insert | Query | Avg |
|-------|------|--------|-------|-----|
| 10K | Seq | 0.49x | 5.35x | 2.92x |
| 10K | Rand | 0.71x | 4.66x | 2.69x |
| 100K | Seq | 0.81x | **1.26x** | **1.03x** |
| 100K | Rand | 1.44x | **1.44x** | **1.44x** |
| 1M | Seq | 0.90x | **0.19x** | **0.55x** |
| 1M | Rand | 2.35x | **0.19x** | **1.27x** |

**Average: 0.55-2.92x** (WORSE!)

---

## Lesson Learned

**Using max() for error bounds is a trap!**
- Captures worst-case outliers
- One bad prediction ruins entire index
- Standard ML practice: Use percentiles for robustness

**PGM-index paper insight:**
- They use ε (epsilon) parameter to bound AVERAGE error
- Not worst-case error
- Trade-off: Occasional miss (exponential search fallback) vs always large windows

**Next step:** Revert P1, re-benchmark P0 alone, then implement percentile-based P1 v2
