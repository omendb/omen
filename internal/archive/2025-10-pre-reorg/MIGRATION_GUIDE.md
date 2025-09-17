# DiskANN → HNSW+ Migration Guide

## Quick Reference: What Changes

### Algorithm Structure
| DiskANN | HNSW+ | Action |
|---------|-------|--------|
| Single graph | Hierarchical layers | Create layer management |
| α-RNG property | M-nearest neighbors | Simpler neighbor selection |
| RobustPrune | Heuristic pruning | Different pruning logic |
| Batch-oriented | Streaming updates | Natural insertion |

## Code Migration Patterns

### 1. Graph Structure
```mojo
# OLD: DiskANN single graph
struct DiskANNGraph:
    var nodes: UnsafePointer[Node]
    var edges: CSRGraph
    var alpha: Float32

# NEW: HNSW hierarchical
struct HNSWIndex:
    var layers: List[Graph]     # Multiple layers
    var entry_point: Int        # Top layer entry
    var M: Int = 16             # Fixed connections
```

### 2. Node Representation
```mojo
# OLD: DiskANN node
struct DiskANNNode:
    var out_edges: List[Int]
    var in_edges: List[Int]  # Bidirectional tracking

# NEW: HNSW node  
struct HNSWNode:
    var id: Int
    var level: Int
    var connections: List[List[Int]]  # Per-layer connections
```

### 3. Distance Calculations (Keep As-Is ✅)
```mojo
# REUSE: SIMD distance functions work for both
alias simd_width = simdwidthof[DType.float32]()

fn euclidean_distance[width: Int](
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32]
) -> Float32:
    # This code stays the same!
```

### 4. Search Algorithm
```mojo
# OLD: DiskANN single-level search
fn diskann_search(query: Vector, L: Int) -> List[Int]:
    # Greedy search with beam width L
    
# NEW: HNSW multi-level search
fn hnsw_search(query: Vector, ef: Int) -> List[Int]:
    # Search from top layer to bottom
    for layer in range(top_layer, -1, -1):
        candidates = search_layer(query, layer)
    return top_k(candidates)
```

### 5. Insertion Logic
```mojo
# OLD: DiskANN batch building
fn build_diskann(vectors: List[Vector]):
    # Build entire graph at once
    # Then run RobustPrune on all edges

# NEW: HNSW streaming insertion
fn insert_hnsw(vector: Vector):
    var level = get_random_level()  # Exponential decay
    # Insert at each layer independently
    for lc in range(level, -1, -1):
        connect_at_layer(vector, lc)
```

## File-by-File Migration

### Files to Keep (with modifications)
```bash
omendb/algorithms/bruteforce.mojo     # Keep for testing
omendb/algorithms/priority_queue.mojo # Useful for HNSW
omendb/core/distance/*.mojo          # All distance functions
omendb/storage/buffer.mojo           # Buffer management
```

### Files to Archive
```bash
# Move to internal/archive/reference/
omendb/algorithms/diskann*.mojo      # ✅ Already archived
omendb/algorithms/robust_prune.mojo  # DiskANN-specific
omendb/algorithms/alpha_rng.mojo     # DiskANN-specific
```

### Files to Create
```bash
omendb/algorithms/hnsw.mojo          # ✅ Created
omendb/algorithms/hnsw_search.mojo   # TODO: Optimize search
omendb/algorithms/hnsw_insert.mojo   # TODO: Optimize insertion
```

## Integration Points

### 1. Update native.mojo
```mojo
# OLD: Using DiskANN
from .algorithms.diskann import DiskANNIndex

# NEW: Using HNSW
from .algorithms.hnsw import HNSWIndex

# Change initialization
# var index = DiskANNIndex(dim, alpha=1.2)
var index = HNSWIndex(dim, M=16)
```

### 2. Python Bindings
```python
# API stays the same!
db = omendb.DB(dimension=1536)
db.add(vectors, ids)
results = db.search(query, k=10)
```

### 3. Storage Layer
```mojo
# Graph storage changes
# OLD: CSR format for DiskANN
# NEW: Adjacency lists per layer for HNSW
```

## Testing Migration

### 1. Correctness Tests
```python
def test_migration_correctness():
    # Build same dataset with both algorithms
    diskann_results = old_index.search(query)
    hnsw_results = new_index.search(query)
    
    # Accept slightly different results (both approximate)
    assert recall(diskann_results, hnsw_results) > 0.90
```

### 2. Performance Tests
```python
def test_performance_improvement():
    # HNSW should be faster for:
    # - Insertion (streaming vs batch)
    # - Search (hierarchical vs flat)
    # - Memory usage (less metadata)
```

## Migration Checklist

### Week 1: Core Structure
- [x] Create HNSWIndex struct
- [ ] Implement layer management
- [ ] Add level assignment logic
- [ ] Basic insertion working

### Week 2: Search Optimization  
- [ ] Implement efficient layer search
- [ ] Add priority queue for candidates
- [ ] Optimize neighbor selection
- [ ] Test recall accuracy

### Week 3: Integration
- [ ] Update native.mojo imports
- [ ] Maintain API compatibility
- [ ] Add migration tool for existing indexes
- [ ] Performance benchmarking

### Week 4: Production
- [ ] Memory optimization
- [ ] Parallel insertion
- [ ] Metadata filtering
- [ ] Documentation

## Key Differences to Remember

1. **HNSW is simpler** - No alpha parameter, no RobustPrune
2. **Insertion is natural** - No batch building required
3. **Search is hierarchical** - Start from top layer
4. **Memory is predictable** - M connections per node
5. **Updates are supported** - Can insert anytime

---
*Follow this guide when migrating DiskANN code to HNSW+*