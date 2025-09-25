# Mojo 25.6 Migration Plan: Database Handle Pattern

## Executive Summary

**Goal**: Migrate from Mojo 25.4 to 25.6 by eliminating global variables and adopting a handle-passing pattern for FFI.

**Status**: ❌ **CANCELLED** - Handle pattern not viable in current Mojo
**Findings**: Integer-to-pointer conversion unsupported in Mojo 25.6 (by design)
**Alternative**: Focus on Dict capacity optimization and algorithm improvements
**Impact**: Continue with single-database architecture (26K+ vec/s performance maintained)

**CRITICAL UPDATE**: Handle pattern testing completed in Mojo 25.6. FFI limitations persist due to language design prioritizing memory safety. See `MOJO_25.6_FFI_LIMITATIONS.md` for technical details.

## Background

### Current State (Mojo 25.4)
- Uses deprecated global variable `__global_db` for FFI
- Dict limited to ~600 vectors before crashes
- Single database instance per process
- Global vars will be removed in future Mojo

### Target State (Mojo 25.6)
- Database handle passed to all functions
- Dict supports 50,000+ vectors reliably
- Multiple independent database instances
- No global variables (future-proof)

## Migration Strategy

### Phase 1: FFI Refactor (Day 1)

#### 1.1 Modify Native Module Exports

**Current Pattern**:
```mojo
var __global_db: UnsafePointer[GlobalDatabase] = UnsafePointer[GlobalDatabase]()

@export
fn add_vector(vector: PythonObject, id: PythonObject) -> PythonObject:
    var db = get_global_db()  # Uses global
    return db[].add_vector(...)
```

**New Pattern**:
```mojo
# NO GLOBAL VARIABLES

@export
fn create_database() -> Int:
    """Create database instance, return handle as integer."""
    var ptr = UnsafePointer[GlobalDatabase].alloc(1)
    ptr.init_pointee_move(GlobalDatabase())
    return int(ptr)

@export
fn add_vector(db_handle: Int, vector: PythonObject, id: PythonObject) -> PythonObject:
    """Add vector using database handle."""
    var db = UnsafePointer[GlobalDatabase](db_handle)
    return db[].add_vector(...)

@export
fn destroy_database(db_handle: Int):
    """Clean up database instance."""
    var db = UnsafePointer[GlobalDatabase](db_handle)
    if db:
        db.destroy_pointee()
        db.free()
```

#### 1.2 Functions to Refactor

**Database Lifecycle**:
- [x] `create_database()` → return Int handle
- [x] `destroy_database(handle: Int)` → cleanup
- [ ] Remove `get_global_db()` entirely
- [ ] Remove `cleanup_global_db()` entirely

**All Operations** (add db_handle: Int as first parameter):
- [ ] `initialize_database(handle, dimension, config)`
- [ ] `add_vector(handle, vector, id)`
- [ ] `add_batch(handle, vectors, ids, metadata)`
- [ ] `search_knn(handle, query, k)`
- [ ] `get_vector(handle, id)`
- [ ] `get_metadata(handle, id)`
- [ ] `delete_vector(handle, id)`
- [ ] `count_vectors(handle)`
- [ ] `get_stats(handle)`
- [ ] `save_index(handle, path)`
- [ ] `load_index(handle, path)`
- [ ] `enable_binary_quantization(handle)`
- [ ] `test_parallel_insertion(handle, n_vectors)`

### Phase 2: Python API Update (Day 1-2)

#### 2.1 Modify DB Class

**Current**:
```python
class DB:
    def __init__(self, path: str):
        self.path = path
        # Uses global database implicitly

    def add(self, vector, id=None, metadata=None):
        return native.add_vector(vector, id)
```

**New**:
```python
class DB:
    def __init__(self, path: str):
        self.path = path
        self._handle = None
        self._dimension = None

    def _ensure_initialized(self, vector_dim: int):
        """Lazy initialization with handle creation."""
        if self._handle is None:
            self._handle = native.create_database()
            native.initialize_database(self._handle, vector_dim, {})
            self._dimension = vector_dim

    def add(self, vector, id=None, metadata=None):
        self._ensure_initialized(len(vector))
        return native.add_vector(self._handle, vector, id)

    def __del__(self):
        """Ensure proper cleanup."""
        if hasattr(self, '_handle') and self._handle is not None:
            try:
                native.destroy_database(self._handle)
            except:
                pass  # Suppress errors during cleanup
```

#### 2.2 Update All Methods

Every method needs to:
1. Check initialization: `self._ensure_initialized()`
2. Pass handle: `native.function(self._handle, ...)`
3. Handle errors gracefully

### Phase 3: Mojo 25.6 Compatibility (Day 2)

#### 3.1 Language Updates

