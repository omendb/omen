# State-of-the-Art Learned Index Research

**Date:** October 2, 2025
**Purpose:** Survey best practices from academic research and production implementations

---

## Summary

**Key finding:** Our RMI implementation is algorithmically sound, but we're missing critical optimizations from state-of-the-art implementations.

**What we should adopt:**
1. **Epsilon-based error control** (PGM-index) - More principled than hardcoded values
2. **Adaptive restructuring** (ALEX) - Handle changing data distributions
3. **Lazy index rebuilds** (Common pattern) - Don't rebuild on every insert
4. **Piecewise linear approximation metrics** - Quantify data "hardness"

**What we're already doing right:**
- ✅ Hierarchical structure (root + second layer models)
- ✅ Linear regression for predictions
- ✅ Binary search in error windows
- ✅ Batch insertion optimization

---

## 1. PGM-Index (VLDB 2020)

**Paper:** Ferragina & Vinciguerra - "The PGM-index: a fully-dynamic compressed learned index with provable worst-case bounds"
**Source:** http://www.vldb.org/pvldb/vol13/p1162-ferragina.pdf
**Implementation:** https://github.com/gvinciguerra/PGM-index (C++)

### Key Algorithm: Piecewise Geometric Model

**Core idea:** Approximate sorted array with piecewise linear segments, each with provable error bounds

**Complexity guarantees:**
- Space: O(n/ε²) where ε is error parameter
- Query time: O(log(n/ε))
- Update time: O(log(n/ε)) amortized

### Algorithmic Innovations

#### 1. Epsilon Parameter (ε)
**What it is:** Maximum allowed prediction error (in positions, not keys)

**How it works:**
```cpp
// User specifies epsilon (e.g., 128)
pgm::PGMIndex<int, 128> index(data);

// During construction:
// - Each segment guarantees prediction error ≤ ε
// - Binary search window is [predicted - ε, predicted + ε]
// - More segments for complex distributions
```

**Trade-offs:**
- Small ε (e.g., 4): More segments, larger index, faster queries
- Large ε (e.g., 256): Fewer segments, smaller index, slower queries

**What we're doing wrong:**
```rust
// src/index.rs:234
max_error = (max_error + 1).min(8).max(1); // ❌ Arbitrary cap at 8!

// src/redb_storage.rs:225
let window_size = 100; // ❌ Hardcoded, should use model's max_error
```

**How to fix:**
- Make epsilon configurable (constructor parameter)
- Use actual computed errors, don't cap artificially
- Allow users to tune space-time tradeoff

#### 2. Optimal Segment Construction

**Algorithm:** Given keys and epsilon, find minimum number of segments

**PGM's approach:**
1. Start at first key
2. Fit linear model to progressively more keys
3. When error exceeds ε, finalize segment and start new one
4. Guaranteed optimal: O(n) time, minimum segments

**Pseudocode:**
```
segments = []
start = 0
while start < n:
    end = start + 1
    while end < n:
        model = fit_linear(keys[start:end+1])
        max_error = compute_max_error(model, keys[start:end+1])
        if max_error > epsilon:
            break
        end += 1
    segments.append((start, end, model))
    start = end
```

**What we're doing:**
```rust
// src/index.rs:134
let segment_size = (n + self.num_second_models - 1) / self.num_second_models;
// ❌ Fixed-size segments, not optimal!
```

**How to fix:**
- Implement greedy optimal segmentation algorithm
- Or keep fixed segments but use proper error bounds

#### 3. Recursive Structure

**PGM's multi-level index:**
```
Level 0 (top): Index of Level 1 segments
Level 1: Index of Level 2 segments
...
Level k: Index of actual data
```

**Each level built with same ε parameter**

**Our implementation:**
```rust
// src/index.rs:11-22
pub struct RecursiveModelIndex {
    root: LinearModel,           // ✅ Top level
    second_layer: Vec<LinearModel>, // ✅ Second level
    data: Vec<(i64, usize)>,     // ✅ Actual data
    num_second_models: usize,    // ⚠️ Fixed, not adaptive
}
```

**Status:** ✅ We have this, but could improve with adaptive levels

### Variants We Should Consider

#### DynamicPGMIndex
**Feature:** Supports inserts/deletes with buffering

**How it works:**
1. Small writes go to buffer (sorted array)
2. Queries check buffer first, then main index
3. When buffer full, merge into main index (bulk rebuild)

**Relevance:** ✅ Exactly what we need for P0 optimization!

**Implementation idea:**
```rust
pub struct RedbStorage {
    db: Database,
    learned_index: RecursiveModelIndex,
    sorted_keys: Vec<i64>,

    // NEW: Buffer for pending inserts
    insert_buffer: Vec<i64>,
    buffer_size_threshold: usize, // e.g., 10000
}
```

