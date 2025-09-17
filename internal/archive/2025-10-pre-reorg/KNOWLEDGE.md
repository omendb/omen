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

## HNSW+ Performance Issues & Fixes

### Fixed Capacity Trap
```mojo
# ❌ WRONG: Fixed capacity limits scale
var vectors = UnsafePointer[Float32].alloc(100000 * dimension)

# ✅ CORRECT: Dynamic growth
fn grow(mut self):
    var new_capacity = self.capacity * 2
    var new_vectors = UnsafePointer[Float32].alloc(new_capacity * self.dimension)
    memcpy(new_vectors, self.vectors, self.size * self.dimension * 4)
    self.vectors.free()
    self.vectors = new_vectors
```

### Search Overhead During Insertion
```mojo
# ❌ WRONG: Exploring too many candidates
var ef = M * 4  # Exploring 64+ candidates per layer!

# ✅ CORRECT: Minimal exploration for insertion
var ef = M  # Only explore M candidates (8 vs 64)
```

### Memory Allocations in Hot Path
```mojo
# ❌ WRONG: Allocating on every operation
fn search():
    var candidates = KNNBuffer(ef)  # Allocation!

# ✅ CORRECT: Object pooling
var search_buffer_pool: List[KNNBuffer]
fn get_buffer() -> KNNBuffer:
    if len(self.pool) > 0:
        return self.pool.pop()
    return KNNBuffer(ef)
```

### Key Performance Targets
- **Insert**: 20,000+ vec/s achievable with fixes
- **Search**: 0.15ms already excellent (keep it)
- **Memory**: 6-8KB/vector is acceptable
- **Scale**: Dynamic growth enables unlimited scale

### Implementation  
- ❌ Not batching operations (FFI overhead)
- ❌ Forgetting to clear() between tests (singleton)
- ❌ Using Mojo stdlib collections (huge overhead)

### Business
- ❌ Competing on performance alone
- ❌ Ignoring PostgreSQL integration need
- ❌ Complex pricing models

## Competitive Intelligence

### Industry Performance Standards (2024-2025 Analysis)
**Insertion Rates (vectors/sec)**:
- FAISS GPU (cuVS): 2.6M vec/s (100M × 96D, batch mode)
- FAISS CPU: 1.0M vec/s (Intel Xeon, IVF-Flat)
- Hnswlib: 600-900K vec/s (128D, SIMD optimized)
- Qdrant: 500K vec/s (128D, recall@10 ≥ 95%)
- Milvus: 400-600K vec/s (768D)
- Weaviate: 200K vec/s (256D)
- **Industry Standard: 25K+ vec/s**
- **OmenDB Current: 5.6K peak, 133 typical** ❌

**State-of-the-Art Optimization Techniques**:
1. **Multi-threading** (ALL competitors): 5-8x gain with 16-core utilization
2. **SIMD optimization** (FAISS, Qdrant): AVX2/AVX-512 intrinsics, 2-3x gain
3. **Memory optimizations** (Weaviate, Milvus): 64-byte alignment, memory mapping, 1.5-2x gain
4. **Optimal parameters** (Pinecone, Qdrant): M=32, efConstruction=200

### Mojo-Native Threading Implementation (CORRECTED)
```mojo
# TRUE parallel insertion using Mojo's native parallelize (NOT Python!)
from algorithm import parallelize
from sys import num_logical_cores

fn get_optimal_workers() -> Int:
    """Hardware-aware worker count: leave 1 core for OS, cap at 16."""
    var cores = num_logical_cores()
    return min(max(1, cores - 1), 16)

fn parallel_insert_bulk(mut self, vectors: UnsafePointer[Float32], n_vectors: Int) -> List[Int]:
    """Native Mojo parallel insertion - 5-8x speedup vs single thread."""
    var results = List[Int]()
    var num_workers = get_optimal_workers()  # 15 on 16-core system
    var chunk_size = n_vectors // num_workers
    
    # Thread-safe chunk results collection
    var chunk_results = List[List[Int]](capacity=num_workers)
    for i in range(num_workers):
        chunk_results.append(List[Int]())
    
    # TRUE PARALLELISM: Mojo native, zero FFI overhead
    @parameter 
    fn process_chunk(worker_id: Int):
        var start = worker_id * chunk_size
        var end = min(start + chunk_size, n_vectors) if worker_id < num_workers - 1 else n_vectors
        var chunk_ptr = vectors + (start * self.dimension)
        var chunk_count = end - start
        
        # Process chunk on this thread (lock-free HNSW regions)
        var chunk_ids = self._insert_chunk_lockfree(chunk_ptr, chunk_count)
        chunk_results[worker_id] = chunk_ids  # Store results
    
    # 🚀 NATIVE MOJO THREADING - No Python, no FFI, true parallelism!
    parallelize[process_chunk](num_workers)
    
    # Merge results from all workers
    for worker_results in chunk_results:
        for id in worker_results[]:
            results.append(id[])
    
    return results  # Expected: 5-8x speedup vs sequential
```

