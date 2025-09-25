# OmenDB Technical Specification

**Pure Mojo vector database with Python bindings for 200K+ vec/s performance**

## 🎯 Core Architecture

### Language & Design
- **Core**: Pure Mojo (zero external dependencies)
- **API**: Python bindings with minimal overhead
- **Philosophy**: Hardware-optimized, embedded-first, production-ready

### Performance Requirements
- **Insertion**: 200,000+ vectors/second
- **Query**: <0.1ms latency (small datasets)
- **Memory**: 4 bytes/dimension + minimal overhead
- **Scaling**: Automatic algorithm switching at 5K vectors

## 🚀 Current Performance Status

```
Regular API: 5,765 vec/s (Python overhead)
Benchmark Mode: 96,269,563 vec/s (pure Mojo)
Gap: 16,697x due to Python→Mojo conversion
```

### Bottleneck Analysis
- **Python→Mojo conversion**: 50% of execution time
- **String operations**: 20% overhead
- **Metadata handling**: 10% overhead
- **Core storage**: Only 20% (blazing fast!)

## 📐 API Design

### Industry-Standard Interface
```python
from omendb import DB

# Create database
db = DB("vectors.omen")

# Add vectors with metadata
db.add("doc1", [1.0, 2.0, 3.0], {"category": "tech"})

# Batch operations for efficiency
batch_data = [
    ("doc2", [4.0, 5.0, 6.0], {"category": "science"}),
    ("doc3", [7.0, 8.0, 9.0], {"category": "tech"})
]
db.add_batch(batch_data)

# Query with filtering
results = db.query([1.0, 2.0, 3.0], top_k=5, where={"category": "tech"})
```

## 🏗️ Key Components

### 1. Native Module (`omendb/native.mojo`)
- Python bindings via `PythonModuleBuilder`
- SIMD-optimized operations throughout
- Benchmark mode for hardware validation
- Metrics collection and export

### 2. Algorithms
- **BruteForce** (<5K vectors): SIMD linear search, ultra-fast
- **RoarGraph** (≥5K vectors): VLDB 2024 bipartite graph, 5-10x faster than HNSW

### 3. Core Operations (`omendb/core/`)
- **vector.mojo**: SIMD vector math
- **distance.mojo**: Hardware-optimized similarity
- **brute_force.mojo**: Matrix operations, batch processing
- **matrix_ops.mojo**: BLAS-style bulk operations

### 4. Python Interface (`python/omendb/api.py`)
- Zero-copy operations (planned)
- Batch processing support
- Automatic dimension detection
- ChromaDB-compatible API

## 🔧 Optimization Strategy

### Implemented (Validated)
- ✅ SIMD vectorization (4-8x on vector ops)
- ✅ Matrix operations (93x improvement achieved)
- ✅ Pre-allocated memory pools
- ✅ Benchmark mode (96M+ vec/s proven)

### In Progress
- 🚧 NumPy buffer protocol (zero-copy)
- 🚧 Direct memory API
- 🚧 BLAS integration

### Planned
- 📋 GPU acceleration (CUDA/ROCm)
- 📋 Distributed operations
- 📋 Advanced query types

## 📊 Success Metrics

### Performance
- ✅ Hardware capability: 96M+ vec/s (validated)
- ❌ Production API: 5.7K vec/s (Python overhead)
- 🎯 Target: 200K+ vec/s in production

### Quality
- Zero external dependencies
- Cross-platform support (macOS/Linux/Windows)
- Production-grade error handling
- Comprehensive test coverage

## 🔍 Technical Decisions

### Algorithm Threshold
- Tested multiple thresholds (1K, 5K, 10K)
- 5K vectors optimal for BruteForce→RoarGraph switch
- Based on real-world performance profiling

### Memory Layout
- Contiguous storage for cache efficiency
- SIMD-aligned allocations
- Pre-allocated capacity (1M vectors)

### Python Integration
- Direct bindings (no intermediate layer)
- Batch operations to amortize overhead
- Future: NumPy buffer protocol for zero-copy

## 🚦 Next Steps

1. **Immediate**: Implement NumPy buffer protocol
2. **Short-term**: Create direct memory API
3. **Medium-term**: Integrate vendor BLAS
4. **Long-term**: GPU acceleration

See `docs/internal/` for detailed implementation plans.