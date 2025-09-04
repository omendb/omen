# OmenDB Error Fixes - Problem → Solution
*Actionable fixes for common OmenDB issues*

## Memory & Scale Issues

### Problem: Crash at 25K vectors
```
SYMPTOM: System crashes when adding ~25K vectors
ERROR: Segmentation fault during vector insertion
ROOT_CAUSE: CSR graph cannot remove edges → unbounded growth
```
**SOLUTION:**
```bash
# Replace CSR with adjacency list
# File: /Users/nick/github/omendb/omendb/algorithms/diskann.mojo
# Replace: CSRGraph with AdjacencyListGraph
# Enable: Edge pruning during RobustPrune phase
```

### Problem: Dict overhead causing memory explosion
```
SYMPTOM: Memory usage 8KB per entry (expected ~80 bytes)
ERROR: OOM at small vector counts
ROOT_CAUSE: Mojo Dict[String, Int] has massive overhead
```
**SOLUTION:**
```mojo
# REPLACE: Dict[String, Int] 
# WITH: SparseMap implementation
# RESULT: 180x memory reduction (8KB → 44 bytes)
# FILES: vector_buffer.mojo:25 (id_to_index mapping)
```

## Algorithm Issues  

### Problem: RobustPrune algorithm incorrect
```
SYMPTOM: Poor search recall, disconnected graph
ERROR: Graph becomes non-navigable at scale  
ROOT_CAUSE: Missing α-RNG property implementation
```
**SOLUTION:**
```mojo
# Current (wrong): if dist_between < candidate_dist * 1.2
# Required (correct): Check α-RNG property for each candidate
# α-RNG: distance(c,q) < α * distance(p,c) for all existing neighbors q
# FILE: core/vamana_pruned.mojo:289-353
```

### Problem: Search uses fixed iterations not convergence
```
SYMPTOM: Suboptimal search results
ERROR: Search terminates before finding optimal path
ROOT_CAUSE: Fixed iteration count vs convergence detection
```
**SOLUTION:**
```mojo
# Current (wrong): while iterations < max_iterations
# Required (correct): while p_star not in visited_set  
# Implement: Working set convergence with W and V sets
# FILE: algorithms/diskann.mojo:622-639
```

## Integration Issues

### Problem: Vamana and PQ disconnected
```
SYMPTOM: PQ compression not working with Vamana
ERROR: Components exist but not integrated
ROOT_CAUSE: No VamanaWithPQ combined struct
```
**SOLUTION:**
```mojo
# Create: VamanaIndexWithPQ combining both algorithms
# Wire: PQ quantization into Vamana graph building
# Test: Memory reduction (target 10x with PQ8)
# FILES: native.mojo (add combined struct)
```

## Performance Issues

### Problem: FFI overhead 8KB per call
```
SYMPTOM: Slow individual vector additions
ERROR: Single add() vs batch add_batch() performance cliff
ROOT_CAUSE: Python→Mojo FFI has high overhead
```
**SOLUTION:**
```python
# AVOID: for vector in vectors: db.add(vector)    # 8KB per call
# USE: db.add_batch(vectors)                       # Single 1.5KB call
# RESULT: 5x performance improvement
```

### Problem: Buffer flush adds vectors one-by-one
```
SYMPTOM: 39ms per 100 vectors during flush
ERROR: Performance cliff at buffer boundary (10K, 100K)
ROOT_CAUSE: Buffer flush calls DiskANN.add() individually
```
**SOLUTION:**
```mojo
# Implement: DiskANNIndex.add_batch() method
# Replace: Individual add() calls with bulk insert
# Result: Remove major bottleneck at boundaries
# FILE: native.mojo:1850-2000 (buffer flush code)
```

## Testing Issues

### Problem: Global singleton causes test interference
```
SYMPTOM: Tests pass individually, fail when run together
ERROR: Segfaults during test suite execution
ROOT_CAUSE: All DB() instances share same VectorStore
```
**SOLUTION:**
```python
# Pattern: Clear state between tests
db1 = DB()
db1.add_batch(vectors, ids=["test1_0", "test1_1"])
# Before next test:
db1.clear()  # Reset global state
db2 = DB()   # Same instance, but cleared
```

## Build Issues

### Problem: Mojo build fails with undefined symbols
```
SYMPTOM: Linking errors during compilation
ERROR: Undefined reference to DiskANN methods
ROOT_CAUSE: Missing native library compilation
```
**SOLUTION:**
```bash
cd /Users/nick/github/omendb/omendb
pixi run mojo build omendb/native.mojo -o python/omendb/native.so --emit shared-lib
# Verify: ls -la python/omendb/native.so
```

## Diagnostic Commands

### Memory Usage Analysis
```bash
# Profile specific operations
with profiler.ProfileBlock("buffer_overflow"):
    ids = db.add_batch(large_batch)

# Check memory at scale
pixi run python -c "
from omendb import DB
db = DB()
db.add_batch(vectors[:20000])  # Test crash point
"
```

### Performance Benchmarking  
```bash
cd /Users/nick/github/omendb/omendb
PYTHONPATH=python:$PYTHONPATH python benchmarks/quick_benchmark.py
```

### Debug Mode
```bash
# Use Mojo's debugging tools
mojo debug native.mojo
# Or in Python context:
cd omendb && PYTHONPATH=python:$PYTHONPATH mojo debug -c "import omendb; db = omendb.DB()"
```

---
*Problem → Solution format for quick error resolution*