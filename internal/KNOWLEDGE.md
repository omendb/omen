# Knowledge Base - Patterns & Gotchas

## Mojo Patterns

### FFI Performance
```mojo
# ✅ Python: Zero overhead
@export
fn search(query: PythonObject) -> PythonObject:
    # Direct access via __array_interface__
    return results

# ✅ C/Rust: ~100ns overhead  
mojo build --emit shared-lib -o libomendb.so
```

### Memory Gotchas
```mojo
# ❌ NEVER use Mojo stdlib collections
Dict[String, Int]  # 8KB per entry!
List[String]       # 5KB per item!

# ❌ Nested Lists cause exponential growth
List[List[Int]]    # Doubles capacity on EACH level!
# When List grows: capacity * 2 allocation
# With nested: parent doubles AND children double = 4x+ memory

# ✅ Use custom implementations
SparseMap          # 180x better than Dict
Fixed arrays       # Predictable memory
InlineArray        # Stack allocated, no heap

# ✅ For graph structures use fixed-size arrays
struct Node:
    var connections: InlineArray[Int, MAX_CONNECTIONS]  # Stack allocated
    var count: Int  # Track actual usage
```

### Pre-allocation Pattern (Critical!)
```mojo
# ❌ DON'T: Let Lists grow dynamically
var nodes = List[Node]()  # Will double: 1→2→4→8→16...

# ✅ DO: Pre-allocate pools
struct NodePool:
    var nodes: UnsafePointer[Node]
    fn __init__(out self, capacity: Int):
        self.nodes = UnsafePointer[Node].alloc(capacity)  # One allocation!
```

### SIMD Optimization
```mojo
# Distance calculations should always use SIMD
fn cosine_distance[size: Int](a: Buffer, b: Buffer) -> Float32:
    var dot = simd[DType.float32, size].splat(0)
    # Vectorized operations 10x faster
```

## HNSW Implementation Notes

### Key Parameters
- **M**: 16 (connections per node) - sweet spot for recall/memory
- **ef_construction**: 200 - higher = better quality, slower build
- **max_M**: 16 - maximum connections in layer 0
- **seed**: Fixed for reproducible builds

### Layer Assignment
```python
# Exponentially decaying probability
level = int(-log(random()) * ml)
# Most nodes in layer 0, few in upper layers
```

### Optimization Opportunities
1. **Parallel insertion** - Each layer independent
2. **SIMD distances** - All distance calcs vectorized
3. **Lock-free updates** - Atomic operations for neighbor lists
4. **Prefetching** - Predict next nodes in search

## Vector Database Market Insights

### What Actually Matters
1. **Developer experience** > Raw performance
2. **Python native** > Language bindings  
3. **PostgreSQL compatible** > Standalone DB
4. **Simple pricing** > Complex tiers

### What Doesn't Matter (Much)
1. **Memory per vector** - RAM is cheap now
2. **Compression** - Unless at massive scale
3. **GPU support** - Most run on CPU
4. **Streaming updates** - Batch is usually fine

## Common Mistakes

### Architecture
- ❌ Over-engineering for scale before product-market fit
- ❌ Custom algorithms when standard ones work
- ❌ Optimizing wrong metrics (memory vs latency)

### Implementation  
- ❌ Not batching operations (FFI overhead)
- ❌ Forgetting to clear() between tests (singleton)
- ❌ Using Mojo stdlib collections (huge overhead)

### Business
- ❌ Competing on performance alone
- ❌ Ignoring PostgreSQL integration need
- ❌ Complex pricing models

## Competitive Intelligence

### pgvector Weaknesses
- Slow index builds (10K vectors/sec)
- No GPU support
- Hard to tune parameters
- Limited to PostgreSQL

### Our Advantages
- 10x faster builds with HNSW+
- Future GPU acceleration
- Python native (no PostgreSQL required)
- Auto-tuning parameters

## Testing Patterns

### Benchmark Correctly
```python
# Always warm up first
for _ in range(10):
    index.search(query)

# Then measure
start = time.time()
for _ in range(1000):
    index.search(query)
qps = 1000 / (time.time() - start)
```

### Test Data
- SIFT1M - Standard 128-dim benchmark
- OpenAI embeddings - 1536-dim real-world
- Random data - For stress testing only

## Deployment Considerations

### Single Node First
- Can handle 10M vectors easily
- Simpler than distributed
- PostgreSQL extension option

### Distribution Later
- Shard by similarity (not random)
- Coordinator nodes for routing
- Eventual consistency fine

## Lessons Learned

1. **DiskANN wrong for streaming** - Batch-oriented by design
2. **HNSW is the standard** - For good reasons
3. **Mojo FFI tricky** - But zero-copy possible
4. **Market wants PostgreSQL** - Not standalone DB
5. **Multimodal is future** - But pure vector first
## Mojo Best Practices (from archive)
# Mojo Best Practices for OmenDB
*Critical documentation for state-of-the-art vector database implementation*

## Critical Type Conversions (Most Common Errors)

