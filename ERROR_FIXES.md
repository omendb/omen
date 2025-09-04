# Error → Fix Quick Reference

*Immediate lookup for common OmenDB issues*

## Critical Issues

### 25K Vector Bottleneck
**Error**: Performance degrades at ~25,000 vectors
**Fix**: 
```mojo
# In omendb/engine/omendb/native.mojo:153
# Increase memory pool from 1MB to 16MB+
self.memory_pool = MemoryPool(16 * 1024 * 1024)

# Check buffer flush at line 1850
# Make flush async, not blocking
```
**Root Cause**: Synchronous buffer flush blocks writes

### Global Singleton Segfault
**Error**: Segmentation fault with duplicate IDs
**Fix**:
```python
# Clear between tests
db = DB()
db.clear()  # MUST do this
db.add_batch(vectors, ids=unique_ids)
```
**Root Cause**: All DB() instances share same VectorStore

## Mojo-Specific Issues

| Error | Fix | Why |
|-------|-----|-----|
| `use of unknown declaration 'int'` | Use `Int` not `int` | Mojo uses capitalized types |
| `use of unknown declaration 'str'` | Use `String` not `str` | Mojo string type |
| `Dict overhead 8KB/entry` | Use `SparseMap` instead | Mojo stdlib inefficient |
| `List overhead 5KB/item` | Use fixed arrays | Mojo stdlib overhead |

## Build Issues

### Pixi Environment
```bash
# Environment not found
pixi install

# Mojo not found
pixi run mojo --version

# Build shared library
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib
```

### Import Errors
```mojo
# ❌ Absolute imports fail
from omendb.core.vector import Vector

# ✅ Relative imports work
from .core.vector import Vector
```

## Performance Issues

### FFI Overhead
```python
# ❌ SLOW: Individual operations (8.3KB/vector)
for v in vectors:
    db.add(v)

# ✅ FAST: Batch operations (1.5KB/vector) 
db.add_batch(vectors)
```

### Memory Leaks
- Check `SparseMap` capacity (line 134)
- Verify `memory_pool` size (line 153)
- Monitor `buffer_size` (line 138)

## Testing Issues

### Test Isolation
```python
# Each test MUST start clean
def test_something():
    db = DB()
    db.clear()  # Critical!
    # ... test code
```

### Dimension Mismatch
```python
# Vectors must match first vector's dimension
first_vector = np.random.rand(384).astype(np.float32)
db.add_batch([first_vector])  # Sets dimension to 384
# All future vectors must be 384-dimensional
```

## Quick Debug Commands

```bash
# Run specific test
pixi run pytest tests/test_basic.py::test_add_vector -xvs

# Check memory usage
pixi run benchmark-memory

# Profile bottleneck
pixi run profile-25k

# Validate DiskANN graph
pixi run validate-graph
```

## When All Else Fails

1. Check `internal/patterns/CONCURRENCY_PATTERNS.md` for buffer issues
2. Check `internal/patterns/STORAGE_PATTERNS.md` for memory issues
3. Check `external/agent-contexts/languages/mojo/MOJO_PATTERNS.md` for language issues
4. Run `pixi run clean && pixi run build` for clean rebuild

## File Locations

- Main engine: `omendb/engine/omendb/native.mojo`
- Buffer code: Lines 1850-2000
- Global state: Lines 78-85
- Memory pool: Line 153
- Python tests: `omendb/engine/tests/`