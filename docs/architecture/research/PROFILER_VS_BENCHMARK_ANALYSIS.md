# Profiler vs Benchmark Discrepancy Analysis

**Date**: January 2025
**Issue**: Profiler shows 3μs queries at 50M, benchmark shows 17μs
**Gap**: 5.7x difference - need to understand why

---

## Results Comparison

### Profiler Results (50M Sequential Data)

**Configuration**:
- Direct ALEX tree access
- Sequential inserts (0, 1, 2, ..., 50M)
- Sequential queries (every 5000th key)
- Sample size: 10,000 queries

**Results**:
```
Total keys: 50,000,000
Num leaves: 16,666,545
Avg keys per leaf: 3.0
Query time: 3.01 μs
```

### Benchmark Results (50M Random UUID Data)

**Configuration**:
- Table wrapper (ALEX + catalog overhead)
- Random inserts (shuffled UUIDs)
- Random queries (1,000 samples)
- Persistent storage (Parquet files)

**Results**:
```
SQLite Query: 7.48 μs
OmenDB Query: 17.18 μs
Ratio: 0.44x (SLOWER)
```

### Key Differences

| Aspect | Profiler | Benchmark | Impact |
|--------|----------|-----------|--------|
| **Data Pattern** | Sequential | Random UUID | High |
| **Access Layer** | Direct ALEX | Table + Catalog | Medium |
| **Persistence** | In-memory | Parquet + WAL | Medium |
| **Query Pattern** | Sequential | Random | High |
| **Sample Size** | 10,000 | 1,000 | Low |

---

## Hypotheses for Discrepancy

### Hypothesis 1: Data Distribution

**Theory**: Random UUID inserts create fragmented tree structure vs sequential inserts.

**Evidence**:
- Sequential inserts: Keys in order, nodes fill left-to-right
- Random inserts: Keys scattered, nodes split unpredictably
- Fragmentation → worse cache locality → slower queries

**Test**: Run profiler with random data pattern

### Hypothesis 2: Table/Catalog Overhead

**Theory**: Table layer adds overhead vs direct ALEX access.

**Breakdown**:
```
Benchmark query path:
1. Catalog lookup (~100ns)
2. Table lock acquisition (~500ns)
3. ALEX tree lookup (~3μs)
4. Value deserialization (~?)
Total: ~3.6μs + overhead

Expected: ~4-5μs, not 17μs
```

**Conclusion**: This explains ~1-2μs, not 14μs difference.

### Hypothesis 3: Cache Effects

**Theory**: Sequential queries have better cache hit rate than random queries.

**Evidence**:
- Sequential: Queries nearby keys → same leaf often cached
- Random: Queries scattered keys → cache misses
- 50M rows → working set exceeds L3 cache

**Expected Impact**: 2-3x difference

### Hypothesis 4: Tree Structure Difference

**Theory**: Random inserts create different tree topology than sequential.

**Test Idea**: Compare tree statistics after random vs sequential insert

**Prediction**:
- Random inserts: More leaves, deeper tree
- Sequential inserts: Fewer leaves, balanced tree

### Hypothesis 5: Storage Layer Overhead

**Theory**: Benchmark reads from Parquet files, profiler is in-memory.

**Evidence**:
- Parquet read: Decompress column, lookup row
- In-memory: Direct array access
- But values are small (20 bytes), shouldn't be 14μs overhead

**Likelihood**: Low - values are cached after first read

---

## Next Steps

### Test 1: Profiler with Random Data

Modify profiler to:
1. Shuffle keys before insert
2. Query random samples (not sequential)
3. Compare results to sequential case

**Expected**: If hypothesis 1/3 are correct, should see ~10-15μs queries

### Test 2: Measure Catalog/Table Overhead

Create minimal benchmark:
1. Direct ALEX tree (current profiler)
2. ALEX + Table wrapper
3. ALEX + Table + Catalog
4. Full benchmark (Table + Catalog + Persistence)

**Expected**: Identify where 14μs overhead comes from

### Test 3: Compare Tree Statistics

After 50M insert:
- Sequential insert: Tree stats
- Random insert: Tree stats
- Compare num_leaves, keys_per_leaf

**Expected**: Random should have more fragmentation

---

## Preliminary Conclusions

**Primary Culprit** (90% confidence): Data distribution + cache effects
- Random data → fragmented tree → poor cache locality
- Sequential data → compact tree → good cache locality
- 5.7x difference is plausible

**Secondary Factors**:
- Table/Catalog overhead: ~1-2μs
- Storage overhead: ~1-2μs
- Total explained: ~3-4μs + 10-12μs (fragmentation/cache) = 13-16μs ✓

**Action**: Run profiler with random data to confirm hypothesis.

---

**Last Updated**: January 2025
**Status**: Hypothesis formed, tests pending
**Next**: Modify profiler for random data pattern
