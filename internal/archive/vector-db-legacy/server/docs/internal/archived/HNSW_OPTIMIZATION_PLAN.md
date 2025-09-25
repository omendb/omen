# HNSW Optimization Plan

**Date**: 2025-07-29  
**Priority**: CRITICAL - Primary bottleneck for production deployment  
**Target**: 50,000+ vec/s construction speed

## 🎯 Executive Summary

Current HNSW implementation achieves 100% recall but construction performance is 568x slower than brute force mode. This document outlines the optimization strategy to achieve competitive performance while maintaining accuracy.

## 📊 Current State Analysis

### Performance Bottleneck Breakdown
```
Brute Force Mode: 5,993 vec/s ✅
HNSW Migration:      11 vec/s ❌ (568x slower)
HNSW Construction:   93 vec/s ❌ (64x slower)
```

### Root Causes
1. **Unvectorized Distance Calculations**: O(M) sequential distance calculations per insertion
2. **Single-threaded Construction**: No parallelization of independent operations
3. **Dynamic Memory Allocation**: Frequent allocations during graph updates
4. **Blocking Migration**: Entire dataset migrated in one blocking operation

## 🚀 Optimization Strategy

### Phase 1: SIMD Vectorization (Week 1)
**Goal**: Vectorize hot path distance calculations

#### Implementation Details
- **File**: `omendb/algorithms/hnsw_fixed.mojo`
- **Function**: `_search_neighbors` and `_select_neighbors_heuristic`
- **Approach**:
  ```mojo
  # Current: Sequential distance calculations
  for candidate in candidates:
      dist = simd_cosine_distance(query, nodes[candidate].vector)
  
  # Optimized: Batch SIMD calculations
  var batch_size = simdwidthof[DType.float32]()
  for i in range(0, len(candidates), batch_size):
      batch_distances = simd_batch_cosine_distance(query, candidates[i:i+batch_size])
  ```

#### Expected Impact
- 5-10x speedup in neighbor selection
- Reduce construction time from 93 vec/s to ~500 vec/s

### Phase 2: Parallel Construction (Week 1-2)
**Goal**: Parallelize independent graph operations

#### Implementation Details
- **Parallel Neighbor Search**: Search multiple graph levels concurrently
- **Parallel Batch Insertion**: Insert multiple vectors simultaneously at different graph positions
- **Code Pattern**:
  ```mojo
  from algorithm import parallelize
  
  # Parallel neighbor searches
  parallelize[search_level](num_levels, num_threads)
  
  # Parallel batch insertions (non-overlapping regions)
  parallelize[insert_batch](batch_size, num_threads)
  ```

#### Expected Impact
- 2-4x speedup on multi-core systems
- Combined with SIMD: ~2,000 vec/s

### Phase 3: Memory Optimization (Week 2)
**Goal**: Eliminate allocation overhead

#### Implementation Details
1. **Pre-allocate Neighbor Lists**:
   ```mojo
   # Pre-allocate based on M parameter
   node.neighbors.reserve(M * 2)  # Bi-directional links
   ```

2. **Memory Pool for Nodes**:
   ```mojo
   struct NodePool:
       var nodes: List[HNSWNode]
       var free_list: List[Int]
       
       fn allocate(self) -> Int:
           # Return pre-allocated node index
   ```

3. **Batch Memory Operations**:
   - Allocate nodes in chunks
   - Reuse temporary buffers

#### Expected Impact
- 20-30% reduction in construction time
- More predictable performance

### Phase 4: Incremental Migration (Week 2)
**Goal**: Non-blocking migration at 5K threshold

#### Implementation Details
```mojo
fn migrate_incremental(self, chunk_size: Int = 100):
    """Migrate vectors in chunks while serving queries."""
    while self.migration_progress < self.brute_index.size:
        # Migrate chunk
        end = min(self.migration_progress + chunk_size, self.brute_index.size)
        self._migrate_chunk(self.migration_progress, end)
        self.migration_progress = end
        
        # Yield to allow queries
        if self.pending_queries > 0:
            return  # Resume later
```

#### Expected Impact
- Migration no longer blocks operations
- Smooth performance transition at 5K threshold

### Phase 5: Algorithm Optimizations (Week 3)
**Goal**: Reduce algorithmic complexity

#### Optimizations
1. **Pruning Strategy**: More aggressive neighbor pruning
2. **Early Termination**: Stop search when improvement unlikely
3. **Approximate Distance**: Use lower precision for initial filtering
4. **Dynamic M**: Adjust connections based on dataset characteristics

## 📈 Performance Targets

### Milestone Timeline
| Week | Optimization | Target Performance |
|------|-------------|--------------------|
| 1 | SIMD Vectorization | 500 vec/s |
| 2 | + Parallel Construction | 2,000 vec/s |
| 2 | + Memory Optimization | 3,000 vec/s |
| 3 | + Algorithm Tuning | 10,000 vec/s |
| 4 | + All Optimizations | 50,000+ vec/s |

### Success Criteria
- Construction: 50,000+ vec/s
- Query: <1ms (maintain current)
- Recall: 100% (maintain current)
- Migration: <1 second for 5K vectors

## 🧪 Testing Strategy

### Performance Benchmarks
```python
# benchmark_suite.py
datasets = [
    ("SIFT-128D", 1_000_000, 128),
    ("GIST-960D", 1_000_000, 960),
    ("Deep-96D", 10_000_000, 96)
]

metrics = [
    "construction_throughput",
    "query_latency_p50",
    "query_latency_p99",
    "recall@10",
    "memory_usage"
]
```

### Regression Prevention
- Automated performance tests in CI
- Benchmark against each optimization
- Compare with Faiss/HNSWlib baseline

## 🔧 Implementation Plan

### Week 1
- [ ] Implement SIMD batch distance calculations
- [ ] Create performance benchmark suite
- [ ] Profile and identify additional bottlenecks

### Week 2  
- [ ] Add Mojo parallelization to construction
- [ ] Implement memory pool allocator
- [ ] Design incremental migration system

### Week 3
- [ ] Implement algorithm optimizations
- [ ] Comprehensive performance testing
- [ ] Documentation and optimization guide

### Week 4
- [ ] Final optimizations and tuning
- [ ] Performance validation on large datasets
- [ ] Release preparation

## 🎯 Risk Mitigation

### Potential Risks
1. **Accuracy Loss**: Ensure optimizations don't affect recall
2. **Memory Usage**: Monitor memory with pre-allocation
3. **Code Complexity**: Keep optimizations maintainable

### Mitigation Strategies
- Comprehensive test suite before each optimization
- Incremental implementation with rollback capability
- Clear documentation of optimization techniques

## 📊 Success Metrics

### Immediate (Week 1)
- 5x improvement in construction speed
- No regression in query performance or recall

### Short-term (Month 1)
- 50,000+ vec/s construction
- Competitive with C++ implementations
- Production deployment ready

### Long-term
- Industry-leading pure Mojo implementation
- Foundation for GPU acceleration
- Showcase Mojo's performance capabilities

---

**Next Action**: Begin SIMD vectorization implementation in `hnsw_fixed.mojo`  
**Owner**: Engineering Team  
**Timeline**: 4 weeks to full optimization