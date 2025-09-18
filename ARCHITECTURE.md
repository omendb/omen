# OmenDB Architecture (CPU-First, October 2025)

## Overview
- **Mojo core + Python bindings**: All hot paths run in Mojo with SIMD; Python marshals batched requests
- **Execution model**: HNSW graph with binary quantization; CPU-only (Mojo has no GPU support)
- **Design philosophy**: Cache locality > SIMD width, AoS > SoA for graph traversal

## Critical Discovery: AoS is Optimal for HNSW
**Industry evidence**: hnswlib (AoS) is 7x faster than FAISS (separated storage)
- **Why**: HNSW has random access patterns, benefits from cache locality
- **Decision**: Keep Array-of-Structures layout for vectors
- **Impact**: Avoided months of wrong SoA optimization

## Current Performance (September 18, 2025)
| Metric | Value | Notes |
|--------|-------|-------|
| **Current throughput** | 2,352 vec/s | Week 2 Day 3 - parallel attempts failed |
| **Search latency** | 0.90 ms | For top-10 results |
| **Recall@10** | 95%+ | Algorithm quality excellent |
| **Status** | Needs competitive optimization | Week 2 focused on wrong bottlenecks |

## 🚨 Week 2 Realization: Implementation Naive vs Algorithm Sound
- **Algorithm Quality**: ✅ State-of-the-art HNSW with 95% recall
- **Implementation Quality**: ❌ Naive parameters (ef_construction=200 vs competitors' 50-100)
- **Missing Optimizations**: Batch processing, proper parallelism, parameter tuning

## Core Components
| Component | Description | Status |
|-----------|-------------|--------|
| `omendb/engine/omendb/algorithms/hnsw.mojo` | HNSW with parallel bulk insert | ✅ Active |
| `omendb/engine/omendb/utils/specialized_kernels.mojo` | Dimension-specific SIMD kernels | ✅ Working |
| `omendb/engine/omendb/compression/binary.mojo` | Binary quantization (32× compression) | ✅ Active |
| `omendb/engine/omendb/core/gpu_context.mojo` | CPU stub (no GPU in Mojo) | ✅ Placeholder |

## Storage Architecture
- **Primary buffer (`self.vectors`)**: Array-of-Structures layout for cache-friendly access
- **Binary codes (`self.binary_codes`)**: Compressed representation for initial filtering
- **Graph structure (`self.graph`)**: Adjacency lists for HNSW connectivity
- **Memory alignment**: 64-byte boundaries for SIMD operations

## Implemented Optimizations

### 1. Zero-Copy FFI ✅
- NumPy buffer protocol via `ctypes.data`
- Direct memory access without copying
- 1.4x speedup, FFI overhead reduced to 10%

### 2. Parallel Graph Construction ✅
```mojo
# Mojo native parallelization
parallelize[process_chunk_parallel](num_chunks)
```
- Chunk-based independent processing
- Hardware-aware (uses N-1 cores)
- 22x speedup for large batches

### 3. Binary Quantization ✅
- 32x memory reduction
- Hamming distance for filtering
- Minimal recall loss (<5%)

## Performance Breakdown (5K vectors)
```
Parallel graph construction: 40%
Distance computations:       25%
Memory operations:          15%
FFI overhead:               10%
Metadata/ID handling:       10%
```

## Roadmap to 25K vec/s

### Phase 1: Cache Optimization (1.5x expected)
- Prefetch next neighbors during traversal
- Better memory access patterns
- Reduce cache misses

### Phase 2: Lock-Free Updates (1.3x expected)
- Atomic operations for graph updates
- Reduce synchronization overhead
- Better parallel scaling

### Phase 3: SIMD Distance Matrix (1.2x expected)
- Vectorized distance computations
- Process multiple distances simultaneously
- Better CPU utilization

### Combined Impact
- Current: 9,504 vec/s
- Target: ~22,000 vec/s
- Gap: 2.3x (achievable!)

## Key Design Decisions

### Why AoS Over SoA
- **Cache locality**: Graph traversal is random access
- **Proven by benchmarks**: hnswlib beats FAISS by 7x
- **Simpler implementation**: No complex memory layouts

### Why Parallel Construction Works
- **Independent chunks**: Minimal synchronization
- **No Python GIL**: Pure Mojo execution
- **Hardware-aware**: Optimal worker count

### Why Binary Quantization
- **Memory efficiency**: 32x reduction
- **Fast filtering**: CPU popcount instruction
- **Quality preserved**: 95%+ recall maintained

## Build & Deploy
```bash
# Build with optimizations
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib -I omendb

# Test performance
pixi run python test_scaling.py

# Benchmark
pixi run python benchmark_competitive.py
```

## Next Actions
1. Implement cache prefetching for graph traversal
2. Add lock-free atomic operations
3. Optimize SIMD distance computations
4. Profile and tune for specific hardware

See `internal/ARCHITECTURE.md` for implementation details and `internal/STATUS.md` for current metrics.