### Idiomatic Mojo SIMD (IMPLEMENTED & TESTED)
```mojo
# ✅ IMPLEMENTED: Let Mojo compiler optimize instead of hand-tuned intrinsics
fn euclidean_distance_optimized(a: UnsafePointer[Float32], b: UnsafePointer[Float32], dim: Int) -> Float32:
    var sum = Float32(0)
    # Simple loop - Mojo compiler will vectorize automatically
    for i in range(dim):
        var diff = a[i] - b[i]
        sum += diff * diff
    return sqrt(sum)
    # ✅ TESTED RESULTS: 1,425 vec/s stable performance, 0.68ms search latency

fn bulk_distances(queries: UnsafePointer[Float32], vectors: UnsafePointer[Float32], 
                 num_queries: Int, num_vectors: Int, dim: Int) -> UnsafePointer[Float32]:
    var results = UnsafePointer[Float32].alloc(num_queries * num_vectors)
    
    # Compiler will detect nested loop parallelization opportunities
    for i in range(num_queries):
        for j in range(num_vectors):
            var query_ptr = queries + (i * dim)
            var vector_ptr = vectors + (j * dim)
            results[i * num_vectors + j] = euclidean_distance_optimized(query_ptr, vector_ptr, dim)
    
    return results
```

### Memory Optimization Patterns (Competitor Analysis)
```mojo
# 64-byte cache line alignment (industry standard)
struct AlignedVectorStorage:
    var data: UnsafePointer[Float32]
    var capacity: Int
    
    fn __init__(out self, num_vectors: Int, dimension: Int):
        var total_size = num_vectors * dimension * sizeof[Float32]()
        var aligned_size = (total_size + 63) & ~63  # 64-byte align
        self.data = UnsafePointer[Float32].alloc(aligned_size // sizeof[Float32]())
        self.capacity = aligned_size // (dimension * sizeof[Float32]())

# Memory mapping for large datasets (Weaviate/Milvus pattern)
fn memory_map_vectors(file_path: String) -> UnsafePointer[Float32]:
    # Use Python mmap until Mojo native support
    var mmap = Python.import_module("mmap")
    var os = Python.import_module("os")
    
    var fd = os.open(file_path, os.O_RDONLY)
    var mapped = mmap.mmap(fd, 0, access=mmap.ACCESS_READ)
    var ptr = UnsafePointer[Float32].from_address(int(mapped._data))
    return ptr
```

### Optimal HNSW Parameters (Competitor Standards)
```mojo
# Pinecone/Qdrant production values
alias OPTIMAL_M = 32                    # vs our current 16
alias OPTIMAL_EF_CONSTRUCTION = 200     # vs our current 100  
alias OPTIMAL_MAX_M = 32                # vs our current 16
alias OPTIMAL_ML = 1.0 / log(2.0)      # Standard multiplier

# Expected performance impact
# M: 32 vs 16 = +50% recall, +20% memory, same speed
# efConstruction: 200 vs 100 = +15% recall, +30% build time
```

### pgvector Weaknesses (Validated)
- Slow index builds (10K vectors/sec vs our potential 41K+)
- No GPU support (Mojo advantage)
- Hard to tune parameters (our auto-tuning)
- Limited to PostgreSQL (our standalone advantage)

