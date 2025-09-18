# HNSW Invariants - NEVER VIOLATE THESE
## Critical Rules That Must Be Preserved for Correctness

## ⛔ ABSOLUTE INVARIANTS

### 1. Hierarchical Navigation is MANDATORY
```mojo
# ✅ CORRECT - Always navigate from top down
var curr = entry_point
for layer in range(entry_level, target_layer, -1):
    curr = search_layer_simple(query, curr, 1, layer)
# Only NOW insert at target layer

# ❌ WRONG - Never skip navigation
var neighbors = search_at_layer(query, target_layer)  # BROKEN!
```

**Why:** Upper layers are "highways" to distant graph regions. Skipping navigation creates disconnected subgraphs, destroying recall (27K vec/s but 1% recall proved this).

### 2. Bidirectional Connections are REQUIRED
```mojo
# ✅ CORRECT - Always add both directions
node.add_connection(layer, neighbor_id)       # A→B
neighbor.add_connection(layer, node_id)       # B→A
prune_connections(neighbor_id, layer, M)      # Maintain bounds

# ❌ WRONG - Missing reverse edge
node.add_connection(layer, neighbor_id)       # Only A→B = broken
```

**Why:** HNSW search traverses edges in both directions. Missing reverse edges make nodes unreachable.

### 3. Entry Point Must Be Highest Level Node
```mojo
# ✅ CORRECT - Update entry point immediately
if new_node_level > current_entry_level:
    entry_point = new_node_id

# ❌ WRONG - Deferred or forgotten update
# ... insert many nodes ...
# update_entry_point_later()  # Too late!
```

**Why:** Search starts from entry point and navigates down. Wrong entry point means search starts in wrong region.

### 4. Progressive Construction is ESSENTIAL
```mojo
# ✅ CORRECT - Graph valid after each insertion
for node in nodes:
    insert_node(node)
    assert graph_is_searchable()  # Can find this node

# ❌ WRONG - Batch construction with deferred connections
insert_all_nodes(nodes)
connect_all_nodes()  # Graph broken until here!
```

**Why:** Each insertion must maintain a valid, searchable graph. Bulk operations that defer connectivity break this.

### 5. Layer Assignment Must Use Exponential Decay
```mojo
# ✅ CORRECT - Exponential probability decay
fn get_random_level() -> Int:
    var level = 0
    while random() < ml and level < max_levels:
        level += 1
    return level

# ❌ WRONG - Uniform or deterministic assignment
level = node_id % max_levels  # Breaks hierarchy!
```

**Why:** Exponential decay creates proper hierarchical structure with few high-level nodes and many low-level nodes.

## 🔴 QUALITY INVARIANTS (95%+ Recall Required)

### 1. M (Connections per Layer) Must Be Sufficient
```yaml
Production Values:
  M: 16         # Bidirectional connections at upper layers
  M0: 32        # Bidirectional connections at layer 0
  ef_construction: 200  # Candidates during construction

Never Use:
  M < 5         # Too sparse, poor connectivity
  M0 < M*2      # Layer 0 needs denser connections
  ef_construction < 100  # Misses good connections
```

### 2. Distance Calculations Must Be Exact During Construction
```mojo
# ✅ CORRECT - Exact distance for graph construction
var dist = euclidean_distance(vec_a, vec_b)

# ❌ WRONG - Approximate distance during construction
var dist = binary_quantized_distance(vec_a, vec_b)  # Builds wrong graph!
```

**Why:** Graph structure depends on accurate distances. Approximations during construction permanently damage quality.

### 3. Visited Tracking Must Be Consistent
```mojo
# ✅ CORRECT - Track visited nodes properly
var visited = Set[Int]()
if node_id not in visited:
    visited.add(node_id)
    process_node(node_id)

# ❌ WRONG - Missing or incorrect tracking
process_node(node_id)  # May visit repeatedly!
```

**Why:** Infinite loops or missed nodes without proper visited tracking.

