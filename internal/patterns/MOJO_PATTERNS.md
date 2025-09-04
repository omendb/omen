# OmenDB Mojo Solutions & Patterns
*OmenDB-specific solutions to Mojo limitations*

## ⚠️ General Mojo Issues
For stdlib memory overhead, known crashes, and language limitations, see:
[agent-contexts/languages/mojo/KNOWN_ISSUES.md](https://github.com/nijaru/agent-contexts/blob/main/languages/mojo/KNOWN_ISSUES.md)

## OmenDB Solutions to Memory Issues

### SparseMap Implementation (180x improvement)
```mojo
# Problem: Dict[String, Int] uses 8KB per entry (100x expected!)
# Discovery: Found during 20K crash debugging (December 2024)
# Impact: 3 Dict instances × 20K entries × 8KB = 480MB overhead alone!
# Solution: Custom SparseMap - 44 bytes per entry

# Where Dict was causing crashes:
metadata_store: Dict[String, String]  # 8KB per metadata entry
quantized_vectors: Dict[String, List[UInt8]]  # 8KB per vector
binary_vectors: Dict[String, List[Bool]]  # 8KB per binary vec

# Fixed with parallel arrays:
struct SparseMetadataMap:
    var keys: List[String]      # 44 bytes per entry total
    var values: List[String]     # vs 8,192 bytes with Dict!
```

### Memory Patterns That Work
```mojo
# Arena allocation for batch operations
var arena = Arena(1024 * 1024)  # 1MB blocks
var batch_data = arena.alloc[Float32](count * 128)

# Fixed-size string pools
struct StringPool:
    var data: UnsafePointer[UInt8]
    var offsets: List[Int]
    
# Product Quantization (16x compression)
struct PQ32:
    var codes: UnsafePointer[UInt8]
    var codebook: Tensor[Float32]
```

## OmenDB FFI Optimizations

### Batch Processing Strategy
```mojo
# Problem: Python FFI has 8KB overhead per call (see KNOWN_ISSUES.md)
# OmenDB Solution: Batch all operations
add_batch(keys, vectors)  # 1,475 bytes/vector vs 8,356 individual

# Using LibC directly where possible
sys.ffi.external_call["mmap", UnsafePointer[Int8]](...)
```

## OmenDB Module Structure

### Monolith Pattern (native.mojo)
Due to Mojo import limitations (see KNOWN_ISSUES.md), OmenDB uses:
- Single `native.mojo` file for global state
- All globals and VectorStore in one place
- Relative imports for other modules

## OmenDB-Specific Crash Fixes

### Safe Unwrap Pattern
```mojo
# Problem: Optional.value() crashes on None (see KNOWN_ISSUES.md)
# OmenDB Solution: Custom safe_unwrap
fn safe_unwrap[T](opt: Optional[T], context: String) -> Result[T, Error]:
    if opt:
        return Ok(opt.value())
    return Error("None value: " + context)
```

### DiskANN Capacity Fix (December 2024 Discovery)
```mojo
# Problem: Crashed at ~16K vectors (exact boundary: 16,050)
# Root Cause: avg_degree mismatch - DiskANN using 16, growth using 64
# Impact: 4x memory explosion at buffer boundaries

# Found in csr_graph.mojo:403:
var avg_degree = 64  # Wrong! Caused 4x memory
# Fixed to:
var avg_degree = 32  # DiskANN paper standard

# Buffer growth pattern discovered:
# 100 → 200 → 400 → 800 → 1600 → 3200 → 6400 → 12800 → 19200
# Crash at 16K when 19200 capacity × 64 degree × 4 bytes = overflow!
```

### CSR Graph Cannot Prune - FUNDAMENTAL LIMITATION
```mojo
# CRITICAL: CSR (Compressed Sparse Row) cannot remove edges!
# Discovery: Root cause of 20K crash (December 2024)

# Why CSR fails for DiskANN:
struct CSRGraph:
    var row_ptr: List[Int]    # [0, 3, 7, 10, ...]
    var col_idx: List[Int]    # [5,8,2, 3,1,9,7, 4,6,0, ...]
    
    # To remove edge 7 from node 1:
    # - Must shift ALL subsequent edges left (640K operations!)
    # - Update ALL subsequent row_ptr indices
    # - O(E) complexity where E = total edges
    
# Solution: Need adjacency list instead
struct AdjacencyGraph:
    var nodes: List[AdjacencyNode]  # Each node has List[Int] neighbors
    # Can remove edges in O(degree) time, not O(E)!
```

### Global Singleton Pattern (Like SQLite!)
```mojo
# CRITICAL DISCOVERY: All DB() instances share same VectorStore!
# Found during debugging when tests segfaulted

# ❌ WRONG - Causes segfault:
db1 = DB()
db1.add_batch(vectors, ids=["vec_0", ...])
db2 = DB()  # Same database, not new!
db2.add_batch(vectors, ids=["vec_0", ...])  # ID collision → crash

# ✅ CORRECT - Clear between tests:
db1 = DB()
db1.add_batch(vectors, ids=["vec_0", ...])
db2 = DB()
db2.clear()  # Reset global state
db2.add_batch(vectors, ids=["vec_0", ...])
```

## Proper DiskANN Implementation Pattern (December 2024)
```mojo
# REQUIRED: True DiskANN needs these components
# Current OmenDB lacks all three!

# 1. Edge Removal Capability
struct ProperGraph:
    fn add_edge(from: Int, to: Int) -> Bool
    fn remove_edge(from: Int, to: Int) -> Bool  # CSR cannot do this!
    fn prune_edges(node: Int, keep: List[Int])  # Essential for bounded degree

# 2. RobustPrune Algorithm
fn robust_prune(node: Int, candidates: List[Int], max_degree: Int) -> List[Int]:
    # Select diverse neighbors, not just closest
    # Maintains graph connectivity
    # Prevents degree explosion
    
# 3. Actual Disk Persistence
fn mmap_graph(path: String) -> Graph:
    # Memory-mapped, not all in RAM
    # Scales to billions of vectors
    # True "Disk"ANN, not "RAM"ANN
```

## Performance Patterns

### SIMD Operations (When Working)
```mojo
fn distance_simd[width: Int](a: SIMD[DType.float32, width], 
                             b: SIMD[DType.float32, width]) -> Float32:
    var diff = a - b
    return sqrt(diff.reduce_add())
```

### Batch Processing
```mojo
# Process in chunks to amortize overhead
alias BATCH_SIZE = 1000

fn process_batch(items: List[T]) -> None:
    for i in range(0, len(items), BATCH_SIZE):
        var end = min(i + BATCH_SIZE, len(items))
        _process_chunk(items[i:end])
```

### Buffer Management
```mojo
# Double buffering for concurrent read/write
struct DoubleBuffer:
    var active: Buffer
    var inactive: Buffer
    
    fn swap(inout self) -> None:
        self.active, self.inactive = self.inactive, self.active
```

## Compiler Limitations

### No Generics (Limited)
```mojo
# Can't do full generic programming
# Must create type-specific implementations
struct IntDict:  # Separate implementation
struct FloatDict:  # Separate implementation
```

### No Async/Await
```mojo
# Use double buffering instead
# Or manual state machines for async patterns
```

### No Thread Safety Primitives
```mojo
# No Arc, Mutex, atomics
# Solution: Single-threaded engine
# Concurrency at server layer (Rust)
```

## Debugging Techniques

### Memory Debugging
```bash
# Enable sanitizers
MOJO_ASAN=1 mojo run program.mojo
MOJO_MEMCHECK=1 mojo run program.mojo

# Debug build
mojo build --debug program.mojo
```

### Common Segfault Causes
1. Buffer overflow - check bounds
2. Use after free - track lifetimes
3. Null pointer - validate before use
4. Graph reallocation - copy old data correctly
5. String handling - use proper length

## Proven Optimizations

### What Worked
1. **SparseMap**: 180x improvement over Dict[String, Int]
2. **Batch operations**: 18x improvement in throughput
3. **Product Quantization**: 16x memory compression
4. **LibC mmap**: 50x faster than Python FFI
5. **CSR Graph**: 79% edge memory reduction

### What Didn't Work
1. Memory pooling - Mojo thread safety issues
2. Full SIMD - Compiler bugs
3. Module splitting - Import system limitations
4. Generic collections - Limited generic support

## Critical Files & Lines

### Performance Hot Paths
- `native.mojo:1850-2000` - VectorStore core
- `native.mojo:500-700` - Buffer management
- `diskann.mojo:200-300` - Search inner loop
- `csr_graph.mojo:150-250` - Edge operations

### Memory Management
- `vector_buffer.mojo:25` - id_to_index Dict
- `memory_mapped.mojo:100-200` - LibC mmap
- `scalar.mojo` - Quantization implementation

## Testing Patterns

### Scale Testing Required
```python
# Always test at scale, not toy examples
def test_at_scale():
    vectors = generate_vectors(100_000, 128)  # Not 100!
    # Small tests hide issues that appear at scale
```

### Performance Measurement
```python
# Run 3x, report median
times = []
for _ in range(3):
    start = time.perf_counter()
    operation()
    times.append(time.perf_counter() - start)
    
result = statistics.median(times)
```

---
*This document captures hard-won Mojo knowledge. Update when new patterns discovered.*