# Real Memory Fix for OmenDB

## The Truth
- **Claimed**: 156 bytes/vector
- **Reality**: 3,080 bytes/vector (20x worse)
- **Python sees**: ~500 bytes/vector
- **Native uses**: ~2,500 bytes/vector (hidden from Python)

## Root Causes (In Order of Impact)

### 1. Vectors Stored Multiple Times
- VectorBuffer stores vectors (Float32 or quantized)
- DiskANN index ALSO stores vectors
- When flush happens, vectors aren't removed from buffer
- Result: 2x or 3x storage

### 2. Quantization Not Working Properly
- Enabled but only 9.8% reduction
- Should be 75% reduction (4 bytes → 1 byte)
- Likely storing both quantized AND original

### 3. Graph Edge Pre-allocation
- Each node pre-allocates for max_edges (64)
- 64 edges × 4 bytes = 256 bytes per node
- Even empty nodes have this overhead

### 4. String ID Storage
- Every vector ID stored as String
- Strings in Mojo have overhead
- Could use integer IDs internally

## Implementation Plan

### Phase 1: Fix Double Storage (Quick Win)
```mojo
# When flushing buffer to index:
fn flush_buffer(mut self):
    # Move vectors to index
    for i in range(buffer.size):
        self.main_index.add(buffer.ids[i], buffer.get_vector(i))
    
    # CRITICAL: Clear buffer after flush
    self.buffer.clear()  # Must actually free memory, not just reset size
```

### Phase 2: Fix Quantization
```mojo
# Ensure ONLY quantized storage when enabled
if use_quantization:
    # ONLY allocate UInt8 storage
    self.quantized_data = alloc(capacity * dimension)  # 128 bytes per vector
    # DO NOT allocate Float32 storage AT ALL
    self.data = UnsafePointer[Float32]()  # Null pointer
```

### Phase 3: Dynamic Graph Growth
```mojo
# Start with zero edges, grow as needed
fn add_edge(mut self, from_node: Int, to_node: Int):
    if self.edge_count[from_node] >= self.edge_capacity[from_node]:
        self.grow_edges(from_node)  # Only grow when needed
```

### Phase 4: Integer IDs Internally
```mojo
# Use integers internally, map strings at API boundary
struct IDMapper:
    var string_to_int: Dict[String, Int]
    var int_to_string: Dict[Int, String]
    var next_id: Int
```

## Expected Results

With ALL fixes:
- Phase 1: 50% reduction (1,500 bytes/vector)
- Phase 2: 75% reduction (375 bytes/vector)
- Phase 3: 20% reduction (300 bytes/vector)
- Phase 4: 10% reduction (270 bytes/vector)

**Target: 200-300 bytes/vector achievable**

## Verification

```python
# Test memory per phase
def verify_memory(phase_name):
    db = DB(":memory:")
    db.enable_quantization()
    
    # Add 10K vectors
    vectors = np.random.randn(10000, 128).astype(np.float32)
    db.add_batch(vectors)
    
    # Measure
    memory_mb = get_memory_usage()
    bytes_per_vector = (memory_mb * 1024 * 1024) / 10000
    
    print(f"{phase_name}: {bytes_per_vector:.0f} bytes/vector")
    return bytes_per_vector < 300  # Target
```

## Priority

1. **Fix double storage** - Biggest impact, easiest fix
2. **Fix quantization** - Second biggest impact
3. **Dynamic edges** - More complex but worthwhile
4. **Integer IDs** - Nice to have