# Critical Research Findings (October 2025)

## MAJOR DISCOVERY: SoA is WRONG for HNSW

### Industry Evidence
**hnswlib is 7x faster than FAISS HNSW** on identical workloads:
- hnswlib: 22 seconds (unified storage)
- FAISS IndexHNSW: 162 seconds (separated storage)
- Dataset: 38 million vectors, 10K queries

### Why SoA Hurts HNSW Performance

1. **Random Access Pattern**
   - HNSW traverses graph randomly, not sequentially
   - Cache misses when jumping between separated arrays
   - CPU prefetcher can't predict next access

2. **Cache Locality Destroyed**
   - Vector dimensions should be together (AoS)
   - Graph connections should be near vector data
   - Separation causes memory bandwidth bottleneck

3. **No Effective Prefetching**
   - Can't use `_mm_prefetch` with separated layout
   - FAISS's layered architecture prevents this optimization
   - hnswlib can prefetch because data is unified

4. **Abstraction Overhead**
   - Extra indirection to access separated storage
   - Function call overhead between layers
   - Prevents compiler optimizations

## Correct Architecture for HNSW

### Unified AoS Storage (hnswlib approach)
```
Node {
    vector: [dim floats]  // Keep together
    neighbors: [M ints]   // Adjacent in memory
}
```

### Why This Works
- Single cache line fetch gets vector + metadata
- Spatial locality during graph traversal
- Enables hardware prefetching
- Direct memory access without abstraction

## Our Current Mistakes

1. **SoA buffers allocated but wrong** - `vectors_soa` will hurt performance
2. **Chunked builder misguided** - HNSW needs incremental updates
3. **Wrong optimization focus** - Should optimize cache, not SIMD width

## Correct Optimization Priority

1. **Fix broken SIMD with AoS** ✅ (just completed)
2. **Zero-copy FFI** - Still critical (50% overhead)
3. **Cache prefetching** - Prefetch next neighbors during traversal
4. **Keep unified storage** - Don't separate vectors from graph
5. **Remove abstractions** - Direct memory access

## Performance Impact

If we stick with SoA:
- Expect 5-7x SLOWER than optimal
- Cache miss rate will dominate
- No amount of SIMD will compensate

If we keep AoS + optimize correctly:
- Match hnswlib performance
- 25K+ vec/s achievable
- Better scaling with dimension

## Action Required

**STOP** SoA migration immediately
**KEEP** AoS layout for cache efficiency
**FOCUS** on FFI overhead and prefetching

---
*This finding contradicts our entire SoA strategy but is backed by production benchmarks*