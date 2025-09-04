# OmenDB Technical Specification
*Last Updated: August 23, 2025*

## Overview

OmenDB is a high-performance vector database engine written in Mojo, designed for state-of-the-art efficiency and performance. Deploy anywhere from embedded applications to distributed clusters.

## Architecture

### Core Design Principles
- **Single Algorithm**: DiskANN (Microsoft Vamana) only - no algorithm switching
- **Embedded First**: No server required, direct library integration
- **Zero Configuration**: Works from 1K to 1B vectors without tuning
- **Memory-Mapped Storage**: Modern approach, no legacy WAL

### Technology Stack
- **Language**: Mojo (Python syntax, C++ performance potential)
- **Algorithm**: DiskANN/Vamana graph-based ANN
- **Storage**: Memory-mapped files with double-buffering
- **API**: Python with numpy zero-copy support
- **Platform**: macOS/Linux (Windows pending Mojo support)

## Algorithm: DiskANN (Vamana)

### Why DiskANN Only
- Scales seamlessly from 1K to 1B vectors
- No index rebuilding at scale thresholds
- Proven at Microsoft Bing scale (billions of vectors)
- Streaming updates match modern data patterns
- Superior to HNSW for dynamic workloads

### Implementation Details
```yaml
Graph Structure:
  - Max degree: 64 (configurable)
  - Entry point: Medoid of dataset
  - Pruning: α-RNG algorithm for edge selection
  - Distance: Cosine similarity (L2 planned)

Build Process:
  - Deferred indexing: Buffer 10K vectors before building
  - Batch operations for efficiency
  - Incremental graph updates

Search Process:
  - Beam search with early termination
  - Typical latency: <1ms for 100K vectors
  - Recall: >95% @ k=10
```

### Algorithm Tuning Parameters
```yaml
Current Settings:
  R: 64          # Max degree (edges per node)
  L: 100         # Search list size
  alpha: 1.2     # Pruning parameter
  
Optimal for 100K-1M vectors:
  R: 32-48       # Lower for faster build
  L: 50-75       # Balance speed/accuracy
  alpha: 1.1-1.3 # Higher = more diverse edges

Buffer Thresholds:
  10K vectors: Trigger index build
  100K vectors: Seal segment, start new
  1M vectors: Consider sharding
```

## Storage Architecture

### Memory-Mapped Design (State-of-Art 2025)
```yaml
Why Not WAL:
  - WAL is 2-3 generations behind current research
  - Write amplification issues
  - No zero-copy reads
  - Poor SSD utilization

Our Approach:
  - Memory-mapped segments for zero-copy access
  - Double-buffering for non-blocking checkpoint
  - Hot buffer → Build segment → Atomic swap
  - Instant checkpoint via buffer swap (microseconds)
```

### Storage Components
```mojo
struct MemoryMappedStorage:
    hot_vectors: Dict[Int, List[Float32]]      # Active writes
    hot_metadata: Dict[Int, Metadata]          # Active metadata
    checkpoint_vectors: Dict[Int, List[Float32]] # Checkpoint buffer
    checkpoint_metadata: Dict[Int, Metadata]    # Checkpoint metadata
    vector_file: FileMapping                   # Memory-mapped vectors
    metadata_file: FileMapping                 # Memory-mapped metadata
```

## Performance Characteristics

### Current Performance
| Operation | Performance | Bottleneck |
|-----------|------------|------------|
| Batch Insert | 85K vec/s | Building index |
| Individual Insert | 3K vec/s | FFI overhead (0.34ms) |
| Search | 0.62ms @ 128D | Graph traversal |
| Checkpoint | 739K vec/s* | I/O bandwidth |
| Memory Usage | 40MB/1M vectors | No quantization yet |

*Checkpoint speed achieved via async double-buffering, needs validation

### Optimization Status
- ✅ Batch operations implemented
- ✅ Zero-copy numpy arrays (via unsafe_get_as_pointer)
- ✅ Async checkpoint with buffer swapping
- ❌ SIMD disabled (Mojo compiler issues)
- ❌ Memory pool disabled (thread safety)
- ❌ Quantization not implemented