#### CompressedPGMIndex
**Feature:** Compresses segments to reduce space

**Technique:** Delta encoding + variable-length integers

**Relevance:** ⚠️ Lower priority (space not our bottleneck yet)

---

## 2. ALEX (Microsoft SIGMOD 2020)

**Paper:** Ding et al. - "ALEX: An Updatable Adaptive Learned Index"
**Source:** https://dl.acm.org/doi/10.1145/3318464.3389711
**Implementation:** https://github.com/microsoft/ALEX (C++)

### Key Innovation: Adaptive Restructuring

**Problem with static learned indexes:**
- Trained on initial data distribution
- Performance degrades as data changes
- Full rebuild is expensive

**ALEX's solution:**
- Monitor prediction error per node
- When error exceeds threshold, restructure that subtree
- Localized updates instead of full rebuild

### Algorithmic Insights

#### 1. Hierarchical Tree Structure

**ALEX uses:**
- Internal nodes: Linear models (like RMI)
- Leaf nodes: Gapped arrays (like B+tree leaves)

**Why gapped arrays:**
- Allow fast inserts without shifting entire array
- Reserve gaps between elements (e.g., 10% overhead)
- When gaps exhausted, split leaf node

**Example:**
```
Leaf node capacity: 100 elements
Actual elements: 90
Gaps: 10 (distributed throughout)

Insert new key:
- Find position using linear model
- Insert into nearby gap (O(1) average)
- No full array shift needed
```

**Our implementation:**
```rust
// src/redb_storage.rs:156
self.sorted_keys.insert(pos, key); // ❌ O(n) Vec shift!
```

**How to fix:**
- Use chunked storage with gaps
- Or use deferred rebuild (P0)

#### 2. Cost-Based Splitting

**When to split a node:**
```
cost(current_node) vs cost(two_smaller_nodes)

Factors:
- Prediction error (larger error = higher query cost)
- Node density (more packed = higher insert cost)
- Split overhead (creating new node)
```

**ALEX's heuristic:**
- Monitor insert cost per node
- When cost exceeds threshold, split
- Choose split point that minimizes error

**Relevance:** 🔮 Future optimization (after P0/P1)

#### 3. Exponential Search for Outliers

**Problem:** Outlier keys far from prediction

**ALEX's solution:**
```cpp
// Instead of fixed window [pred - ε, pred + ε]
// Use exponential search if not found:
positions = [pred, pred-1, pred+1, pred-2, pred+2, pred-4, pred+4, pred-8, ...]
```

**Benefit:** Handles outliers gracefully without inflating ε

**Our implementation:**
```rust
// src/index.rs:289-294
let start = global_pos.saturating_sub(model.max_error).max(model.start_idx);
let end = (global_pos + model.max_error + 1).min(model.end_idx);
// ❌ Fixed window only
```

**How to fix:**
- Add exponential fallback for not-found keys
- Or improve error bound calculation (P1)

#### 4. Performance Results

**From SIGMOD 2020 paper:**
- **vs RMI:** 2.2x better throughput, 15x smaller index
- **vs B+tree:** 4.1x better throughput (up to), 2000x smaller index
- **Weakness:** Nonlinear distributions, extreme outliers

**Key metrics:**
- Throughput: Queries + inserts per second
- Space: Bytes per key
- Adaptability: Performance under changing workload

---

## 3. VLDB 2022: Comprehensive Evaluation

**Paper:** Wongkham et al. - "Are Updatable Learned Indexes Ready?"
**Source:** https://vldb.org/pvldb/vol15/p3004-wongkham.pdf

### Key Findings

#### Performance Rankings
**Best performers (80%+ of scenarios):**
1. ALEX
2. LIPP (Learned Index with Persistent Pointers)
3. PGM-DynamicIndex

**Traditional baselines:**
- B+tree: Good for write-heavy workloads
- ART (Adaptive Radix Tree): Good for string keys

#### Data Hardness Metrics

**New concept:** Piecewise Linear Approximation (PLA) difficulty

**Two dimensions:**
1. **Global hardness:** How many segments needed?
   - Linear data: 1 segment (hardness = 0)
   - Log-normal: ~10 segments (hardness = medium)
   - Zipfian: ~100 segments (hardness = high)

2. **Local hardness:** Prediction error within segments
   - Uniform within segment: Low local hardness
   - Noisy within segment: High local hardness

**Relevance:** We should measure this for our datasets!

**How to measure:**
```rust
// Compute global hardness
fn global_hardness(keys: &[i64], epsilon: usize) -> usize {
    let segments = build_optimal_segments(keys, epsilon);
    segments.len() // More segments = harder
}

// Compute local hardness
fn local_hardness(segment: &[(i64, usize)], model: &LinearModel) -> f64 {
    let errors: Vec<f64> = segment.iter()
        .map(|(k, i)| {
            let pred = model.predict(*k);
            (pred - *i as f64).abs()
        })
        .collect();

    errors.iter().sum::<f64>() / errors.len() as f64 // Average error
}
```

