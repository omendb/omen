# OmenDB Memory Management Architecture

## Critical Fix: Static Pointer Pattern (v0.1.0)

**Problem**: Mojo v25.4.0 global variables cause double-free crashes with complex objects
**Solution**: Static pointer pattern with lazy initialization

## Implementation

```mojo
// Before (broken):
var _global_db = VectorStore()  // Complex destructor causes double-free

// After (fixed):
var _global_db_ptr: UnsafePointer[VectorStore] = UnsafePointer[VectorStore]()
var _module_initialized: Bool = False

@always_inline
fn get_global_db() -> UnsafePointer[VectorStore]:
    if not _module_initialized:
        _global_db_ptr = UnsafePointer[VectorStore].alloc(1)
        _global_db_ptr.init_pointee_move(VectorStore())
        _module_initialized = True
    return _global_db_ptr
```

## Usage Pattern

- **Before**: `_global_db.add()` 
- **After**: `get_global_db()[].add()`
- **Performance**: Zero overhead after initialization
- **Memory**: Never freed (process lifetime)

## Files Updated

- `omendb/native.mojo` - Main database storage
- `omendb/core/metrics.mojo` - Metrics storage  
- `omendb/core/memory_pool.mojo` - Pool storage
- `omendb/core/blas_integration.mojo` - BLAS storage

## Server Edition Compatibility

✅ **FFI Ready**: C++/Rust can call `get_global_db()`
✅ **Zero Overhead**: Same performance as direct globals
✅ **Process Isolation**: Works with server process pools
✅ **Future Proof**: Easy migration when Mojo gets module state

## Migration Path

When Mojo adds proper module state (~2026):
```mojo
// Easy conversion:
var _module_db: VectorStore  // Proper module global
fn get_global_db() -> UnsafePointer[VectorStore]:
    return UnsafePointer.address_of(_module_db)
```

**Status**: Production ready, eliminates memory crashes