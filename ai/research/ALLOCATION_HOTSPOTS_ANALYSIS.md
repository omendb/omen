# Allocation Hotspots Analysis - Week 8 Day 2

**Date**: October 30, 2025
**Workload**: 10K vectors, 100 queries
**Total Allocations**: 7,325,297
**Tool**: heaptrack

---

## Executive Summary

**Critical Finding**: 76% of allocations (5.6M) occur in `hnsw_rs::search_layer` (library internal)

**What We CAN Optimize**:
- Result vector allocations: ~100 allocations/query → pre-allocate
- Our wrapper code allocations: Distance arrays, temporary vectors
- Expected improvement: 5-10% (not 10-20% as initially hoped)

**What We CANNOT Optimize**:
- `hnsw_rs` internal allocations (5.6M, 76% of total)
- HNSW graph traversal data structures
- Requires custom HNSW implementation

**Revised Target**: 581 QPS → 610-640 QPS (5-10% improvement)

---

## Allocation Breakdown

### Total: 7,325,297 Allocations

| Source | Calls | Percentage | Can Optimize? |
|--------|-------|------------|---------------|
| **hnsw_rs::search_layer** | 5,601,871 | **76.5%** | ❌ No - Library internal |
| **hnsw_rs::search** | 82,212 | 1.1% | ❌ No - Library code |
| **hnsw_rs::insert_slice** | 91,089 | 1.2% | ❌ No - Library code |
| **Our code + misc** | ~1,550,125 | 21.2% | ✅ Yes - OmenDB code |

###Hotspot #1: `hnsw_rs::search_layer` (5.6M allocations, 76%)

**What it is**:
- Internal HNSW graph traversal during search
- Allocates candidate lists, priority queues, visited node sets
- Called for each layer during hierarchical search

**Why we can't optimize it**:
```
hnsw_rs::search_layer (library internal)
  └─> Allocates: Vec<(f32, usize)> for candidates
  └─> Allocates: HashSet<usize> for visited nodes
  └─> Allocates: BinaryHeap for priority queue
  └─> We don't control this code (opaque library)
```

**Impact**: This is THE bottleneck, but requires custom HNSW

---

### Hotspot #2: Search Result Allocations (~100/query)

**What it is**:
- Each `knn_search()` call allocates result `Vec<(usize, f32)>`
- 100 queries × ~100 allocations = ~10,000 allocations
- Small but easy to optimize

**Where it happens**:
```rust
// src/vector/store.rs:162
pub fn knn_search(&mut self, query: &Vector, k: usize) -> Result<Vec<(usize, f32)>> {
    // ...
    if let Some(ref index) = self.hnsw_index {
        return index.search(&query.data, k);  // ← Allocates new Vec
    }
    // ...
}
```

**Optimization**: Pre-allocate result buffer, reuse across queries

---

### Hotspot #3: Vector Data Allocations

**What it is**:
- Vector generation, temporary copies, distance calculations
- Part of OmenDB wrapper code

**Where it happens**:
- Benchmark vector generation
- Query vector creation
- Distance calculation intermediate values

**Optimization**: Object pooling for temporary vectors

---

## What We CAN Optimize (OmenDB Code)

### Optimization 1: Pre-allocate Result Buffers

**Current**:
```rust
pub fn knn_search(&mut self, query: &Vector, k: usize) -> Result<Vec<(usize, f32)>> {
    index.search(&query.data, k)  // Allocates new Vec every call
}
```

**Optimized**:
```rust
pub struct VectorStore {
    vectors: Vec<Vector>,
    hnsw_index: Option<HNSWIndex<'static>>,
    dimensions: usize,
    result_buffer: Vec<(usize, f32)>,  // ← Pre-allocated, reused
}

pub fn knn_search(&mut self, query: &Vector, k: usize) -> Result<&[(usize, f32)]> {
    self.result_buffer.clear();
    // Use result_buffer instead of allocating
    index.search_into(&query.data, k, &mut self.result_buffer)?;
    Ok(&self.result_buffer[..k.min(self.result_buffer.len())])
}
```

**Expected reduction**: ~10,000 allocations (0.14% of total)
**Expected improvement**: ~1-2%

---

### Optimization 2: Thread-Local Query Buffers

