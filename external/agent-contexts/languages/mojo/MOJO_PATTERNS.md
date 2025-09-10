# Mojo Patterns & Best Practices

*Consolidated patterns for high-performance Mojo development*
*Updated: Feb 2025 - Mojo v25.5 with v25.6 preview*

## Critical Type Conversions

### ❌ WRONG vs ✅ CORRECT
```mojo
❌ int(value)     → ERROR: use of unknown declaration 'int'
✅ Int(value)     → Mojo integer type

❌ str(value)     → ERROR: use of unknown declaration 'str'
✅ String(value)  → Mojo string type

❌ float(value)   → ERROR: use of unknown declaration 'float'
✅ Float32(value) → Explicit precision required

❌ bool(value)    → ERROR: use of unknown declaration 'bool'
✅ Bool(value)    → Mojo boolean type
```

## Function Type Selection

```
IF Python_interop_needed → def function_name():
IF performance_critical  → fn function_name():
IF memory_management     → fn with owned/mut parameters
```

## Memory Management Patterns

### Single Storage Principle
```mojo
# ❌ WRONG - Double storage wastes memory
struct BadBuffer:
    var original: UnsafePointer[Float32]
    var quantized: UnsafePointer[UInt8]  # BOTH stored = waste

# ✅ CORRECT - Single storage with flag
struct GoodBuffer:
    var data: UnsafePointer[Float32]     # Only one
    var is_quantized: Bool               # Flag for type
```

### Move Semantics Over Copy
```mojo
# ❌ WRONG - Copying data
fn process(self, buffer: Buffer):
    var data = buffer.get_data()  # COPY

# ✅ CORRECT - Moving data  
fn process(mut self, owned buffer: Buffer):
    self.consume(buffer^)  # MOVE ownership
```

### Lazy Allocation
```mojo
# ❌ WRONG - Pre-allocate everything
self.nodes = UnsafePointer[Node].alloc(1_000_000)  # Too much upfront

# ✅ CORRECT - Grow as needed
self.capacity = 100
self.nodes = UnsafePointer[Node].alloc(self.capacity)
# Double capacity when full
```

## Import Patterns

### Module Imports
```mojo
❌ from python import numpy       → Slow FFI overhead
✅ from tensor import Tensor      → Native Mojo types

❌ from core.vector import Vector → Absolute imports fail
✅ from .core.vector import Vector → Relative imports work
```

### Import Rules
- Every directory needs `__init__.mojo` with re-exports
- Use relative imports (`.module` or `..parent.module`)
- No module-level variables allowed (still not supported as of v25.5)
- Threading primitives not available yet (no mutexes/locks)

## New in Mojo v25.5 (Current Release)

### Parametric Aliases - Simplify Type Definitions
```mojo
# ❌ OLD - Repetitive type patterns
var a: SIMD[DType.float32, 1]
var b: SIMD[DType.float32, 1]

# ✅ NEW - Use parametric aliases
alias Scalar[DT: DType] = SIMD[1, DT]
alias Float32 = Scalar[DType.float32]
var a: Float32
var b: Float32
```

### Default Trait Methods - Reduce Boilerplate
```mojo
# Traits can now have default implementations
trait Comparable:
    fn __eq__(self, other: Self) -> Bool: ...
    fn __ne__(self, other: Self) -> Bool:
        return not self.__eq__(other)  # Default implementation
```

### SIMD Improvements
```mojo
# __bool__ now works with SIMD vectors
if my_simd_vector:
    # Evaluates to True if any element is non-zero
    process()
```

## Coming in v25.6 (Preview)

### Trait Unions - Better Type Constraints
```mojo
# FUTURE - Not available yet
alias CopyableAndMovable = Copyable & Movable
struct MyStruct[T: CopyableAndMovable]: ...
```

### requires Keyword - Compile-Time Validation
```mojo
# FUTURE - Not available yet
struct HNSW[dim: Int, capacity: Int]
  requires dim > 0, "dimension must be positive"
  requires capacity.is_power_of_two(), "capacity must be power of 2"
```

## Performance Critical Patterns

### Collection Overhead (CRITICAL)
```mojo
# ❌ AVOID in hot paths - massive overhead
Dict[String, Int]  # 8KB per entry (100x expected)
List[String]       # 5KB per item (100x expected)

# ✅ USE INSTEAD
Custom SparseMap   # 180x improvement over Dict
Fixed arrays       # Instead of List[String]
Batch operations   # Amortize FFI overhead
```