### ❌ WRONG - Python-style (causes compilation errors)
```mojo
var my_int = int(some_value)        # ERROR: use of unknown declaration 'int'
var my_str = str(some_value)        # ERROR: use of unknown declaration 'str'
var my_float = float(some_value)    # ERROR: use of unknown declaration 'float'
```

### ✅ CORRECT - Mojo-style
```mojo
var my_int = Int(some_value)        # Capital I
var my_str = String(some_value)     # String not str
var my_float = Float32(some_value)  # Explicit precision
```

## Memory Management Best Practices

### Single Storage Principle
**CRITICAL**: Never store the same data multiple times
```mojo
# ❌ WRONG - Double storage
struct BadBuffer:
    var data: UnsafePointer[Float32]          # Original vectors
    var quantized: UnsafePointer[UInt8]       # Quantized vectors
    # Storing BOTH wastes memory!

# ✅ CORRECT - Single storage
struct GoodBuffer:
    var data: UnsafePointer[Float32]          # Only if NOT quantized
    var quantized: UnsafePointer[UInt8]       # Only if quantized
    var use_quantization: Bool                # Flag to know which
```

### Move Semantics Over Copy
```mojo
# ❌ WRONG - Copying data
fn flush_buffer(self, buffer: VectorBuffer):
    for i in range(buffer.size):
        var vector = buffer.get_vector(i)     # COPY
        self.index.add(vector)                # Another COPY
        
# ✅ CORRECT - Moving data
fn flush_buffer(mut self, owned buffer: VectorBuffer):
    # Move entire buffer to index
    self.index.consume_buffer(buffer^)        # MOVE ownership
    # Buffer is now empty, no duplication
```

### Lazy Allocation Pattern
```mojo
# ❌ WRONG - Pre-allocate everything
fn __init__(out self):
    self.nodes = UnsafePointer[Node].alloc(1_000_000)  # 1M nodes upfront!
    self.edges = UnsafePointer[Edge].alloc(32_000_000) # 32M edges!

# ✅ CORRECT - Start small, grow as needed
fn __init__(out self):
    self.capacity = 100                      # Start small
    self.nodes = UnsafePointer[Node].alloc(self.capacity)
    
fn add_node(mut self):
    if self.size >= self.capacity:
        self._grow()                          # Double when needed
```

## State-of-the-Art Memory Targets

### Industry Standards (FAISS, ScaNN, Qdrant)
- **Float32 vectors**: 512 bytes/vector (128D × 4 bytes)
- **8-bit quantized**: 136 bytes/vector (128D × 1 byte + metadata)
- **Binary quantized**: 20 bytes/vector (128D / 8 bits + metadata)

### OmenDB Achieved (Aug 26, 2025)
- **With quantization**: 208 bytes/vector ✅
- **Without quantization**: 800 bytes/vector

### Current Problem (Aug 29, 2025)
- **With quantization**: 2,800+ bytes/vector ❌ (13x regression!)

## Common Mojo Performance Patterns

### SIMD Operations
```mojo
# Hardware-adaptive SIMD width
alias simd_width = simdwidthof[DType.float32]()

fn dot_product(a: UnsafePointer[Float32], b: UnsafePointer[Float32], dim: Int) -> Float32:
    var sum = SIMD[DType.float32, simd_width](0)
    
    # Process SIMD-width chunks
    for i in range(0, dim, simd_width):
        var va = a.load[width=simd_width](i)
        var vb = b.load[width=simd_width](i)
        sum = va.fma(vb, sum)
    
    return sum.reduce_add()
```

### Zero-Copy FFI
```mojo
# ❌ WRONG - Copying from Python
fn add_vector(py_array: PythonObject):
    var vector = List[Float32]()
    for i in range(len(py_array)):
        vector.append(Float32(py_array[i]))  # COPY each element!

# ✅ CORRECT - Direct pointer access
fn add_vector(data_ptr: UnsafePointer[Float32], size: Int):
    # Use pointer directly, no copy
    self.buffer.add_view(data_ptr, size)
```

## Critical Architecture Decisions

### 1. Buffer → Index Flow (NO DUPLICATION)
```mojo
struct VectorStore:
    var buffer: VectorBuffer         # Temporary holding
    var index: DiskANNIndex          # Permanent storage
    
    fn flush(mut self):
        # MOVE vectors from buffer to index
        self.index.consume(self.buffer^)
        self.buffer = VectorBuffer()  # New empty buffer
```

### 2. Quantization Storage (EITHER/OR)
```mojo
struct CSRGraph:
    # Store EITHER full precision OR quantized, never both
    var vectors: UnsafePointer[Float32]      # Only if NOT quantized
    var quantized: UnsafePointer[UInt8]      # Only if quantized
    var use_quantization: Bool
    
    fn memory_bytes(self) -> Int:
        if self.use_quantization:
            return self.num_nodes * (self.dimension + 8)  # 136 bytes/vector
        else:
            return self.num_nodes * self.dimension * 4     # 512 bytes/vector
```

### 3. Proper Pre-allocation Sizes
```mojo
# Based on empirical analysis of production workloads
alias DEFAULT_BUFFER_SIZE = 100        # Flush frequently
alias INITIAL_INDEX_CAPACITY = 100     # Start small
alias GROWTH_FACTOR = 1.5               # Grow moderately
alias MAX_DEGREE = 16                   # Edges per node (not 32 or 64!)
```

