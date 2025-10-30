# Custom HNSW: SOTA Research & Implementation Guide (Oct 30, 2025)

**Research Goal**: Design custom HNSW implementation incorporating SOTA optimization techniques
**Context**: Week 8 profiling revealed 76% allocations + 23.41% cache misses in hnsw_rs library (cannot optimize)
**Decision**: Build custom HNSW for full control over cache, allocation, and SOTA features
**Target Performance**: 1000+ QPS (vs current 581 QPS, Qdrant's 626 QPS)

---

## Executive Summary

### Key Findings

1. **ALL serious competitors use custom HNSW implementations** (Qdrant, Milvus, Weaviate, LanceDB)
2. **Memory fetching is THE bottleneck**: 40% of query time spent fetching vectors (NeurIPS 2022)
3. **SOTA techniques identified**:
   - Delta Encoding: 30% memory reduction (Qdrant 1.13)
   - Extended RaBitQ: 3-9 bit arbitrary quantization (SIGMOD 2025)
   - AVX512: 20-30% speedup over AVX2 (Milvus Knowhere)
   - Graph reordering: 1.15-1.18x speedup via BFS/DFS layout
   - Arena allocators: Eliminate per-node allocation overhead

4. **Critical optimizations for custom implementation**:
   - Cache-line alignment (64-byte boundaries)
   - Software prefetching (_mm_prefetch for next nodes)
   - Contiguous memory layout (flattened index, not pointer-based)
   - SIMD distance calculations (AVX2/AVX512)
   - Thread-local buffers (eliminate query allocations)

---

## Competitor Analysis

### Qdrant (Rust, 25.5k GitHub stars) ⭐ BEST IN CLASS

**Architecture**:
- Custom HNSW in Rust with GPU acceleration (NVIDIA, AMD, Intel via Vulkan)
- 10x faster GPU indexing vs CPU for equivalent hardware price
- Custom storage backend (fixed-block, replaces RocksDB)

**Key Innovations (2025)**:

1. **Delta Encoding for HNSW** (Oct 2024):
   - Compress graph links by storing differences between consecutive values
   - **30% memory reduction** with no performance degradation
   - Low CPU overhead (unlike gzip/lz4) - crucial for frequent traversal
   - **Lesson**: HNSW graph structure has patterns suitable for compression

2. **GPU Indexing**:
   - Custom, vendor-agnostic implementation (not third-party library)
   - Supports all major GPU vendors via Vulkan API
   - **10x faster index building** than CPU

3. **Custom Storage Engine**:
   - Fixed-block architecture with data/mask/region/tracker layers
   - Constant-time read/write regardless of data size
   - Eliminates RocksDB compaction latency spikes

**Performance**: 626 QPS (our benchmark target)

**Source Code**: https://github.com/qdrant/qdrant (Rust, 87.6%)
- HNSW likely in `src/` directory (custom implementation)
- Built from scratch, not wrapping hnswlib

**What We Should Copy**:
- ✅ Delta encoding for graph compression
- ✅ Custom storage backend for predictable latency
- ✅ Rust implementation patterns

---

### Milvus Knowhere (C++, Vector DB market leader)

**Architecture**:
- Knowhere: Separate vector search engine
- Integrates FAISS, HNSW, DiskANN
- C++ with extensive SIMD optimizations

**Key Innovations**:

1. **SIMD Instruction Set Support** (2024-2025):
   - SSE4.2, AVX2, AVX512 support
   - **AVX512: 20-30% faster** than AVX2
   - Auto-detection: Chooses best SIMD for CPU
   - PowerPC architecture support

2. **HNSW INT8 Support**:
   - Quantized integer arithmetic
   - Reduces memory + improves cache efficiency

3. **Bitset Mechanism**:
   - Soft-delete vectors (mark as "1" in bitset)
   - Skip during search without rebuilding index
   - Critical for real-time updates

4. **Query Batching**:
   - Combines small queries into large batches
   - "Improves QPS obviously" (per docs)
   - Better instruction-level parallelism

**Performance**:
- hnswlib faster than FAISS HNSW (internal testing)
- AVX512: +20-30% vs AVX2

**Source Code**: https://github.com/milvus-io/knowhere (C++)
- Uses hnswlib-derived code with custom optimizations

**What We Should Copy**:
- ✅ AVX512 support (if not using AVX2 yet)
- ✅ Query batching for QPS improvement
- ✅ Bitset mechanism for soft-deletes

---

### LanceDB (Rust, Columnar format for ML/LLMs)

**Architecture**:
- Lance: Columnar data format in Rust
- Custom HNSW implementation (IVF_HNSW_SQ index)
- Native Rust, Python, TypeScript support

**Key Insights**:
- "Vast majority of database vendors opt for custom implementation of HNSW"
- Pinecone and Lance both **rewritten from ground up in Rust**
- Focus on columnar storage for ML workloads

**Performance**: Not extensively documented

**Source Code**:
- https://github.com/lancedb/lance (core format)
- https://github.com/lancedb/lancedb (database)

**What We Should Copy**:
- ✅ Rust-first approach (we're already doing this)
- ✅ Custom implementation vs library wrapping

---

### Weaviate (Go, Enterprise-focused)

**Architecture**:
- Custom HNSW in Go
- Overcomes hnswlib limitations (durability, CRUD, pre-filtering)

**Key Innovations**:

1. **Memory Allocation Optimization** (2024):
   - Switched from dynamic → static allocations
   - Instructs Go runtime to allocate exact number of elements
   - **10-30% memory reduction**

2. **Cache Prefilling**:
   - Proposed: HNSW_STARTUP_WAIT_FOR_VECTOR_CACHE=true
   - Synchronous cache loading before accepting queries
   - Prevents zero-downtime issues during rolling updates

**Performance**: Thousands of queries per second (unspecified)

**Source Code**: https://github.com/weaviate/weaviate (Go)

**What We Should Copy**:
- ✅ Static allocation patterns (pre-allocate, reuse)
- ✅ Cache prefilling strategy (warm cache before queries)

---

## SOTA Algorithms & Techniques

### 1. Extended RaBitQ (SIGMOD 2025) ⭐ HIGHEST PRIORITY

**Paper**: "Practical and Asymptotically Optimal Quantization of High-Dimensional Vectors"
**Authors**: Jianyang Gao et al., NTU Vector Database Group
**Status**: Accepted to SIGMOD 2025

**Key Innovations**:

1. **Arbitrary Compression Rates**:
   - Supports **3, 4, 5, 7, 8, 9 bits per dimension**
   - Current binary quantization: 1 bit (32x compression)
   - E-RaBitQ: 4x to 32x compression (configurable)

2. **No Computational Overhead**:
   - "Computation is exactly the same as classical scalar quantization"
   - Drop-in replacement for our current BinaryQuantization
   - Asymptotically optimal error bounds

3. **Best in Low-Bit Regime**:
   - "Especially significant improvement in 2-bit to 6-bit" range
   - High recall without reranking (no computational penalty)
   - Better accuracy at same memory footprint

**Implementation**:
- C++ (96% of codebase)
- Requires AVX512 instruction set
- IVF index structure (configurable cluster count)

**Performance Claims**:
- Better than scalar quantization at equivalent compression
- No additional distance computation overhead
- Proven via peer review (SIGMOD acceptance)

**GitHub**: https://github.com/VectorDB-NTU/Extended-RaBitQ
**ArXiv**: https://arxiv.org/pdf/2409.09913

**Integration Plan**:
1. Replace `BinaryQuantization` struct with `ExtendedRaBitQ`
2. Add compression rate parameter (default: 4-bit = 8x compression)
3. Implement AVX512 distance calculations (reuse SIMD patterns)
4. Backward compatibility: Support current 1-bit mode

**Expected Impact**:
- 2-3x better accuracy at same memory vs binary quantization
- OR: 2x less memory at same accuracy
- Critical for competitive positioning ("SOTA quantization")

---

### 2. Delta Encoding for HNSW Graph (Qdrant, Oct 2024) ⭐ HIGH PRIORITY

**Problem**: HNSW graph links consume significant memory (1.5-2x vector data)

**Solution**: Store differences between consecutive node IDs instead of absolute IDs

**Why It Works**:
- HNSW neighbors often have similar IDs (construction order locality)
- Differences are small → compress with fewer bits
- Example: Neighbors [1000, 1003, 1007] → store [1000, +3, +4]

**Implementation**:
- Simple integer delta encoding (not gzip/lz4)
- Low CPU overhead (critical for traversal)
- Compress on save, decompress on load (or lazy)

**Performance (Qdrant)**:
- **30% memory reduction** for HNSW graph
- No measurable performance degradation
- Works with all graph sizes

**Integration Plan**:
1. Store neighbor lists as delta-encoded arrays
2. Decompress during traversal (or pre-decompress hot nodes)
3. Compress during serialization

**Expected Impact**:
- 30% less memory for HNSW graph
- Enables larger in-memory graphs (10M → 14M vectors)

---

### 3. Graph Reordering for Cache Efficiency (NeurIPS 2022) ⭐ MEDIUM PRIORITY

**Paper**: "Graph Reordering for Cache-Efficient Near Neighbor Search"
**Key Finding**: 40% of HNSW query time spent fetching vectors from memory

**Problem**: HNSW nodes allocated in insertion order → random access → cache misses

**Solution**: Reorder node IDs based on graph structure

**Approaches**:

1. **BFS Ordering**:
   - Traverse graph breadth-first, assign consecutive IDs
   - Neighbors likely in same cache line
   - **1.18x speedup** in graph processing tasks

2. **DFS Ordering**:
   - Depth-first traversal
   - **1.15x speedup**

3. **RCM (Reverse Cuthill-McKee)**:
   - Bandwidth reduction algorithm
   - **1.15x speedup**

**Implementation**:
1. Build HNSW with temporary IDs
2. Run BFS/DFS from entry point, assign new IDs
3. Reorder vectors and neighbor lists
4. Store reordered layout

**Expected Impact**:
- 15-18% query speedup (via better cache locality)
- Complements SIMD and other optimizations
- One-time cost during index build

---

### 4. Flattened Index with Prefetching (Industry Standard)

**Technique**: Store HNSW nodes + vectors in contiguous memory (not pointers)

**Benefits**:
1. **Hardware prefetching**: Sequential memory access triggers CPU prefetchers
2. **Cache-line alignment**: Pack nodes into 64-byte boundaries
3. **Software prefetching**: `_mm_prefetch` for upcoming nodes

**Implementation Pattern**:

```rust
#[repr(align(64))]  // Cache-line aligned
struct HNSWNode {
    vector: [f32; 1536],           // 6144 bytes
    neighbors: [u32; MAX_NEIGHBORS], // e.g., 64 neighbors = 256 bytes
    level: u8,
    // Total: ~6400 bytes → allocate in contiguous arena
}

// During traversal:
fn search_layer(&self, query: &[f32], entry_id: usize) {
    let next_id = self.get_next_candidate();

    // Prefetch next node (20-30 elements ahead)
    let prefetch_id = self.peek_candidate_at(20);
    unsafe {
        use core::arch::x86_64::_mm_prefetch;
        let node_ptr = self.nodes.as_ptr().add(prefetch_id);
        _mm_prefetch(node_ptr as *const i8, _MM_HINT_T0);
    }

    // Process current node...
}
```

**Expected Impact**:
- 10-20% query speedup (mask memory latency)
- Critical for our 23.41% LLC miss rate

---

### 5. Arena Allocators for Graph Construction

**Problem**: Per-node allocations cause fragmentation + overhead

**Solution**: Allocate large blocks, sub-allocate from arena

**Rust Options**:

1. **typed-arena** (Single-type arena):
   - Perfect for HNSW nodes (all same type)
   - Runs Drop implementations (unlike bumpalo)
   - Enables cycles (graph structures)
   - Example:
     ```rust
     use typed_arena::Arena;

     let arena = Arena::new();
     let node1 = arena.alloc(HNSWNode { ... });
     let node2 = arena.alloc(HNSWNode { ... });
     // All nodes deallocated together when arena dropped
     ```

2. **bumpalo** (Heterogeneous arena):
   - More flexible but less type-safe
   - Doesn't run Drop by default
   - Most downloaded arena crate

**Integration Plan**:
1. Use typed-arena for HNSW node allocation
2. Pre-allocate arena capacity: `Arena::with_capacity(10_000_000)`
3. Single deallocation on index drop

**Expected Impact**:
- Eliminate 5.6M allocations (76% of current total)
- 10-20% build speedup
- 5-10% query speedup (better locality)

---

## Cache Optimization Techniques

### Cache-Line Alignment (64 bytes)

**Technique**: Align hot data structures to cache-line boundaries

```rust
#[repr(align(64))]
struct QueryContext {
    candidate_list: Vec<(f32, u32)>,  // Priority queue
    visited: FixedBitSet,              // Visited nodes
    result_buffer: Vec<(u32, f32)>,   // Results
}
```

**Why**: Prevents false sharing between threads, improves prefetching

---

### Selective Cache Admission (SHINE paper, 2024)

**Technique**: Don't cache all nodes equally

**Policy**:
- Base-level nodes: 1% admission probability (too many to cache all)
- Upper-level nodes: Always admit (critical for navigation)
- Frequently accessed nodes: Eventually make it to cache

**Implementation**:
```rust
fn should_cache(&self, node_id: usize, level: u8) -> bool {
    if level > 0 {
        return true;  // Always cache upper levels
    }

    // Base level: 1% admission probability
    self.rng.gen_bool(0.01) || self.access_count[node_id] > threshold
}
```

**Expected Impact**: 1.7-2.3x throughput (SHINE paper results)

---

### Thread-Local Query Buffers

**Problem**: Per-query allocations for distance arrays, candidate lists

**Solution**: Thread-local buffers, reused across queries

```rust
thread_local! {
    static QUERY_BUFFERS: RefCell<QueryBuffers> = RefCell::new(QueryBuffers {
        candidates: Vec::with_capacity(500),
        visited: FixedBitSet::with_capacity(1_000_000),
        distances: Vec::with_capacity(100),
    });
}

pub fn search(&self, query: &[f32], k: usize) -> Vec<(u32, f32)> {
    QUERY_BUFFERS.with(|buffers| {
        let mut buf = buffers.borrow_mut();
        buf.candidates.clear();  // Reuse allocation
        buf.visited.clear();

        // Use pre-allocated buffers...
        self.search_with_buffers(query, k, &mut buf)
    })
}
```

**Expected Impact**:
- Eliminate 10,000-50,000 allocations per benchmark
- 5-10% query speedup

---

## SIMD Optimization Patterns

### AVX2 vs AVX512

**Current**: AVX2 (hnsw-simd feature)
- 256-bit vectors (8 × f32)
- Widely available (Intel Haswell+, AMD Zen+)

**Upgrade**: AVX512
- 512-bit vectors (16 × f32)
- **20-30% faster** than AVX2 (Milvus Knowhere data)
- Requires newer CPUs (Intel Skylake-X+, AMD Zen 4+)

**Implementation Strategy**:
1. Support both AVX2 and AVX512 (runtime detection)
2. Fallback to AVX2 if AVX512 unavailable
3. Test on Fedora (i9-13900KF has AVX512)

**Code Pattern** (distance calculation):
```rust
#[cfg(target_feature = "avx512f")]
unsafe fn l2_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    use core::arch::x86_64::*;

    let mut sum = _mm512_setzero_ps();
    for i in (0..a.len()).step_by(16) {
        let va = _mm512_loadu_ps(a.as_ptr().add(i));
        let vb = _mm512_loadu_ps(b.as_ptr().add(i));
        let diff = _mm512_sub_ps(va, vb);
        sum = _mm512_fmadd_ps(diff, diff, sum);
    }

    _mm512_reduce_add_ps(sum)
}

#[cfg(target_feature = "avx2")]
unsafe fn l2_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    // Current implementation (simdeez_f in hnsw_rs)
    // ... 8-wide SIMD ...
}

pub fn l2_distance(a: &[f32], b: &[f32]) -> f32 {
    #[cfg(target_feature = "avx512f")]
    { unsafe { l2_distance_avx512(a, b) } }

    #[cfg(not(target_feature = "avx512f"))]
    { unsafe { l2_distance_avx2(a, b) } }
}
```

---

## Implementation Roadmap

### Phase 1: Foundation (Weeks 9-10) - 2 weeks

**Goal**: Match current hnsw_rs performance with custom implementation

**Tasks**:
1. Basic HNSW structure (nodes, layers, neighbor lists)
2. Insert operation (greedy search + neighbor selection)
3. Search operation (hierarchical traversal)
4. Serialization/deserialization (persist to disk)
5. Parallel building (reuse existing patterns)

**Success Criteria**:
- ✅ 99%+ recall (match hnsw_rs)
- ✅ <2.5ms p95 query latency (competitive with current 2.08ms)
- ✅ 500+ QPS (match current SIMD baseline)
- ✅ All existing tests pass (142 tests)

**Expected Performance**: 500-600 QPS (baseline parity)

---

### Phase 2: Cache Optimization (Weeks 11-12) - 2 weeks

**Goal**: Reduce cache misses from 23.41% to <15%

**Optimizations**:
1. Cache-line alignment (64-byte boundaries)
2. Flattened index (contiguous memory, not pointers)
3. Software prefetching (_mm_prefetch)
4. Graph reordering (BFS layout)

**Success Criteria**:
- ✅ LLC cache misses <15% (from 23.41%)
- ✅ <1.8ms p95 query latency (15-20% improvement)
- ✅ 650-700 QPS (beat Qdrant's 626 QPS)

**Expected Performance**: 650-700 QPS (+15-20%)

---

### Phase 3: Allocation Optimization (Week 13) - 1 week

**Goal**: Eliminate unnecessary allocations

**Optimizations**:
1. Arena allocators (typed-arena for node construction)
2. Thread-local query buffers
3. Pre-allocated result vectors

**Success Criteria**:
- ✅ Allocations <2M (from 7.3M, eliminate 5.6M in search_layer)
- ✅ <1.6ms p95 query latency
- ✅ 750-800 QPS

**Expected Performance**: 750-800 QPS (+30-40% cumulative)

---

### Phase 4: SIMD Enhancement (Week 14) - 1 week

**Goal**: AVX512 support (if not already enabled)

**Optimizations**:
1. AVX512 distance calculations (16-wide vs 8-wide)
2. Runtime CPU detection
3. Fallback to AVX2

**Success Criteria**:
- ✅ 20-30% faster distance calculations (Milvus benchmark)
- ✅ <1.4ms p95 query latency
- ✅ 850-900 QPS

**Expected Performance**: 850-900 QPS (+46-55% cumulative)

---

### Phase 5: Advanced Features (Weeks 15-17) - 3 weeks

**Goal**: SOTA quantization and compression

**Implementations**:

1. **Extended RaBitQ** (Week 15-16):
   - Replace BinaryQuantization with E-RaBitQ
   - Support 3-9 bits per dimension
   - AVX512 distance calculations
   - **Impact**: 2-3x better accuracy at same memory

2. **Delta Encoding** (Week 17):
   - Compress HNSW graph neighbor lists
   - 30% memory reduction (Qdrant benchmark)
   - **Impact**: 10M → 14M vectors in same RAM

**Success Criteria**:
- ✅ E-RaBitQ: 4-bit (8x compression) with >90% recall
- ✅ Delta encoding: 30% graph memory reduction
- ✅ 900-950 QPS (quantization improves throughput via less memory access)

**Expected Performance**: 900-950 QPS (+55-64% cumulative)

---

### Phase 6: Scale & Polish (Weeks 18-19) - 2 weeks

**Goal**: Billion-scale support (HNSW-IF) + production readiness

**Implementations**:

1. **HNSW-IF** (Hybrid in-memory/disk):
   - Random 20% centroids in HNSW (in-memory)
   - 80% neighbors on disk (posting lists)
   - Two-phase search (HNSW → disk scan)
   - **Impact**: 10M → 1B+ vectors with modest RAM

2. **Production polish**:
   - Comprehensive error handling
   - Monitoring/observability hooks
   - Documentation
   - Benchmarking suite

**Success Criteria**:
- ✅ Support 1B vectors with <32GB RAM
- ✅ 1000+ QPS at 10M scale
- ✅ Production-ready code quality

**Expected Performance**: 1000+ QPS (+72-100% cumulative)

---

## Performance Projection Summary

| Phase | Weeks | Key Optimization | QPS Target | vs Qdrant | vs Baseline |
|-------|-------|------------------|------------|-----------|-------------|
| Current (SIMD) | - | hnsw_rs + SIMD | 581 | 93% | Baseline |
| Phase 1 (Foundation) | 9-10 | Custom HNSW core | 500-600 | 80-96% | -14% to +3% |
| Phase 2 (Cache) | 11-12 | Cache optimization | 650-700 | 104-112% ⭐ | +12-20% |
| Phase 3 (Allocation) | 13 | Arena allocators | 750-800 | 120-128% ⭐ | +29-38% |
| Phase 4 (SIMD) | 14 | AVX512 | 850-900 | 136-144% ⭐ | +46-55% |
| Phase 5 (Quantization) | 15-17 | E-RaBitQ + Delta | 900-950 | 144-152% ⭐ | +55-64% |
| Phase 6 (Scale) | 18-19 | HNSW-IF | 1000+ | 160%+ ⭐ | +72%+ |

**Total Timeline**: 10-11 weeks (Weeks 9-19)

**Risk**: Medium (custom implementation complexity, but well-researched)

**Fallback**: Phases 1-4 already competitive (650-900 QPS), Phases 5-6 optional

---

## Key References

### Papers

1. **Extended RaBitQ**: https://arxiv.org/pdf/2409.09913
   - SIGMOD 2025, arbitrary compression rates

2. **Graph Reordering for HNSW**: https://papers.neurips.cc/paper_files/paper/2022/file/fb44a668c2d4bc984e9d6ca261262cbb-Paper-Conference.pdf
   - NeurIPS 2022, cache-efficient layouts

3. **SHINE**: https://arxiv.org/html/2507.17647v1
   - 2024, cache admission policies, 40% time in memory fetches

4. **HNSW Original**: https://arxiv.org/abs/1603.09320
   - Malkov & Yashunin 2016, foundational paper

### Competitor Implementations

1. **Qdrant**: https://github.com/qdrant/qdrant (Rust, 25.5k stars)
   - Delta encoding, GPU indexing, custom storage

2. **Milvus Knowhere**: https://github.com/milvus-io/knowhere (C++)
   - AVX512, query batching, HNSW INT8

3. **LanceDB**: https://github.com/lancedb/lance (Rust)
   - Columnar format, custom HNSW

4. **Weaviate**: https://github.com/weaviate/weaviate (Go)
   - Static allocations, cache prefilling

### Rust Libraries

1. **typed-arena**: https://github.com/thomcc/rust-typed-arena
   - Single-type arena allocator for graph nodes

2. **bumpalo**: https://github.com/fitzgen/bumpalo
   - General-purpose bump allocator

### Technical Articles

1. **Qdrant 1.13 Release**: https://qdrant.tech/blog/qdrant-1.13.x/
   - Delta encoding, GPU indexing details

2. **Vespa HNSW-IF**: https://blog.vespa.ai/vespa-hybrid-billion-scale-vector-search/
   - Hybrid in-memory/disk approach

3. **Arenas in Rust**: https://manishearth.github.io/blog/2021/03/15/arenas-in-rust/
   - Comprehensive guide to arena patterns

---

## Decision Matrix: What to Implement When

### MUST HAVE (Competitive Parity)

| Feature | Rationale | Phase | Priority |
|---------|-----------|-------|----------|
| Custom HNSW core | Unblock all optimizations | 1 | ⭐⭐⭐⭐⭐ |
| Cache-line alignment | 23.41% cache misses | 2 | ⭐⭐⭐⭐⭐ |
| Software prefetching | 40% time in memory | 2 | ⭐⭐⭐⭐⭐ |
| Arena allocators | 76% allocations in library | 3 | ⭐⭐⭐⭐⭐ |
| Thread-local buffers | Eliminate query allocations | 3 | ⭐⭐⭐⭐ |
| AVX512 support | 20-30% speedup (Milvus data) | 4 | ⭐⭐⭐⭐ |

### SHOULD HAVE (Competitive Advantage)

| Feature | Rationale | Phase | Priority |
|---------|-----------|-------|----------|
| Extended RaBitQ | SOTA quantization (SIGMOD 2025) | 5 | ⭐⭐⭐⭐⭐ |
| Delta encoding | 30% memory reduction (Qdrant) | 5 | ⭐⭐⭐⭐ |
| Graph reordering | 15-18% speedup (NeurIPS) | 2 | ⭐⭐⭐ |
| Query batching | "Obvious" QPS improvement (Milvus) | 6 | ⭐⭐⭐ |

### NICE TO HAVE (Scale & Features)

| Feature | Rationale | Phase | Priority |
|---------|-----------|-------|----------|
| HNSW-IF | Billion-scale support | 6 | ⭐⭐⭐⭐ |
| Bitset soft-deletes | Real-time updates (Milvus) | 6 | ⭐⭐ |
| Cache admission policy | 1.7-2.3x throughput (SHINE) | 6 | ⭐⭐ |

---

## Actionable Next Steps

### Week 9 Day 1 (Today):

1. **✅ Research complete** - This document
2. **Create technical specification**:
   - HNSW node structure (cache-aligned)
   - Neighbor list format (fixed-size arrays vs vectors)
   - Layer organization (entry point, level distribution)
   - Memory layout (arena allocation strategy)

3. **Set up development branch**:
   ```bash
   git checkout -b feature/custom-hnsw
   ```

4. **Create module structure**:
   ```
   src/vector/
   ├── hnsw/
   │   ├── mod.rs          # Public API
   │   ├── node.rs         # HNSWNode structure
   │   ├── graph.rs        # Graph management
   │   ├── build.rs        # Index construction
   │   ├── search.rs       # Query execution
   │   ├── serialize.rs    # Persistence
   │   └── prefetch.rs     # Cache optimization
   ```

### Week 9 Day 2-5 (Foundation):

1. Implement basic HNSW structure
2. Port insert logic from hnsw_rs patterns
3. Port search logic
4. Write unit tests (distance, recall, correctness)
5. Benchmark against hnsw_rs baseline

### Week 10-19 (Incremental Optimization):

Follow roadmap phases 2-6, measuring performance after each phase

---

## Success Metrics

### Performance Targets

| Metric | Current | Phase 1 | Phase 4 | Phase 6 | vs Qdrant |
|--------|---------|---------|---------|---------|-----------|
| QPS | 581 | 500-600 | 850-900 | 1000+ | +60% |
| p95 latency | 2.08ms | <2.5ms | <1.4ms | <1.2ms | -40% |
| Cache misses | 23.41% | 20% | <15% | <10% | Better |
| Allocations | 7.3M | 5M | <2M | <1M | Better |
| Memory/vector | 1.50 B | 1.50 B | 1.50 B | 1.05 B | Better |

### Competitive Positioning

**After Phase 2** (Week 12): Beat Qdrant (650-700 QPS vs 626 QPS)
**After Phase 4** (Week 14): 36-44% faster than Qdrant (850-900 QPS)
**After Phase 6** (Week 19): 60%+ faster than Qdrant (1000+ QPS)

**Marketing Claims** (Post-Phase 6):
- "1000+ QPS - 60% faster than Qdrant"
- "SOTA quantization (SIGMOD 2025 Extended RaBitQ)"
- "Billion-scale support with <32GB RAM"
- "PostgreSQL-compatible - only vector DB at this scale"

---

## Risk Assessment

### Technical Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Custom HNSW slower than library | Low | High | Benchmark each phase, can abort if falling behind |
| AVX512 bugs/compatibility | Medium | Medium | Fallback to AVX2, test on Fedora first |
| Arena allocator lifetime issues | Medium | Medium | Use typed-arena (well-tested), extensive testing |
| Extended RaBitQ integration complex | Medium | Medium | Phase 5 optional, core already competitive |

### Schedule Risks

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Phase 1 takes >2 weeks | Medium | High | Strong existing hnsw_rs reference, port patterns |
| Optimization phases stack slowly | Low | Medium | Each phase independent, can parallelize |
| Hitting performance wall | Low | High | Conservative estimates, research-backed techniques |

### Fallback Plan

**If Phase 1 fails** (can't match hnsw_rs):
- Abort custom HNSW, fork hnsw_rs instead
- Implement arena allocators + thread-local buffers on fork
- Target: 650-700 QPS (compromise)

**If Phase 2-3 underwhelm** (<650 QPS):
- Skip to Phase 4 (AVX512) for quick win
- Re-evaluate optimization priorities
- Still ahead of original 581 QPS baseline

**If schedule slips**:
- Phase 1-4 = 8 weeks (not 6) → still competitive
- Phase 5-6 optional (SOTA features, not required for Qdrant-beating)

---

## Conclusion

**Strategic Decision: VALIDATED ✅**

1. **All serious competitors use custom HNSW** (industry standard)
2. **Clear path to 1000+ QPS** (10-11 weeks, research-backed)
3. **Incremental validation** (each phase independently testable)
4. **Low risk** (fallback options, conservative estimates)

**Competitive Advantage**:
- Week 12: Beat Qdrant (650-700 QPS vs 626 QPS) ⭐
- Week 14: 36-44% faster than Qdrant (850-900 QPS) ⭐⭐
- Week 19: 60%+ faster + SOTA features (1000+ QPS) ⭐⭐⭐

**Proceed with custom HNSW implementation**

---

**Status**: Research complete, ready for implementation
**Next**: Design technical specification (Week 9 Day 1)
**Timeline**: 10-11 weeks (Weeks 9-19)
**Target**: 1000+ QPS, 60% faster than Qdrant, PostgreSQL-compatible
