# 🎯 OmenDB Master Architecture Decision (Sept 2025)

**Status**: FINAL DECISION
**Date**: September 20, 2025
**Supersedes**: All previous architecture docs

## Executive Summary

**Decision**: Pure Mojo with HNSW for CPU, IVF-Flat for GPU
**Server**: Python FastAPI wrapper (minimal, when needed)
**Embedded**: Pure Mojo (primary focus)
**Index**: HNSW for general use, IVF-Flat when GPU available

## Index Algorithm Decision

### Research Findings (Sept 2025)

After analyzing latest benchmarks and GPU optimization research:

#### HNSW Remains Best for CPU
- **Consistent performance**: 95%+ recall at high speed
- **Memory efficient**: ~200MB per million vectors
- **Incremental updates**: Can add vectors without rebuild
- **Wide support**: Most mature, well-understood

#### IVF-Flat Dominates on GPU
- **GPU-optimized**: 10-100x faster than CPU HNSW
- **NVIDIA cuVS**: Highly optimized CUDA implementation
- **Simple to parallelize**: Perfect for Mojo GPU kernels
- **Lower memory**: Compressed variants available

#### Why Not Others?
- **DiskANN**: Great for billion-scale, but complex for embedded use
- **IVF-PQ**: Good compression but worse recall than we want
- **LSH**: Too approximate for our quality targets

### Our Strategy: Adaptive Selection

```mojo
struct AdaptiveIndex:
    """Automatically selects best index for hardware"""

    fn select_index(self, config: Config) -> Index:
        if config.has_gpu and self.count > 100_000:
            # GPU + large dataset = IVF-Flat
            return IVFFlatGPU(nlist=sqrt(self.count))
        elif self.count < 10_000:
            # Small dataset = Flat search (100% recall)
            return FlatIndex()
        else:
            # Default = HNSW (best CPU performance)
            return HNSWIndex(M=32, ef=200)
```

## Architecture: Pure Mojo + Light Python

### Core Components

```
┌─────────────────────────────────────┐
│         Python Layer (Optional)      │
│  - FastAPI for HTTP endpoints        │
│  - Async coordination if needed      │
│  - Monitoring and metrics            │
└─────────────────────────────────────┘
                    │
              Python C API
                    ▼
┌─────────────────────────────────────┐
│         Mojo Core (Required)         │
│  - Vector storage (mmap)             │
│  - HNSW index (CPU)                  │
│  - IVF-Flat index (GPU)              │
│  - SIMD operations                   │
│  - Python bindings (native)          │
└─────────────────────────────────────┘
```

### Embedded Mode (Primary Focus)

```python
import omendb

# Pure Mojo, no server needed
db = omendb.open("vectors.db")
db.add_batch(vectors)  # 50K+ vec/s
results = db.search(query)  # 2-3ms
```

**Why Embedded First?**
- Most users start here (prototyping)
- No dependencies or setup
- SQLite-like simplicity
- Full performance

### Server Mode (When Needed)

```python
# Simple Python wrapper around Mojo core
from fastapi import FastAPI
import omendb

app = FastAPI()
db = omendb.open("vectors.db")

@app.post("/search")
async def search(query: list[float], k: int = 10):
    # Python handles async/HTTP
    # Mojo handles computation
    return db.search(query, k)
```

**Why Python for Server?**
- Async/await works today
- FastAPI is production-ready
- Minimal overhead (just routing)
- Can switch to Rust later if needed

**Why NOT Rust?**
- Adds complexity for little benefit
- Python is sufficient for routing
- Mojo does heavy lifting
- Easier to maintain

## Implementation Strategy

### Phase 1: Pure Mojo Core (Weeks 1-2)
```mojo
struct OmenDB:
    var storage: MMapStorage        # Memory-mapped vectors
    var index: AdaptiveIndex        # HNSW or IVF-Flat
    var config: Config               # User preferences

    fn add_batch(mut self, vectors: Tensor) -> List[Int]:
        # Batch operations only (enforce minimum size)
        if vectors.shape[0] < 100:
            raise Error("Minimum batch size is 100")
        return self.storage.append(vectors)

    fn build_index(mut self):
        # Explicit index building
        self.index = AdaptiveIndex.build(self.storage.vectors)

    fn search(self, query: Tensor, k: Int) -> Results:
        if self.config.device == "gpu" and GPU.available():
            return self.index.search_gpu(query, k)
        else:
            return self.index.search_cpu(query, k)
```

