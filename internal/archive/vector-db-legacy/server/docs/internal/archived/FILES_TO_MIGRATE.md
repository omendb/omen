# Files to Migrate from Public to Private Repository

## Files That Should Be MOVED to omendb-server/

### Advanced Optimizations (Competitive Advantage)
```
omendb/algorithms/faiss_style_distance.mojo     → Keep public (educational value)
omendb/algorithms/hnsw_batch_optimized.mojo    → Keep public (core feature)
omendb/algorithms/hnsw_simd_optimized.mojo     → Keep public (standard optimization)
omendb/algorithms/hnsw_ultra.mojo              → MOVE to private (advanced/experimental)
omendb/core/incremental_migration.mojo         → MOVE to private (server-specific)
```

### Enterprise Test Files
```
test/validation/test_enterprise_scale.py        → MOVE to private
test/validation/test_large_scale.py            → MOVE to private  
examples/getting_started/production_scale.py    → MOVE to private
```

### Potential GPU/Distributed Features
```
Any future files with:
- GPU acceleration code
- Distributed algorithms
- Multi-node coordination
- Advanced monitoring
- Performance profiling tools
```

## Files That Should STAY Public

### Core Algorithm (Market Education)
```
omendb/algorithms/hnsw.mojo                    → Stay public
omendb/algorithms/hnsw_batch_optimized.mojo    → Stay public
omendb/core/brute_force.mojo                  → Stay public
omendb/native.mojo                             → Stay public
```

### Basic Features
```
All basic CRUD operations
Standard benchmarks
Simple examples
Public documentation
```

## Migration Strategy

1. **Copy files** to server repo first
2. **Modify imports** in server version
3. **Remove from public** after server integration working
4. **Update public docs** to mention "advanced features in server edition"

## Rationale

**Keep Public**: Features that educate the market and show our competence
**Move Private**: Features that provide unfair competitive advantage

The goal is to have enough in public to prove we're legitimate while keeping our secret sauce private.