### FFI Optimization
```mojo
# ❌ BAD - Individual FFI calls (8KB overhead each)
for i in range(count):
    add_vector(keys[i], vectors[i])

# ✅ GOOD - Batch FFI calls
add_batch(keys, vectors)  # Amortize overhead

# ✅ BETTER - LibC FFI (50x faster)
sys.ffi.external_call["mmap", UnsafePointer[Int8]](...)
```

### SIMD Operations
```mojo
alias simd_width = simdwidthof[DType.float32]()

fn dot_product(a: UnsafePointer[Float32], b: UnsafePointer[Float32], dim: Int) -> Float32:
    var sum = SIMD[DType.float32, simd_width](0)
    for i in range(0, dim, simd_width):
        var va = a.load[width=simd_width](i)
        var vb = b.load[width=simd_width](i)
        sum = va.fma(vb, sum)
    return sum.reduce_add()
```

## Known Workarounds

### Dict Iteration Bug
```mojo
# ❌ CRASHES with bus error
for item in dict.items():
    process(item)

# ✅ WORKAROUND - Manual key iteration
for key in known_keys:
    if key in dict:
        process(dict[key])
```

### Optional Handling
```mojo
# ❌ CRASHES on None
var value = optional.value()

# ✅ SAFE pattern
if opt:
    var value = opt.value()
else:
    handle_none()
```

### Memory Allocation Limits
```mojo
# ❌ CRASHES at 26-27K with nested lists
var adjacency: List[List[Int]]  # Hard limit

# ✅ SOLUTION - Flat arrays with offsets
var edges: UnsafePointer[UInt32]  # CSR format
var offsets: List[Int]
```

### Convention Changes (v25.5+)
```mojo
# ❌ OLD - Using owned in __del__
fn __del__(owned self):
    self.data.free()

# ✅ NEW - Using deinit convention
fn __del__(deinit self):
    self.data.free()
```

## Debugging Techniques

### Enable Sanitizers
```bash
MOJO_ASAN=1 mojo run program.mojo    # Address sanitizer
MOJO_MEMCHECK=1 mojo run program.mojo # Memory checker
mojo build --debug program.mojo       # Debug symbols
```

### Common Segfault Causes
1. Buffer overflow → Check bounds
2. Use after free → Track lifetimes  
3. Null pointer → Validate before use
4. String handling → Use proper length

## AI Assistant Integration

### Project Setup
```bash
# Create AI assistant context
mkdir -p .cursor
curl -o .cursor/rules https://docs.modular.com/max/cursorules
echo "@docs.modular.com/llms-mojo.txt" >> .cursor/context
```

### Error → Solution Mappings
| Error | Root Cause | Fix |
|-------|------------|-----|
| `use of unknown declaration 'int'` | Python syntax | Use `Int()` |
| `use of unknown declaration 'str'` | Python syntax | Use `String()` |
| `cannot implicitly convert` | Type mismatch | Explicit conversion |
| `use of uninitialized value` | No initialization | Assign at declaration |

## Proven Optimizations

### What Worked
- **SparseMap**: 180x improvement over Dict[String, Int]
- **Batch operations**: 18x improvement in throughput
- **Product Quantization**: 16x memory compression
- **LibC mmap**: 50x faster than Python FFI
- **CSR Graph**: 79% edge memory reduction

### What Failed
- Memory pooling → Thread safety issues
- Full SIMD → Compiler bugs
- Module splitting → Import limitations
- Generic collections → Limited support

## Critical Limitations (As of v25.5)

### Still Not Available
1. **Module-level variables** - Global singleton pattern unreliable
2. **Threading primitives** - No mutexes, locks, or thread-safe collections
3. **Relative imports in builds** - Complex build configurations needed
4. **Full generic support** - Limited generic collection types

### Workarounds
```mojo
# Global singleton workaround
var __global_db: UnsafePointer[Database]  # __ prefix suppresses warning

# Threading workaround
# Use single-threaded operations only
# Or use process-based parallelism
```

## Quick Command Sequences

### Convert Python-style code
```bash
rg "int\(" --type mojo -l | xargs sed -i 's/int(/Int(/g'
rg "str\(" --type mojo -l | xargs sed -i 's/str(/String(/g'
rg "float\(" --type mojo -l | xargs sed -i 's/float(/Float32(/g'
```

### Find performance issues
```bash
rg "def " --type mojo           # Python-style functions
rg "import.*python" --type mojo # FFI usage
rg "Dict\[|List\[" --type mojo  # Slow collections
```

---
*Consolidated from OmenDB production experience - Update when new patterns discovered*