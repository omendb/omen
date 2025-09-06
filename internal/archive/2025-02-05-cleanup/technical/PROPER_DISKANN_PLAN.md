# Plan for Correct DiskANN Implementation
*December 2024 - How to Actually Fix This*

## Option A: Fix Current Implementation (4-6 weeks)

### Phase 1: Fix RobustPrune (1 week)
```python
def robust_prune(p, candidates, R, alpha, graph):
    """
    Correct RobustPrune from DiskANN paper
    """
    pruned = []
    candidates.sort(by_distance_to_p)
    
    for c in candidates:
        if len(pruned) >= R:
            break
            
        # Check α-RNG property
        should_add = True
        for q in pruned:
            if distance(c, q) < alpha * distance(p, c):
                # c is too close to existing neighbor q
                should_add = False
                break
        
        if should_add:
            pruned.append(c)
            
    # Ensure we have at least R/2 edges for connectivity
    while len(pruned) < R/2 and candidates:
        pruned.append(candidates.pop(0))
        
    return pruned
```

### Phase 2: Fix Search (1 week)
```python
def search_layer(query, entry_point, L, graph):
    """
    Correct beam search from Vamana paper
    """
    W = {entry_point}  # Working set
    V = set()          # Visited set
    
    while True:
        # Get closest unvisited node from W
        p_star = min(W - V, key=lambda x: distance(query, x))
        
        if p_star in V:
            break  # All nodes in W are visited
            
        V.add(p_star)
        
        # Check neighbors of p_star
        for neighbor in graph[p_star]:
            if neighbor not in V:
                W.add(neighbor)
                
                # Prune W to size L
                if len(W) > L:
                    W.remove(max(W, key=lambda x: distance(query, x)))
    
    return sorted(W, key=lambda x: distance(query, x))[:k]
```

### Phase 3: Fix Build (1 week)
```python
def build_vamana(data, R, L, alpha):
    """
    Correct Vamana index construction
    """
    # 1. Initialize with random graph
    graph = create_random_graph(data, R)
    
    # 2. Calculate medoid
    medoid = find_medoid(data)
    
    # 3. Multiple iterations to improve graph
    for iteration in range(2):  # Usually 2 passes sufficient
        for i, point in enumerate(data):
            # Search for neighbors
            candidates = search_layer(point, medoid, L, graph)
            
            # Prune to get best R neighbors
            new_neighbors = robust_prune(i, candidates, R, alpha, graph)
            
            # Update edges (bidirectional)
            for neighbor in new_neighbors:
                graph[i].add(neighbor)
                
                # Reverse edge with pruning
                reverse_candidates = graph[neighbor] + [i]
                graph[neighbor] = robust_prune(
                    neighbor, reverse_candidates, R, alpha, graph
                )
    
    return graph, medoid
```

### Phase 4: Add Disk Persistence (2 weeks)
```python
class DiskANNIndex:
    def __init__(self, dim, metric='l2'):
        self.graph_file = mmap.mmap(...)  # Memory-mapped graph
        self.vector_file = mmap.mmap(...) # Compressed vectors
        
    def add(self, vector):
        # Compress with PQ
        compressed = self.pq_encode(vector)
        # Write to disk
        self.vector_file.write(compressed)
        # Update graph
        self.update_graph(...)
```

## Option B: Use Microsoft's DiskANN (1-2 weeks)

### Step 1: Create Mojo Bindings
```mojo
@external
fn diskann_build(
    data: UnsafePointer[Float32],
    num_points: Int,
    dim: Int,
    index_path: String,
    R: Int,
    L: Int,
    alpha: Float32
) -> Int32

@external  
fn diskann_search(
    index_path: String,
    query: UnsafePointer[Float32],
    K: Int,
    L: Int,
    indices: UnsafePointer[Int32],
    distances: UnsafePointer[Float32]
) -> Int32
```

### Step 2: Wrap in Mojo Class
```mojo
struct DiskANN:
    var index_path: String
    var dim: Int
    
    fn build(self, data: List[List[Float32]]):
        # Convert to C layout
        # Call diskann_build
        
    fn search(self, query: List[Float32], k: Int):
        # Call diskann_search
        # Convert results
```

## Option C: Switch to HNSW (2-3 weeks)

HNSW might be easier to implement correctly:
- Simpler algorithm
- No complex pruning
- Well-documented
- Many reference implementations

## Recommended Approach

### Short Term (This Week)
1. **Be Honest**: Update all docs to reflect that this is NOT DiskANN
2. **Set Expectations**: 20K vectors is the limit, not production ready
3. **Stop Claiming**: Remove "DiskANN" from descriptions

### Medium Term (This Month)
**Option B**: Use Microsoft's implementation
- Fastest path to correct behavior
- Battle-tested code
- Actual disk persistence
- Scales to billions

### Long Term (Next Quarter)
**Option A**: Fix implementation IF needed
- Only if Mojo bindings insufficient
- Significant effort required
- High risk of more bugs

## Testing Plan

### Correctness Tests
```python
def test_robust_prune():
    # Test α-RNG property maintained
    # Test connectivity preserved
    # Test degree bounds respected

def test_search_quality():
    # Compare with brute force
    # Should find >95% of true neighbors
    # Test on standard datasets (SIFT, GIST)

def test_scale():
    # Test with 1M+ vectors
    # Memory should stay bounded
    # Search time should be logarithmic
```

## Success Criteria

A correct DiskANN implementation should:
1. ✅ Find 95%+ recall at 10
2. ✅ Scale to 1M+ vectors
3. ✅ Search in <5ms at 1M scale
4. ✅ Use <100 bytes per vector
5. ✅ Build in <10 minutes for 1M vectors
6. ✅ Support incremental updates
7. ✅ Persist to disk

Currently we achieve: **0 of 7** ❌

## Conclusion

**Current state is not fixable with minor tweaks.**

The implementation needs fundamental algorithmic corrections. The fastest path to a working system is using Microsoft's DiskANN with Mojo bindings.

Continuing to claim this is DiskANN is misleading. It's a simple graph index with basic pruning, not the sophisticated system described in the paper.

---

*Recommendation: Use Option B (Microsoft's DiskANN) for immediate results.*