## Memory Debugging Checklist

When memory usage exceeds targets:

1. **Check for double storage**
   - Are vectors stored in both buffer AND index?
   - Are we keeping both quantized AND original?

2. **Check pre-allocations**
   - Search for `.alloc(` with large numbers
   - Look for `* 1000` or `* 10000` multiplications
   - Verify DEFAULT constants aren't too large

3. **Check for copies vs moves**
   - Look for `vector.append` in loops
   - Check if `owned` keyword is used for transfers
   - Verify `^` operator for moves

4. **Measure actual allocations**
   ```python
   import psutil
   process = psutil.Process()
   before = process.memory_info().rss
   # Operation
   after = process.memory_info().rss
   print(f"Allocated: {(after-before)/1024/1024:.1f} MB")
   ```

## Quick Reference: Python ↔ Mojo

| Python | Mojo | Notes |
|--------|------|-------|
| `int()` | `Int()` | Capital I |
| `str()` | `String()` | Full name |
| `float()` | `Float32()` or `Float64()` | Explicit precision |
| `list.append()` | `list.append()` | Same, but check for copies |
| `del obj` | `obj^` to move, `__del__` for cleanup | RAII pattern |
| `@property` | `fn get_x(self)` | No properties yet |
| `**kwargs` | Not supported | Use explicit parameters |

## Common Compilation Errors → Solutions

- `use of unknown declaration 'int'` → Use `Int()`
- `use of unknown declaration 'str'` → Use `String()`  
- `cannot convert Float64 to Float32` → Use `Float32(value)`
- `could not deduce parameter` → Add type annotations
- `no attribute 'get'` → Use index access `[0]` for tuples
- `unable to locate module` → Use relative imports within packages

## Performance Verification

Always verify optimizations with actual measurements:

```python
# Test memory efficiency
vectors_added = 1000
memory_used_mb = (after - before) / (1024 * 1024)
bytes_per_vector = (memory_used_mb * 1024 * 1024) / vectors_added

# Success criteria
assert bytes_per_vector <= 200  # Quantized target
assert bytes_per_vector <= 600  # Non-quantized target
```

---
*Remember: State-of-the-art means matching FAISS/ScaNN efficiency while maintaining Mojo's safety and performance guarantees.*
## Multimodal Architecture Patterns

### Query Planning
```mojo
# Selectivity-based execution order
if filter_selectivity < 0.01:  # Very selective
    # Filter first, then vector search
    candidates = metadata_filter(filters)
    results = vector_search(candidates, query)
elif text_selectivity < 0.05:  # Specific text
    # Text search first
    candidates = text_search(text_query)
    results = vector_filter(candidates, query)
else:
    # Vector search first (default)
    candidates = vector_search(query, ef=500)
    results = apply_filters(candidates, filters)
```

### Storage Tiering
```mojo
# Automatic data movement based on age/access
struct TieredStorage:
    var hot: NVMe    # Last 7 days, <1ms latency
    var warm: SSD     # 7-30 days, <10ms latency  
    var cold: S3      # >30 days, <100ms latency
    
    fn auto_tier(self):
        # Move based on access patterns
        if last_access > 30_days:
            move_to_cold()
```

### Competitive Advantages

#### vs MongoDB Atlas
- 10x faster (<10ms vs 50-100ms latency)
- 10x cheaper (efficient storage)
- Open source option available
- GPU compilation path

#### vs LanceDB  
- GPU support (Mojo exclusive)
- Python-native (no FFI overhead)
- Better text search (proper BM25)
- ACID transactions support

### Mojo Workarounds

#### Missing async/await
```mojo
# Use thread pool pattern
from python import ThreadPoolExecutor

fn async_search(queries: List[Query]):
    with ThreadPoolExecutor() as pool:
        futures = [pool.submit(search, q) for q in queries]
        return [f.result() for f in futures]
```

#### Limited stdlib
```mojo
# Implement only what we need
struct SimpleDict[K, V]:
    var keys: DynamicVector[K]
    var values: DynamicVector[V]
    # 100x more efficient than stdlib Dict
```

## Production Considerations

### Cloud Architecture
- **Control plane**: API routing, shard management
- **Data plane**: Auto-scaling compute nodes
- **Storage plane**: Tiered S3/SSD/RAM

### Cost Model (1M vectors)
- Storage: $30/month (tiered)
- Compute: $400/month (auto-scaled)
- Can charge: $5,000/month (platform pricing)

### Marketing Position
- Internal: "Fast multimodal database competing with MongoDB Atlas"
- External: "Open source multimodal database for AI applications"
- Never mention competitors by name in public materials

## Development Priorities

### Month 1: Core
- HNSW+ with SIMD
- Metadata filtering
- Python bindings

### Month 2: Multimodal
- BM25 text search
- Query planner
- SQL interface

### Month 3: Production
- Distributed sharding
- Cloud deployment
- Monitoring

### Month 4: Differentiation
- GPU compilation
- Advanced query optimization
- Enterprise features