## API Design

### Python Interface
```python
import omendb
import numpy as np

# Simple API
db = omendb.DB()
vectors = np.random.rand(1000, 128).astype(np.float32)
ids = db.add_batch(vectors)
results = db.search(query_vector, limit=10)

# Persistence
db.set_persistence("/path/to/db.omen")
db.checkpoint()  # Instant via buffer swap
```

### Key Design Decisions
1. **Batch-first API**: Individual operations discouraged
2. **Numpy native**: Zero-copy for performance
3. **No collections**: Single DB instance (like SQLite)
4. **Explicit checkpoint**: User controls persistence

## Mojo-Specific Considerations

### Current Limitations
- **No SIMD**: Compiler issues prevent vectorization (2-3x performance left)
- **FFI Overhead**: 0.34ms per call limits individual operations
- **No True Async**: Using double-buffering workaround
- **Thread Safety**: Global state issues with tcmalloc
- **Memory Pool**: Disabled due to thread safety

### FFI Zero-Copy Implementation
```mojo
# WRONG approach (doesn't work):
var address = Int(numpy_array.__array_interface__["data"][0])
var ptr = UnsafePointer[Float32](address)  # ❌ No constructor

# CORRECT approach (67K vec/s):
var ctypes_data = numpy_array.ctypes.data
var ptr = ctypes_data.unsafe_get_as_pointer[DType.float32]()  # ✅

# Direct memory access - no Python object overhead
for i in range(total_elements):
    var value = ptr.load(i)  # ~0.05μs vs 5μs via python.float()
```

### Double-Buffer Checkpoint
```mojo
struct MemoryMappedStorage:
    # Double buffers for instant swap
    var hot_vectors: Dict[Int, List[Float32]]      # Active writes
    var checkpoint_vectors: Dict[Int, List[Float32]] # Checkpoint buffer
    
    fn checkpoint_async(mut self) raises -> Bool:
        # INSTANT BUFFER SWAP - microseconds not seconds!
        self.checkpoint_vectors, self.hot_vectors = \
            self.hot_vectors, self.checkpoint_vectors
        return True  # Returns immediately
```

### Future Optimizations (Blocked by Mojo)
```mojo
# Waiting for Mojo to support:
@vectorize  # SIMD operations
async fn    # True async/await
Arc/Mutex   # Thread-safe primitives
Module vars # For collections API

# Quantization plan (3.8x memory reduction):
var vector: List[Float32]   # Current: 128 * 4 = 512 bytes
var quantized: List[UInt8]  # Target: 128 * 1 = 128 bytes
var scale: Float32           # + 4 bytes
var offset: Float32          # + 4 bytes = 136 total
```

## Competitive Analysis Summary

*For detailed market analysis, benchmarks, and feature comparison matrix, see [COMPETITIVE_ANALYSIS.md](./COMPETITIVE_ANALYSIS.md)*

### Quick Positioning
- **vs Qdrant**: OmenDB is embedded-first (like SQLite), Qdrant is server-based (like PostgreSQL)
- **vs Chroma**: OmenDB has 10x better memory efficiency, targets production use
- **vs Weaviate**: OmenDB is lightweight embedded, Weaviate is enterprise server
- **vs Pinecone**: OmenDB is local/self-hosted, Pinecone is cloud-only

### Our Differentiators
- **Memory Efficiency**: 29MB/100K vectors full precision, 7MB quantized
- **Embedded First**: True zero-dependency embedding
- **Single Algorithm**: DiskANN scales 1K→1B without switching
- **Mojo Performance**: Potential for C++ speeds with Python syntax
- **Pragmatic Defaults**: Full precision by default, quantization when needed

### Current Gaps (See ROADMAP in TODO.md)
- Thread safety (blocked by Mojo)
- REST/gRPC APIs (Python-only currently)
- Advanced filtering (basic metadata only)
- Production monitoring

