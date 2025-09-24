# OmenDB Development Context

## 🚨 Architecture: Pure Mojo + HNSW/IVF-Flat

**Decision Date**: September 20, 2025
**Status**: Ready to implement

## Key Documents (Load in Order)

1. **`internal/ARCHITECTURE.md`** - Complete architecture (START HERE)
2. **`internal/STATUS.md`** - Current performance metrics
3. **`internal/TODO.md`** - 4-week implementation plan
4. **`internal/DECISIONS.md`** - Decision history
5. **`internal/RESEARCH.md`** - Research findings

## Project Structure & Context

### Architecture Summary (September 20, 2025)
```yaml
Decision: Pure Mojo (no FFI overhead)
CPU Index: HNSW (95% recall, 2-3ms search)
GPU Index: IVF-Flat (90-95% recall, <1ms search)
Server: Python FastAPI wrapper (when needed)
Target: 50K+ vec/s bulk load, 2-3ms search
Timeline: 4 weeks to production
```

### Active Blockers
1. **Bulk insertion broken** - Creates disconnected graphs (0% recall)
2. **No real parallelism** - Everything runs sequentially despite "parallel" names
3. **Segmented breaks at scale** - 0% recall at 3000+ vectors

### Repository Structure
```
omendb/core/
├── CLAUDE.md                  # This file - AI agent context
├── internal/                  # AI working documentation
│   ├── TODO.md               # Active tasks (edit in place)
│   ├── STATUS.md             # Current state (edit in place)
│   ├── RESEARCH.md           # Algorithm research (append findings)
│   ├── DECISIONS.md          # Architecture log (append only)
│   └── archive/              # Historical logs
├── omendb/engine/            # Main Mojo vector database
│   ├── omendb/algorithms/    # HNSW, segmented_hnsw
│   ├── python/omendb/        # Python bindings
│   └── benchmarks/           # Performance tests
├── zendb/                    # Rust hybrid database (secondary)
└── external/agent-contexts/  # Decision trees and patterns
```

## Development Setup

### Build Commands (Mojo/Pixi)
```bash
# Build engine
cd omendb/engine
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib -I omendb

# Run benchmarks
pixi run python benchmarks/final_validation.py  # Main metric
pixi run python test_current_perf.py           # Performance at scale

# Quick tests
pixi run python test_binary_quantization_quick.py
```

### Performance Validation
```bash
# Expected results (as of Sept 19, 2025):
# 100 vectors (flat): ~2K vec/s, 100% recall
# 1000 vectors (HNSW): ~5K vec/s, 95% recall
# 2000+ vectors (segmented): ~3.3K vec/s, 100% recall
# 3000+ vectors: Performance degrades, may drop to 0% recall
```

## Code Conventions

### HNSW Invariants (NEVER VIOLATE)
```mojo
// ✅ CORRECT - Always navigate from entry point down
var curr = entry_point
for layer in range(entry_level, target_layer, -1):
    curr = search_layer(query, curr, 1, layer)

// ❌ WRONG - Never skip hierarchical navigation
var neighbors = find_nearest_in_layer(query, target_layer)  // Breaks recall!
```

### Mojo Patterns
```mojo
// Memory management
UnsafePointer[T].alloc(size)    # Manual allocation
ptr.free()                      # Manual cleanup
memcpy(dest, src, bytes)        # Efficient copying

// SIMD (working)
_fast_distance_between_nodes()   # Use this, not _distance_between_nodes()

// Parallelism (future)
parallelize[func](0, count)      # True parallelism when implemented
```

### Performance Measurement
```python
# Always measure with proper timing
import time
start = time.time()
result = operation()
elapsed = time.time() - start
rate = count / elapsed if elapsed > 0 else 0
print(f"Rate: {rate:.0f} operations/sec")
```

## Testing

### Required Tests Before Commits
```bash
# Build must pass
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib -I omendb

# Core functionality test
pixi run python benchmarks/final_validation.py

# Performance regression check
pixi run python test_current_perf.py
```

### Success Criteria
- **Build**: Must compile without errors (warnings OK)
- **Recall**: Must maintain >95% on small datasets (<1K vectors)
- **Performance**: Must not regress below current baseline
- **Memory**: Must not crash or leak memory

