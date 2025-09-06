# Technical Architecture - Multimodal HNSW+ Database

## Core Architecture Decisions ✅

### Algorithm: HNSW+ 
**Why**: Industry standard, streaming updates, proven at scale
**Parameters**:
```mojo
M = 16              # Connections per node
ef_construction = 200  # Build quality
max_M0 = M * 2      # Layer 0 connections
```

### Language: Mojo Core + Rust Server
**Why**: GPU compilation (Mojo) + mature HTTP (Rust)
```
omendb/engine/  # Mojo - algorithms, SIMD, GPU
omendb/server/  # Rust - HTTP/gRPC, networking
```

### Storage: Tiered Architecture
```mojo
struct TieredStorage:
    var hot: MemoryMapped   # Last 7 days, NVMe
    var warm: DiskStorage   # 7-30 days, SSD
    var cold: ObjectStorage # >30 days, S3
```

## Multimodal Components

### 1. Vector Index (HNSW+)
```mojo
struct HNSWIndex:
    var layers: List[Graph]
    var vectors: UnsafePointer[Float32]
    var entry_point: Int
    
    fn search(query: Vector, k: Int) -> List[Result]:
        # O(log n) search complexity
```

### 2. Text Search (BM25)
```mojo
struct BM25Index:
    var inverted_index: Dict[Term, List[DocId]]
    var doc_frequencies: Dict[Term, Int]
    
    fn search(query: String) -> List[DocId]:
        # Standard BM25 scoring
```

### 3. Metadata Store (B-tree)
```mojo
struct MetadataStore:
    var columns: ColumnStore
    var indexes: Dict[String, BTree]
    
    fn filter(predicates: List[Predicate]) -> List[Id]:
        # Fast filtering on structured data
```

### 4. Query Planner
```mojo
fn plan_query(query: HybridQuery) -> ExecutionPlan:
    # Estimate selectivity
    if filter_selectivity < 0.01:
        return FilterFirst_ThenVector()
    elif text_selectivity < 0.05:
        return TextFirst_ThenVector()
    else:
        return VectorFirst_ThenFilter()
```

## Implementation Roadmap

### Phase 1: HNSW+ Core (Week 1)
```mojo
# Files to create/modify
omendb/algorithms/hnsw.mojo       # ✅ Created
omendb/algorithms/priority_queue.mojo  # TODO: Optimize
omendb/storage/vector_store.mojo  # TODO: Create
```

### Phase 2: Multimodal (Week 2-3)
```mojo
# Add components
omendb/text/bm25.mojo            # BM25 implementation
omendb/metadata/btree.mojo       # Metadata indexes
omendb/query/planner.mojo        # Query optimization
```

### Phase 3: Production (Week 4)
```mojo
# Production features
omendb/storage/tiered.mojo       # Hot/warm/cold tiers
omendb/monitoring/metrics.mojo   # Prometheus metrics
omendb/api/sql_parser.mojo       # SQL interface
```

## Performance Optimization

### SIMD Everywhere
```mojo
alias simd_width = simdwidthof[DType.float32]()

fn distance[width: Int](a: Pointer, b: Pointer) -> Float32:
    var sum = SIMD[DType.float32, width](0)
    # Vectorized operations
```

### Memory Management
```mojo
# RAII pattern for safety
struct Buffer:
    fn __del__(owned self):
        self.data.free()  # Automatic cleanup
```

### Batch Operations
```mojo
# Minimize FFI overhead
fn add_batch(vectors: List[Vector]):
    # Single FFI call, not loop
```

## Cloud Architecture

### Deployment Tiers
```yaml
Control Plane:
  - API Gateway
  - Query Router
  - Shard Manager

Data Plane:
  - Vector Nodes (GPU-capable)
  - Text Nodes (CPU-optimized)
  - Storage Nodes (High IOPS)
```

### Cost Model
| Component | Monthly Cost | Can Charge |
|-----------|-------------|------------|
| Storage (1M vectors) | $30 | $300 |
| Compute | $400 | $4000 |
| Network | $70 | $700 |
| **Total** | **$500** | **$5000** |

## Testing Strategy

### Unit Tests
```python
def test_hnsw_insertion():
    index = HNSWIndex(dimension=128)
    vectors = generate_random(1000, 128)
    index.add_batch(vectors)
    assert index.size() == 1000

def test_recall_at_10():
    # Must achieve >95% recall
```

### Benchmarks
```python
def benchmark_vs_pgvector():
    # Target: 10x faster builds
    # Target: 10x faster queries
    # Target: 10x less memory
```

### Integration Tests
```python
def test_multimodal_query():
    # Test vector + text + metadata
    results = db.search(
        vector=embedding,
        text="smartphone",
        filters={"price": {"$lt": 1000}}
    )
```

## Common Pitfalls to Avoid

1. **Don't use Mojo stdlib collections** (huge overhead)
2. **Batch all operations** (FFI overhead)
3. **Profile early** (find bottlenecks)
4. **Test incrementally** (don't wait)
5. **Document patterns** (for future AI agents)

---
*Reference this for all technical implementation decisions*