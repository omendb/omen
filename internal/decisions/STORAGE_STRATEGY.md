# Decision: Storage Strategy - Memory First, Disk Later
*Storage architecture decision for OmenDB vector engine*

## Decision Summary
**CHOSEN**: Memory-first storage with planned disk persistence  
**REJECTED**: Disk-only storage, Hybrid from start  
**STATUS**: ✅ Phase 1 implemented, Phase 2 in progress  
**DATE**: December 2024

## Context

OmenDB needed a storage strategy that balances:
- **Performance**: Fast vector operations and search
- **Scale**: Handle 100K+ vectors efficiently  
- **Complexity**: Manageable implementation and debugging
- **Memory**: Reasonable memory usage per vector

## Options Considered

### Option 1: Pure Disk Storage (DiskANN Style)
```
All data on disk:
├── Vectors: Memory-mapped binary files
├── Graph: Memory-mapped adjacency structures  
├── Metadata: SQLite or embedded DB
└── Indices: B+ trees for lookups
```

### Option 2: Pure Memory Storage  
```
All data in RAM:
├── Vectors: In-memory arrays
├── Graph: In-memory adjacency structures
├── Metadata: Hash maps/dictionaries  
└── No persistence (ephemeral)
```

### Option 3: Memory-First Hybrid (CHOSEN)
```
Phase 1 - Memory (Current):
├── Vectors: In-memory with efficient structures
├── Graph: Memory-optimized adjacency representation
├── Metadata: Custom SparseMap (not Dict)
└── Buffer: Batch operations for performance

Phase 2 - Disk Spillover (Planned):  
├── Hot Data: Keep in memory (recent vectors)
├── Cold Data: Spill to memory-mapped files
├── Graph: Hybrid in-memory/disk representation
└── Metadata: Disk-backed with memory cache
```

## Why Memory-First Strategy

### ✅ Advantages

#### Development Velocity
- **Simpler Implementation**: No file I/O, memory mapping complexity
- **Easier Debugging**: All data structures visible in memory
- **Faster Iteration**: No disk format compatibility issues

#### Performance Benefits
- **Zero I/O Overhead**: All operations in memory bandwidth
- **No Page Faults**: Predictable performance characteristics  
- **Batch Optimizations**: Can optimize memory access patterns

#### Algorithm Validation
- **Focus on Correctness**: Validate algorithms before adding disk complexity
- **Memory Profiling**: Understand actual memory usage patterns
- **Bottleneck Identification**: Find real performance limits

### ❌ Disadvantages
- **Scale Limits**: Everything must fit in available RAM
- **No Persistence**: Data lost on restart (development only)
- **Memory Pressure**: Large datasets require substantial RAM

## Current Implementation

### Memory Structures
```mojo
# Vector Storage
struct VectorBuffer:
    var vectors: List[List[Float32]]    # Raw vectors in memory
    var ids: SparseMap                  # String->Int mapping (not Dict!)
    var metadata: SparseMap             # Vector metadata
    
# Graph Storage  
struct VamanaGraph:
    var adjacency: CSRGraph             # ⚠️ Cannot prune edges (being replaced)
    var node_count: Int
    var edge_capacity: Int
```

### Memory Optimizations Applied
1. **Dict Replacement**: SparseMap reduces overhead 180x (8KB → 44 bytes)
2. **Batch Operations**: Reduce FFI overhead from 8KB to 1.5KB per batch
3. **Contiguous Arrays**: Better cache performance than fragmented Lists

### Current Limitations
- **25K Vector Limit**: CSR graph cannot remove edges → unbounded growth
- **No Persistence**: All data ephemeral (development constraint)
- **Memory Growth**: Some structures still grow unbounded

## Planned Disk Integration (Phase 2)

### Hybrid Architecture Design
```mojo
# Hot/Cold Data Separation
struct HybridVectorStore:
    # Hot data (recent, frequently accessed)
    var hot_vectors: List[List[Float32]]     # Keep in memory
    var hot_graph: AdjacencyListGraph       # Memory-resident graph
    
    # Cold data (older, less accessed)
    var cold_vectors: MemoryMappedFile      # Disk-backed vectors
    var cold_graph: DiskBasedGraph          # Memory-mapped graph
    
    # Unified interface
    fn search(query: List[Float32]) -> List[Result]:
        # Search hot data first (fast)
        hot_results = self.hot_graph.search(query)
        # Extend with cold data if needed (slower)  
        if len(hot_results) < k:
            cold_results = self.cold_graph.search(query)
        return merge(hot_results, cold_results)
```