## Git Workflow

### Commit Format
```
type: concise description

Types: feat, fix, perf, docs, refactor, test
Example: "perf: reduce ef_construction to 50 for 2x speedup"
```

### Before Each Commit
1. Check `git status` and `git diff`
2. Run build and basic tests
3. Update `internal/STATUS.md` if performance changes
4. Update `internal/TODO.md` if tasks completed

## HNSW-Specific Patterns

### Safe Optimization Approach
```
1. Start with working code (867 vec/s baseline)
2. Profile to find bottlenecks
3. Optimize preserving ALL invariants
4. Validate recall remains >95%
5. Commit only if both speed AND quality improve
```

### Parameter Tuning (Proven Settings)
```mojo
// Competitive settings (based on Qdrant benchmarks)
alias M = 16                    # Connections per layer
alias max_M0 = 32              # Layer 0 connections
alias ef_construction = 50      # Build candidates (not 200!)
alias ef_search = 150          # Search candidates
```

### Common Anti-Patterns
```mojo
// ❌ NEVER do these (guaranteed to break recall):
// 1. Skip hierarchical navigation
// 2. Defer bidirectional connections
// 3. Batch without preserving graph validity
// 4. Use approximate distances during construction
// 5. Parallel updates to shared graph state
```

## Project Context

### Mission
Build the fastest vector database in Mojo with state-of-the-art performance (20K+ vec/s) while maintaining high quality (90%+ recall).

### Current Challenge
We have excellent recall (100%) but need 6x performance improvement to be competitive. The path is clear: implement proper bulk construction and segment parallelism.

### Key Insights Discovered
1. **Segments > Shared Parallelism**: Independent segments work, shared graph doesn't
2. **Quality > Speed**: 100% recall at 3K vec/s beats 0% recall at 27K vec/s
3. **Parameters Matter**: ef_construction=50 not 200
4. **Navigation Critical**: NEVER skip layer traversal in HNSW

### Technical Debt
- Bulk insertion completely broken (creates disconnected graphs)
- No real parallelism (everything sequential)
- Zero-copy FFI not implemented (10% overhead)
- Test coverage incomplete

### Competitive Position
```
Current: Matches Chroma (3-5K vec/s)
Need: Match Qdrant/Weaviate (15-25K vec/s)
Achievable: Yes, with segment parallelism + proper bulk construction
Timeline: 2-3 weeks with focus
```

## Quick Decision Trees

### IF performance regression:
1. Check if bulk insertion was enabled (usually causes 0% recall)
2. Verify SIMD functions are being called
3. Ensure ef_construction hasn't been increased
4. Test at multiple scales (100, 1K, 5K vectors)

### IF recall drops:
1. Check hierarchical navigation not skipped
2. Verify bidirectional connections maintained
3. Ensure proper entry point management
4. Test with individual insertion (known working)

### IF memory crashes:
1. Check lazy initialization pattern
2. Verify pointer lifecycle management
3. Use individual insertion until bulk fixed
4. Check for double-free or use-after-free

### IF optimizing performance:
1. Profile first, don't guess bottlenecks
2. Test with working algorithms before implementing new ones
3. Preserve ALL HNSW invariants
4. Validate recall at every step

## Examples & Patterns

### High-Performance Vector Insertion
```python
# Current working pattern (individual insertion)
import numpy as np
from python.omendb import DB

vectors = np.random.rand(1000, 128).astype(np.float32)
db = DB("/tmp/test.omen")

# This works reliably
for i, vector in enumerate(vectors):
    db.add(vector, id=f"vec_{i}")
```

### Proper Performance Testing
```python
# Always test at multiple scales
test_sizes = [100, 500, 1000, 2000, 5000]
for n in test_sizes:
    vectors = generate_test_vectors(n, 128)

    start = time.time()
    insert_vectors(vectors)
    rate = n / (time.time() - start)

    recall = test_recall(vectors)
    print(f"{n} vectors: {rate:.0f} vec/s, {recall:.1f}% recall")
```

---

*This file represents the current truth about OmenDB development as of September 19, 2025*