## 🔥 PERFORMANCE INVARIANTS (Don't Break While Optimizing)

### 1. Memory Safety
```mojo
# ✅ CORRECT - Proper lifecycle management
var ptr = UnsafePointer[Float32].alloc(size)
# ... use ptr ...
ptr.free()  # Always free

# ❌ WRONG - Dangling pointers
var ptr = get_vector(node_id)
resize_pool()  # ptr now invalid!
use(ptr)  # SEGFAULT!
```

### 2. Thread Safety for Shared State
```mojo
# ✅ CORRECT - Synchronize shared state
with lock:
    graph.add_edge(a, b)

# ❌ WRONG - Concurrent modification
parallel_for i in range(n):
    graph.add_edge(i, j)  # RACE CONDITION!
```

### 3. Batch Size Limits
```mojo
# ✅ CORRECT - Bounded batches
var batch_size = min(1000, remaining)

# ❌ WRONG - Unbounded allocation
var batch_size = total_vectors  # OOM at scale!
```

## 📊 VALIDATION REQUIREMENTS

### After Every Optimization
```python
def validate_invariants(index):
    # 1. Test hierarchical structure
    assert entry_point_is_highest_level()

    # 2. Test bidirectional connections
    for edge in all_edges:
        assert reverse_edge_exists(edge)

    # 3. Test search quality
    recall = measure_recall_at_10()
    assert recall >= 0.95

    # 4. Test progressive construction
    for i in range(100):
        insert_vector(random_vector())
        assert can_find_vector()
```

### Performance Regression Tests
```python
def test_performance():
    vectors = generate_test_vectors(10000)

    start = time.time()
    insert_bulk(vectors)
    duration = time.time() - start

    rate = 10000 / duration
    assert rate > 800  # Minimum vec/s

    recall = measure_recall()
    assert recall > 0.95  # Quality requirement
```

## 🚨 COMMON VIOLATIONS AND CONSEQUENCES

| Violation | Consequence | Example Speed | Example Recall |
|-----------|-------------|---------------|----------------|
| Skip hierarchical navigation | Disconnected subgraphs | 27,604 vec/s | 1% |
| Missing bidirectional edges | Unreachable nodes | 18,234 vec/s | 0.1% |
| Wrong entry point | Search starts wrong | 9,607 vec/s | 15% |
| Batch without connections | Broken until complete | 22,803 vec/s | 1.5% |
| Approximate distances in construction | Wrong graph structure | 15,000 vec/s | 60% |

## ✅ SAFE OPTIMIZATIONS (Won't Break Invariants)

### Can Optimize:
- Distance calculations (SIMD, vectorization)
- Memory allocation (pre-allocation, pooling)
- Cache locality (data layout, prefetching)
- Parallel independent work (segments, allocation)
- Batch operations (vector copying, serialization)

### Cannot Change:
- Hierarchical navigation order
- Bidirectional connection requirement
- Entry point management
- Progressive construction
- Layer assignment probability

## 🎯 THE GOLDEN RULE

**If an optimization breaks ANY invariant, it's not an optimization - it's a bug.**

Better to have:
- 1,000 vec/s with 95% recall (useful)

Than:
- 50,000 vec/s with 10% recall (useless)

## Summary Checklist

Before committing any HNSW optimization:

- [ ] Hierarchical navigation preserved?
- [ ] Bidirectional connections maintained?
- [ ] Entry point correctly updated?
- [ ] Graph valid after each insertion?
- [ ] Layer assignment uses exponential decay?
- [ ] Distance calculations exact during construction?
- [ ] Memory properly managed?
- [ ] Thread safety ensured?
- [ ] Recall ≥ 95% verified?
- [ ] No segfaults or crashes?

**If any answer is NO, the optimization is REJECTED.**

---
*These invariants are based on failed attempts that achieved high speed but destroyed quality. They are NON-NEGOTIABLE.*