### Our Competitive Advantages
- **Mojo GPU compilation**: Unique 2-5x acceleration path
- **Zero-copy Python FFI**: No serialization overhead vs competitors
- **Idiomatic SIMD**: Compiler optimization vs hand-tuned complexity
- **16-core threading**: Match industry standard parallelization
- **Memory efficiency**: SparseMap gives 180x improvement over naive approaches

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

## Storage Implementation Lessons (Feb 2025)

### Critical Discovery: 373x Overhead Catastrophe
The original `memory_mapped.mojo` (1,168 lines) pre-allocated 64MB minimum causing:
- 100 vectors → 112MB (373x overhead!)
- Memory reporting broken (always 64 bytes)
- Files couldn't grow dynamically

### Solution: Storage V2
Rewrote in ~300 lines with:
- **1.00008x overhead** (essentially perfect)
- Dynamic file growth
- Accurate memory reporting
- 100% data integrity at 100K vectors

### Current Performance
- **Throughput**: 439 vec/s (Python I/O bottleneck)
- **Recovery**: 41K vec/s
- **Scale**: Works well up to 100K vectors

### Optimization Findings (Feb 2025)
1. **Batch writes**: Only 1.02x speedup (Python I/O still bottleneck)
2. **PQ Compression**: Works but needs re-compression after training
3. **Real bottleneck**: Python FFI overhead, need direct mmap

### What Actually Works
- Storage V2: 1.00008x overhead (perfect!)
- Recovery: 41K vec/s (excellent)
- Compression: PQ code exists and works
- Throughput: Stuck at ~440 vec/s due to Python I/O

### Next Priority
**Must implement direct mmap** to bypass Python:
```mojo
# Current bottleneck:
self.data_file.write(bytes)  # Python FFI overhead

# Solution from memory_mapped.mojo:
external_call["mmap"](...)  # Direct system call
```

### Key Pattern
```mojo
# Simple is better - don't over-engineer
struct VectorStorage:
    var data_file: PythonObject  # Simple Python I/O
    var index_file: PythonObject  # Separate index
    var id_map: Dict[String, Int]  # In-memory index
```

## HNSW Quality Patterns (Critical Learnings)

### Bulk Insertion Must Navigate Hierarchy
```mojo
// ❌ WRONG: Direct connection without navigation
for node in bulk_nodes:
    connect_to_random_nodes(node)

// ✅ RIGHT: Navigate from entry point through layers
var curr_nearest = self.entry_point
for lc in range(entry_level, layer, -1):
    curr_nearest = self._search_layer_simple(vector, curr_nearest, 1, lc)
```

### Graph Connectivity Requires Sufficient Sampling
```mojo
// ❌ WRONG: Sample too few nodes
var sample_size = min(20, self.size)

// ✅ RIGHT: Sample at least 100 nodes or 10% of graph
var sample_size = min(self.size, max(100, self.size // 10))
```

### Use Correct Search Parameters
```mojo
// ❌ WRONG: Use M for search (way too small)
var ef = M  // Only 16!

// ✅ RIGHT: Use ef_construction for quality
var ef = ef_construction  // 200 for good recall
```

### Bidirectional Connections Need Pruning
```mojo
// ❌ WRONG: Add reverse connection without pruning
neighbor[].add_connection(layer, node_id)

// ✅ RIGHT: Add connection then prune to maintain capacity
neighbor[].add_connection(layer, node_id)
self._prune_connections(neighbor_id, layer, M_layer)
```

### Adaptive Strategy for Small Datasets
```mojo
// ❌ WRONG: Always use HNSW regardless of size
if true: use_hnsw()

// ✅ RIGHT: Use flat buffer for small datasets
if dataset_size < 500:
    use_flat_buffer()  // 100% recall, 2-4x faster
else:
    use_hnsw()  // Scalable for large datasets
```

### Migration Requires Careful Memory Management
- Small migrations (<500): Work perfectly
- Large migrations (1000+): Can segfault if not batched
- Solution: Batch large migrations or use pure bulk insertion

### Quality Testing Must Use Realistic Queries
- Exact match testing: Can hide real issues
- Random queries: Better represent production use
- Always test recall@1 and recall@10

