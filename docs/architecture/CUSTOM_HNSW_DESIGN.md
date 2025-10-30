# Custom HNSW Architecture Design

**Date**: October 30, 2025 (Week 9 Day 1)
**Goal**: Design custom HNSW implementation to reach 1000+ QPS (60% faster than Qdrant's 626 QPS)
**Current Performance**: 581 QPS (93% of Qdrant, with hnsw_rs library)
**Target**: 10-15 weeks to SOTA implementation

---

## Executive Summary

**Why Custom Implementation**:
- ALL serious competitors use custom HNSW (Qdrant, Milvus, Weaviate, LanceDB)
- hnsw_rs library bottlenecks identified:
  - 23.41% LLC cache misses (poor memory layout)
  - 76% of allocations in hnsw_rs (cannot optimize)
  - No control over SOTA features (Extended RaBitQ, delta encoding)

**Key Design Principles**:
1. **Cache-First**: Optimize for memory locality (reduce 23.41% LLC misses to <15%)
2. **Zero-Copy**: Contiguous memory layout, avoid pointer chasing
3. **SIMD-Native**: AVX2/AVX512 distance calculations built-in
4. **Allocation-Aware**: Arena allocators, thread-local buffers
5. **SOTA-Ready**: Foundation for Extended RaBitQ, delta encoding, HNSW-IF

**Performance Projections**:
| Phase | Target QPS | vs Qdrant | Timeline |
|-------|-----------|-----------|----------|
| Phase 1: Foundation | 500-600 QPS | 80-96% | Weeks 9-10 |
| Phase 2: Cache Optimization | 650-700 QPS | 104-112% | Weeks 11-12 |
| Phase 3: Allocation Optimization | 750-800 QPS | 120-128% | Week 13 |
| Phase 4: SIMD Enhancement | 850-900 QPS | 136-144% | Week 14 |
| Phase 5: SOTA Features | 900-1000+ QPS | 144-160%+ | Weeks 15-19 |

---

## Table of Contents

1. [Core Data Structures](#core-data-structures)
2. [Memory Layout Strategy](#memory-layout-strategy)
3. [Cache Optimization Techniques](#cache-optimization-techniques)
4. [Allocation Strategy](#allocation-strategy)
5. [SIMD Integration](#simd-integration)
6. [Algorithm Implementation](#algorithm-implementation)
7. [Serialization](#serialization)
8. [SOTA Features Roadmap](#sota-features-roadmap)
9. [API Design](#api-design)
10. [Implementation Phases](#implementation-phases)

---

## Core Data Structures

### Design Philosophy

**Goal**: Minimize cache misses and maximize SIMD efficiency

**Key Decisions**:
- **Flattened index**: Store all nodes in contiguous Vec (not pointers)
- **Cache-line aligned**: 64-byte boundaries for hot data
- **Separate hot/cold**: Frequently accessed data together, rarely accessed data separate
- **Index-based**: Use u32 node IDs instead of pointers (4 bytes vs 8 bytes)

---

### HNSWIndex Structure

```rust
/// Custom HNSW index optimized for cache efficiency
pub struct HNSWIndex {
    /// Flattened node storage (cache-friendly, contiguous memory)
    nodes: Vec<HNSWNode>,

    /// Dimensions per vector
    dimensions: usize,

    /// HNSW construction parameters
    params: HNSWParams,

    /// Entry point (top-level node)
    entry_point: Option<u32>,

    /// Maximum level in graph
    max_level: usize,

    /// Distance function (L2, cosine, dot product)
    distance_fn: DistanceFunction,

    /// Thread-local query buffers (avoid per-query allocations)
    query_buffers: ThreadLocal<RefCell<QueryBuffers>>,
}

/// HNSW construction parameters
#[derive(Clone)]
pub struct HNSWParams {
    /// Number of bidirectional links per node (M)
    pub m: usize,

    /// Size of dynamic candidate list (ef_construction)
    pub ef_construction: usize,

    /// Normalization factor for level assignment
    pub ml: f32,

    /// Random seed for reproducibility
    pub seed: u64,
}

impl Default for HNSWParams {
    fn default() -> Self {
        Self {
            m: 48,                    // Qdrant uses 32-64
            ef_construction: 200,     // Higher = better recall, slower build
            ml: 1.0 / (48 as f32).ln(), // 1/ln(M)
            seed: 42,
        }
    }
}
```

---

### HNSWNode Structure (Cache-Optimized)

**Design Goals**:
1. Cache-line align hot data (64 bytes)
2. Separate frequently accessed (neighbors, level) from rarely accessed (vector data)
3. Use indices instead of pointers (4 bytes vs 8 bytes, better cache utilization)

```rust
/// HNSW node with cache-optimized layout
///
/// Hot data (first 64 bytes = 1 cache line):
/// - Node ID (4 bytes)
/// - Level (1 byte)
/// - Neighbor counts (8 bytes for 8 levels max)
/// - First M neighbors inline (48 * 4 = 192 bytes... too large)
///
/// Design: Separate hot (metadata) from cold (neighbors, vector)
#[repr(C, align(64))]  // Cache-line aligned
pub struct HNSWNode {
    /// Node ID (u32 = 4 bytes, supports 4 billion vectors)
    pub id: u32,

    /// Current level (0 to max_level)
    pub level: u8,

    /// Neighbor counts per level (u8 = 1 byte per level, max 8 levels)
    pub neighbor_counts: [u8; 8],

    /// Padding to 64-byte cache line
    _padding: [u8; 50],
}

/// Neighbors stored separately in contiguous arrays
/// (access neighbors only when needed, not in every node fetch)
pub struct NeighborLists {
    /// Flattened neighbor storage: [level0_neighbors, level1_neighbors, ...]
    /// Each neighbor is u32 node ID
    neighbors: Vec<u32>,

    /// Offsets into neighbors vec for each node's layers
    /// offsets[node_id * MAX_LEVELS + level] = start offset
    offsets: Vec<u32>,
}

/// Vectors stored separately (quantized or full precision)
pub enum VectorStorage {
    /// Full precision f32 vectors (6144 bytes for 1536D)
    FullPrecision(Vec<Vec<f32>>),

    /// Binary quantized (192 bytes for 1536D, 32x compression)
    BinaryQuantized {
        /// Quantized vectors (1 bit per dimension)
        quantized: Vec<Vec<u8>>,

        /// Original vectors for reranking (optional, 6144 bytes)
        original: Option<Vec<Vec<f32>>>,

        /// Quantization thresholds (1536 f32 = 6144 bytes)
        thresholds: Vec<f32>,
    },
}
```

**Why This Layout**:
- **64-byte cache-line alignment**: HNSWNode fits in 1 cache line
- **Hot data first**: ID, level, neighbor counts accessed frequently
- **Neighbors separate**: Only fetch when traversing graph (not in every node access)
- **Vectors separate**: Largest data (6KB per vector), only fetch when computing distances
- **Index-based**: u32 node IDs (4 bytes) vs pointers (8 bytes) = 50% memory savings for links

**Memory Breakdown (10M vectors, M=48)**:
| Component | Size per Node | Total (10M) | Notes |
|-----------|---------------|-------------|-------|
| HNSWNode | 64 bytes | 640 MB | Cache-aligned metadata |
| Neighbors | ~600 bytes | 6 GB | M=48, ~6 levels avg |
| Vectors (f32) | 6144 bytes | 61 GB | 1536D full precision |
| Vectors (BQ) | 192 bytes | 1.9 GB | 32x compression |
| **Total (f32)** | **~6800 bytes** | **68 GB** | Full precision |
| **Total (BQ)** | **~850 bytes** | **8.5 GB** | Binary quantized |

**Comparison to hnsw_rs** (pointer-based):
- hnsw_rs: ~7500 bytes/node (pointers + allocation overhead)
- Custom: ~850 bytes/node (BQ) = **8.8x more memory efficient**
- Custom: ~6800 bytes/node (f32) = **10% more efficient** (less overhead)

---

### QueryBuffers (Thread-Local, Reusable)

**Problem**: Per-query allocations cause 7.3M allocations for 10K benchmark (54K temporary)

**Solution**: Thread-local buffers reused across queries

```rust
/// Reusable buffers for query execution (eliminate per-query allocations)
pub struct QueryBuffers {
    /// Candidate set for greedy search (pre-allocated, capacity = ef_construction)
    candidates: BinaryHeap<Reverse<Candidate>>,

    /// Visited set (bitset for O(1) lookups)
    visited: FixedBitSet,

    /// Result buffer (k nearest neighbors)
    results: Vec<Candidate>,

    /// Prefetch queue (next nodes to prefetch)
    prefetch_queue: VecDeque<u32>,
}

#[derive(PartialEq, PartialOrd, Eq, Ord)]
struct Candidate {
    /// Negative distance (for min-heap via Reverse)
    distance: OrderedFloat<f32>,

    /// Node ID
    node_id: u32,
}

impl QueryBuffers {
    /// Reset buffers for next query (O(1) via clear)
    fn reset(&mut self) {
        self.candidates.clear();
        self.visited.clear();
        self.results.clear();
        self.prefetch_queue.clear();
    }
}
```

**Benefits**:
- **Zero allocations per query**: Reuse same buffers
- **Thread-safe**: ThreadLocal ensures one buffer per thread
- **Fast reset**: clear() is O(1) for most structures
- **Expected impact**: Reduce 7.3M → <2M allocations (eliminate 5.6M in search_layer)

---

## Memory Layout Strategy

### Goal: Reduce Cache Misses from 23.41% to <15%

**Current Problem** (hnsw_rs):
- Nodes allocated in insertion order
- Random access during graph traversal
- Poor spatial locality → 23.41% LLC misses

**Solution: BFS Graph Reordering**

**Algorithm** (NeurIPS 2022, "Graph Reordering for Cache-Efficient Near Neighbor Search"):
1. Build HNSW with temporary IDs
2. Run BFS from entry point, assign consecutive IDs
3. Reorder nodes[] and neighbors[] arrays
4. Result: Neighbors likely in same cache line

**Expected Impact**: 15-18% query speedup (NeurIPS benchmark)

```rust
impl HNSWIndex {
    /// Reorder graph for better cache locality (run after construction)
    pub fn reorder_for_cache_efficiency(&mut self) {
        // 1. BFS traversal from entry point
        let new_ids = self.bfs_reordering();

        // 2. Remap node IDs
        let old_to_new: HashMap<u32, u32> = new_ids.iter().enumerate()
            .map(|(new_id, &old_id)| (old_id, new_id as u32))
            .collect();

        // 3. Reorder nodes array
        let mut new_nodes = Vec::with_capacity(self.nodes.len());
        for &old_id in &new_ids {
            new_nodes.push(self.nodes[old_id as usize].clone());
            new_nodes.last_mut().unwrap().id = old_to_new[&old_id];
        }
        self.nodes = new_nodes;

        // 4. Remap neighbor IDs
        for neighbors in &mut self.neighbor_lists.neighbors {
            for neighbor_id in neighbors {
                *neighbor_id = old_to_new[neighbor_id];
            }
        }

        // 5. Update entry point
        if let Some(entry) = self.entry_point {
            self.entry_point = Some(old_to_new[&entry]);
        }
    }

    fn bfs_reordering(&self) -> Vec<u32> {
        let mut new_ids = Vec::with_capacity(self.nodes.len());
        let mut visited = FixedBitSet::with_capacity(self.nodes.len());
        let mut queue = VecDeque::new();

        // Start from entry point
        if let Some(entry) = self.entry_point {
            queue.push_back(entry);
            visited.set(entry as usize, true);
        }

        // BFS traversal
        while let Some(node_id) = queue.pop_front() {
            new_ids.push(node_id);

            // Visit all neighbors (all levels)
            for level in 0..=self.nodes[node_id as usize].level {
                for &neighbor_id in self.get_neighbors(node_id, level) {
                    if !visited[neighbor_id as usize] {
                        visited.set(neighbor_id as usize, true);
                        queue.push_back(neighbor_id);
                    }
                }
            }
        }

        new_ids
    }
}
```

---

## Cache Optimization Techniques

### 1. Software Prefetching

**Goal**: Hide memory latency by prefetching upcoming nodes

**When to prefetch**: 20-30 elements ahead in candidate queue (typical L2→RAM latency)

```rust
impl HNSWIndex {
    fn search_layer(
        &self,
        query: &[f32],
        entry_points: &[u32],
        ef: usize,
        level: usize,
        buffers: &mut QueryBuffers,
    ) -> Vec<u32> {
        // ... setup ...

        while let Some(Reverse(current)) = buffers.candidates.pop() {
            // **PREFETCH OPTIMIZATION**: Prefetch upcoming nodes
            if buffers.candidates.len() >= 20 {
                // Peek at node 20 elements ahead
                let prefetch_candidates: Vec<_> = buffers.candidates.iter()
                    .skip(19)
                    .take(5)
                    .collect();

                for candidate in prefetch_candidates {
                    let prefetch_node_id = candidate.0.node_id;
                    let node_ptr = &self.nodes[prefetch_node_id as usize] as *const HNSWNode;

                    unsafe {
                        // Prefetch into L1 cache (_MM_HINT_T0)
                        #[cfg(target_arch = "x86_64")]
                        core::arch::x86_64::_mm_prefetch(
                            node_ptr as *const i8,
                            core::arch::x86_64::_MM_HINT_T0
                        );
                    }
                }
            }

            // Process current node...
            let neighbors = self.get_neighbors(current.node_id, level);
            for &neighbor_id in neighbors {
                if buffers.visited[neighbor_id as usize] {
                    continue;
                }

                let distance = self.compute_distance(query, neighbor_id);
                // ... add to candidates ...
            }
        }

        // ... return results ...
    }
}
```

**Expected Impact**: 10-15% improvement (mask memory latency)

---

### 2. Cache-Line Alignment

**Ensure hot data structures align to 64-byte cache lines**:

```rust
// HNSWNode already aligned (see above)
#[repr(C, align(64))]
pub struct HNSWNode { ... }

// Align vectors if stored inline (not recommended due to size)
#[repr(C, align(64))]
pub struct AlignedVector {
    data: [f32; 1536],
}

// For neighbor lists, pack tightly (no alignment needed)
// Access pattern is sequential, not random
```

---

### 3. Batch Query Processing

**Goal**: Process multiple queries together to improve instruction-level parallelism and cache reuse

```rust
impl HNSWIndex {
    /// Batch query processing (process N queries together)
    pub fn batch_search(
        &self,
        queries: &[Vec<f32>],
        k: usize,
        ef: usize,
    ) -> Vec<Vec<SearchResult>> {
        // Group queries by entry point level for better cache locality
        // Process queries in batches of 8-16 (CPU pipeline depth)
        queries.chunks(16)
            .flat_map(|batch| {
                batch.par_iter()
                    .map(|query| self.search(query, k, ef))
                    .collect::<Vec<_>>()
            })
            .collect()
    }
}
```

**Expected Impact**: 5-10% improvement (Milvus Knowhere: "Improves QPS obviously")

---

## Allocation Strategy

### Goal: Reduce Allocations from 7.3M to <2M

### 1. Arena Allocator for Graph Construction

**Problem**: Per-node allocations during construction (fragmentation + overhead)

**Solution**: Arena allocator (typed-arena crate)

```rust
use typed_arena::Arena;

pub struct HNSWBuilder {
    /// Arena for node construction (all nodes deallocated together)
    node_arena: Arena<HNSWNode>,

    /// Arena for neighbor lists
    neighbor_arena: Arena<Vec<u32>>,

    /// HNSW parameters
    params: HNSWParams,
}

impl HNSWBuilder {
    pub fn new(params: HNSWParams) -> Self {
        Self {
            node_arena: Arena::with_capacity(1_000_000), // Pre-allocate 1M nodes
            neighbor_arena: Arena::with_capacity(1_000_000),
            params,
        }
    }

    pub fn insert_node(&mut self, vector: Vec<f32>) -> &mut HNSWNode {
        // Allocate from arena (fast, no fragmentation)
        let node = self.node_arena.alloc(HNSWNode {
            id: self.next_id(),
            level: self.random_level(),
            neighbor_counts: [0; 8],
            _padding: [0; 50],
        });

        node
    }

    pub fn finalize(self) -> HNSWIndex {
        // Convert arena-allocated data to Vec (contiguous, cache-friendly)
        let nodes: Vec<HNSWNode> = self.node_arena.iter()
            .cloned()
            .collect();

        HNSWIndex {
            nodes,
            // ... rest of initialization ...
        }
    }
}
```

**Benefits**:
- **Fast allocation**: Bump pointer, O(1)
- **No fragmentation**: Contiguous memory blocks
- **Batch deallocation**: Drop arena, all nodes freed at once
- **Expected impact**: 10-15% faster construction

---

### 2. Thread-Local Query Buffers (Already Covered Above)

See [QueryBuffers](#querybuffers-thread-local-reusable)

---

## SIMD Integration

### Current State: SIMD Enabled (Week 8)

**Achieved**: 3.6x improvement with AVX2 SIMD (hnsw_rs simdeez_f)

**Goal**: Native SIMD in custom implementation

### AVX2 Distance Calculation (Baseline)

```rust
#[cfg(target_feature = "avx2")]
pub unsafe fn l2_distance_avx2(a: &[f32], b: &[f32]) -> f32 {
    use core::arch::x86_64::*;

    debug_assert_eq!(a.len(), b.len());
    debug_assert!(a.len() % 8 == 0); // AVX2 = 8 floats

    let mut sum = _mm256_setzero_ps();

    for i in (0..a.len()).step_by(8) {
        let va = _mm256_loadu_ps(a.as_ptr().add(i));
        let vb = _mm256_loadu_ps(b.as_ptr().add(i));
        let diff = _mm256_sub_ps(va, vb);
        sum = _mm256_fmadd_ps(diff, diff, sum);
    }

    // Horizontal sum
    let sum_high = _mm256_extractf128_ps(sum, 1);
    let sum_low = _mm256_castps256_ps128(sum);
    let sum128 = _mm_add_ps(sum_high, sum_low);
    let sum64 = _mm_add_ps(sum128, _mm_movehl_ps(sum128, sum128));
    let sum32 = _mm_add_ss(sum64, _mm_shuffle_ps(sum64, sum64, 0x1));

    _mm_cvtss_f32(sum32)
}
```

---

### AVX512 Distance Calculation (Future, Week 14)

**Expected**: 20-30% faster than AVX2 (Milvus benchmark)

```rust
#[cfg(target_feature = "avx512f")]
pub unsafe fn l2_distance_avx512(a: &[f32], b: &[f32]) -> f32 {
    use core::arch::x86_64::*;

    debug_assert_eq!(a.len(), b.len());
    debug_assert!(a.len() % 16 == 0); // AVX512 = 16 floats

    let mut sum = _mm512_setzero_ps();

    for i in (0..a.len()).step_by(16) {
        let va = _mm512_loadu_ps(a.as_ptr().add(i));
        let vb = _mm512_loadu_ps(b.as_ptr().add(i));
        let diff = _mm512_sub_ps(va, vb);
        sum = _mm512_fmadd_ps(diff, diff, sum);
    }

    // Horizontal reduction (single instruction!)
    _mm512_reduce_add_ps(sum)
}
```

---

### Runtime SIMD Selection

```rust
pub enum DistanceFunction {
    L2,
    Cosine,
    DotProduct,
}

impl HNSWIndex {
    fn compute_distance(&self, query: &[f32], node_id: u32) -> f32 {
        let vector = self.get_vector(node_id);

        // Runtime CPU feature detection
        #[cfg(target_arch = "x86_64")]
        {
            if is_x86_feature_detected!("avx512f") {
                return unsafe { l2_distance_avx512(query, vector) };
            }

            if is_x86_feature_detected!("avx2") {
                return unsafe { l2_distance_avx2(query, vector) };
            }
        }

        // Fallback: scalar implementation
        l2_distance_scalar(query, vector)
    }
}
```

---

## Algorithm Implementation

### HNSW Core Algorithm

**Reference**: Malkov & Yashunin, "Efficient and robust approximate nearest neighbor search using Hierarchical Navigable Small World graphs" (2018)

### Insert Algorithm

```rust
impl HNSWIndex {
    pub fn insert(&mut self, vector: Vec<f32>) -> u32 {
        let node_id = self.nodes.len() as u32;
        let level = self.random_level();

        // Create new node
        self.nodes.push(HNSWNode {
            id: node_id,
            level,
            neighbor_counts: [0; 8],
            _padding: [0; 50],
        });

        // Store vector
        match &mut self.vectors {
            VectorStorage::FullPrecision(vecs) => vecs.push(vector),
            VectorStorage::BinaryQuantized { quantized, original, thresholds } => {
                let quant = Self::quantize_vector(&vector, thresholds);
                quantized.push(quant);
                if let Some(orig) = original {
                    orig.push(vector);
                }
            }
        }

        // Insert into graph
        if self.entry_point.is_none() {
            // First node becomes entry point
            self.entry_point = Some(node_id);
            return node_id;
        }

        // Search for nearest neighbors at each level
        let entry = self.entry_point.unwrap();
        let mut nearest = vec![entry];

        // Descend from top level to target level
        for lc in (level + 1..=self.max_level).rev() {
            nearest = self.search_layer(&vector, &nearest, 1, lc);
        }

        // Insert at levels 0..=level
        for lc in (0..=level).rev() {
            let candidates = self.search_layer(
                &vector,
                &nearest,
                self.params.ef_construction,
                lc
            );

            let m = if lc == 0 { self.params.m * 2 } else { self.params.m };
            let neighbors = self.select_neighbors_heuristic(node_id, &candidates, m, lc);

            // Add bidirectional links
            self.add_bidirectional_links(node_id, &neighbors, lc);

            // Prune neighbors if needed
            for &neighbor_id in &neighbors {
                let neighbor_neighbors = self.get_neighbors_mut(neighbor_id, lc);
                if neighbor_neighbors.len() > m {
                    let pruned = self.prune_connections(neighbor_id, neighbor_neighbors, m, lc);
                    self.set_neighbors(neighbor_id, lc, pruned);
                }
            }

            nearest = candidates.into_iter().take(1).collect();
        }

        // Update max level if needed
        if level > self.max_level {
            self.max_level = level;
            self.entry_point = Some(node_id);
        }

        node_id
    }

    fn random_level(&self) -> u8 {
        let mut rng = thread_rng();
        let uniform: f32 = rng.gen();
        (-uniform.ln() * self.params.ml).floor() as u8
    }
}
```

---

### Search Algorithm

```rust
impl HNSWIndex {
    pub fn search(&self, query: &[f32], k: usize, ef: usize) -> Vec<SearchResult> {
        if self.entry_point.is_none() {
            return Vec::new();
        }

        // Get thread-local buffers (zero allocation)
        let buffers = self.query_buffers.get_or(|| RefCell::new(QueryBuffers::new(
            self.params.ef_construction,
            self.nodes.len(),
        )));
        let mut buffers = buffers.borrow_mut();
        buffers.reset();

        // Start from entry point, descend to layer 0
        let entry = self.entry_point.unwrap();
        let mut nearest = vec![entry];

        // Greedy search at each layer (find 1 nearest)
        for level in (1..=self.max_level).rev() {
            nearest = self.search_layer(query, &nearest, 1, level, &mut buffers);
            buffers.reset();
        }

        // Beam search at layer 0 (find ef nearest)
        let candidates = self.search_layer(query, &nearest, ef, 0, &mut buffers);

        // Return k nearest
        candidates.into_iter()
            .take(k)
            .map(|node_id| SearchResult {
                id: node_id,
                distance: self.compute_distance(query, node_id),
            })
            .collect()
    }
}

pub struct SearchResult {
    pub id: u32,
    pub distance: f32,
}
```

---

### Neighbor Selection Heuristic

**Algorithm**: Simple or Heuristic (Malkov 2018, Section 4)

**Heuristic version** (better recall, used by Qdrant):

```rust
impl HNSWIndex {
    fn select_neighbors_heuristic(
        &self,
        node_id: u32,
        candidates: &[u32],
        m: usize,
        level: usize,
    ) -> Vec<u32> {
        if candidates.len() <= m {
            return candidates.to_vec();
        }

        let query_vector = self.get_vector(node_id);
        let mut result = Vec::with_capacity(m);
        let mut working_set: BinaryHeap<_> = candidates.iter()
            .map(|&id| {
                let dist = self.compute_distance(query_vector, id);
                Reverse((OrderedFloat(dist), id))
            })
            .collect();

        // Heuristic: Prioritize diverse neighbors
        while result.len() < m && !working_set.is_empty() {
            let Reverse((dist, candidate_id)) = working_set.pop().unwrap();

            // Check if candidate is closer to query than to existing neighbors
            let mut good = true;
            for &result_id in &result {
                let dist_to_result = self.compute_distance_between(candidate_id, result_id);
                if dist_to_result < dist.0 {
                    good = false;
                    break;
                }
            }

            if good {
                result.push(candidate_id);
            }
        }

        // Fill remaining slots if needed
        while result.len() < m && !working_set.is_empty() {
            let Reverse((_, candidate_id)) = working_set.pop().unwrap();
            result.push(candidate_id);
        }

        result
    }
}
```

---

## Serialization

### Design Goals

1. **Fast load**: <10s for 1M vectors (already achieved 6.02s with hnsw_rs)
2. **Portable**: Platform-independent format
3. **Versioned**: Support format evolution
4. **Compressed**: Optional delta encoding for neighbors (Week 17)

### File Format

```
HNSWIndexFile {
    magic: [u8; 8] = b"HNSWIDX\0",
    version: u32 = 1,
    dimensions: u32,
    num_nodes: u32,
    max_level: u8,
    params: HNSWParams,

    // Node metadata (64 bytes * num_nodes)
    nodes: Vec<HNSWNode>,

    // Neighbor lists (delta-encoded in Phase 5)
    neighbor_lists: NeighborLists,

    // Vector storage (quantized or full)
    vectors: VectorStorage,
}
```

### Save/Load Implementation

```rust
impl HNSWIndex {
    pub fn save(&self, path: &Path) -> Result<()> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        // Write header
        writer.write_all(b"HNSWIDX\0")?;
        writer.write_all(&1u32.to_le_bytes())?; // version
        writer.write_all(&(self.dimensions as u32).to_le_bytes())?;
        writer.write_all(&(self.nodes.len() as u32).to_le_bytes())?;
        writer.write_all(&[self.max_level])?;

        // Write params
        bincode::serialize_into(&mut writer, &self.params)?;

        // Write nodes (fast: contiguous memory)
        let nodes_bytes = unsafe {
            std::slice::from_raw_parts(
                self.nodes.as_ptr() as *const u8,
                self.nodes.len() * std::mem::size_of::<HNSWNode>()
            )
        };
        writer.write_all(nodes_bytes)?;

        // Write neighbor lists
        bincode::serialize_into(&mut writer, &self.neighbor_lists)?;

        // Write vectors
        bincode::serialize_into(&mut writer, &self.vectors)?;

        Ok(())
    }

    pub fn load(path: &Path) -> Result<Self> {
        let file = File::open(path)?;
        let mut reader = BufReader::new(file);

        // Read and verify header
        let mut magic = [0u8; 8];
        reader.read_exact(&mut magic)?;
        if &magic != b"HNSWIDX\0" {
            return Err(Error::InvalidMagic);
        }

        let mut version_bytes = [0u8; 4];
        reader.read_exact(&mut version_bytes)?;
        let version = u32::from_le_bytes(version_bytes);
        if version != 1 {
            return Err(Error::UnsupportedVersion(version));
        }

        // Read dimensions, num_nodes, max_level
        // ... (similar pattern) ...

        // Read nodes (fast: memmap or bulk read)
        let mut nodes = vec![HNSWNode::default(); num_nodes];
        let nodes_bytes = unsafe {
            std::slice::from_raw_parts_mut(
                nodes.as_mut_ptr() as *mut u8,
                nodes.len() * std::mem::size_of::<HNSWNode>()
            )
        };
        reader.read_exact(nodes_bytes)?;

        // Read neighbor lists
        let neighbor_lists: NeighborLists = bincode::deserialize_from(&mut reader)?;

        // Read vectors
        let vectors: VectorStorage = bincode::deserialize_from(&mut reader)?;

        Ok(HNSWIndex {
            nodes,
            neighbor_lists,
            vectors,
            dimensions,
            max_level,
            params,
            entry_point: Some(0), // Assume first node is entry
            query_buffers: ThreadLocal::new(),
            distance_fn: DistanceFunction::L2,
        })
    }
}
```

**Expected Performance**:
- Save 1M vectors: <10s (validated with hnsw_rs)
- Load 1M vectors: <10s (6.02s achieved with hnsw_rs)

---

## SOTA Features Roadmap

### Phase 5: Advanced Features (Weeks 15-19)

#### 1. Extended RaBitQ (Weeks 15-16)

**Paper**: "Extended RaBitQ: Versatile Product Quantization" (SIGMOD 2025)

**Benefits**:
- Arbitrary compression rates (3-9 bits per dimension)
- 2-3x better accuracy than binary quantization at same memory
- OR: 2x less memory at same accuracy

**Implementation**:

```rust
pub struct ExtendedRaBitQ {
    /// Bits per dimension (3-9 bits, configurable)
    bits: u8,

    /// Quantization thresholds (2^bits levels per dimension)
    thresholds: Vec<Vec<f32>>,

    /// Quantized vectors (packed bits)
    quantized: Vec<Vec<u8>>,

    /// Original vectors for reranking (optional)
    original: Option<Vec<Vec<f32>>>,
}

impl ExtendedRaBitQ {
    pub fn quantize_4bit(vector: &[f32], thresholds: &[Vec<f32>]) -> Vec<u8> {
        // Pack 2 values per byte (4 bits each)
        vector.chunks(2)
            .enumerate()
            .map(|(i, chunk)| {
                let v0 = Self::quantize_value_4bit(chunk[0], &thresholds[i * 2]);
                let v1 = if chunk.len() > 1 {
                    Self::quantize_value_4bit(chunk[1], &thresholds[i * 2 + 1])
                } else {
                    0
                };
                (v0 << 4) | v1
            })
            .collect()
    }

    fn quantize_value_4bit(value: f32, thresholds: &[f32]) -> u8 {
        // Binary search in 16 thresholds
        thresholds.binary_search_by(|t| t.partial_cmp(&value).unwrap())
            .unwrap_or_else(|i| i) as u8
    }

    /// AVX512 distance calculation (16-way SIMD)
    #[cfg(target_feature = "avx512f")]
    unsafe fn distance_avx512(
        a: &[u8],
        b: &[u8],
        thresholds: &[Vec<f32>],
    ) -> f32 {
        // Unpack 4-bit values, compute distances with SIMD
        // ... (similar to binary quantization, but 16 levels) ...
    }
}
```

**Expected Impact**:
- 4-bit (8x compression): 90-92% recall (vs 70% for 1-bit)
- 8-bit (4x compression): 95-97% recall (near full precision)
- Memory: 8-32x reduction vs full precision

---

#### 2. Delta Encoding for Graph (Week 17)

**Source**: Qdrant 1.13 (October 2024)

**Benefit**: 30% memory reduction for neighbor lists

**Algorithm**:
```
Original neighbors: [1000, 1003, 1007, 1050]
Delta encoded: [1000, +3, +4, +43]
Compressed: Variable-length encoding for deltas
```

**Implementation**:

```rust
pub struct DeltaEncodedNeighbors {
    /// Base IDs (one per node)
    base_ids: Vec<u32>,

    /// Delta-encoded neighbor offsets (variable-length)
    deltas: Vec<u8>,

    /// Offsets into deltas vec
    offsets: Vec<u32>,
}

impl DeltaEncodedNeighbors {
    pub fn encode(neighbors: &[u32]) -> Vec<u8> {
        if neighbors.is_empty() {
            return Vec::new();
        }

        let mut encoded = Vec::new();
        encoded.extend_from_slice(&neighbors[0].to_le_bytes()); // Base ID

        // Encode deltas with variable-length encoding
        for window in neighbors.windows(2) {
            let delta = window[1] - window[0];
            Self::encode_varint(delta, &mut encoded);
        }

        encoded
    }

    fn encode_varint(value: u32, output: &mut Vec<u8>) {
        // Variable-length encoding (1-5 bytes for u32)
        let mut val = value;
        loop {
            let mut byte = (val & 0x7F) as u8;
            val >>= 7;
            if val != 0 {
                byte |= 0x80; // More bytes follow
            }
            output.push(byte);
            if val == 0 {
                break;
            }
        }
    }

    pub fn decode(encoded: &[u8]) -> Vec<u32> {
        if encoded.len() < 4 {
            return Vec::new();
        }

        let mut neighbors = Vec::new();
        let base_id = u32::from_le_bytes([encoded[0], encoded[1], encoded[2], encoded[3]]);
        neighbors.push(base_id);

        let mut pos = 4;
        while pos < encoded.len() {
            let (delta, bytes_read) = Self::decode_varint(&encoded[pos..]);
            neighbors.push(neighbors.last().unwrap() + delta);
            pos += bytes_read;
        }

        neighbors
    }

    fn decode_varint(input: &[u8]) -> (u32, usize) {
        let mut result = 0u32;
        let mut shift = 0;
        let mut bytes_read = 0;

        for &byte in input {
            bytes_read += 1;
            result |= ((byte & 0x7F) as u32) << shift;
            if byte & 0x80 == 0 {
                break;
            }
            shift += 7;
        }

        (result, bytes_read)
    }
}
```

**Expected Impact**:
- 30% memory reduction for neighbor lists (Qdrant benchmark)
- 10M vectors: 6 GB → 4.2 GB neighbors (save 1.8 GB)
- No performance degradation (decompress during traversal, or pre-decompress hot nodes)

---

## API Design

### Public API

```rust
use omen::vector::{HNSWIndex, HNSWParams, SearchResult, DistanceFunction};

fn main() -> Result<()> {
    // 1. Create index
    let params = HNSWParams {
        m: 48,
        ef_construction: 200,
        ..Default::default()
    };

    let mut index = HNSWIndex::new(1536, params, DistanceFunction::L2);

    // 2. Insert vectors (single or batch)
    let vector = vec![0.1; 1536];
    let id = index.insert(vector)?;

    // Batch insert (parallel, 16x faster)
    let vectors: Vec<Vec<f32>> = load_vectors("embeddings.bin")?;
    index.batch_insert(&vectors)?;

    // 3. Search
    let query = vec![0.2; 1536];
    let results = index.search(&query, k=10, ef=50)?;

    for result in results {
        println!("ID: {}, Distance: {}", result.id, result.distance);
    }

    // 4. Save/Load
    index.save("index.hnsw")?;
    let loaded = HNSWIndex::load("index.hnsw")?;

    // 5. Optimize for queries (BFS reordering)
    index.reorder_for_cache_efficiency();

    Ok(())
}
```

---

### Integration with VectorStore

```rust
// src/vector/store.rs
pub struct VectorStore {
    /// Custom HNSW index (replaces hnsw_rs)
    index: HNSWIndex,

    /// RocksDB storage backend
    storage: Arc<RwLock<RocksDB>>,

    // ... rest of fields ...
}

impl VectorStore {
    pub fn insert(&mut self, vector: Vec<f32>) -> Result<u32> {
        // Insert into HNSW
        let id = self.index.insert(vector.clone())?;

        // Persist to RocksDB
        self.storage.write().unwrap().put(
            format!("vector:{}", id).as_bytes(),
            &bincode::serialize(&vector)?
        )?;

        Ok(id)
    }

    pub fn knn_search(&self, query: &[f32], k: usize) -> Result<Vec<SearchResult>> {
        self.index.search(query, k, self.ef_search)
    }

    pub fn save_to_disk(&self, path: &Path) -> Result<()> {
        self.index.save(path)
    }

    pub fn load_from_disk(path: &Path, storage: Arc<RwLock<RocksDB>>) -> Result<Self> {
        let index = HNSWIndex::load(path)?;

        Ok(VectorStore {
            index,
            storage,
            // ... rest of initialization ...
        })
    }
}
```

---

## Implementation Phases

### Phase 1: Foundation (Weeks 9-10) - 2 weeks

**Goal**: Match current hnsw_rs performance (500-600 QPS)

**Tasks**:
1. **Core data structures** (2 days):
   - HNSWNode, NeighborLists, VectorStorage
   - Basic memory layout (not yet optimized)

2. **Insert algorithm** (2 days):
   - Level assignment (random_level)
   - Greedy search (search_layer)
   - Neighbor selection (simple version first)
   - Bidirectional links

3. **Search algorithm** (1 day):
   - Hierarchical traversal
   - Beam search at layer 0
   - Return k nearest

4. **Serialization** (1 day):
   - Save/load format
   - Bincode for simplicity

5. **Testing** (2 days):
   - Port existing 142 tests to custom implementation
   - Correctness: 99%+ recall
   - Performance: match hnsw_rs (500-600 QPS)

**Success Criteria**:
- ✅ 99%+ recall (match hnsw_rs)
- ✅ 500-600 QPS (parity with hnsw_rs + SIMD)
- ✅ <2.5ms p95 query latency
- ✅ All 142 tests pass
- ✅ Save/load works (1M vectors in <10s)

**Expected Performance**: 500-600 QPS (baseline parity)

---

### Phase 2: Cache Optimization (Weeks 11-12) - 2 weeks

**Goal**: Reduce cache misses from 23.41% to <15%, reach 650-700 QPS

**Tasks**:
1. **BFS graph reordering** (2 days):
   - Implement reorder_for_cache_efficiency()
   - Benchmark improvement

2. **Software prefetching** (1 day):
   - Add _mm_prefetch hints
   - Tune prefetch distance (20-30 elements)

3. **Cache-line alignment** (1 day):
   - Verify HNSWNode alignment
   - Pack structures efficiently

4. **Profile and iterate** (3 days):
   - Measure cache miss rate
   - Identify remaining bottlenecks
   - Fine-tune layout

**Success Criteria**:
- ✅ LLC cache misses <15% (from 23.41%)
- ✅ 650-700 QPS (beat Qdrant's 626 QPS)
- ✅ <1.8ms p95 query latency (15-20% improvement)

**Expected Performance**: 650-700 QPS (+30-40% from baseline)

---

### Phase 3: Allocation Optimization (Week 13) - 1 week

**Goal**: Reduce allocations from 7.3M to <2M, reach 750-800 QPS

**Tasks**:
1. **Arena allocators** (2 days):
   - Implement HNSWBuilder with typed-arena
   - Benchmark construction speed

2. **Thread-local query buffers** (2 days):
   - Implement QueryBuffers
   - ThreadLocal integration
   - Verify zero per-query allocations

3. **Profile and verify** (1 day):
   - Measure allocation count
   - Benchmark query performance

**Success Criteria**:
- ✅ Allocations <2M (from 7.3M, eliminate 5.6M)
- ✅ 750-800 QPS
- ✅ <1.6ms p95 query latency

**Expected Performance**: 750-800 QPS (+50-60% cumulative)

---

### Phase 4: SIMD Enhancement (Week 14) - 1 week

**Goal**: AVX512 support (if available), reach 850-900 QPS

**Tasks**:
1. **AVX512 implementation** (2 days):
   - l2_distance_avx512()
   - Runtime CPU detection
   - Fallback to AVX2

2. **Testing** (1 day):
   - Test on Fedora (i9-13900KF has AVX512)
   - Verify correctness

3. **Benchmarking** (2 days):
   - Measure improvement
   - Compare AVX2 vs AVX512

**Success Criteria**:
- ✅ 20-30% faster distance calculations (Milvus benchmark)
- ✅ 850-900 QPS
- ✅ <1.4ms p95 query latency

**Expected Performance**: 850-900 QPS (+70-80% cumulative)

---

### Phase 5: SOTA Features (Weeks 15-19) - 5 weeks

**Goal**: Extended RaBitQ + Delta Encoding, reach 900-1000+ QPS

**Tasks**:

**Weeks 15-16: Extended RaBitQ**
1. Implement quantization (3 days)
2. AVX512 distance calculations (2 days)
3. Testing and tuning (2 days)

**Week 17: Delta Encoding**
1. Implement encode/decode (2 days)
2. Integration with save/load (1 day)
3. Benchmark memory savings (1 day)

**Weeks 18-19: Scale Validation**
1. 10M vector benchmark (2 days)
2. 100M vector validation (3 days)
3. Performance tuning (2 days)

**Success Criteria**:
- ✅ E-RaBitQ: 4-bit (8x compression) with >90% recall
- ✅ Delta encoding: 30% graph memory reduction
- ✅ 900-1000+ QPS (quantization improves throughput)
- ✅ 10M vectors validated
- ✅ 100M vectors validated (stretch goal)

**Expected Performance**: 900-1000+ QPS (+80-100%+ cumulative)

---

## Performance Projections Summary

| Phase | Weeks | Target QPS | vs Qdrant (626 QPS) | Cumulative Improvement | Key Optimizations |
|-------|-------|------------|---------------------|------------------------|-------------------|
| Baseline (hnsw_rs) | - | 581 QPS | 93% | - | SIMD enabled |
| Phase 1: Foundation | 9-10 | 500-600 QPS | 80-96% | 0% (parity) | Core implementation |
| Phase 2: Cache | 11-12 | 650-700 QPS | 104-112% | +30-40% | BFS reordering, prefetching |
| Phase 3: Allocation | 13 | 750-800 QPS | 120-128% | +50-60% | Arena allocators, thread-local buffers |
| Phase 4: SIMD | 14 | 850-900 QPS | 136-144% | +70-80% | AVX512 (if available) |
| Phase 5: SOTA | 15-19 | 900-1000+ QPS | 144-160%+ | +80-100%+ | Extended RaBitQ, delta encoding |

**Final Target**: 1000+ QPS (60% faster than Qdrant, Week 19)

---

## Testing Strategy

### Correctness Tests

**Port existing 142 tests**:
- Distance calculations (10 tests)
- HNSW recall (5 tests)
- Binary Quantization (7 tests)
- Serialization (10 tests)
- Concurrency (85 tests, MVCC)
- Input validation (25 tests)

**New tests for custom implementation**:
- BFS reordering correctness
- Cache-line alignment verification
- Thread-local buffer correctness
- Extended RaBitQ quantization accuracy
- Delta encoding round-trip

---

### Performance Benchmarks

**Microbenchmarks**:
- Distance calculation (AVX2 vs AVX512)
- Cache miss rate (perf stat)
- Allocation count (heaptrack)
- Memory bandwidth utilization

**System benchmarks**:
- 10K, 100K, 1M vector insertion
- Query latency (p50, p95, p99)
- QPS throughput
- Memory usage
- Save/load time

---

### Profiling

**Continuous profiling** after each phase:
```bash
# CPU profiling
cargo flamegraph --bin benchmark_custom_hnsw -- 100000

# Cache profiling
perf stat -e cache-references,cache-misses,LLC-loads,LLC-load-misses \
    ./target/release/benchmark_custom_hnsw 100000

# Memory profiling
heaptrack ./target/release/benchmark_custom_hnsw 100000
```

---

## Risk Mitigation

| Risk | Impact | Mitigation |
|------|--------|------------|
| **Performance regression** | Slower than hnsw_rs | Phase 1: Match baseline first, then optimize |
| **Correctness issues** | Wrong results | Port all 142 tests, add new tests |
| **Memory bugs** | Crashes, data corruption | ASAN validation (40 tests clean) |
| **Implementation complexity** | Takes longer than 10-15 weeks | Phased approach, each phase independently valuable |
| **AVX512 not available** | Can't reach 850-900 QPS target | AVX2 still gets to 750-800 QPS (competitive) |

---

## Success Metrics

### Minimum Success (Week 10)
- ✅ Custom HNSW working (500-600 QPS)
- ✅ All 142 tests pass
- ✅ Parity with hnsw_rs baseline

### Target Success (Week 14)
- ✅ 750-850 QPS (cache + allocation + SIMD optimization)
- ✅ Beat Qdrant (626 QPS) by 20-36%
- ✅ <1.5ms p95 query latency

### Stretch Success (Week 19)
- ✅ 1000+ QPS (SOTA features)
- ✅ Beat Qdrant by 60%+
- ✅ Extended RaBitQ working
- ✅ Delta encoding working
- ✅ 10M-100M vectors validated

---

## Next Steps

**Week 9 Day 1** (TODAY):
1. ✅ Design complete (this document)
2. Create module structure: `src/vector/custom_hnsw/`
3. Implement basic data structures (HNSWNode, NeighborLists, HNSWParams)
4. Write skeleton insert/search (compile, don't optimize yet)

**Week 9 Day 2-3**:
1. Implement insert algorithm (greedy search, neighbor selection)
2. Implement search algorithm (hierarchical traversal)
3. Write basic tests (10-20 tests for correctness)

**Week 9 Day 4-5**:
1. Implement serialization (save/load)
2. Port existing tests (142 tests)
3. Benchmark vs hnsw_rs baseline

**Week 10**:
1. Fix any correctness issues
2. Optimize until parity with hnsw_rs (500-600 QPS)
3. Complete Phase 1 milestone

---

**Design Status**: ✅ COMPLETE
**Next Action**: Create `src/vector/custom_hnsw/mod.rs` and begin implementation
**Timeline**: Weeks 9-19 (10-15 weeks to 1000+ QPS)
**Confidence**: HIGH (all techniques validated by competitors, clear roadmap)

---

**Last Updated**: October 30, 2025 (Week 9 Day 1)
**Author**: Claude Code + Nick (OmenDB)
**References**:
- `ai/research/CUSTOM_HNSW_SOTA_RESEARCH_2025.md` (12,500 words)
- `ai/research/PROFILING_ANALYSIS_WEEK8.md` (current bottlenecks)
- `ai/research/OPTIMIZATION_STRATEGY.md` (engine-first approach)