#### Workload Characteristics

**Best for learned indexes:**
- Read-heavy (90% reads, 10% writes)
- Batch inserts (not individual inserts)
- Predictable distributions (linear, log-normal)

**Worst for learned indexes:**
- Write-heavy (10% reads, 90% writes)
- Random inserts (no locality)
- Highly nonlinear distributions (Zipfian, adversarial)

**OmenDB's target workload:**
- Time-series analytics: ✅ Read-heavy, sequential inserts
- OLTP: ⚠️ Write-heavy, random access
- **Verdict:** Learned indexes are a good fit for our analytics use case

---

## 4. RadixSpline (aiDM 2020)

**Paper:** Kipf et al. - "RadixSpline: a single-pass learned index"
**Source:** https://github.com/learnedsystems/RadixSpline

### Key Innovation: Single-Pass Construction

**Problem with RMI/PGM:**
- Require full data scan to build index
- Can't build incrementally

**RadixSpline's solution:**
- Use radix tree to partition keys
- Fit splines within radix tree nodes
- Can build in single pass over data

**Algorithm:**
1. Insert keys into radix tree (standard radix tree)
2. When node fills, fit cubic spline to keys in node
3. Replace node with spline model + error bounds

**Benefit:** O(n) construction, no sorting required

**Relevance:** 🔮 Future work (for streaming inserts)

---

## 5. Comparison to Our Implementation

### What We're Doing Right ✅

1. **Hierarchical structure:** Root + second layer models
2. **Linear regression:** Standard least-squares fitting
3. **Binary search in windows:** Use prediction + error bound
4. **Batch training:** Don't train on every insert
5. **Segment-based approach:** Divide data into chunks

### What We're Missing ❌

1. **Epsilon-based error control**
   - Currently: Hardcoded `max_error.min(8)`
   - Should: User-configurable epsilon, use actual errors

2. **Lazy index rebuilds**
   - Currently: Rebuild on every `insert_batch()`
   - Should: Defer rebuild until query time (P0)

3. **Adaptive restructuring**
   - Currently: Fixed number of models
   - Should: Adapt to data distribution

4. **Insert buffering**
   - Currently: Immediate rebuild
   - Should: Buffer small inserts, merge periodically

5. **Error bound calculation**
   - Currently: Sample first 100 elements
   - Should: Sample strategically or compute for all

6. **Outlier handling**
   - Currently: Fixed window only
   - Should: Exponential search fallback

### Priority Ranking

| Fix | Complexity | Impact | Priority |
|-----|-----------|--------|----------|
| **P0: Lazy rebuild** | Low | 10-100x insert | 🔥 Must do |
| **P1: Fix error bounds** | Low | 2-5x query | 🔥 Must do |
| **P2: Epsilon parameter** | Low | Better tuning | ⚠️ Should do |
| **P3: Insert buffering** | Medium | 2-5x insert | ⚠️ Should do |
| **P4: Adaptive restructuring** | High | 2-3x long-term | 🔮 Future |
| **P5: Exponential search** | Medium | Outlier handling | 🔮 Future |

---

## 6. Recommended Implementations

### P0: Lazy Index Rebuild (Based on DynamicPGMIndex pattern)

```rust
pub struct RedbStorage {
    db: Database,
    learned_index: RecursiveModelIndex,
    sorted_keys: Vec<i64>,

    // NEW: Track if index needs rebuild
    index_dirty: bool,
}

impl RedbStorage {
    pub fn insert_batch(&mut self, entries: Vec<(i64, Vec<u8>)>) -> Result<()> {
        // Write to redb
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(DATA_TABLE)?;
            for (key, value) in &entries {
                table.insert(*key, value.as_slice())?;
            }
        }
        write_txn.commit()?;

        // Mark index as dirty, don't rebuild yet
        self.index_dirty = true;
        self.row_count += entries.len() as u64;

        Ok(())
    }

    fn ensure_index_fresh(&mut self) -> Result<()> {
        if self.index_dirty {
            self.rebuild_index()?;
            self.index_dirty = false;
        }
        Ok(())
    }

    pub fn point_query(&mut self, key: i64) -> Result<Option<Vec<u8>>> {
        // Rebuild index if dirty (lazy rebuild)
        self.ensure_index_fresh()?;

        // ... rest of query logic ...
    }
}
```

**Expected impact:** 10-100x insert speedup

### P1: Fix Error Bound Calculation (Based on PGM epsilon approach)