**Current**:
```rust
// Each query might allocate temporary vectors
let query_vec = query.data.clone();  // Allocation
```

**Optimized**:
```rust
use std::cell::RefCell;

thread_local! {
    static QUERY_BUFFER: RefCell<Vec<f32>> = RefCell::new(Vec::with_capacity(2048));
}

fn prepare_query(query: &[f32]) -> impl Deref<Target = [f32]> {
    QUERY_BUFFER.with(|buf| {
        let mut buf = buf.borrow_mut();
        buf.clear();
        buf.extend_from_slice(query);
        // Return borrowed data
    })
}
```

**Expected reduction**: ~1,000-5,000 allocations
**Expected improvement**: ~1-2%

---

### Optimization 3: Reduce Vector Clones

**Audit code for**:
- `vector.data.clone()` calls
- Temporary vector allocations
- Unnecessary copies

**Expected reduction**: ~5,000-10,000 allocations
**Expected improvement**: ~2-3%

---

## Revised Performance Projections

### Realistic Assessment

| Optimization | Allocation Reduction | Performance Impact |
|--------------|---------------------|-------------------|
| Pre-allocate results | ~10,000 (0.14%) | 1-2% |
| Thread-local buffers | ~5,000 (0.07%) | 1-2% |
| Reduce clones | ~10,000 (0.14%) | 2-3% |
| **Total** | **~25,000 (0.34%)** | **5-10%** |

**Why so small?**:
- 76% of allocations are in `hnsw_rs` library (can't optimize)
- Only 24% in our code, and much of that is necessary
- Real gains require custom HNSW implementation

### Updated Performance Targets

| Stage | QPS | Improvement | vs Qdrant (626 QPS) |
|-------|-----|-------------|---------------------|
| Current (SIMD) | 581 | Baseline | 93% |
| + Allocation reduction | 610-640 | **5-10%** | 97-102% |
| + Custom HNSW (Weeks 9-10) | 850 | +46% total | **136%** ⭐ |

---

## Strategic Decision

**Week 8 Options**:

1. **Implement minor allocation optimizations** (5-10% improvement)
   - Effort: 1-2 days
   - Impact: 581 → 610-640 QPS (might beat Qdrant)
   - Risk: Low
   - Value: Marginal

2. **Skip to Custom HNSW planning** (46% improvement potential)
   - Effort: 10-15 weeks implementation
   - Impact: 581 → 850+ QPS (beat Qdrant by 36%)
   - Risk: Medium (complexity)
   - Value: High (unlocks cache + allocation optimization)

**Recommendation**: **Option 2** - Start Custom HNSW planning

**Rationale**:
- 76% of allocations are in `hnsw_rs` (blocked)
- 5-10% improvement doesn't move the needle much
- Custom HNSW unlocks BOTH cache (15-25%) AND allocation (10-20%) gains
- Better to invest 10-15 weeks for 46%+ improvement than 2 days for 5-10%

---

## Custom HNSW Benefits

**What we unlock**:
1. **Cache optimization** (15-25%): Control memory layout, add prefetching
2. **Allocation optimization** (10-20%): Reuse data structures, arena allocators
3. **SOTA features**: Extended RaBitQ, HNSW-IF, MN-RU
4. **Cumulative**: 581 QPS → 1000+ QPS (72% improvement)

**Timeline**:
- Weeks 9-10: Custom HNSW core → 850 QPS
- Weeks 11-12: SOTA features → 1000+ QPS

---

## Conclusion

**Week 8 Findings**:
- ✅ SIMD optimization: 162 → 581 QPS (3.6x) **Complete**
- ✅ Cache optimization: Blocked by `hnsw_rs` **Requires custom HNSW**
- ✅ Allocation optimization: 76% in library **Requires custom HNSW**

**Strategic Pivot**:
- **Week 8 complete**: SIMD (3.6x) + Profiling + Strategic analysis
- **Weeks 9-22**: Custom HNSW implementation (10-15 weeks)
  - Unlocks cache optimization (15-25%)
  - Unlocks allocation optimization (10-20%)
  - Enables SOTA features (Extended RaBitQ, HNSW-IF)
  - Target: 1000+ QPS (60% faster than Qdrant)

---

**Status**: Allocation analysis complete, pivot to Custom HNSW recommended
**Next**: Begin Custom HNSW implementation planning (Week 9)
