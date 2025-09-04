# Fix for Double Vector Storage Issue

## Problem
Vectors are being stored in two places:
1. `VectorStore.vector_store` dictionary (line 102, 546, 768 in native.mojo)
2. CSR graph inside `DiskANNIndex.graph` (line 70 in diskann_csr.mojo)

This causes:
- **2x memory usage** for vector data
- 778MB for 100K vectors instead of <100MB target
- Unnecessary memory copies and allocations

## Solution

### Step 1: Remove vector_store dictionary writes

In `native.mojo`, remove these lines:
- Line 546: `self.vector_store[id] = vector`
- Line 768: `self.vector_store[id] = vector`

### Step 2: Update vector retrieval

Modify `get_vector()` function (line 1738) to retrieve from CSR graph:

```mojo
fn get_vector(vector_id: PythonObject) raises -> PythonObject:
    """Get vector data by ID from CSR graph."""
    try:
        var id = String(vector_id)
        var db = get_global_db()[]
        
        # Check if vector exists in quantized forms first
        if db.use_binary_quantization and id in db.binary_vectors:
            # Existing binary dequantization code
            ...
        elif db.use_quantization and id in db.quantized_vectors:
            # Existing scalar dequantization code
            ...
        elif id in db.id_to_idx:
            # NEW: Retrieve from CSR graph instead of vector_store
            var idx = db.id_to_idx[id]
            var vec_ptr = db.main_index.graph.get_vector_ptr(idx)
            
            # Convert pointer to List[Float32]
            vector_data = List[Float32]()
            for i in range(db.dimension):
                vector_data.append(vec_ptr[i])
        else:
            return PythonObject(None)
```

### Step 3: Fix CSR graph memory tracking

In `csr_graph.mojo`, update `memory_bytes()` to be called from ComponentMemoryStats:

```mojo
fn memory_bytes(self) -> Int:
    """Calculate total memory usage."""
    # Vector storage
    var vector_bytes = self.num_nodes * self.dimension * 4
    
    # CSR structure
    var offset_bytes = (self.num_nodes + 1) * 4
    var edge_bytes = self.num_edges * 4
    
    # ID storage
    var id_bytes = 0
    for i in range(len(self.node_ids)):
        id_bytes += len(self.node_ids[i]) + 24
    
    return vector_bytes + offset_bytes + edge_bytes + id_bytes
```

### Step 4: Update ComponentMemoryStats

In `diskann_csr.mojo`, update memory stats:

```mojo
fn _update_memory_stats(mut self):
    """Update memory statistics."""
    # Get CSR graph memory
    var graph_bytes = self.graph.memory_bytes()
    self.memory_stats.graph_memory = graph_bytes
    
    # Memory pool usage
    self.memory_stats.pool_memory = self.memory_pool.total_allocated
```

## Expected Results

After this fix:
- **Memory reduction**: 778MB → ~100MB for 100K vectors (7.8x reduction)
- **No duplicate storage**: Vectors stored only in CSR graph
- **Accurate tracking**: ComponentMemoryStats shows real usage
- **Performance maintained**: Direct pointer access for retrieval

## Testing

Run these tests after implementation:
```bash
# Test memory usage
python benchmarks/test_double_storage.py

# Verify retrieval still works
python benchmarks/test_retrieval.py

# Check memory tracking
python benchmarks/memory_investigation.py
```

## Implementation Priority

1. **First**: Remove vector_store writes (immediate 50% memory savings)
2. **Second**: Update retrieval to use CSR graph
3. **Third**: Fix memory tracking for accurate reporting