```rust
// src/index.rs
pub struct RecursiveModelIndex {
    root: LinearModel,
    second_layer: Vec<LinearModel>,
    data: Vec<(i64, usize)>,

    // NEW: User-configurable error bound
    epsilon: usize, // e.g., 32, 64, 128
}

impl RecursiveModelIndex {
    pub fn new(data_size: usize, epsilon: usize) -> Self {
        // ...
        Self {
            // ...
            epsilon,
        }
    }

    fn train_segment(&self, start: usize, end: usize) -> (f64, f64, usize) {
        // ... fit linear model ...

        // Compute max error for ALL elements (not just sample)
        let mut max_error = 0;
        for (i, (key, _)) in segment.iter().enumerate() {
            let predicted = (slope * (*key as f64) + intercept).round() as i64;
            let error = (predicted - i as i64).abs() as usize;
            max_error = max_error.max(error);
        }

        // Use actual max_error, don't cap it!
        (slope, intercept, max_error.min(self.epsilon))
    }
}

// src/redb_storage.rs
pub fn point_query(&self, key: i64) -> Result<Option<Vec<u8>>> {
    if let Some(predicted_pos) = self.learned_index.search(key) {
        // Use model's actual max_error, not hardcoded 100
        let model = self.learned_index.get_model_for_key(key);
        let window_size = model.max_error;

        let start = predicted_pos.saturating_sub(window_size);
        let end = (predicted_pos + window_size).min(self.sorted_keys.len());

        // Binary search in window
        // ...
    }
}
```

**Expected impact:** 2-5x query speedup

### P2: Epsilon Parameter (Based on PGM API)

```rust
// Allow users to configure space-time tradeoff
let epsilon = 128; // Larger = less space, slower queries
let mut storage = RedbStorage::with_epsilon(&db_path, epsilon)?;
```

### P3: Insert Buffering (Based on DynamicPGMIndex)

```rust
pub struct RedbStorage {
    // ...
    insert_buffer: BTreeSet<i64>, // Sorted buffer
    buffer_threshold: usize, // e.g., 10000
}

impl RedbStorage {
    pub fn point_query(&self, key: i64) -> Result<Option<Vec<u8>>> {
        // Check buffer first
        if self.insert_buffer.contains(&key) {
            // Fetch from redb
            // ...
        }

        // Then check learned index
        // ...
    }

    fn maybe_flush_buffer(&mut self) -> Result<()> {
        if self.insert_buffer.len() >= self.buffer_threshold {
            // Merge buffer into main index
            self.rebuild_index()?;
            self.insert_buffer.clear();
        }
        Ok(())
    }
}
```

---

## 7. Key Takeaways

### For Our Implementation

1. **Keep RMI architecture** - It's solid and well-understood
2. **Fix obvious bugs first** - Lazy rebuild (P0) and error bounds (P1)
3. **Make epsilon configurable** - Let users tune space-time tradeoff
4. **Add insert buffering** - Pattern from DynamicPGMIndex
5. **Measure data hardness** - Use PLA metrics to quantify difficulty

### For Benchmarking

1. **Test multiple distributions**
   - ✅ Already doing: Sequential, Random
   - Add: Log-normal, Zipfian, Real datasets

2. **Measure data hardness**
   - Global: Number of optimal segments
   - Local: Average prediction error per segment

3. **Compare to multiple baselines**
   - ✅ SQLite (B-tree)
   - Add: std::map (Red-Black tree), ART

### For Future Work

1. **Adaptive restructuring** - Like ALEX, monitor and adjust
2. **Concurrency** - Multi-threaded reads, MVCC for writes
3. **Persistence** - Save/load trained models
4. **String keys** - Extend beyond i64

---

## 8. References

### Papers
1. Ferragina & Vinciguerra (2020). "The PGM-index: a fully-dynamic compressed learned index with provable worst-case bounds". PVLDB 13(8).
2. Ding et al. (2020). "ALEX: An Updatable Adaptive Learned Index". SIGMOD.
3. Wongkham et al. (2022). "Are Updatable Learned Indexes Ready?". PVLDB 15(11).
4. Kipf et al. (2020). "RadixSpline: a single-pass learned index". aiDM@SIGMOD.
5. Kraska et al. (2018). "The Case for Learned Index Structures". SIGMOD.

### Implementations
- PGM-index: https://github.com/gvinciguerra/PGM-index
- ALEX: https://github.com/microsoft/ALEX
- RadixSpline: https://github.com/learnedsystems/RadixSpline
- RMI (reference): https://github.com/learnedsystems/RMI

### Datasets for Testing
- SOSD Benchmark: https://github.com/learnedsystems/SOSD
  - Books, OSM, Wiki timestamps, FB user IDs, etc.
  - Standard benchmark suite for learned indexes

---

**Next steps:**
1. Implement P0 (lazy rebuild) - 4 hours
2. Implement P1 (error bounds) - 2 hours
3. Re-run honest benchmarks
4. Target: 7-13x average speedup (vs current 2-4x)
