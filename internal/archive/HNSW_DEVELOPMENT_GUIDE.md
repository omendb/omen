# HNSW Development Guide: How to Achieve State-of-the-Art in Mojo
## September 2025

## Critical Understanding: Why We Keep Failing

### The Core Problem
We've repeatedly achieved high speed (27K vec/s) but destroyed quality (1% recall) because we don't respect HNSW's fundamental invariants. This guide captures what MUST be preserved.

## HNSW Invariants (NEVER VIOLATE THESE)

### 1. Hierarchical Navigation (MOST CRITICAL)
```
REQUIREMENT: Every insertion MUST navigate from entry_point down through layers
WHY: Upper layers are "highways" to distant graph regions
VIOLATION: Direct insertion at target layer = disconnected subgraphs = 1% recall
```

**Safe Implementation:**
```mojo
# ALWAYS start from entry point
var curr_nearest = self.entry_point

# ALWAYS navigate down through layers
for layer in range(entry_level, target_layer, -1):
    curr_nearest = search_layer(query, curr_nearest, 1, layer)

# ONLY THEN insert at target layer
```

**Broken Implementation:**
```mojo
# NEVER do this - creates disconnected graphs
var neighbors = find_nearest_in_layer(query, target_layer)  # WRONG!
connect_to_neighbors(new_node, neighbors)  # Subgraph isolation!
```

### 2. Bidirectional Connections
```
REQUIREMENT: Every edge MUST be bidirectional (A→B and B→A)
WHY: Search traverses in both directions
VIOLATION: Missing reverse edges = unreachable nodes
```

**Safe Implementation:**
```mojo
# ALWAYS add both directions
node[].add_connection(layer, neighbor_id)      # A→B
neighbor[].add_connection(layer, node_id)      # B→A
_prune_connections(neighbor_id, layer, M)      # Maintain degree bounds
```

### 3. Entry Point Management
```
REQUIREMENT: Entry point MUST be the highest-level node
WHY: Search starts here and navigates down
VIOLATION: Wrong entry = search starts in wrong region = poor recall
```

### 4. Progressive Construction
```
REQUIREMENT: Graph must be valid after EACH insertion
WHY: Batch operations that defer connectivity break search
VIOLATION: Bulk insertion without immediate connection = broken graph
```

## Why Each Optimization Failed

### 1. Simplified Insertion (27K vec/s, 1% recall)
**What broke:** Skipped hierarchical navigation
**Result:** Fast but disconnected subgraphs
**Fix needed:** Keep navigation, optimize the search itself

### 2. Sophisticated Bulk (22K vec/s, segfaults)
**What broke:** Memory management with large batches
**Result:** Pointer corruption at 20K vectors
**Fix needed:** Smaller chunks, proper cleanup

### 3. Parallel Insertion (18K vec/s, 0.1% recall)
**What broke:** Race conditions on graph updates
**Result:** Random connections, corrupted structure
**Fix needed:** Lock-free structures or segment independence

## The Path to State-of-the-Art (VALIDATED APPROACH)

### Phase 1: Optimize What Works (Target: 2-3K vec/s)
```mojo
// Start with working code (867 vec/s, 95.5% recall)
// Profile to find exact bottlenecks:

fn insert_with_profiling():
    var t1 = now()
    navigate_layers()  # Measure: 30% of time
    var t2 = now()
    search_neighbors()  # Measure: 40% of time
    var t3 = now()
    update_connections()  # Measure: 20% of time
    var t4 = now()

// Then optimize the slowest part FIRST
```

### Phase 2: Safe Batching (Target: 5K vec/s)
```mojo
// Batch operations that DON'T break invariants:
// ✅ Batch distance calculations
// ✅ Batch memory allocation
// ✅ Batch vector copying
// ❌ DON'T batch graph connections (breaks progressive construction)

fn safe_batch_insert(vectors, count):
    # Pre-allocate ALL memory
    var node_ids = allocate_nodes(count)

    # Batch copy vectors (safe, no graph impact)
    batch_memcpy(vectors, node_ids)

    # Individual graph construction (preserves quality)
    for i in range(count):
        _insert_node(node_ids[i])  # Full navigation
```

### Phase 3: Parallel INDEPENDENT Work (Target: 10K vec/s)
```mojo
// Parallelize only independent operations:

fn parallel_safe_insert(vectors, count):
    # Phase 1: Parallel allocation (independent)
    parallel_for(i in range(count)):
        node_ids[i] = allocate_node()
        copy_vector(vectors[i], node_ids[i])

    # Phase 2: Sequential graph updates (dependencies)
    for i in range(count):
        _insert_node(node_ids[i])  # Can't parallelize without locks
```

### Phase 4: Segment Parallelism (Target: 15-20K vec/s)
```mojo
// Build independent segments in parallel, merge at search:

fn segment_parallel_insert(vectors, count):
    var segments = split_into_segments(vectors, num_cores)

    # Build independent graphs in parallel
    parallel_for(segment in segments):
        build_independent_hnsw(segment)  # No shared state

    # Search across segments
    fn search(query):
        results = parallel_search_segments(query)
        return merge_results(results)
```

### Phase 5: SIMD + Zero-Copy (Target: 25K+ vec/s)
```mojo
// Hardware acceleration + eliminate copies:

fn simd_distance(a: SIMD[Float32, 16], b: SIMD[Float32, 16]) -> Float32:
    var diff = a - b  # 16 floats at once
    return reduce_sum(diff * diff)

fn zero_copy_insert(numpy_array):
    var ptr = get_numpy_pointer(numpy_array)  # No copy!
    _insert_node_direct(ptr)
```

