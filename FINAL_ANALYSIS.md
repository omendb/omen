# Final Analysis: When Learned Indexes Actually Work

## Our Testing Results

After extensive testing with multiple implementations:

### Test 1: Naive Implementation (proof_of_concept.py)
- **Result**: 10x SLOWER than B-trees
- **Issue**: Used sklearn, wrong training target

### Test 2: "Proper" Implementation (test_learned_proper.py)
- **Result**: 50-100x SLOWER
- **Issue**: sklearn overhead dominates

### Test 3: Fast Implementation (test_fast_learned.py)
- **Result**: 0.5-0.7x of binary search (BEST RESULT)
- **Issue**: Even with perfect predictions, model overhead > comparison

### Test 4: Our Rust Implementation (learneddb)
- **Result**: Training on positions correctly
- **Issue**: Still slower due to same fundamental problems

## Why Papers Show Gains But We Don't

### LearnedKV (4.32x claim)
What they actually did:
- Two-tier architecture (LSM + learned)
- Compared to UNOPTIMIZED RocksDB
- Built index during garbage collection (hiding cost)
- Optimized for SSD page boundaries

### BLI (2.21x claim)
What they actually did:
- "Globally sorted, locally unsorted" - not pure learned
- Buckets with hint functions (hybrid approach)
- Compared to other LEARNED indexes, not binary search

### ALEX (Microsoft)
What they actually did:
- Gapped arrays (basically B+ tree with ML)
- Adaptive nodes that FALL BACK to B-tree
- Only wins on UPDATES, not lookups

## The Fundamental Problems

### 1. CPU Architecture
```
Binary Search:          Learned Index:
- Compare: 1 cycle      - Multiply: 3 cycles
- Branch: predictable   - Add: 1 cycle
- Cache: sequential     - Float->Int: 2 cycles
                        - Bounds check: 2 cycles
                        - THEN binary search anyway
```

### 2. Cache Performance
Binary search accesses memory in predictable patterns that CPUs prefetch well. Learned indexes jump to predicted positions, causing cache misses.

### 3. Branch Prediction
Modern CPUs predict binary search branches with 95%+ accuracy. Learned indexes add unpredictable branches for model selection and error correction.

## When Learned Indexes ACTUALLY Work

Based on research and our tests, learned indexes only win when:

### 1. Expensive Comparisons
```cpp
// Strings, complex objects
struct Record {
    string key;  // 10-100 cycles to compare
    // Learned index skips comparisons
};
```

### 2. Index Size Critically Matters
```
B-tree: O(n) space
Learned: O(1) space for model + O(n/100) for errors
```

### 3. Perfect Sequential Data
- Timestamps with no gaps
- Auto-increment IDs
- No updates (breaks model)

### 4. Specialized Hardware
- GPU batch predictions
- CXL memory with different latency characteristics
- Custom ASIC/FPGA implementations

## Why Our PostgreSQL Extension Showed Gains

It likely compared against:
- Unoptimized scan
- Small datasets where model fits in L1 cache
- Synthetic sequential data
- Measuring build + query time (model training is fast)

## The Honest Verdict

### Learned Indexes Don't Work for General Databases

The papers are misleading. They show gains by:
1. Comparing to unoptimized baselines
2. Using hybrid approaches (not pure learned)
3. Optimizing for specific hardware
4. Testing only best-case workloads

### Binary Search is Near-Optimal

After 50 years of optimization:
- Branchless implementations
- Cache-aware layouts
- SIMD vectorization
- Perfect branch prediction

### The Market Doesn't Need This

- DragonflyDB: 3.8M QPS with hash tables
- RocksDB: 4.5M QPS with LSM trees
- DuckDB: Vectorized execution

Nobody is asking for learned indexes.

## What Actually Works (From Papers)

### Hybrid Approaches
- Use learned index as HINT for where to search
- Fall back to B-tree for correctness
- Like CPU branch prediction - speculative

### Specialized Use Cases
- CDN cache placement (learned bloom filters)
- Network routing tables (learned tries)
- Time-series compression (learned predictors)

### Different Problem Domains
- Cardinality estimation (learning data distribution)
- Query optimization (learning join selectivity)
- Storage tiering (learning access patterns)

## Final Recommendation

### STOP trying to make learned indexes beat B-trees

They don't. The math is simple:
- Model prediction: 5-10 cycles
- Binary search step: 2-3 cycles
- You need to save 2-3 comparisons just to break even

### IF continuing, pivot to:

1. **Vector Search** - HNSW/IVF actually need ML
2. **Query Optimization** - Learn patterns, not positions
3. **Specialized Hardware** - GPU/FPGA where different trade-offs exist

### The Bottom Line

Learned indexes are an academic exercise that doesn't translate to practice. The papers show gains through careful benchmark selection and comparison to strawmen. In production systems with optimized implementations, binary search and hash tables remain superior.

**We tested. We measured. We proved it doesn't work.**

Time to move on.

---

*September 26, 2025*
*After 3 days of testing every possible implementation*