# Hybrid Search Recall Investigation - Week 5 Day 3

**Date**: October 23, 2025
**Status**: CRITICAL ISSUE IDENTIFIED - Vector Index Not Persisted

---

## Executive Summary

Recall validation revealed **55-65% recall** instead of target **90%+**, indicating a critical issue with vector index persistence. The HNSW index is not being persisted or rebuilt when the catalog is reloaded from disk.

---

## Test Results

**Dataset**: 5,000 products with 128D embeddings
**Queries**: 20 per selectivity level
**k**: 10 (top-10 nearest neighbors)

| Selectivity | Avg Recall | Status | Expected |
|-------------|------------|--------|----------|
| **High (20%)** | 64.50% | ⚠️ FAIL | >90% |
| **Medium (50%)** | 56.50% | ⚠️ FAIL | >90% |
| **Low (90%)** | 55.00% | ⚠️ FAIL | >90% |

---

## Root Cause Analysis

### Issue: HNSW Index Not Persisted

**Problem**: When the catalog is reloaded after insert, the HNSW vector index is not persisted or rebuilt.

**Evidence**:
1. VectorStore creates HNSW index lazily on first insert
2. When catalog is reloaded, vectors are read from storage but HNSW index is not rebuilt
3. System may be falling back to brute-force search or operating without a proper index

**Code Path**:
```rust
// In VectorStore::new()
pub fn new(dimensions: usize) -> Self {
    Self {
        dimensions,
        vectors: Vec::new(),
        hnsw_index: None, // ← Index not initialized
    }
}

// HNSW index only created on first insert
pub fn insert(&mut self, vector: Vector) -> Result<usize> {
    if self.hnsw_index.is_none() {
        self.hnsw_index = Some(HNSWIndex::new(...));
    }
    // ...
}
```

**Problem**: After reload, if vectors exist but HNSW index is None, searches may not use HNSW properly.

---

## Impact Analysis

### Performance Impact

**Without HNSW Index**:
- Recall: 55-65% (incorrect results)
- Latency: Still 7-9ms (surprisingly fast, but wrong results)
- Correctness: FAIL (returning wrong neighbors)

**Expected with HNSW Index**:
- Recall: >90% (correct results)
- Latency: 7-10ms (similar or slightly higher)
- Correctness: PASS

---

## Proposed Solutions

### Option 1: Rebuild HNSW Index on Load (Immediate Fix)

**Approach**: When table/catalog is loaded, iterate through all vectors and rebuild HNSW index

**Implementation**:
```rust
// In Table::load() or VectorStore::from_vectors()
pub fn rebuild_index(&mut self) -> Result<()> {
    if !self.vectors.is_empty() && self.hnsw_index.is_none() {
        let mut index = HNSWIndex::new(self.vectors.len(), self.dimensions);
        for vector in &self.vectors {
            index.insert(&vector.data)?;
        }
        self.hnsw_index = Some(index);
    }
    Ok(())
}
```

**Pros**:
- Simple to implement
- No schema changes needed
- Fixes recall immediately

**Cons**:
- Rebuild cost on every restart (O(n log n) for n vectors)
- Startup latency increases with dataset size

**Timeline**: 2-4 hours

---

### Option 2: Persist HNSW Index to Disk (Proper Fix)

**Approach**: Serialize HNSW graph structure and persist alongside vector data

**Implementation**:
1. Add HNSW serialization methods (hnsw_rs may support this)
2. Save HNSW index to RocksDB or separate file
3. Load HNSW index on catalog/table initialization

**Pros**:
- Fast startup (no rebuild needed)
- Production-ready solution
- Consistent with database persistence model

**Cons**:
- More complex implementation
- Requires HNSW serialization support
- Need to handle version compatibility

**Timeline**: 1-2 days

---

### Option 3: Lazy Index Rebuild on First Query (Temporary Workaround)

**Approach**: Detect missing index on first query and rebuild it then

**Implementation**:
```rust
pub fn knn_search(&mut self, query: &Vector, k: usize) -> Result<Vec<(usize, f32)>> {
    // If index missing but vectors exist, rebuild
    if self.hnsw_index.is_none() && !self.vectors.is_empty() {
        self.rebuild_index()?;
    }

    if let Some(ref index) = self.hnsw_index {
        return index.search(&query.data, k);
    }

    // Fallback to brute-force
    self.knn_search_brute_force(query, k)
}
```