**Deprecation Fixes**:
```mojo
// Old (25.4)
fn __del__(owned self):

// New (25.6)
fn __del__(inout self):  // or use 'deinit'

// Old
@value
struct Entry:

// New
@fieldwise_init
struct Entry(Copyable, Movable):

// Old
return results  // Implicit copy

// New
return results^  // Explicit move
return results.copy()  // Explicit copy
```

**SIMD Updates**:
```mojo
// Old
simdwidthof[DType.float32]()

// New
simdwidthof[Float32]()
```

#### 3.2 Dict Improvements

No code changes needed - Dict automatically handles 50K+ entries in 25.6.

### Phase 4: Testing & Validation (Day 2-3)

#### 4.1 Unit Tests

Create handle-specific tests:
```python
def test_multiple_databases():
    db1 = DB("/tmp/db1.omen")
    db2 = DB("/tmp/db2.omen")

    # Add different data
    db1.add([1, 2, 3], "vec1")
    db2.add([4, 5, 6], "vec2")

    # Verify isolation
    assert db1.count() == 1
    assert db2.count() == 1

def test_handle_cleanup():
    db = DB("/tmp/test.omen")
    handle = db._handle
    db.add([1, 2, 3], "vec1")
    del db  # Should cleanup

    # Verify handle invalid (would crash if not cleaned)
```

#### 4.2 Performance Validation

```python
def test_performance_at_scale():
    sizes = [100, 1000, 5000, 10000, 50000]
    for n in sizes:
        db = DB(f"/tmp/test_{n}.omen")
        vectors = np.random.rand(n, 128).astype(np.float32)

        start = time.time()
        db.add_batch(vectors, ids=[f"vec_{i}" for i in range(n)])
        elapsed = time.time() - start

        print(f"{n} vectors: {n/elapsed:.0f} vec/s")
        assert db.count() == n  # Verify all inserted
```

#### 4.3 Migration Validation

- [ ] All tests pass with handle pattern
- [ ] Performance unchanged or better
- [ ] 50,000+ vectors supported
- [ ] No memory leaks
- [ ] Multiple DB instances work

## Implementation Checklist

### Day 1: Core Refactor
- [ ] Create feature branch: `feat/mojo-25.6-migration`
- [ ] Refactor native.mojo to handle pattern
- [ ] Update Python API to pass handles
- [ ] Basic testing (100-1000 vectors)

### Day 2: Mojo 25.6 Updates
- [ ] Update pixi.toml to Mojo 25.6
- [ ] Fix deprecation warnings
- [ ] Fix breaking changes
- [ ] Scale testing (10K-50K vectors)

### Day 3: Validation & Polish
- [ ] Full test suite passes
- [ ] Performance benchmarks
- [ ] Documentation updates
- [ ] Code review & cleanup
- [ ] Merge to main

## Risk Mitigation

### Risk 1: Handle Corruption
**Mitigation**: Add magic number validation:
```mojo
struct GlobalDatabase:
    var magic: UInt32  # Set to 0xDEADBEEF

fn validate_handle(db_handle: Int) -> Bool:
    var db = UnsafePointer[GlobalDatabase](db_handle)
    return db and db[].magic == 0xDEADBEEF
```

### Risk 2: Python Cleanup Failures
**Mitigation**: Context manager pattern:
```python
class DB:
    def __enter__(self):
        return self

    def __exit__(self, *args):
        if self._handle:
            native.destroy_database(self._handle)
            self._handle = None

# Usage
with DB("/tmp/test.omen") as db:
    db.add([1, 2, 3], "vec1")
```

### Risk 3: Performance Regression
**Mitigation**:
- Benchmark before/after at multiple scales
- Profile hot paths
- Rollback plan if >5% regression

## Rollback Plan

If issues arise:
1. **Immediate**: Revert to main branch (25.4)
2. **Short-term**: Apply only Dict fixes without handle pattern
3. **Long-term**: Consider Rust if Mojo unstable

## Success Criteria

- [ ] 50,000+ vectors supported (vs 600 current)
- [ ] No performance regression
- [ ] All tests passing
- [ ] Multiple DB instances working
- [ ] Clean build with Mojo 25.6

## Future Considerations

### When Module-Level Vars Arrive (2026+)

**Option 1: Keep Handle Pattern** (Recommended)
- Already working perfectly
- Better architecture (testable, multiple instances)
- Standard practice in systems programming

**Option 2: Add Convenience Global**
```mojo
var default_db: Optional[Int] = None  # Module-level var

fn get_default_db() -> Int:
    if not default_db:
        default_db = create_database()
    return default_db.value()
```

But handle pattern remains primary API.

## Conclusion

This migration solves our immediate capacity problem (600 → 50K+ vectors) while creating better architecture that will serve us regardless of Mojo's future direction. The handle pattern is standard practice in systems programming and provides benefits beyond just Mojo compatibility.

**Estimated effort**: 2-3 days
**Risk level**: Low
**Reward**: 83x capacity increase + future-proof architecture