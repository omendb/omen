# Proposed Fix for 10K Performance Cliff

## Problem Analysis
The performance cliff at 10K vectors is caused by the buffer flush operation in `_flush_buffer_to_main()`.

### Current Implementation (SLOW)
```mojo
# Lines 816-834 in native.mojo
for i in range(buffer_size):
    var id = buffer_ids[i]
    var vector = List[Float32]()
    # ... prepare vector ...
    _ = self.main_index.add(id, vector)  # ❌ EXPENSIVE! One by one
```

**Why it's slow:**
- Each `add()` call to DiskANN involves:
  1. Beam search to find nearest neighbors (O(log n))
  2. Connecting the new node to neighbors
  3. RobustPrune to maintain graph quality
- For 10K vectors, this means 10K expensive graph operations
- Measured: 39ms per 100 vectors = ~4 seconds for 10K vectors!

## Proposed Solution

### Option 1: Batch Graph Construction (Recommended)
```mojo
fn _flush_buffer_to_main(mut self) raises:
    """Flush buffer using batch graph construction."""
    if self.buffer.size == 0:
        return
    
    # FAST PATH: Build a temporary graph from buffer vectors
    var temp_graph = DiskANNIndex(
        dimension=self.dimension,
        expected_nodes=self.buffer.size,
        use_quantization=self.use_quantization
    )
    
    # Add all buffer vectors to temp graph (fast batch construction)
    for i in range(self.buffer.size):
        temp_graph.add(self.buffer.ids[i], get_buffer_vector(i))
    
    if self.main_index.size() > 0:
        # MERGE: Connect temp graph to main graph efficiently
        self.main_index.merge_graph(temp_graph)
    else:
        # INITIAL: temp graph becomes main index
        self.main_index = temp_graph
    
    # Clear buffer
    self.buffer.clear()
```

**Benefits:**
- Batch construction is much faster than incremental
- Graph merge can be optimized (connect representatives, not all nodes)
- Maintains graph quality through proper pruning

### Option 2: Deferred Graph Building
```mojo
fn _flush_buffer_to_main(mut self) raises:
    """Flush buffer but defer graph construction."""
    
    # Just store vectors, don't build connections yet
    for i in range(self.buffer.size):
        self.main_index.add_vector_only(self.buffer.ids[i], vector)
    
    # Build graph connections in batch when needed
    if self.main_index.needs_rebuild():
        self.main_index.rebuild_graph_batch()
```

**Benefits:**
- Instant flush (just memory copy)
- Graph built only when needed (on search or explicit build)
- Can use parallel construction

### Option 3: Increase Default Buffer Size (Quick Fix)
```mojo
# Change default buffer size from 10000 to 100000
alias __buffer_size = 100000  # Was 10000
```

**Benefits:**
- Simple one-line change
- Delays the cliff to 100K vectors
- Gives time to implement proper solution

**Drawbacks:**
- Doesn't solve the problem, just delays it
- Uses more memory (100K vectors in buffer)

## Implementation Priority

1. **Immediate**: Increase buffer size to 100K (Option 3)
   - Quick fix to unblock users
   - One-line change

2. **Short-term**: Implement batch graph construction (Option 1)
   - Proper solution
   - Requires adding merge_graph() method to DiskANN

3. **Long-term**: Consider deferred building (Option 2)
   - Best for bulk loading scenarios
   - More complex implementation

## Expected Performance Impact

Current: 95K vec/s → 10K vec/s at 10K boundary (10x degradation)
With fix: 95K vec/s sustained to 100K+ vectors (no cliff)

## Testing Plan

1. Test with various buffer sizes (100, 1K, 10K, 100K)
2. Measure flush time at each size
3. Verify graph quality after batch construction
4. Test search accuracy after merge