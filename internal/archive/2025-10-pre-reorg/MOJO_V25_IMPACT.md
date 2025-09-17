# Mojo v25.5 Impact on OmenDB System Design
## February 2025

## Executive Summary

Mojo v25.5 brings useful improvements but **critical limitations remain** that affect our architecture.

### 🚨 Critical Findings
1. **Module-level variables NOT coming** - Our global singleton pattern remains problematic
2. **Threading primitives NOT available** - No parallel graph updates possible
3. **Import system still limited** - storage_v3 has import issues

### ✅ Improvements We Can Use
1. **Parametric aliases** - Simplify our SIMD type definitions
2. **deinit convention** - Update all __del__ methods
3. **SIMD __bool__** - Better vector operations

## System Design Impacts

### 1. Global Singleton Pattern (CRITICAL)
**Current**: Using `__global_db` with warning suppression
**Problem**: Not officially supported, may break
**Impact**: Core architecture at risk

```mojo
# Current workaround (fragile)
var __global_db: UnsafePointer[GlobalDatabase]

# Alternative if it breaks
struct DatabaseHandle:
    var db: UnsafePointer[GlobalDatabase]
    
# Pass handle explicitly to all functions
fn operation(handle: DatabaseHandle): ...
```

### 2. Parallelization Strategy
**Current**: Using `parallelize` for batch operations
**Problem**: No mutexes for thread-safe graph updates
**Impact**: Limited to embarrassingly parallel operations

```mojo
# Can do: Parallel distance calculations
parallelize[compute_distance](num_vectors)

# Cannot do: Parallel graph insertions
# No way to lock/unlock graph edges safely
```

### 3. Import Architecture
**Current**: storage_v3 can't import PQ compression
**Problem**: Relative imports don't work in builds
**Solution**: Inline critical code (as we did with SimplePQ)

## Immediate Actions Required

### 1. Update Conventions (Low Risk)
```bash
# Find all old conventions
grep -r "fn __del__(owned self)" omendb/

# Should update to:
fn __del__(deinit self):
```

**Files to update**: 32 files use old convention

### 2. Simplify Types with Aliases (Medium Impact)
```mojo
# Add to core types module
alias SimdFloat32 = SIMD[DType.float32, simdwidthof[DType.float32]()]
alias ScalarFloat32 = SIMD[DType.float32, 1]

# Use throughout codebase
var sum: SimdFloat32 = 0  # Instead of SIMD[DType.float32, simd_width](0)
```

### 3. Storage Integration Decision (HIGH PRIORITY)
**Option A**: Keep storage_v3 with inline compression
- ✅ Works now
- ❌ Code duplication

**Option B**: Fix imports with build system
- ✅ Clean architecture
- ❌ Complex build configuration

**Option C**: Abandon storage_v3, optimize storage_v2
- ✅ Simpler
- ❌ Still has FFI overhead

## Performance Implications

### What's Possible
- **10,000 vec/s** with direct mmap (storage_v3)
- **Single-threaded HNSW** updates only
- **Batch operations** for read-heavy workloads

### What's NOT Possible
- **100,000+ vec/s** (needs real parallelism)
- **Concurrent graph updates** (no mutexes)
- **Multiple database instances** (global singleton issue)

## Architectural Recommendations

### 1. Accept Current Limitations
- Stay with global singleton (document risk)
- Single-threaded graph updates
- Inline critical code to avoid imports

### 2. Design for Future
- Abstract database handle for easy migration
- Prepare parallel-ready algorithms
- Keep modular design for threading addition

### 3. Focus on What Works
- **storage_v3 integration** - 10x speedup available
- **PQ compression** - 96x reduction working
- **SIMD optimizations** - Already fast

## Risk Assessment

| Component | Risk Level | Mitigation |
|-----------|------------|------------|
| Global singleton | HIGH | Prepare handle-based alternative |
| No threading | MEDIUM | Design parallel-ready, run single-threaded |
| Import issues | LOW | Inline critical code |
| Convention changes | LOW | Simple refactor |

## Decision: Proceed with Current Architecture

### Rationale
1. **10x speedup available NOW** with storage_v3
2. **Threading won't come soon** (not in roadmap)
3. **Global singleton works** (with __ prefix)
4. **Better to ship** than wait for perfect language features

### Next Sprint Actions
1. **Integrate storage_v3** despite import issues (inline code)
2. **Update to deinit convention** in critical paths
3. **Add parametric aliases** for cleaner code
4. **Document limitations** clearly

## Bottom Line

Mojo v25.5 doesn't solve our fundamental issues (global state, threading) but provides nice-to-have improvements. We should:
- **Use what works** (parametric aliases, SIMD improvements)
- **Work around limitations** (inline code, single-threading)
- **Ship with known constraints** (document thoroughly)

The language is evolving but we can't wait. Our current architecture is sufficient for v1 with 10,000 vec/s performance.