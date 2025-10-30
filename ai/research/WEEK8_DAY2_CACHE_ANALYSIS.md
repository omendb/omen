# Week 8 Day 2: Cache Optimization Analysis

**Date**: October 30, 2025
**Goal**: Implement cache optimization for 15-25% improvement
**Status**: Analysis complete - Strategy pivot required

---

## Executive Summary

**Critical Finding**: Cache optimization blocked by `hnsw_rs` library limitations

**Problem**:
- 23.41% LLC cache misses occur in `hnsw_rs` HNSW graph traversal
- We don't control `hnsw_rs` internal memory layout
- HNSW nodes are allocated/managed by library (opaque implementation)

**Solution**: Two-track approach
1. **Week 8**: Implement allocation reduction (feasible now, 10-20% improvement)
2. **Weeks 9-10**: Build custom HNSW for full cache control (15-25% additional)

**Revised Week 8 Target**: 581 QPS → 697 QPS via allocation reduction (not cache)

---

## Analysis

### Where Cache Misses Occur

**From Profiling**:
- 23.41% LLC misses (last-level cache)
- 11.22% L1 cache misses
- 54-69% backend bound (CPU waiting on memory)

**Root Cause**:
```
HNSW graph traversal (hnsw_rs internals)
  └─> Random node access (pointer chasing)
      └─> Nodes scattered in memory (library-allocated)
          └─> Cache misses (we can't control layout)
```

### What We Control vs Library

| Component | Owner | Can Optimize? |
|-----------|-------|---------------|
| HNSW graph nodes | `hnsw_rs` library | ❌ No - Opaque |
| HNSW graph edges | `hnsw_rs` library | ❌ No - Opaque |
| HNSW traversal order | `hnsw_rs` library | ❌ No - Library logic |
| Vector storage (`Vec<Vector>`) | OmenDB | ✅ Yes - Minor impact |
| Query allocations | OmenDB | ✅ Yes - **High impact** |
| Distance calculations | `hnsw_rs` (SIMD) | ✅ Already optimized |

### Why Cache Optimization Needs Custom HNSW

**Cache-friendly HNSW requires**:
1. Control over node allocation (contiguous memory)
2. Control over node layout (BFS/DFS ordering)
3. Control over edge storage (packed arrays vs pointers)
4. Prefetching hints during traversal
5. Hot/cold data separation

**All blocked by `hnsw_rs` library** - Can only be done with custom implementation.

---

## Revised Week 8 Strategy

### What We CAN Optimize (This Week)

**Priority 1: Allocation Reduction** (10-20% improvement)
- ✅ Feasible with current hnsw_rs
- ✅ High impact (7.3M allocations identified)
- ✅ Low risk (straightforward optimization)
- ✅ Immediate results

**Tasks**:
1. Thread-local buffer pools for query distances
2. Reuse candidate list buffers
3. Pre-allocate result vectors
4. Arena allocator for temporary data

**Expected**: 581 QPS → 697 QPS (20% improvement, beat Qdrant's 626!)

---

### What We CANNOT Optimize (Blocked)

**Cache Optimization** (15-25% improvement)
- ❌ Blocked by hnsw_rs internal layout
- ❌ Requires custom HNSW implementation
- ❌ 10-15 week effort (Weeks 9-22)

**Why defer**:
- Can't modify hnsw_rs internals
- Forking library is high risk
- Custom HNSW already planned (decision made)

---

## Updated Performance Projections

### Revised Week 8 Timeline

| Stage | QPS | Improvement | vs Qdrant (626 QPS) | Feasible? |
|-------|-----|-------------|---------------------|-----------|
| Current (SIMD) | 581 | Baseline | 93% | ✅ Complete |
| + Allocation reduction | 697 | +20% | **111% of Qdrant** ⭐ | ✅ Week 8 |
| + Custom HNSW core | 850 | +46% | **136% of Qdrant** ⭐ | ✅ Weeks 9-10 |
| + SOTA features | 1000+ | +72% | **160% of Qdrant** ⭐ | ✅ Weeks 11-12 |

**Week 8 Target** (revised): 697 QPS via allocation reduction
**Weeks 9-10**: Custom HNSW → 850 QPS via cache optimization
**Weeks 11-12**: Extended RaBitQ + HNSW-IF → 1000+ QPS

---

## Decision: Implement Allocation Reduction First

**Rationale**:
1. **Immediate wins**: 10-20% improvement this week
2. **Beat Qdrant**: 697 QPS > 626 QPS (11% faster)
3. **De-risk**: Validate optimization approach before custom HNSW
4. **Feasible**: No library limitations, full control
5. **Foundation**: Buffer pooling useful for custom HNSW too

**Alternative rejected**: Fork hnsw_rs (high risk, maintenance burden)

---

## Next Steps

**Week 8 Day 2-3** (Allocation Reduction):
1. Profile allocation hotspots (detailed heaptrack analysis)
2. Implement thread-local buffer pools
3. Add query buffer reuse
4. Benchmark improvements
5. **Target**: 697 QPS (beat Qdrant)

**Weeks 9-10** (Custom HNSW):
1. Implement basic HNSW core
2. Cache-friendly node layout
3. Prefetching hints
4. **Target**: 850 QPS (cache optimization unlocked)

**Weeks 11-12** (SOTA Features):
1. Extended RaBitQ integration
2. HNSW-IF for billion-scale
3. **Target**: 1000+ QPS

---

## Lessons Learned

**Library limitations**:
- Can't optimize what you don't control
- Performance-critical code needs full ownership
- Confirms custom HNSW decision was correct

**Optimization priorities**:
- Do what's feasible first (allocation reduction)
- Build foundation for next phase (custom HNSW)
- Incremental wins > blocked on perfect solution

---

**Status**: Analysis complete, pivoting to allocation reduction (Priority 2 → Priority 1)
**Next**: Implement buffer pooling and allocation reduction (Week 8 Day 2-3)
**Target**: 697 QPS (beat Qdrant's 626 QPS by 11%)