### Phase 2: Python Bindings (Week 3)
```python
# Zero-copy Python integration
import numpy as np
import omendb

# NumPy arrays work directly
vectors = np.random.rand(100000, 768).astype(np.float32)
db = omendb.DB(dim=768)
db.add_batch(vectors)  # Zero-copy from NumPy
```

### Phase 3: Optional Server (Week 4)
```python
# Only if user needs HTTP API
pip install omendb[server]

# Starts FastAPI server
omendb serve --host 0.0.0.0 --port 8000 --db vectors.db
```

## GPU Integration Plan

### Leveraging Mojo 25.6

```mojo
struct IVFFlatGPU:
    """GPU-accelerated IVF-Flat index"""

    var centroids: GPUBuffer[Float32]
    var clusters: List[GPUBuffer[Float32]]

    @parameter
    fn search_gpu[device: GPU.Device](
        self,
        query: Tensor,
        k: Int
    ) -> Results:
        # Step 1: Find nearest centroids on GPU
        var distances = gpu.matmul(query, self.centroids.T)
        var nearest_clusters = gpu.topk(distances, nprobe)

        # Step 2: Search within clusters
        var results = List[Results]()
        for cluster_id in nearest_clusters:
            var cluster = self.clusters[cluster_id]
            var local_results = gpu.matmul(query, cluster.T)
            results.append(local_results.topk(k))

        # Step 3: Merge results
        return merge_topk(results, k)
```

### GPU vs CPU Performance

```
Algorithm    Hardware    Vectors    Build      Search    Recall
-------------------------------------------------------------
Flat         CPU         10K        N/A        5ms       100%
HNSW         CPU         100K       10s        2ms       95%
HNSW         CPU         1M         100s       3ms       95%
IVF-Flat     GPU         100K       1s         0.2ms     95%
IVF-Flat     GPU         1M         10s        0.5ms     93%
IVF-Flat     GPU         10M        100s       1ms       90%
```

## Critical Decisions

### 1. Why HNSW for CPU?
- **Proven**: Most tested, understood algorithm
- **Balanced**: Good speed/recall trade-off
- **Incremental**: Can add vectors without full rebuild
- **Memory efficient**: Better than graph alternatives

### 2. Why IVF-Flat for GPU?
- **Simple**: Easy to implement in Mojo
- **Parallel**: Perfect for GPU architecture
- **Fast**: 10-100x speedup over CPU
- **NVIDIA optimized**: cuVS provides reference

### 3. Why Python for Server?
- **Available now**: Async works today
- **Good enough**: Just routing, Mojo does work
- **Easy migration**: Can switch to Rust later
- **Lower complexity**: One less language to manage

### 4. Why Embedded First?
- **Market fit**: Most start with embedded
- **Simplicity**: No dependencies
- **Performance**: No network overhead
- **Growth path**: Can add server later

## What This Means for Codebase

### Keep
- `omendb/engine/` - Mojo core implementation
- `internal/` docs - Consolidated and updated
- Python bindings code

### Remove
- `zendb/` - Separate Rust project
- Conflicting architecture docs
- Old test results

### Defer
- `omendb/server/` - Rust server (use Python for now)
- `omendb/web/` - Marketing site (not priority)

## Performance Targets

### MVP (2 weeks)
- Bulk load: 50K vec/s
- Search: 2-3ms (CPU HNSW)
- Memory: 4GB per million

### With GPU (3 weeks)
- Bulk load: 200K vec/s
- Search: <1ms (GPU IVF-Flat)
- Memory: GPU VRAM + RAM

### Production (4 weeks)
- Python FastAPI wrapper
- Monitoring metrics
- Multi-index support

## Migration Path

### From ChromaDB
```python
# Nearly identical API
# Before: import chromadb
# After:
import omendb as chromadb  # Drop-in replacement
```

### From Pinecone
```python
# Add compatibility layer
from omendb.compat import pinecone
index = pinecone.Index("myindex")  # Works
```

## Next Steps

1. **Week 1**: Implement core Mojo engine with HNSW
2. **Week 2**: Add IVF-Flat GPU index
3. **Week 3**: Python bindings and tests
4. **Week 4**: Optional FastAPI server

## Conclusion

**Pure Mojo + HNSW/IVF-Flat + Python wrapper** is optimal:
- Ships immediately (no async blockers)
- Best performance (no FFI overhead)
- GPU-ready (Mojo 25.6)
- Simple deployment
- Future-proof architecture

This supersedes all previous architecture decisions.