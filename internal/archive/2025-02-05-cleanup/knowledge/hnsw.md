# HNSW+ Algorithm Implementation

## Core Algorithm Overview

### Hierarchical Navigable Small World (HNSW)
**Paper**: Efficient and robust approximate nearest neighbor search using Hierarchical Navigable Small World graphs (Malkov & Yashunin, 2018)

**Key Properties**:
- O(log n) search complexity
- Natural streaming updates 
- Memory-efficient (2-4 bytes/vector)
- Industry proven (8+ years production)

## Mojo Implementation Strategy

### Core Structures
```mojo
struct HNSWIndex:
    var layers: List[Graph]         # Layer 0 (densest) to top layer
    var M: Int = 16                 # Max connections per layer
    var ef_construction: Int = 200  # Search candidates during build
    var entry_point: Int           # Top layer entry node
    var level_multiplier: Float32 = 1.0 / ln(2.0)

struct Graph:
    var nodes: UnsafePointer[Node] 
    var size: Int
    var capacity: Int

struct Node:
    var vector_id: Int            # Index into vector storage
    var connections: List[Int]    # Neighbor node IDs
    var level: Int               # Which layer this node belongs to
```

### Layer Assignment
```mojo
fn get_random_level(self) -> Int:
    # Exponential decay: most nodes at level 0
    var level = 0
    while random_float() < 0.5 and level < MAX_LEVELS:
        level += 1
    return level
```

## Search Algorithm (Core Performance)

### Search Flow
1. **Entry**: Start at top layer entry point
2. **Greedy traversal**: Each layer until layer 1
3. **Candidate selection**: ef closest candidates at layer 0
4. **Result extraction**: k best candidates

```mojo
fn search[simd_width: Int](
    self, 
    query: UnsafePointer[Float32], 
    k: Int, 
    ef: Int = -1
) -> List[SearchResult]:
    if ef == -1: ef = max(k, self.ef_construction)
    
    var current_closest = self.entry_point
    
    # Traverse layers from top to 1
    for layer in range(self.get_node_level(self.entry_point), 0, -1):
        current_closest = self.greedy_search_layer(query, current_closest, 1, layer)
    
    # Search layer 0 with ef candidates  
    var candidates = self.greedy_search_layer(query, current_closest, ef, 0)
    return candidates.take_top_k(k)
```

### Distance Calculation (SIMD Optimized)
```mojo
fn euclidean_distance[simd_width: Int](
    a: UnsafePointer[Float32],
    b: UnsafePointer[Float32], 
    dim: Int
) -> Float32:
    var sum = SIMD[DType.float32, simd_width](0)
    
    for i in range(0, dim, simd_width):
        var va = a.load[width=simd_width](i)
        var vb = b.load[width=simd_width](i)
        var diff = va - vb
        sum = diff.fma(diff, sum)  # diff² + sum
    
    return sqrt(sum.reduce_add())
```

## Insertion Algorithm

### Insert Flow
1. **Level assignment**: Random level for new node
2. **Search**: Find insertion points at each level
3. **Connect**: Add bidirectional edges
4. **Prune**: Maintain M connections using heuristic

```mojo
fn insert(mut self, vector_id: Int, vector: UnsafePointer[Float32]):
    var level = self.get_random_level()
    var new_node_id = self.add_node(vector_id, level)
    
    var current_closest = self.entry_point
    
    # Find insertion point at each level
    for lc in range(min(level, self.get_node_level(self.entry_point)), -1, -1):
        var candidates = self.greedy_search_layer(vector, current_closest, self.ef_construction, lc)
        
        # Select M neighbors using heuristic
        var selected = self.select_neighbors(candidates, self.M if lc > 0 else self.M * 2, lc)
        
        # Create bidirectional connections
        self.connect_node(new_node_id, selected, lc)
        current_closest = selected[0]  # Best candidate for next layer
    
    # Update entry point if needed
    if level > self.get_node_level(self.entry_point):
        self.entry_point = new_node_id
```

### Neighbor Selection Heuristic
```mojo
fn select_neighbors(candidates: List[SearchResult], M: Int, level: Int) -> List[Int]:
    # Heuristic 4 (best): Prefer diverse neighbors to avoid clustering
    var selected = List[Int]()
    
    for candidate in candidates:
        if selected.size >= M:
            break
            
        var should_add = True
        
        # Check diversity: reject if too close to existing neighbors
        for existing in selected:
            var dist_to_existing = self.distance(candidate.id, existing)
            if dist_to_existing < candidate.distance:
                should_add = False
                break
        
        if should_add:
            selected.append(candidate.id)
    
    return selected
```

## Performance Characteristics

### Expected Performance (vs pgvector)
| Metric | pgvector | HNSW+ (Mojo) | Improvement |
|--------|----------|--------------|-------------|
| Build rate | 10K/sec | 100K/sec | 10x |
| Search QPS | 1K/sec | 10K/sec | 10x |
| Memory/vector | 40+ bytes | 2-4 bytes | 10-20x |
| Search latency | 50-100ms | <10ms | 5-10x |

### Memory Layout
```mojo
# Optimized memory layout for cache efficiency
struct CompactNode:
    var vector_id: Int32           # 4 bytes
    var level: UInt8               # 1 byte  
    var connection_count: UInt8    # 1 byte
    var connections: UnsafePointer[Int32]  # Separate allocation
    # Total: 6 bytes + connections * 4 bytes
    # vs 40+ bytes in typical implementations
```

## GPU Acceleration Path

### Batch Operations
```mojo
fn batch_search_gpu[batch_size: Int](
    queries: UnsafePointer[Float32],
    k: Int
) -> UnsafePointer[SearchResult]:
    # Future: GPU kernel for batch distance calculations
    # Current: CPU SIMD implementation
    # Same HNSW structure, different distance kernel
```

### Why HNSW Works for GPU
- **Batch-friendly**: Multiple queries can share graph traversal
- **Memory access**: Predictable patterns good for GPU caching
- **SIMD natural**: Distance calculations are embarrassingly parallel

## Comparison with Alternatives

### HNSW vs DiskANN
| Aspect | HNSW+ | DiskANN |
|--------|-------|---------|
| Streaming updates | ✅ Natural | ❌ Batch-oriented |
| Memory efficiency | ✅ 2-4 bytes/vector | ✅ <1 byte (PQ only) |
| Search complexity | O(log n) | O(log n) |
| Industry adoption | ✅ Universal | ❌ Limited (Microsoft only) |
| Mojo advantages | ✅ SIMD, GPU future | ❌ Complex graph algorithms |

### HNSW vs CAGRA
| Aspect | HNSW+ | CAGRA |
|--------|-------|--------|
| Platform | CPU + GPU | GPU only |
| Streaming | ✅ Yes | ❌ Batch rebuilds |
| Memory | 2-4 bytes/vector | Higher (exact unclear) |
| Maturity | ✅ 8+ years | ❌ New research |

## Implementation Milestones

### Week 1: Core Structure
- [ ] Basic HNSW index with single layer
- [ ] Euclidean distance with SIMD
- [ ] Simple insertion and search

### Week 2: Multi-layer + Optimization
- [ ] Hierarchical layers with level assignment
- [ ] Neighbor selection heuristic
- [ ] Memory-mapped persistence

### Week 3: Production Features
- [ ] Concurrent search support
- [ ] Batch insertion optimization
- [ ] Zero-copy Python FFI integration

### Week 4: Benchmarking
- [ ] Performance testing vs pgvector
- [ ] Memory usage validation
- [ ] Scale testing (1M+ vectors)

---
*HNSW+ implementation guide for production vector database*