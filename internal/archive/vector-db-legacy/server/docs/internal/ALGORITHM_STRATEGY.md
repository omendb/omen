# Algorithm Strategy: HNSW Implementation & RoarGraph Archive

## Executive Summary

**Decision**: Replace RoarGraph with HNSW in public OmenDB for competitive performance
**Rationale**: RoarGraph is 0.4-0.6x slower than HNSW for pure vector search
**Future**: Keep RoarGraph in server edition for cross-modal search (its intended purpose)

## Performance Reality Check

### Current RoarGraph Performance
- **Construction**: 15-20K vec/s (with O(n²) training overhead)
- **Query**: 0.14ms @50K vectors
- **Issue**: Training query computation dominates construction time

### HNSW Expected Performance
- **Construction**: 50K+ vec/s (O(log n) incremental)
- **Query**: 0.01-0.03ms @50K vectors
- **Advantage**: No training phase, proven algorithm

### Faiss Baseline
- **Construction**: 28-52K vec/s (HNSW implementation)
- **Query**: 0.01-0.03ms
- **Target**: Match or exceed with Mojo optimizations

## Implementation Strategy

### Phase 1: Clean HNSW Implementation
1. **Remove RoarGraph from native.mojo**
2. **Implement HNSW from scratch in Mojo**
3. **Archive RoarGraph in omendb-server/**

### Phase 2: Mojo-Specific Optimizations
1. **Compile-time graph structure**
2. **Hardware-adaptive SIMD**
3. **Cache-aligned memory layout**
4. **Zero-cost abstractions**

### Phase 3: Competitive Features
1. **Persistence** (.omen format)
2. **Collections** (namespaces)
3. **Metadata filtering**
4. **Multiple distance metrics**

## Code Architecture

```
omendb/algorithms/
├── brute_force.mojo      # Keep for small datasets
├── hnsw.mojo            # NEW: Core implementation
├── hnsw_index.mojo      # NEW: Index structure
└── distance.mojo        # Shared distance functions

omendb-server/algorithms/archived/
├── roar_graph.mojo      # Archived for future cross-modal
├── bipartite.mojo       # RoarGraph support
└── README.md           # Explanation of archive
```

## Competitive Positioning

### vs Faiss
- **Goal**: Match or exceed performance
- **Advantage**: Better developer experience, Mojo optimizations
- **Strategy**: HNSW + compile-time optimizations

### vs ChromaDB
- **Goal**: Feature parity + better performance
- **Missing**: Collections, metadata filtering, persistence
- **Strategy**: Implement core features first

### vs Pinecone/Qdrant
- **Goal**: Embedded alternative with comparable features
- **Advantage**: No server required, instant startup
- **Strategy**: Focus on embedded use cases

## Success Metrics

1. **Performance**: ≥50K vec/s construction, ≤0.03ms query
2. **Features**: Collections, filtering, persistence
3. **Correctness**: 100% recall for exact search
4. **Stability**: No crashes, proper error handling