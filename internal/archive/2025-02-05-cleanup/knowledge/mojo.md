# Mojo Language Patterns & Best Practices

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

## Memory Management Patterns

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

## SIMD Optimization Patterns

### Hardware-Adaptive SIMD
```mojo
# Always use hardware-adaptive SIMD width
alias simd_width = simdwidthof[DType.float32]()

fn dot_product(a: UnsafePointer[Float32], b: UnsafePointer[Float32], dim: Int) -> Float32:
    var sum = SIMD[DType.float32, simd_width](0)
    
    # Process SIMD-width chunks
    for i in range(0, dim, simd_width):
        var va = a.load[width=simd_width](i)
        var vb = b.load[width=simd_width](i)
        sum = va.fma(vb, sum)  # Fused multiply-add
    
    return sum.reduce_add()
```

### Distance Calculation Template
```mojo
# Template for all distance functions
fn cosine_distance[simd_width: Int](
    a: UnsafePointer[Float32], 
    b: UnsafePointer[Float32], 
    dim: Int
) -> Float32:
    var dot = SIMD[DType.float32, simd_width](0)
    var norm_a = SIMD[DType.float32, simd_width](0)
    var norm_b = SIMD[DType.float32, simd_width](0)
    
    for i in range(0, dim, simd_width):
        var va = a.load[width=simd_width](i)
        var vb = b.load[width=simd_width](i)
        dot = va.fma(vb, dot)
        norm_a = va.fma(va, norm_a)
        norm_b = vb.fma(vb, norm_b)
    
    return dot.reduce_add() / (norm_a.reduce_add().sqrt() * norm_b.reduce_add().sqrt())
```

## FFI Patterns

### Zero-Copy Python Integration
```mojo
# ❌ WRONG - Copying from Python
fn add_vector(py_array: PythonObject):
    var vector = List[Float32]()
    for i in range(len(py_array)):
        vector.append(Float32(py_array[i]))  # COPY each element!

# ✅ CORRECT - Direct pointer access
fn add_vector_zero_copy(py_array: PythonObject) raises:
    var interface = py_array.__array_interface__
    var data_ptr = int(interface["data"][0])
    var ptr = DTypePointer[DType.float32](data_ptr)
    # Use pointer directly, no copy
    self.buffer.add_view(ptr, size)
```

### C API Export Pattern
```mojo
# Export C-compatible API
@export("C")
fn omen_create_index(dimension: Int32) -> UnsafePointer[HNSWIndex]:
    var index = HNSWIndex(dimension=Int(dimension))
    return UnsafePointer.address_of(index)

@export("C") 
fn omen_search(
    index_ptr: UnsafePointer[HNSWIndex],
    query_ptr: UnsafePointer[Float32],
    k: Int32,
    results: UnsafePointer[Int32]
):
    var index = index_ptr.load()
    var results_list = index.search(query_ptr, Int(k))
    # Copy results to C array
    for i in range(len(results_list)):
        results.offset(i).store(results_list[i])
```

## Performance Patterns

### Memory Pool Pattern
```mojo
struct MemoryPool:
    var blocks: List[UnsafePointer[UInt8]]
    var block_size: Int
    var current_offset: Int
    
    fn allocate[T: AnyType](mut self, count: Int) -> UnsafePointer[T]:
        var bytes_needed = count * sizeof[T]()
        if self.current_offset + bytes_needed > self.block_size:
            self._allocate_new_block()
        
        var ptr = self.blocks[-1].offset(self.current_offset).bitcast[T]()
        self.current_offset += bytes_needed
        return ptr
```

## Common Compilation Errors → Solutions

| Error | Solution |
|-------|----------|
| `use of unknown declaration 'int'` | Use `Int()` |
| `use of unknown declaration 'str'` | Use `String()` |
| `cannot convert Float64 to Float32` | Use `Float32(value)` |
| `could not deduce parameter` | Add explicit type annotations |
| `no attribute 'get'` | Use index access `[0]` for tuples |
| `unable to locate module` | Use relative imports within packages |

## Stdlib Gotchas

### Avoid These (Massive Overhead)
```mojo
# ❌ NEVER use - huge memory overhead
Dict[String, Int]   # 8KB per entry!
List[String]        # 5KB per item!
Optional[String]    # Also problematic
```

### Use These Instead
```mojo
# ✅ Custom implementations
SparseMap          # 180x better than Dict
Fixed arrays       # Predictable memory
UnsafePointer      # Manual but efficient
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

---
*Critical Mojo patterns for high-performance vector database implementation*