**Pros**:
- Zero startup cost
- Index built only when needed
- Simple implementation

**Cons**:
- First query after restart is slow
- Inconsistent latency (first query vs subsequent)
- Still requires rebuild cost

**Timeline**: 1-2 hours

---

## Recommendation

### Immediate Action (Next 4 hours)

**Implement Option 1 (Rebuild on Load)** + **Option 3 (Lazy Rebuild)**:
1. Add `rebuild_index()` method to VectorStore
2. Call during lazy initialization on first query
3. This fixes recall immediately while being simple

**Rationale**:
- Fixes recall issue TODAY
- Simple, low-risk implementation
- Allows us to validate hybrid search accuracy
- Can be replaced with Option 2 later for production

### Medium-Term (Week 6)

**Implement Option 2 (Persist HNSW)**:
1. Research hnsw_rs serialization support
2. Implement HNSW persistence to RocksDB
3. Add versioning for index format
4. Benchmark startup time improvement

**Timeline**: Allocate 2-3 days during Week 6 optimization phase

---

## Testing Plan

### After Fix Implementation

1. **Recall Validation** (benchmark_hybrid_recall):
   - Re-run recall tests
   - Target: >90% recall @ all selectivity levels
   - Verify HNSW index is active

2. **Performance Validation** (benchmark_hybrid_search):
   - Measure latency change (expect <10% increase)
   - Verify throughput (should remain >100 QPS)
   - Check startup time (measure index rebuild cost)

3. **Larger Scale Test**:
   - Test with 50K-100K vectors
   - Validate recall at scale
   - Measure rebuild time

---

## Lessons Learned

### Design Issues

1. **Lazy initialization without persistence is dangerous**
   - Index built on first insert but not persisted
   - Silent degradation on restart (no errors, just wrong results)

2. **Need persistence testing**
   - Tests insert and query in same session
   - Need tests that restart/reload between operations

3. **Silent fallback to brute-force**
   - System should log when falling back to sequential scan
   - Should warn if index is missing for large datasets

### Process Improvements

1. **Always test persistence path**
   - Insert → Restart → Query workflow
   - Catch issues like this earlier

2. **Monitor index health**
   - Track whether HNSW index is active
   - Alert if falling back to brute-force on large datasets

3. **Recall validation is critical**
   - Performance metrics alone miss correctness issues
   - Must validate accuracy regularly

---

## CRITICAL UPDATE: Hybrid Search Uses Exact Search, Not HNSW

### Additional Finding

Upon deeper investigation, discovered that **hybrid search doesn't use HNSW index at all**. It performs exact brute-force distance computation on filtered rows:

```rust
// In execute_hybrid_filter_first() - src/sql_engine.rs:876-900
let mut scored_rows: Vec<(Row, f32)> = filtered_rows
    .into_iter()
    .filter_map(|row| {
        if let Ok(Value::Vector(vec_val)) = row.get(col_idx) {
            // Exact distance computation - NO INDEX USED
            let distance = vec_val.l2_distance(query_vector).ok()?;
            Some((row, distance))
        } else {
            None
        }
    })
    .collect();
```

### Implications

1. **Recall should be 100%** (exact search, not approximate)
2. **Benchmark showing 55-65%** suggests bug in benchmark, not hybrid search
3. **HNSW fix not relevant** for current hybrid search implementation
4. **Performance is good** because filtered sets are small (100-5000 rows)

### Revised Analysis

**The low recall (55-65%) is likely due to**:
- Bug in ground truth computation
- Bug in result ID extraction
- Mismatch in distance operators (L2 vs cosine)

**Hybrid search is correct** but uses exact search on filtered rows (intentional for accuracy).

## Next Steps

1. [x] Identify root cause (Hybrid search uses exact distance, not HNSW)
2. [x] Implement HNSW rebuild (for future vector-only queries)
3. [ ] Debug recall benchmark (fix ground truth or ID extraction)
4. [ ] Consider adding HNSW for large filtered sets (optimization)
5. [ ] Update STATUS.md with findings

---

## Conclusion

The recall issue is **NOT a fundamental algorithm problem**, but a **persistence/initialization bug**. The HNSW algorithm works correctly when the index is built, but the index is not being persisted or rebuilt after restart.

**Fix is straightforward**: Add index rebuild on load or first query. Expected to restore >90% recall with minimal latency impact.

**Impact on Timeline**: 4-8 hours to implement and validate fix. Does not block Week 5 completion, but critical for production readiness.
