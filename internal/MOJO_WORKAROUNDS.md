# OmenDB-Specific Mojo Workarounds

## Critical Context for AI Agents  
**This file tracks OmenDB-specific Mojo workarounds. For general Mojo patterns, see @external/agent-contexts/languages/mojo/MOJO_PATTERNS.md**

## Project-Specific Solutions

## Language Limitations & Solutions

### 1. No async/await (Coming Q2 2025)
```mojo
# ❌ Not available yet
async fn fetch_data():
    await some_operation()

# ✅ Workaround: Thread pools
from python import ThreadPoolExecutor

fn parallel_search(queries: List[Query]) -> List[Result]:
    var executor = ThreadPoolExecutor(max_workers=8)
    var futures = List[Future]()
    
    for query in queries:
        futures.append(executor.submit(search_single, query))
    
    var results = List[Result]()
    for future in futures:
        results.append(future.result())
    
    return results
```

### 2. Limited stdlib collections
```mojo
# ❌ NEVER use these (massive overhead)
Dict[String, Int]   # 8KB per entry!
List[String]        # 5KB per item!

# ✅ Implement custom collections
struct CompactDict[K: AnyType, V: AnyType]:
    var keys: DynamicVector[K]
    var values: DynamicVector[V]
    var size: Int
    
    fn get(self, key: K) -> Optional[V]:
        for i in range(self.size):
            if self.keys[i] == key:
                return self.values[i]
        return None
```

### 3. No HTTP server
```python
# ✅ Workaround: Python FastAPI wrapper
# server.py
from fastapi import FastAPI
import omendb  # Our Mojo module

app = FastAPI()

@app.post("/search")
def search(query: dict):
    return omendb.search(query["vector"], query["k"])
```

### 4. String operations limited
```mojo
# ❌ No regex, limited string methods
# ✅ Workaround: Use Python for text processing
fn process_text(text: String) -> String:
    var py = Python.import_module("re")
    var processed = py.sub(r"\s+", " ", text)
    return String(processed)
```

### 5. No native JSON parsing
```mojo
# ✅ Workaround: Use Python json module
fn parse_config(json_str: String) -> PythonObject:
    var json = Python.import_module("json")
    return json.loads(json_str)
```

### 6. Type conversion gotchas
```mojo
# ❌ Python-style (compilation error)
var x = int(value)
var s = str(value)

# ✅ Mojo-style
var x = Int(value)
var s = String(value)
var f = Float32(value)  # Explicit precision
```

### 7. No generators/yield
```mojo
# ❌ Not available
fn generate_batches():
    for batch in data:
        yield batch

# ✅ Workaround: Return list or use callback
fn process_batches(data: List, callback: fn(Batch)):
    for i in range(0, len(data), batch_size):
        var batch = data[i:i+batch_size]
        callback(batch)
```

### 8. Limited error handling
```mojo
# ❌ No custom exceptions yet
# ✅ Workaround: Return Result type
struct Result[T: AnyType]:
    var value: Optional[T]
    var error: Optional[String]
    
    fn is_ok(self) -> Bool:
        return self.error is None
```

## FFI Patterns

### Python Integration (Zero-overhead)
```mojo
@export
fn search_vectors(
    query_ptr: Int,  # Python passes pointer as int
    dim: Int,
    k: Int
) -> PythonObject:
    # Convert pointer to Mojo type
    var query = DTypePointer[DType.float32](query_ptr)
    
    # Do computation
    var results = hnsw_search(query, dim, k)
    
    # Return Python list
    return results.to_python()
```

### C/Rust Integration
```mojo
# Build shared library
# mojo build --emit-library -o libomendb.so engine.mojo

@export("C")
fn omen_search(
    query: UnsafePointer[Float32],
    dim: Int32,
    k: Int32,
    results: UnsafePointer[Int32]
) -> Int32:
    # Implementation
    return 0  # Success
```

## Memory Management Patterns

### Manual memory with RAII
```mojo
struct Buffer:
    var data: UnsafePointer[Float32]
    var size: Int
    
    fn __init__(out self, size: Int):
        self.size = size
        self.data = UnsafePointer[Float32].alloc(size)
    
    fn __del__(owned self):
        self.data.free()  # Automatic cleanup
```

### Avoid memory leaks
```mojo
# ❌ Leak - no free
var ptr = UnsafePointer[Float32].alloc(1000)
# ptr never freed!

# ✅ RAII pattern
var buffer = Buffer(1000)
# Automatically freed when out of scope
```

## Performance Workarounds

### SIMD when loops are slow
```mojo
# ❌ Slow scalar loop
for i in range(size):
    result[i] = a[i] + b[i]

# ✅ SIMD vectorization
alias simd_width = simdwidthof[DType.float32]()
for i in range(0, size, simd_width):
    var va = a.load[width=simd_width](i)
    var vb = b.load[width=simd_width](i)
    (va + vb).store(result, i)
```

### Batch operations for FFI
```mojo
# ❌ Many FFI calls (slow)
for vector in vectors:
    index.add(vector)  # FFI call each time

# ✅ Single batch call
index.add_batch(vectors)  # One FFI call
```

## Current Blockers (No Workaround)

1. **No package manager** - Manual dependency management
2. **No debugger** - Print debugging only
3. **No profiler** - Manual timing with time.now()
4. **Limited IDE support** - VS Code extension basic

## When to Use Python Instead

Use Python for:
- HTTP server (FastAPI)
- Complex string/regex operations
- JSON/YAML parsing
- File I/O beyond basics
- Database connections
- Third-party integrations

Use Mojo for:
- Core algorithms (HNSW+)
- SIMD operations
- Memory-critical paths
- GPU kernels (future)
- Performance-critical loops

## Testing Patterns

```python
# test_omendb.py
import omendb  # Our Mojo module
import numpy as np

def test_search():
    # Python handles test framework
    index = omendb.Index(dimension=128)
    
    # NumPy arrays work directly
    vectors = np.random.rand(1000, 128).astype(np.float32)
    index.add_batch(vectors)
    
    query = np.random.rand(128).astype(np.float32)
    results = index.search(query, k=10)
    
    assert len(results) == 10
```

## Build Commands

```bash
# Development build
mojo build omendb/engine.mojo -o omendb.so

# Release build (optimized)
mojo build --release omendb/engine.mojo -o omendb.so

# With debug symbols
mojo build -g omendb/engine.mojo -o omendb.so

# Create Python package
mojo package omendb -o dist/
```

## Common Errors & Solutions

| Error | Solution |
|-------|----------|
| `use of unknown declaration 'int'` | Use `Int()` not `int()` |
| `cannot convert Float64 to Float32` | Use explicit `Float32(value)` |
| `no attribute 'append'` | Check if using Mojo List vs Python list |
| `cannot deduce parameter` | Add explicit type annotations |
| `unable to locate module` | Check import paths, use relative imports |

## Version Compatibility

**Current Mojo version**: 24.5 (as of Feb 2025)
**Minimum required**: 24.4
**GPU support**: Coming in 25.1 (estimated)

## Links & Resources

- [Mojo Manual](https://docs.modular.com/mojo/manual/)
- [Mojo stdlib](https://docs.modular.com/mojo/stdlib/)
- [Community Discord](https://discord.gg/modular)
- Our patterns: `internal/KNOWLEDGE.md#mojo-patterns`

---
*Always check this file when implementing Mojo code. Update when finding new workarounds.*