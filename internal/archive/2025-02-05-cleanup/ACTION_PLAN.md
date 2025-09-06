# 🎯 OmenDB Action Plan - HNSW+ Implementation
*Last Updated: 2025-02-05*

## 📊 Strategic Pivot: DiskANN → HNSW+

### Why HNSW+
- **Market fit**: Every successful vector DB uses HNSW
- **Mojo advantages**: SIMD, parallelism, future GPU support
- **Business model**: CPU open source, GPU cloud premium
- **Timeline**: 4 weeks to production vs 6+ for IP-DiskANN

## Phase 1: HNSW+ Core (Weeks 1-2)

### Week 1: Core Algorithm
```mojo
struct HNSWIndex:
    var layers: List[Graph]         # Hierarchical structure
    var entry_point: Int            # Top layer entry
    var M: Int = 16                 # Connections per layer
    var ef_construction: Int = 200  # Build parameter
    
    fn insert(self, vector, level):
        # Parallel layer insertion
        # SIMD distance calculations
        # Lock-free neighbor updates
```

| Task | Priority | Location | Time |
|------|----------|----------|------|
| Design hierarchical structure | P0 | New file | 2 days |
| Implement layer management | P0 | New file | 2 days |
| Port neighbor selection | P0 | Reference HNSWLIB | 1 day |

### Week 2: Search & Optimization
| Task | Priority | Optimization | Time |
|------|----------|--------------|------|
| Hierarchical search | P0 | SIMD distances | 2 days |
| Parallel construction | P0 | Multi-threaded | 1 day |
| Python bindings | P0 | Zero-copy FFI | 2 days |

## Phase 2: Production Features (Weeks 3-4)

### Week 3: Integration
```python
# Simple Python API
from omendb import Index
index = Index(dimension=1536, algorithm="hnsw+")
index.add(vectors, ids)           # <100ms for 10K vectors
results = index.search(query, k)  # <10ms latency
```

| Feature | Implementation | Reference |
|---------|---------------|-----------|
| Persistence | Memory-mapped files | RocksDB pattern |
| Deletion | Tombstones + compaction | pgvector approach |
| Updates | Delete + reinsert | Standard pattern |

### Week 4: Benchmarking
| Benchmark | Target | Baseline |
|-----------|--------|----------|
| Build speed | 100K vectors/sec | pgvector: 10K/sec |
| Query latency | <10ms @ 95% recall | pgvector: 50ms |
| Memory usage | 2 bytes/vector | pgvector: 4 bytes |

## Phase 3: GPU Acceleration (Month 2)

### GPU Optimizations (Cloud Only)
```mojo
@parameter
if target == "GPU":
    # GPU kernels for distance calculation
    fn simd_distance_gpu[size: Int](a: Buffer, b: Buffer) -> Float32:
        # Parallel GPU computation
        # 100x faster than CPU
```

| Component | CPU Performance | GPU Performance | Speedup |
|-----------|----------------|-----------------|---------|
| Distance calc | 1M/sec | 100M/sec | 100x |
| Graph build | 100K/sec | 1M/sec | 10x |
| Search | 10K QPS | 50K QPS | 5x |

## Phase 4: Business Model (Month 2-3)

### Open Source (CPU)
- Algorithm: HNSW+ 
- Performance: 10x pgvector
- Language bindings: Python, C, Rust
- License: Apache 2.0

### Cloud Platform (GPU)
- Algorithm: HNSW+ GPU-accelerated
- Performance: 100x pgvector
- Features: Managed, auto-scaling
- Pricing: $0.50/million vectors/month

## 📈 Success Metrics

| Metric | Current (DiskANN) | HNSW+ Target | Industry Best |
|--------|-------------------|--------------|---------------|
| **Algorithm** | Broken at 25K | Production ready | HNSW standard |
| **Build rate** | 10K/sec | 100K/sec | 50K/sec |
| **Query QPS** | Unknown | 10K (CPU), 50K (GPU) | 100K |
| **Memory/vector** | 288 bytes | 2 bytes | 1-4 bytes |
| **Market fit** | Experimental | Production proven | ✓ |

## 🔧 FFI Strategy (Answering Your Question)

### Mojo → Python (Native, Zero Overhead)
```mojo
@export
fn search_vectors(query: PythonObject) -> PythonObject:
    # Direct Python interop, no FFI overhead
    return results
```

### Mojo → C (Shared Library)
```bash
mojo build --emit shared-lib -o libomendb.so
```
```c
// C header
typedef struct OmenIndex* omen_index_t;
omen_index_t omen_create(int dimension);
void omen_search(omen_index_t index, float* query, int* results);
```

### Mojo → Rust (Via C ABI)
```rust
#[link(name = "omendb")]
extern "C" {
    fn omen_create(dimension: i32) -> *mut OmenIndex;
    fn omen_search(index: *mut OmenIndex, query: *const f32, results: *mut i32);
}
```

**FFI Performance**: 
- Python: ~0 overhead (native integration)
- C/Rust: ~100ns per call (negligible for batch ops)

## 📚 Key Implementation Files

| Component | Location | Status |
|-----------|----------|--------|
| Current DiskANN | `omendb/engine/omendb/algorithms/diskann.mojo` | Replace |
| New HNSW+ | `omendb/engine/omendb/algorithms/hnsw.mojo` | Create |
| Python API | `omendb/engine/python/omendb/api.py` | Update |
| Benchmarks | `omendb/engine/benchmarks/hnsw_benchmark.py` | Create |

## 🚀 Getting Started

```bash
# Create new HNSW implementation
cd omendb/engine/omendb/algorithms
touch hnsw.mojo

# Start with basic structure
# Reference: https://github.com/nmslib/hnswlib

# Build and test
cd ../..
pixi run mojo build omendb/native.mojo -o python/omendb/native.so
pixi run benchmark-quick
```

## 📝 Decision Summary

**Algorithm**: HNSW+ (not IP-DiskANN)
- Proven in production
- Better market fit
- Mojo strengths apply perfectly

**Language**: Mojo for core
- Python native integration
- Future GPU support
- SIMD optimizations built-in

**Business**: Open/Cloud split
- Broad adoption via open source
- Revenue from GPU acceleration
- Clear upgrade path

---
*Next review: After Week 1 implementation*