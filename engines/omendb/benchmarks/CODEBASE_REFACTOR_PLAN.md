# Comprehensive Codebase Refactoring Plan

## Current Issues

### 1. Core Directory Bloat (36 files!)
**Problem**: Too many files, unclear organization
```
core/
├── distance.mojo           ✓ Keep (used)
├── distance_functions.mojo  ? Duplicate?
├── simd_operations.mojo    ? Unused
├── simd_utils.mojo         ? Duplicate?
├── blas_integration.mojo   ? Unused
├── blas_modular_optimized.mojo ? Duplicate?
├── brute_force.mojo        ? Should be in algorithms/
├── gpu_context.mojo        ? Unused (no GPU support)
├── visited_list_pool.mojo  ? Unused
├── norm_cache.mojo         ? Unused
├── heap_topk.mojo          ? Should be in algorithms/
└── ... 25 more files
```

### 2. Empty Directories
```
optimization/
└── __init__.mojo  # Empty directory!
```

### 3. Duplicate/Dead Code
- Multiple distance implementations
- Multiple BLAS implementations  
- Multiple SIMD attempts
- Unused optimizations

### 4. Poor Organization
- native.mojo: 3000+ lines
- Algorithms mixed with core
- Storage mixed with operations
- No clear module boundaries

## Immediate Actions (While Fixing Critical Issues)

### Clean Up Dead Code
```bash
# Files to DELETE (unused/redundant):
omendb/optimization/            # Empty directory
omendb/core/simd_operations.mojo    # Broken SIMD
omendb/core/simd_utils.mojo         # Duplicate
omendb/core/gpu_context.mojo        # No GPU support
omendb/core/visited_list_pool.mojo  # Unused
omendb/core/norm_cache.mojo         # Unused  
omendb/core/blas_modular_optimized.mojo  # Duplicate
omendb/core/mixed_precision.mojo    # Not implemented
omendb/core/optimized_memory_ops.mojo  # Unused
omendb/core/optimized_metadata_ops.mojo # Unused
omendb/core/performance_profiler.mojo  # Unused
omendb/core/wal_storage.mojo        # Deprecated
```

### Consolidate Duplicates
```bash
# Merge these:
distance.mojo + distance_functions.mojo → distance.mojo
blas_integration.mojo + matrix_ops.mojo → matrix_ops.mojo
scalar_quantization.mojo + quantization.mojo → quantization.mojo
```

## Proposed New Structure

```
omendb/
├── __init__.mojo           # Package init
├── native.mojo             # ONLY FFI exports (200 lines)
│
├── database/               # Core database logic
│   ├── __init__.mojo
│   ├── db.mojo            # Main Database struct (400 lines)
│   ├── operations.mojo    # Add/Search/Delete operations
│   └── state.mojo         # Global state management
│
├── algorithms/             # Search algorithms  
│   ├── __init__.mojo
│   ├── diskann.mojo       # Main DiskANN
│   ├── diskann_csr.mojo   # CSR variant
│   └── bruteforce.mojo    # Fallback algorithm
│
├── storage/                # Persistence layer
│   ├── __init__.mojo
│   ├── coordinator.mojo   # Storage coordination
│   ├── buffer.mojo        # Vector buffer
│   ├── mmap.mojo          # Memory-mapped storage
│   └── checkpoint.mojo    # Checkpoint logic
│
├── core/                   # Essential components only
│   ├── __init__.mojo
│   ├── vector.mojo        # Vector type
│   ├── distance.mojo      # Distance metrics
│   ├── metadata.mojo      # Metadata handling
│   ├── quantization.mojo  # All quantization
│   └── memory.mojo        # Memory tracking
│
└── utils/                  # Utilities
    ├── __init__.mojo
    ├── metrics.mojo       # Performance metrics
    └── errors.mojo        # Error handling
```

## Refactoring Steps

