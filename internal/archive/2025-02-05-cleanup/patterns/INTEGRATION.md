# OmenDB Component Integration Guide
*How to connect OmenDB components correctly*

## Core Component Relationships

```
VectorBuffer ──→ DiskANNIndex ──→ VamanaGraph
     │               │                │
     ├─ SparseMap     ├─ RobustPrune   ├─ CSR Graph (BROKEN)
     ├─ Quantization  ├─ Search        └─ Adjacency List (NEEDED)
     └─ FFI Layer     └─ Build
                       │
                       └─ ProductQuantization (DISCONNECTED)
```

## Integration Patterns

### PATTERN: Vamana + PQ Integration
```mojo
# Current State: Components exist but disconnected
struct VamanaIndex:       # ✅ Working
struct ProductQuantizer:  # ✅ Working but separate

# Required Integration:
struct VamanaWithPQ:
    var vamana: VamanaIndex
    var quantizer: ProductQuantizer
    var compressed_vectors: List[UInt8]
    
    fn add_node(self, vector: List[Float32], id: String):
        # 1. Quantize vector first
        let quantized = self.quantizer.quantize(vector)
        # 2. Store compressed version  
        self.compressed_vectors.append(quantized)
        # 3. Build graph with original vector
        self.vamana.add_node(vector, id)
        
    fn search(self, query: List[Float32], k: Int):
        # 1. Search graph with full precision
        let candidates = self.vamana.search(query, k)
        # 2. Re-rank with compressed vectors if needed
        return candidates
```

### PATTERN: Buffer → Index Integration  
```mojo
# Current Problem: Buffer flushes one-by-one
struct VectorBuffer:
    fn flush_to_index(self):
        # ❌ CURRENT (slow): Individual adds
        for i in range(self.size):
            self.index.add_node(self.vectors[i], self.ids[i])
            
        # ✅ REQUIRED (fast): Batch add
        self.index.add_batch(
            self.vectors.slice(0, self.size),
            self.ids.slice(0, self.size)
        )
```

### PATTERN: Graph Structure Integration
```mojo
# Current Problem: CSR cannot remove edges
struct CSRGraph:  # ❌ Cannot prune edges
    var neighbors: List[List[Int]]  # Fixed after creation

# Required: Adjacency List  
struct AdjacencyListGraph:  # ✅ Can add/remove edges
    var adjacency: Dict[Int, List[Int]]
    
    fn prune_edges(self, node: Int, max_degree: Int):
        # Can actually remove edges during RobustPrune
        if self.adjacency[node].size > max_degree:
            # Remove worst edges based on α-RNG
            self.adjacency[node] = robust_prune(...)
```

## Integration Commands

### Connect Vamana to PQ
```bash
# 1. Test current state
cd /Users/nick/github/omendb/omendb
python -c "
from omendb.algorithms.vamana import VamanaIndex
from omendb.compression.product_quantization import ProductQuantizer
# Test both work independently
"

# 2. Create integration test
cat > tests/test_vamana_pq.py << 'EOF'
def test_vamana_with_pq():
    vamana = VamanaIndex(...)
    pq = ProductQuantizer(...)
    # Test memory reduction
    assert memory_usage_with_pq < memory_usage_without_pq / 5
EOF

# 3. Implement combined struct
# Edit: native.mojo - add VamanaWithPQ implementation
```

### Fix Buffer Flush Integration
```bash
# 1. Profile current flush performance
python -c "
import time
db = omendb.DB()
start = time.time()
db.add_batch(vectors[9999:10010])  # Trigger flush
print(f'Flush time: {time.time() - start}s')
"

# 2. Implement batch flush
# Edit: native.mojo:1850-2000 (VectorStore.flush method)
# Replace: for-loop with add_batch call

# 3. Benchmark improvement
# Should see 10x+ improvement in flush time
```

### Replace CSR with Adjacency List
```bash
# 1. Create new graph implementation
# File: omendb/graph/adjacency_list.mojo
# Implement: add_edge, remove_edge, get_neighbors

# 2. Update DiskANNIndex to use new graph
# File: omendb/algorithms/diskann.mojo  
# Replace: CSRGraph with AdjacencyListGraph

# 3. Test edge pruning capability
python -c "
graph = AdjacencyListGraph()
graph.add_edge(0, 1)
graph.remove_edge(0, 1)  # Should work (unlike CSR)
assert graph.get_degree(0) == 0
"
```

## Integration Testing

### Test Component Connections
```python
# Pattern: Test each integration point
def test_buffer_to_index():
    buffer = VectorBuffer(capacity=1000)
    buffer.add_batch(vectors[:500])
    
    # Test flush integration
    old_size = index.size()
    buffer.flush_to_index()
    assert index.size() == old_size + 500
    
def test_vamana_pq_memory():
    # Without PQ
    vamana_only = VamanaIndex(vectors)
    memory_without = measure_memory(vamana_only)
    
    # With PQ
    vamana_pq = VamanaWithPQ(vectors, pq_subvectors=8)  
    memory_with = measure_memory(vamana_pq)
    
    # Should see significant reduction
    assert memory_with < memory_without / 5
```

### Integration Benchmarks
```bash
# End-to-end performance test
cd /Users/nick/github/omendb/omendb
python benchmarks/integration_benchmark.py

# Should test:
# 1. Buffer → Index batch performance  
# 2. Vamana + PQ memory usage
# 3. Graph build with new structure
# 4. Search quality maintenance
```

## Integration Failure Patterns

### Anti-Pattern: Component Mismatch
```mojo
# ❌ WRONG: Different data types
buffer.vectors: List[Float32]    
index.vectors: List[Float64]     # Type mismatch

# ✅ CORRECT: Consistent types
buffer.vectors: List[Float32]
index.vectors: List[Float32]
```

### Anti-Pattern: Memory Layout Mismatch  
```mojo
# ❌ WRONG: Non-contiguous memory
buffer: List[List[Float32]]      # List of lists (fragmented)

# ✅ CORRECT: Contiguous memory  
buffer: FlatArray[Float32]       # Single contiguous block
```

### Anti-Pattern: Missing Error Handling
```mojo
# ❌ WRONG: No integration validation
vamana.add_batch(buffer.vectors)  # What if sizes don't match?

# ✅ CORRECT: Validate integration
if buffer.size != buffer.ids.size:
    raise "Vector-ID count mismatch"
vamana.add_batch(buffer.vectors, buffer.ids)
```

## Integration Decision Tree

```
IF adding new vectors:
    IF count < 1000:
        → Add to VectorBuffer (fast)
    ELIF buffer nearly full:
        → Trigger batch flush to DiskANN
        → Use VamanaWithPQ if memory constrained
    ELSE:
        → Direct add to index

IF search query:
    IF using PQ compression:
        → Search graph + re-rank with compressed
    ELSE:
        → Direct graph search

IF graph growing unbounded:
    → CRITICAL: Replace CSR with AdjacencyList  
    → Enable proper edge pruning
```

---
*Component integration patterns for OmenDB architecture*