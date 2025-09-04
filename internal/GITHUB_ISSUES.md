# GitHub Issues to Create
*Documentation for issues that need to be filed*

## 🔴 P0 Issues (Critical - Algorithm Correctness)

### Issue 1: RobustPrune Algorithm is Incorrect
**Title**: `[CRITICAL] RobustPrune algorithm doesn't implement α-RNG property`
**Labels**: `bug`, `critical`, `algorithm`, `diskann`

**Description**:
Our current RobustPrune implementation only checks distance thresholds, but doesn't implement the α-RNG (Relative Neighborhood Graph) property required by DiskANN.

**Current (Wrong) Implementation**:
```mojo
if dist_between < candidate_dist * 1.2:
    is_diverse = False
```

**Required (Correct) Implementation**:
For each candidate `c` and existing neighbor `q`, reject `c` if:
- `distance(c, q) < α * distance(p, c)` AND 
- Need to maintain graph connectivity
- Need bidirectional pruning consideration

**Impact**: Graph becomes disconnected, poor search quality, unbounded degree growth.

**Files**: `core/vamana_pruned.mojo:289-353`

---

### Issue 2: Search Algorithm Uses Wrong Termination Condition  
**Title**: `[CRITICAL] Beam search uses fixed iterations instead of convergence`
**Labels**: `bug`, `critical`, `algorithm`, `diskann`

**Description**:
Our search terminates after fixed iterations, but DiskANN beam search should continue until the working set stops improving.

**Current (Wrong)**:
```mojo
while not beam.is_empty() and iterations < max_iterations:
```

**Required (Correct)**:
```python
while True:
    p_star = closest_unvisited_in_working_set()
    if p_star in visited_set:
        break  # Convergence reached
```

**Impact**: Poor recall, suboptimal search results.

**Files**: `algorithms/diskann.mojo:622-639`

---

### Issue 3: Graph Build Process is Incorrect
**Title**: `[CRITICAL] Missing random graph initialization and multi-pass construction`
**Labels**: `bug`, `critical`, `algorithm`, `diskann`  

**Description**:
DiskANN should start with random graph and iterate multiple times until convergence. We start empty and do single pass.

**Missing Components**:
1. Random graph initialization (R random edges per node)
2. Medoid calculation and maintenance  
3. Multiple build iterations (typically 2 passes)
4. Convergence detection

**Impact**: Poor graph quality, suboptimal connectivity.

**Files**: `algorithms/diskann.mojo:add_node`, `core/vamana_pruned.mojo:add_node`

---

## 🟡 P1 Issues (Important - Missing Features)

### Issue 4: No Disk Persistence Component
**Title**: `[FEATURE] Add memory-mapped graph storage for true "DiskANN"`
**Labels**: `enhancement`, `diskann`, `scalability`

**Description**:
Despite being called "DiskANN", everything is stored in memory. Need:
- Memory-mapped graph files
- Compressed vector storage (Product Quantization)
- Lazy loading of graph components

**Impact**: Cannot scale beyond memory limits.

---

### Issue 5: Missing Product Quantization
**Title**: `[FEATURE] Add Product Quantization for vector compression`
**Labels**: `enhancement`, `diskann`, `compression`

**Description**:
Essential component of DiskANN for disk storage efficiency. Need PQ32 or similar.

---

### Issue 6: Still Crashes at 25K Vectors
**Title**: `[BUG] System crashes at ~25K vectors despite "pruning"`
**Labels**: `bug`, `memory`, `scalability`

**Description**:
Even with our incorrect pruning, system should theoretically handle more than 25K vectors. Other memory issues likely exist.

---

## 📊 P2 Issues (Testing & Validation)

### Issue 7: No Correctness Testing
**Title**: `[TEST] Add recall benchmarks against ground truth`
**Labels**: `testing`, `quality`

**Description**:
Need tests that verify search quality:
- Recall@10 should be >95%
- Test on standard datasets (SIFT, GIST)
- Compare with brute force ground truth

---

### Issue 8: No Scale Testing
**Title**: `[TEST] Add automated testing at 100K+ vectors`
**Labels**: `testing`, `scalability`

**Description**:
Real DiskANN should handle millions of vectors. Need continuous integration tests.

---

## 🛠 How to Create Issues

1. Go to https://github.com/[your-org]/omendb/issues
2. Click "New Issue"  
3. Copy title and description from above
4. Add appropriate labels
5. Assign to appropriate milestone
6. Link to relevant files/PRs

## Priority Order

**Week 1**: Issues 1, 2, 3 (Algorithm correctness)
**Week 2**: Issues 4, 5 (Missing features)
**Week 3**: Issues 6, 7, 8 (Testing & validation)

**Total Estimated Effort**: 4-6 weeks for complete fix

---

*These issues represent the fundamental problems that prevent OmenDB from being a correct DiskANN implementation.*