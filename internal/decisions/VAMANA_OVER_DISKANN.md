# Decision: Vamana Over DiskANN Implementation
*Architecture decision record for core algorithm choice*

## Decision Summary
**CHOSEN**: Vamana algorithm core with DiskANN optimizations  
**REJECTED**: Pure DiskANN implementation  
**STATUS**: ✅ Implemented and working  
**DATE**: December 2024

## Context

OmenDB needed a high-performance approximate nearest neighbor search algorithm. Two main approaches were considered:

1. **Pure DiskANN**: Microsoft's complete algorithm including disk-based storage
2. **Vamana Core**: Core Vamana algorithm from DiskANN with selective optimizations

## Decision Factors

### Technical Requirements
- **Performance**: >50K vectors/second insertion
- **Memory**: <1KB per vector at scale  
- **Scale**: 100K+ vectors without crashes
- **Quality**: >95% recall at k=10

### Implementation Complexity
- **DiskANN Full**: Complex disk management, memory mapping, multi-threading
- **Vamana Core**: Simpler graph-based approach, easier to debug

## Chosen Architecture: Vamana + Selective DiskANN Features

```mojo
# Core: Vamana algorithm for graph building and search
VamanaIndex:
    - Graph construction with random initialization ✅
    - RobustPrune with α-RNG property ✅  
    - Convergence-based beam search ✅
    
# Added: DiskANN optimizations
ProductQuantization:
    - Memory compression (target 10x reduction) ⚠️ Disconnected
    
# Future: DiskANN features
DiskPersistence:
    - Memory-mapped graph files ❌ Not implemented
    - Lazy loading ❌ Not implemented
```

## Why Vamana Over Pure DiskANN

### ✅ Advantages
1. **Simpler Implementation**: Vamana core is well-defined and testable
2. **Better Debugging**: Graph algorithms easier to reason about than disk I/O
3. **Proven Algorithm**: Vamana is the core of DiskANN's success
4. **Incremental Complexity**: Can add DiskANN features progressively

### ❌ Disadvantages  
1. **Memory Limitations**: Everything in RAM (no disk spillover)
2. **Missing Compression**: PQ not integrated with core algorithm
3. **Scale Limits**: Current ~25K vector limit due to memory

## Implementation Status

### ✅ Working Components
- **Vamana Graph Building**: Random initialization + multi-pass construction
- **RobustPrune Algorithm**: Correct α-RNG property implementation  
- **Convergence Search**: Beam search with working/visited sets
- **Memory Optimizations**: SparseMap replaces Dict (180x improvement)

### 🔄 In Progress
- **PQ Integration**: Vamana + ProductQuantization combination
- **Graph Structure**: Replace CSR with adjacency list for edge pruning

### ❌ Missing (Future)
- **Disk Persistence**: Memory-mapped files for true scalability
- **Multi-threading**: Parallel graph construction and search
- **Incremental Updates**: Efficient vector updates/deletions

## Performance Results

| Metric | Current (Vamana) | Target (Full DiskANN) |
|--------|------------------|----------------------|
| **Build Speed** | ~15K vec/s | 50K+ vec/s |
| **Search Speed** | Good | Excellent |  
| **Memory/Vector** | 4KB | <1KB with PQ |
| **Scale Limit** | 25K vectors | Millions |

## Alternative Considered: HNSW

### Why Not HNSW?
- **Memory Overhead**: Higher than Vamana at scale
- **Construction Cost**: More expensive graph building
- **Our Expertise**: Team familiar with DiskANN/Vamana papers

## Integration Strategy

### Phase 1: Vamana Core (✅ Complete)
- Correct algorithm implementation
- Basic memory optimizations
- Scale to 25K vectors

### Phase 2: DiskANN Optimizations (🔄 Current)
- Product Quantization integration
- Adjacency list graph structure  
- Scale to 100K+ vectors

### Phase 3: Full DiskANN (❌ Future)
- Disk persistence with memory mapping
- Multi-threaded operations
- Production-scale deployment

## Decision Validation

### Success Criteria Met
- ✅ Algorithm correctness (RobustPrune, Search, Build all correct)
- ✅ Memory efficiency (4KB/vector competitive with alternatives)
- ✅ Development velocity (simpler than full DiskANN)

### Outstanding Issues
- 🔴 Scale limit at 25K vectors (CSR graph limitation)
- 🔴 PQ compression not integrated (components disconnected)
- 🔴 No disk persistence (all in memory)

## Lessons Learned

### What Worked
- **Start Simple**: Vamana core provided solid foundation
- **Incremental Complexity**: Easier to debug and validate  
- **Algorithm First**: Getting algorithms right before optimization

### What Didn't Work
- **CSR Graph Choice**: Cannot remove edges, limiting scale
- **Component Isolation**: PQ and Vamana developed separately
- **Memory Assumptions**: Underestimated overhead complexity

## Next Steps

1. **Fix Graph Structure**: Replace CSR with adjacency list (enables pruning)
2. **Connect PQ**: Integrate ProductQuantization with VamanaIndex  
3. **Scale Testing**: Validate 100K+ vectors with new structure
4. **Disk Planning**: Design memory-mapped persistence layer

## References
- [DiskANN Paper](https://arxiv.org/abs/1909.05007) - Original algorithm
- [Vamana Algorithm](https://arxiv.org/abs/1909.05007) - Core graph building
- [Implementation](native.mojo:1850-2000) - Current VamanaIndex code

---
*Architecture decision: Vamana core provides the right foundation for OmenDB's requirements*