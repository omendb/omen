# Profiling Analysis - Week 8 Day 1

**Date**: October 30, 2025
**Workload**: 10,000 vectors (1536D), 100 queries
**Hardware**: Fedora i9-13900KF (24-core), 32GB RAM
**Configuration**: SIMD enabled (hnsw-simd feature)

---

## Executive Summary

**Critical Bottlenecks Identified**:
1. **Memory-bound execution**: 54-69% backend_bound (CPU waiting on memory)
2. **High cache misses**: 23.41% LLC misses (poor memory locality)
3. **Excessive allocations**: 7.3M allocations for 10K benchmark

**Top 3 Optimizations** (Expected 20-40% improvement):
1. **Cache optimization** (memory layout, prefetching) - 15-25% improvement
2. **Allocation reduction** (object pooling, arena allocators) - 10-20% improvement
3. **Memory access patterns** (sequential batching) - 5-10% improvement

**Target**: 581 QPS → 700-815 QPS (exceed Qdrant's 626 QPS)

---

## Profiling Data

### Performance Counters (perf stat)

| Metric | Value | Analysis |
|--------|-------|----------|
| **Backend Bound** | 54.1-69.4% | ⚠️ **CRITICAL** - CPU stalled on memory/execution |
| **L1 Cache Misses** | 11.22% | ⚠️ Moderate - Room for improvement |
| **LLC Cache Misses** | 23.41% | ⚠️⚠️ **VERY HIGH** - Poor memory locality |
| **Branch Misses** | 0.51-0.55% | ✅ Excellent - Not a bottleneck |
| **IPC** | 0.82-0.92 | ⚠️ Low - Memory stalls dominate |
| **Context Switches** | 8,652 | ✅ Low - No contention |

### Memory Allocations (heaptrack)

| Metric | Value | Analysis |
|--------|-------|----------|
| **Total Allocations** | 7,325,297 | ⚠️ Very high for 10K vectors |
| **Temporary Allocations** | 53,966 | ⚠️ Opportunities for pooling |
| **Leaked Allocations** | 663 | ⚠️ Minor issue, should fix |
| **Peak Memory** | Unknown | Need detailed heaptrack analysis |

### Query Performance (10K vectors)

| Metric | Value | vs 100K Baseline |
|--------|-------|------------------|
| **Query avg** | 1.05ms | 1.72ms (smaller dataset faster) |
| **Query p95** | 1.21ms | 2.08ms |
| **Query p99** | 1.26ms | 2.26ms |
| **Build speed** | 24,050 vec/sec | 6,540 vec/sec |

---

## Bottleneck Analysis

### 1. Memory-Bound Execution (54-69% Backend Bound) ⚠️⚠️ CRITICAL

**What it means**:
- CPU is idle waiting for memory operations
- Memory bandwidth or latency is the bottleneck
- Not compute-bound (SIMD is working, but memory is limiting)

**Root causes**:
- Poor cache utilization (23.41% LLC misses confirms this)
- Random memory access patterns in HNSW graph traversal
- Pointer chasing (navigating graph edges)

**Solutions**:
1. **Improve memory layout**: Store HNSW graph nodes contiguously
2. **Prefetching**: Add hints for upcoming node accesses
3. **Batch processing**: Process multiple queries together to improve locality

**Expected impact**: 15-25% improvement

---

### 2. High Cache Misses (23.41% LLC) ⚠️⚠️ CRITICAL

**What it means**:
- 23.41% of last-level cache accesses miss → go to RAM (100x slower)
- Poor spatial/temporal locality in data access

**Root causes**:
- HNSW graph structure: Nodes scattered in memory
- Random access during graph traversal
- Large vectors (1536D = 6KB per vector) thrashing cache

**Solutions**:
1. **Memory layout optimization**:
   - Store graph nodes in traversal order (BFS/DFS layout)
   - Align data structures to cache line boundaries (64 bytes)
   - Separate hot/cold data (frequently accessed vs rarely accessed)

2. **Prefetching**:
   ```rust
   // Prefetch next layer nodes during traversal
   std::intrinsics::prefetch_read_data(next_node_ptr, 3);
   ```

3. **Data structure compaction**:
   - Pack HNSW node data (reduce padding)
   - Use indices instead of pointers where possible

**Expected impact**: 15-20% improvement

---

### 3. Excessive Allocations (7.3M) ⚠️ HIGH

**What it means**:
- 7.3M allocations for 10K vectors + 100 queries
- Allocations cause memory fragmentation and overhead
- Temporary allocations (54K) indicate repeated alloc/dealloc

**Root causes**:
- Per-query buffer allocations (distance arrays, candidate lists)
- Temporary vectors during distance calculations
- HNSW traversal data structures allocated per-query

**Solutions**:
1. **Object pooling**:
   ```rust
   // Reuse query buffers across requests
   thread_local! {
       static QUERY_BUFFER: RefCell<Vec<f32>> = RefCell::new(Vec::with_capacity(1536));
       static CANDIDATE_BUFFER: RefCell<Vec<usize>> = RefCell::new(Vec::with_capacity(500));
   }
   ```

2. **Arena allocators**:
   - Allocate large blocks, sub-allocate from arena
   - Reset arena after query completion (no individual frees)

3. **Pre-allocation**:
   - Allocate buffers once during VectorStore initialization
   - Reuse across all queries

**Expected impact**: 10-20% improvement

---

## Optimization Priority

### Priority 1: Cache Optimization (Highest Impact)

**Tasks**:
1. Profile memory access patterns with `perf record -e cache-misses`
2. Analyze HNSW graph layout (are nodes contiguous?)
3. Implement cache-friendly memory layout
4. Add prefetching hints for graph traversal
5. Align data structures to cache lines

**Expected improvement**: 15-25%
**Effort**: 3-5 days
**Risk**: Medium (requires careful testing for correctness)

---

### Priority 2: Allocation Reduction (High Impact)

**Tasks**:
1. Identify hot allocation paths (use heaptrack detailed analysis)
2. Implement thread-local buffer pools for queries
3. Replace per-query allocations with reused buffers
4. Consider arena allocator for HNSW construction

**Expected improvement**: 10-20%
**Effort**: 2-3 days
**Risk**: Low (straightforward optimization)

---

### Priority 3: Memory Access Patterns (Medium Impact)

**Tasks**:
1. Analyze query batching opportunities
2. Implement batch query processing (process N queries together)
3. Optimize HNSW traversal order for better locality

**Expected improvement**: 5-10%
**Effort**: 2-3 days
**Risk**: Low

---

## Performance Projections

### Current Performance (with SIMD)
- **QPS**: 581 (from 100K benchmark: 1.72ms avg)
- **Query p95**: 2.08ms
- **Build**: 6,540 vec/sec

### After Cache Optimization (+20%)
- **QPS**: 697
- **Query avg**: 1.43ms
- **Query p95**: 1.73ms

### After Allocation Reduction (+15%)
- **QPS**: 802
- **Query avg**: 1.24ms
- **Query p95**: 1.50ms

### After Memory Access (+8%)
- **QPS**: 866
- **Query avg**: 1.15ms
- **Query p95**: 1.39ms

### Cumulative Improvement
**Target**: 581 QPS → 866 QPS (1.49x improvement, 49% faster)

**vs Qdrant**: 866 QPS vs 626 QPS = **38% faster than Qdrant!** ⭐

---

## Next Steps (Week 8 Day 2-4)

**Day 2** (Cache Optimization):
1. Profile cache-misses in detail
2. Analyze HNSW graph memory layout
3. Implement cache-friendly node storage
4. Add prefetching hints
5. Benchmark improvement

**Day 3** (Allocation Reduction):
1. Detailed heaptrack analysis (identify hot paths)
2. Implement thread-local buffer pools
3. Replace hot allocations with pooled buffers
4. Benchmark improvement

**Day 4** (Memory Access + Integration):
1. Implement query batching
2. Optimize traversal order
3. Integration testing
4. Full 100K benchmark comparison

---

## Success Criteria

| Metric | Current | Target | Stretch |
|--------|---------|--------|---------|
| **QPS** | 581 | 700+ (beat Qdrant) | 850+ |
| **Query p95** | 2.08ms | 1.5ms | 1.2ms |
| **Cache misses** | 23.41% | <15% | <10% |
| **Allocations** | 7.3M | <5M | <3M |

---

## Files Generated

- `flamegraph_queries.svg` - CPU hotspots visualization (153KB)
- `perf_stat_output.txt` - Performance counters (3.9KB)
- `heaptrack.benchmark_pgvector_comparison.70920.zst` - Memory allocations (19MB)

---

**Status**: Profiling complete, optimization priorities identified
**Next**: Implement cache optimization (Priority 1)
