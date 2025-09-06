# Technology Stack Decisions

## Feb 4, 2025: Mojo Core Engine

### Context
25K vector bottleneck blocking production. Considering complete rewrite in Rust for better FFI performance vs fixing Mojo architecture.

### Options Evaluated
1. **Rust Rewrite**
   - Pros: PyO3 zero-copy, mature ecosystem, proven in Qdrant/LanceDB
   - Cons: Lose Mojo advantages, restart from scratch, 1-2 month effort
   
2. **Fix Mojo Architecture** ✅
   - Pros: Keep existing code, Mojo has zero-copy available, GPU native
   - Cons: Less mature ecosystem, some language features in development

3. **Hybrid Approach**
   - Pros: Best of both worlds
   - Cons: Complexity, two codebases to maintain

### Decision: Mojo with Optimized FFI

#### Rationale
- **Zero-copy FFI exists**: Mojo supports `__array_interface__` for numpy arrays
- **Future GPU support**: Same Mojo code compiles to both CPU and GPU
- **Python native**: No FFI layer needed for Python integration
- **Performance potential**: SIMD built-in, no GIL, compiled performance
- **Differentiation**: Most vector DBs use Rust/C++, Mojo provides uniqueness

#### FFI Strategy
```mojo
# Python: Zero overhead (native integration)
@export
fn search_vectors(query: PythonObject) -> PythonObject:
    # Direct Python interop, no marshalling
    return results

# C/Rust: Shared library (~100ns overhead)
mojo build --emit shared-lib -o libomendb.so
```

#### Performance Targets
- Python bindings: ~0 overhead (native)
- C/Rust bindings: ~100ns per call (acceptable for batch ops)
- GPU compilation: Future Mojo feature for 100x speedup

### Consequences
- Can solve current bottleneck with proper async patterns
- Keep innovation advantage with new language
- May need workarounds for missing Mojo features
- Community/ecosystem still growing, but improving

---

## Language Binding Strategy

### Python (Primary) ✅
**Approach**: Native Mojo → Python integration
**Overhead**: ~0 (no FFI layer)
**Use case**: Data science, ML workflows, rapid prototyping

### C (Secondary)
**Approach**: Shared library export
**Overhead**: ~100ns per call
**Use case**: System integration, other language bindings

### Rust (Tertiary) 
**Approach**: Via C ABI
```rust
#[link(name = "omendb")]
extern "C" {
    fn omen_create(dimension: i32) -> *mut OmenIndex;
    fn omen_search(index: *mut OmenIndex, query: *const f32) -> *mut i32;
}
```
**Use case**: Integration with Rust applications, high-performance servers

---

## Build System: Pixi + Mojo

### Current Setup ✅
```bash
# Mojo build
pixi run mojo build omendb/native.mojo -o python/omendb/native.so

# Python environment
pixi shell  # Conda-based, includes Mojo toolchain

# Testing
pixi run benchmark-quick
```

### Why Pixi
- **Mojo integration**: Official Modular toolchain support
- **Reproducible**: Locked environment across machines
- **Python compatible**: Works with existing Python ecosystem

### Build Targets
- **Development**: `pixi run mojo build` (debug symbols)
- **Production**: `mojo build --release` (optimized)
- **GPU**: `mojo build --target=gpu` (future)

---

## Memory Management Strategy

### Approach: RAII + Manual
```mojo
struct VectorBuffer:
    var data: UnsafePointer[Float32]
    
    fn __init__(out self, size: Int):
        self.data = UnsafePointer[Float32].alloc(size)
    
    fn __del__(owned self):
        self.data.free()  # Automatic cleanup
```

### Key Principles
1. **No leaks**: Every alloc() has corresponding free()
2. **Move semantics**: Use `^` operator to transfer ownership
3. **SIMD first**: All distance calculations vectorized
4. **Avoid Mojo stdlib**: Dict/List have huge overhead

---
*All major technology decisions documented here*