# OmenDB Status (October 2025)

## Performance Update
- **Baseline**: 427 vec/s (NumPy zero-copy working)
- **Previous**: 298 vec/s (Python lists, slow path)
- **Speedup**: 1.4x with zero-copy FFI ✅
- **Target**: 25K+ vec/s (need 60x more)

## Major Findings

### 1. SoA is WRONG for HNSW
- Industry benchmarks: hnswlib (AoS) is 7x faster than FAISS (SoA)
- Cache locality > SIMD width for graph traversal
- **Decision**: Keep AoS layout ✅

### 2. Zero-Copy FFI Working
- NumPy buffer protocol implemented ✅
- Direct memory access via ctypes ✅
- 1.4x speedup achieved (limited by other bottlenecks)

### 3. Real Bottleneck: Graph Construction
- HNSW graph building: ~70% of time
- FFI overhead: only ~10% now
- Need parallel construction and cache optimization

## Next Priorities

1. **Parallel graph construction** - Expected 2-3x speedup
2. **Cache prefetching** - Expected 1.5x speedup
3. **Batch metadata processing** - Expected 1.2x speedup
4. **Combined target**: 2,000+ vec/s achievable

See `internal/ZERO_COPY_ANALYSIS.md` for detailed breakdown.