### Disk Storage Format (Planned)
```
data/
├── vectors/
│   ├── hot.mmap          # Recent vectors (memory-mapped)
│   ├── cold_001.mmap     # Older vector chunks
│   └── cold_002.mmap
├── graph/
│   ├── adjacency.mmap    # Graph structure
│   └── metadata.mmap     # Node/edge metadata
└── indices/
    ├── vector_ids.idx    # ID → position mapping
    └── timestamps.idx    # Access time tracking
```

## Migration Strategy

### Phase 1: Memory Optimization (✅ Current)
- Replace inefficient structures (Dict → SparseMap)
- Fix graph limitations (CSR → AdjacencyList)
- Scale to 100K vectors in memory
- Perfect algorithms before disk complexity

### Phase 2: Selective Disk Storage (⚠️ Next)  
- Add memory-mapped vector storage
- Implement hot/cold data separation  
- Maintain memory performance for active data
- Add optional persistence for development

### Phase 3: Production Disk Storage (❌ Future)
- Full disk-based storage with memory caching
- Efficient startup from persisted state
- Transaction logging for consistency  
- Multi-process safe file locking

## Performance Implications

### Current (Memory-Only) Performance
| Operation | Speed | Memory Usage |
|-----------|-------|--------------|
| **Vector Insert** | ~15K vec/s | 4KB/vector |
| **Search** | Excellent | Zero I/O |
| **Startup** | Instant | No loading |

### Planned (Hybrid) Performance  
| Operation | Speed | Memory Usage |
|-----------|-------|--------------|
| **Hot Insert** | ~15K vec/s | 1KB/vector (with PQ) |
| **Cold Insert** | ~5K vec/s | Minimal memory |
| **Hot Search** | Excellent | Zero I/O |
| **Cold Search** | Good | Some I/O |

## Alternative Rejected: Disk-First

### Why Not Disk-First?
- **Implementation Complexity**: Memory mapping, file management, consistency
- **Development Overhead**: Harder to debug, slower iteration cycles  
- **Premature Optimization**: Algorithms need validation before disk complexity
- **Performance Uncertainty**: Unknown I/O patterns until algorithms stable

## Validation Metrics

### Phase 1 Success Criteria (Current)
- ✅ Handle 25K vectors efficiently in memory
- ✅ Memory usage <5KB per vector competitive
- ✅ Algorithm correctness validated
- 🔄 Scale to 100K+ vectors (in progress)

### Phase 2 Success Criteria (Planned)
- Handle 1M+ vectors with memory/disk hybrid
- <2KB memory per vector average (including disk-backed)
- Search performance degradation <20% vs pure memory
- Data persistence across restarts

## Decision Impact

### Positive Outcomes  
- **Rapid Algorithm Development**: Focus on correctness over I/O complexity
- **Clear Memory Profile**: Understand actual usage patterns  
- **Performance Baseline**: Know memory-bound performance limits

### Technical Debt Created
- **No Persistence**: Development data lost on restart
- **Memory Scaling**: Cannot handle massive datasets yet
- **Single-Process**: No shared storage across processes

## Next Actions

1. **Complete Memory Phase**: Fix CSR→AdjacencyList, integrate PQ compression
2. **Design Disk Phase**: Plan memory-mapped storage architecture  
3. **Prototype Hybrid**: Test hot/cold data separation performance
4. **Migrate Gradually**: Add disk backing without breaking memory performance

## References
- [Memory-Mapped Files](https://en.wikipedia.org/wiki/Memory-mapped_file)
- [DiskANN Storage](https://github.com/microsoft/DiskANN) - Reference disk implementation
- [SparseMap Implementation](native.mojo) - Current memory optimization

---
*Storage decision: Memory-first enables algorithm validation, disk-second enables scale*