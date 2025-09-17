# OmenDB Status (October 2025)

## Critical Discovery
**SoA is WRONG for HNSW** - Industry benchmarks show hnswlib (AoS) is 7x faster than FAISS HNSW (separated storage). See `CRITICAL_FINDINGS.md` for full analysis.

## Current Metrics (After SIMD Fix)
- **Build status**: ✅ Compiles successfully
- **Import fixes**: Replaced broken `advanced_simd` with `simd_distance`
- **Performance**: 427 vec/s (down from 763 due to generic SIMD)
- **Test status**: Binary quantization test passes

## Immediate Actions Taken
1. ✅ Fixed broken imports (advanced_simd → simd_distance)
2. ✅ Disabled adaptive_search (import failed)
3. ✅ Used specialized_kernels for common dimensions
4. ✅ Build now succeeds

## Critical Path Forward

### 1. Zero-Copy FFI (50% overhead)
```python
# Current: Python list → Mojo copy
vectors = [[1,2,3], [4,5,6]]  # Slow!

# Target: NumPy buffer protocol
vectors = np.array(data)  # Direct memory access
```

### 2. Keep AoS Layout
- DON'T migrate to SoA (will hurt cache locality)
- Keep vectors together for graph traversal
- Optimize for random access pattern

### 3. Cache Prefetching
```mojo
# Prefetch next neighbors during traversal
fn traverse_with_prefetch(node_id: Int):
    var neighbors = get_neighbors(node_id)
    for i in range(len(neighbors)):
        prefetch(get_vector(neighbors[i]))  # Load into cache
        process_node(neighbors[i])
```

## Performance Analysis

### Why Performance Dropped
- Generic `simd_l2_distance` vs specialized kernels
- No cache optimization yet
- FFI overhead still present (50%)

### Expected After Optimizations
- Zero-copy FFI: 2x speedup (850 vec/s)
- Better SIMD usage: 2-3x (1,700-2,550 vec/s)
- Cache prefetching: 1.5x (3,825 vec/s)
- Combined: ~4-5K vec/s achievable

## Blockers Resolved
- ✅ Compilation errors fixed
- ✅ SIMD imports working
- ✅ Tests passing

## Next Sprint Focus
1. Implement NumPy buffer protocol for zero-copy
2. Add cache prefetching to graph traversal
3. Optimize specialized kernels usage
4. Keep AoS layout (don't implement SoA)

## Commands
```bash
# Build
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib -I omendb

# Test
pixi run python test_binary_quantization_quick.py
pixi run python test_simd_performance.py

# Current: 427 vec/s (working but slow)
# Target: 25K+ vec/s (after optimizations)
```

---
*Critical insight: Cache locality > SIMD width for HNSW*