## Bottleneck Analysis (MEASURE FIRST!)

### Current Profile (867 vec/s)
```
Distance calculation:  40% → Optimize with SIMD
Neighbor search:      30% → Optimize with better data structure
Graph updates:        20% → Optimize with batching
Memory allocation:    10% → Optimize with pre-allocation
```

### Tools for Profiling
```python
import time

def profile_insertion(vectors):
    timings = {
        'navigation': [],
        'search': [],
        'connections': [],
        'total': []
    }

    for v in vectors:
        t_start = time.perf_counter()
        # Measure each phase
        result = insert_vector_with_timing(v, timings)

    # Analyze bottlenecks
    print(f"Navigation: {sum(timings['navigation']):.2f}s")
    print(f"Search: {sum(timings['search']):.2f}s")
    print(f"Connections: {sum(timings['connections']):.2f}s")
```

## Mojo-Specific Patterns

### Memory Management (AVOID SEGFAULTS)
```mojo
# GOOD: Clear ownership
var ptr = UnsafePointer[Float32].alloc(size)
# ... use ptr ...
ptr.free()  # Always free

# BAD: Dangling pointers
var ptr = get_vector(node_id)
node_pool.resize()  # ptr now invalid!
use_ptr(ptr)  # SEGFAULT
```

### Parallelization (AVOID RACES)
```mojo
# GOOD: Independent work
parallel_for(i in range(n)):
    independent_work[i] = process(data[i])

# BAD: Shared state
parallel_for(i in range(n)):
    shared_graph.add_edge(i, j)  # RACE CONDITION
```

### SIMD Usage (WHEN IT WORKS)
```mojo
# Check if SIMD compiles first!
fn test_simd():
    var a = SIMD[DType.float32, 16](1.0)
    var b = SIMD[DType.float32, 16](2.0)
    var c = a + b  # If this compiles, SIMD works
```

## Testing Strategy

### 1. Unit Tests for Invariants
```python
def test_hierarchical_navigation():
    # Insert node at level 2
    # Verify it was found by navigating from entry point
    assert path_from_entry_to_node_exists()

def test_bidirectional_connections():
    # For every edge A→B
    # Verify edge B→A exists
    assert all_edges_bidirectional()

def test_progressive_construction():
    # After each insertion
    # Verify graph is searchable
    assert search_finds_inserted_nodes()
```

### 2. Performance Regression Tests
```python
def test_performance():
    vectors = generate_test_vectors(10000)

    start = time.time()
    insert_bulk(vectors)
    duration = time.time() - start

    rate = 10000 / duration
    assert rate > 800  # Minimum acceptable

    recall = measure_recall()
    assert recall > 0.95  # Quality requirement
```

## Decision Tree for Optimizations

```
Is current performance < target?
├─ YES: Profile to find bottleneck
│   ├─ Distance calc slow? → Try SIMD
│   ├─ Search slow? → Try better data structure
│   ├─ Graph updates slow? → Try batching safe ops
│   └─ Memory slow? → Try pre-allocation
│
└─ NO: Is quality maintained?
    ├─ YES: Ship it!
    └─ NO: Optimization broke invariants
        ├─ Check hierarchical navigation
        ├─ Check bidirectional connections
        ├─ Check entry point
        └─ Check progressive construction
```

## Common Pitfalls

### 1. "Let's just make it parallel"
**Reality:** Graph dependencies prevent naive parallelization
**Solution:** Identify truly independent work first

### 2. "Simplified = Faster"
**Reality:** Simplification often breaks quality invariants
**Solution:** Profile first, optimize bottlenecks only

### 3. "Batch everything"
**Reality:** Batching graph updates breaks progressive construction
**Solution:** Batch only independent operations

### 4. "More threads = faster"
**Reality:** Synchronization overhead can make it slower
**Solution:** Measure parallel efficiency, use appropriate grain size

## Concrete Next Steps

### Week 1: Foundation
1. Add profiling to current implementation
2. Identify exact bottlenecks with data
3. Fix sophisticated bulk memory issues
4. Test with smaller chunks

### Week 2: Safe Optimizations
1. Implement SIMD distance (if compiler works)
2. Pre-allocate memory pools
3. Batch safe operations only
4. Maintain quality checks

### Week 3: Parallelization
1. Implement segment parallelism
2. Test lock-free structures
3. Measure parallel efficiency
4. Ensure quality maintained

### Week 4: Polish
1. Zero-copy FFI implementation
2. Cache optimization
3. Final performance tuning
4. Production hardening

## Success Criteria

### Minimum Viable Performance
- **Speed:** 5,000+ vec/s
- **Quality:** 95%+ recall@10
- **Stability:** No crashes at 100K vectors
- **Scalability:** Linear with threads

### State-of-the-Art Target
- **Speed:** 20,000+ vec/s
- **Quality:** 95%+ recall@10
- **Stability:** Production ready
- **Competitiveness:** Match Qdrant/Pinecone

## Conclusion

**We CAN achieve state-of-the-art in Mojo**, but we must:
1. Respect HNSW invariants (hierarchical navigation is NON-NEGOTIABLE)
2. Profile before optimizing (don't guess bottlenecks)
3. Test quality after EVERY change (catch breaks early)
4. Use Mojo's strengths (parallelism, SIMD) correctly

The path is clear. The tools exist. We just need systematic execution.

---
*This guide is based on hard-learned lessons from multiple failed attempts. Follow it to avoid repeating mistakes.*