### Phase 1: Clean (While Fixing Issues)
1. Delete unused files (12 files)
2. Merge duplicate implementations (3 merges)
3. Move misplaced files (brute_force → algorithms/)

### Phase 2: Modularize native.mojo
1. Extract Database struct → database/db.mojo
2. Move operations → database/operations.mojo
3. Keep only FFI exports in native.mojo

### Phase 3: Organize Storage
1. Consolidate storage implementations
2. Fix memory-mapped recovery
3. Create clear storage interface

### Phase 4: Simplify Core
1. Keep only essential types
2. Remove experimental code
3. Consolidate quantization

## Files to Keep vs Delete

### KEEP (Actually Used)
```
✓ native.mojo (refactor to FFI only)
✓ algorithms/diskann.mojo
✓ algorithms/diskann_csr.mojo  
✓ core/vector.mojo
✓ core/distance.mojo
✓ core/metadata.mojo
✓ core/vector_buffer.mojo
✓ core/memory_tracker.mojo
✓ core/quantization.mojo
✓ core/csr_graph.mojo
✓ storage/memory_mapped_storage.mojo
```

### DELETE (Unused/Redundant)
```
✗ optimization/* (empty directory)
✗ core/simd_*.mojo (broken)
✗ core/gpu_context.mojo (no GPU)
✗ core/visited_list_pool.mojo
✗ core/norm_cache.mojo
✗ core/blas_modular_optimized.mojo
✗ core/mixed_precision.mojo
✗ core/optimized_*.mojo
✗ core/performance_profiler.mojo
✗ core/wal_storage.mojo
✗ ffi_exports.mojo (duplicate)
✗ ffi_optimization.mojo (unused)
✗ python_integration.mojo (duplicate)
✗ python_interop.mojo (duplicate)
```

## Benefits After Refactoring

### Code Quality
- 50% less code (remove dead files)
- Clear module boundaries
- Single responsibility per file
- Easy to navigate

### Performance
- Faster compilation (fewer files)
- Better optimization opportunities
- Clearer hot paths

### Maintainability
- Find bugs faster
- Add features cleanly
- Test individual modules
- Onboard new developers

## Action Plan

### Week 1: Fix Critical Issues + Clean
- Fix memory-mapped recovery (TODO stubs)
- Fix vector normalization
- Apply quantization
- **Delete 12+ unused files**
- **Merge 3 duplicate implementations**

### Week 2: Modularize
- Split native.mojo into modules
- Create database/ structure
- Organize storage layer
- Update all imports

### Week 3: Polish
- Add comprehensive tests
- Update documentation
- Performance optimization
- Final cleanup

## Example: native.mojo After Refactoring

### Before: 3000+ lines of everything
```mojo
# native.mojo - EVERYTHING
struct VectorStore { ... }  # 800 lines
fn add_vector() { ... }     # 200 lines  
fn search_vector() { ... }  # 300 lines
fn checkpoint() { ... }     # 150 lines
... # 1500+ more lines
```

### After: 200 lines of FFI only
```mojo
# native.mojo - ONLY FFI exports
from database import Database

var _db: Optional[Database] = None

@export
fn add_vector(id: PythonObject, vector: PythonObject) -> PythonObject:
    return get_db().add(String(id), to_list(vector))

@export
fn search_vector(query: PythonObject, k: PythonObject) -> PythonObject:
    return get_db().search(to_list(query), Int(k))
```

## Metrics for Success

### Before Refactoring
- 36 files in core/
- 3000+ lines in native.mojo
- 12+ unused files
- 5+ duplicate implementations

### After Refactoring  
- 5 files in core/
- 200 lines in native.mojo
- 0 unused files
- 0 duplicate implementations

## Conclusion

The codebase has accumulated significant technical debt:
- **50% of files are unused**
- **native.mojo is 15x too large**
- **No clear module boundaries**

Refactoring will reduce codebase by ~50% while improving clarity and maintainability.