## Quantization Strategy

### Competitor Approaches

| Database | Quantization Options | Default | Performance Impact | Memory Savings |
|----------|---------------------|---------|-------------------|----------------|
| **Qdrant** | Scalar (8-bit), Product (PQ) | OFF | 10-20% slower | 4-32x |
| **Weaviate** | Product Quantization (PQ) | OFF | 15-30% slower | 8-96x |
| **Pinecone** | Automatic (pod-based) | Varies | Transparent | 4-16x |
| **Milvus** | IVF_SQ8, PQ, SQ8H | OFF | 5-25% slower | 4-64x |
| **Chroma** | None | N/A | N/A | N/A |
| **Faiss** | SQ8, PQ, OPQ, LSQ | OFF | Highly variable | 4-256x |

### Why Competitors Default to OFF
1. **Accuracy first**: Users blame the DB for bad results (2-5% recall drop typical)
2. **Gradual scaling**: Most start small (<100K vectors) where memory isn't critical
3. **Complexity**: Quantization adds tuning parameters and debugging difficulty
4. **Hardware trends**: RAM cost dropping faster than accuracy requirements rising

### OmenDB Quantization Design

```python
# Default: Full precision for accuracy
db = omendb.DB()  # No quantization

# Explicit opt-in when scale demands it
db = omendb.DB()
db.enable_quantization()  # Scalar 8-bit quantization
```

#### Implementation Details
- **Method**: Scalar quantization (8-bit) with per-vector scale/offset
- **Storage**: 1 byte/dim + 8 bytes/vector overhead
- **Search**: On-the-fly dequantization during distance computation
- **Trade-offs**:
  - Memory: 4x reduction (128D: 512B → 136B per vector)
  - Speed: ~15% slower due to dequantization
  - Accuracy: 2-3% recall drop typical

#### When to Enable Quantization

| Dataset Size | Memory Usage | Recommendation |
|-------------|--------------|----------------|
| <100K vectors | <5GB | Full precision |
| 100K-1M | 5-50GB | Consider quantization |
| >1M | >50GB | Quantization recommended |
| Embedded/Edge | Limited | Always quantize |

### Future Quantization Roadmap
1. **Binary quantization**: 32x reduction for initial filtering
2. **Product quantization**: 16-64x reduction with better accuracy
3. **Learned quantization**: Adaptive to data distribution
4. **Mixed precision**: Critical vectors full, rest quantized

## Future Roadmap

### Phase 1: Validation (Current)
- Comprehensive benchmarking at scale
- Thread safety validation
- Production hardening

### Phase 2: Optimization
- Quantization for 3x memory reduction
- SIMD when Mojo supports it
- GPU exploration for batch operations

### Phase 3: Scale
- Distributed sharding (if needed)
- Cloud offering
- Enterprise features

## Testing & Benchmarking

### Standard Performance Test
```python
# Always use this for comparable results
import numpy as np
import time

vectors = np.random.rand(10000, 128).astype(np.float32)
queries = vectors[:100]  # First 100 as queries

# Insert performance
start = time.perf_counter()
ids = db.add_batch(vectors)
insert_time = time.perf_counter() - start
print(f"Insert: {len(vectors)/insert_time:.0f} vec/s")

# Search performance
start = time.perf_counter()
for q in queries:
    results = db.search(q, limit=10)
search_time = time.perf_counter() - start
print(f"Search: {search_time/len(queries)*1000:.2f} ms/query")
```

## Risk Analysis

### Technical Risks
- **Mojo Immaturity**: Language still evolving, missing features
- **Single Algorithm**: No fallback if DiskANN underperforms
- **Memory Usage**: 3x higher than competitors without quantization

### Mitigation Strategies
- Conservative Mojo usage (proven patterns only)
- DiskANN proven at scale (Microsoft Bing)
- Quantization implementation planned

---

*This specification represents the consolidated technical truth as of August 23, 2025.*