# HNSW+ Migration from DiskANN Codebase

## Current Codebase Analysis

### What We Keep ✅
```mojo
// Distance functions (excellent SIMD implementation)
omendb/engine/omendb/distance/
├── euclidean.mojo       # Keep: optimized SIMD
├── cosine.mojo          # Keep: optimized SIMD  
├── dot_product.mojo     # Keep: optimized SIMD

// Memory management patterns
omendb/engine/omendb/storage/
├── buffer.mojo          # Keep: async flush patterns work
├── memory_pool.mojo     # Keep: allocation strategies good

// Python FFI layer
omendb/engine/omendb/bindings/
├── python_interface.mojo # Keep: zero-copy interface proven
```

### What We Replace ❌
```mojo
// Algorithm-specific code (DiskANN → HNSW+)
omendb/engine/omendb/diskann/
├── vamana.mojo          # Replace: DiskANN algorithm
├── robust_prune.mojo    # Replace: different neighbor selection  
├── alpha_rng.mojo       # Replace: HNSW uses different RNG

// Graph structures (different topology)
omendb/engine/omendb/graph/
├── bidirectional_graph.mojo  # Replace: HNSW uses different structure
├── node.mojo                 # Modify: different node representation
```

## Migration Strategy

### Phase 1: Core HNSW+ Implementation (Week 1)

#### New File Structure
```
omendb/engine/omendb/hnsw/
├── index.mojo           # Main HNSWIndex struct
├── layer.mojo           # Layer management
├── search.mojo          # Search algorithms  
├── insert.mojo          # Insertion logic
├── neighbors.mojo       # Neighbor selection heuristics
└── persistence.mojo     # Memory-mapped storage
```

#### Key Structure Changes
```mojo
// OLD: DiskANN bidirectional graph
struct DiskANNGraph:
    var nodes: UnsafePointer[DiskANNNode] 
    var edges: UnsafePointer[Edge]
    var alpha: Float32

struct DiskANNNode:
    var vector_id: Int
    var out_edges: List[Int]  # Only outgoing
    var in_edges: List[Int]   # Only incoming (bidirectional)

// NEW: HNSW hierarchical layers
struct HNSWIndex:
    var layers: List[HNSWLayer]
    var M: Int = 16
    var ef_construction: Int = 200  
    var entry_point: Int

struct HNSWNode:
    var vector_id: Int
    var level: Int
    var connections: List[Int]  # Bidirectional by nature
```

### Phase 2: Algorithm Translation (Week 1-2)

#### Search Algorithm Migration
```mojo
// OLD: DiskANN greedy search with alpha parameter
fn diskann_search(
    query: UnsafePointer[Float32],
    start_node: Int,
    L: Int,  # Search list size
    alpha: Float32  # DiskANN-specific parameter
) -> List[SearchResult]:
    # Single-layer greedy search with alpha pruning

// NEW: HNSW hierarchical search
fn hnsw_search(
    query: UnsafePointer[Float32], 
    k: Int,
    ef: Int = -1  # Search parameter (replaces L)
) -> List[SearchResult]:
    # Multi-layer search from top to bottom
    var current_closest = self.entry_point
    
    # Descend layers
    for layer in range(self.get_max_layer(), 0, -1):
        current_closest = self.search_layer(query, current_closest, 1, layer)
    
    # Search layer 0 with ef candidates
    return self.search_layer(query, current_closest, max(ef, k), 0)
```

#### Insertion Algorithm Migration
```mojo
// OLD: DiskANN insertion with RobustPrune
fn diskann_insert(vector_id: Int, vector: UnsafePointer[Float32]):
    # Find insertion point with greedy search
    # Use RobustPrune to maintain alpha-RNG property
    # Update bidirectional edges

// NEW: HNSW insertion with level assignment  
fn hnsw_insert(vector_id: Int, vector: UnsafePointer[Float32]):
    var level = self.get_random_level()  # Exponential distribution
    
    # Search for insertion points at each level
    var candidates = self.search_for_insertion(vector, level)
    
    # Connect using HNSW heuristic (not RobustPrune)
    for lc in range(level + 1):
        var selected = self.select_neighbors_heuristic(candidates[lc], self.get_M(lc))
        self.connect_bidirectional(vector_id, selected, lc)
```

### Phase 3: Performance Optimization (Week 2)

#### Memory Layout Optimization
```mojo
// Optimize for cache efficiency (HNSW nodes accessed together)
struct CompactHNSWNode:
    var vector_id: Int32           # 4 bytes
    var level: UInt8               # 1 byte
    var connection_count: UInt8    # 1 byte  
    # Total: 6 bytes vs DiskANN's 16+ bytes per node
    
    # Connections stored separately for memory efficiency
    var connections: UnsafePointer[Int32]  
```

#### SIMD Distance Reuse
```mojo
// Keep existing optimized distance functions
alias simd_width = simdwidthof[DType.float32]()

fn euclidean_distance[width: Int](
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32], 
    dim: Int
) -> Float32:
    # EXISTING CODE - already optimized, no changes needed
```

### Phase 4: Integration & Testing (Week 3)

#### API Compatibility Layer
```mojo
// Maintain same Python interface
@export
fn search_vectors(
    query_ptr: UnsafePointer[Float32],
    k: Int32,
    results_ptr: UnsafePointer[Int32]
):
    # OLD: Called diskann_search internally
    # NEW: Call hnsw_search internally
    # Same external interface, different implementation
```

#### Migration Testing Strategy
```python
# Test suite to validate migration
def test_hnsw_vs_diskann_accuracy():
    # Compare search results on same dataset
    # Accept slightly different results (both approximate)
    # Ensure recall@10 >= 95% for both
    
def test_performance_improvement():  
    # Measure build time: HNSW should be 2-5x faster
    # Measure query time: Similar or better
    # Measure memory: HNSW should use less
```

## Migration Timeline

### Week 1: Foundation
- [ ] Create `omendb/hnsw/` directory structure
- [ ] Implement basic HNSWIndex struct and layer management
- [ ] Port distance functions (no changes needed)
- [ ] Basic single-layer search working

### Week 2: Algorithm Complete
- [ ] Multi-layer hierarchical search
- [ ] Level assignment and insertion logic
- [ ] Neighbor selection heuristics
- [ ] Memory-mapped persistence

### Week 3: Integration
- [ ] Python FFI compatibility maintained
- [ ] Performance benchmarking vs DiskANN
- [ ] Memory usage optimization
- [ ] Production-ready error handling

### Week 4: Metadata Extension
- [ ] Add metadata filtering hooks
- [ ] Implement hybrid search capabilities  
- [ ] ZenDB pattern integration
- [ ] Full multimodal API

## Risk Mitigation

### Compatibility Risks
- **API changes**: Keep same Python interface, change internals only
- **File format**: Implement migration tool for existing indexes
- **Performance regression**: Benchmark at each step

### Technical Risks  
- **HNSW complexity**: Well-documented algorithm, many references
- **Memory efficiency**: HNSW typically more efficient than DiskANN
- **Build time**: HNSW should be faster (no alpha-RNG complexity)

---
*Complete DiskANN → HNSW+ migration strategy*