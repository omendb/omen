# ALEX Implementation Plan - October 2025

## Executive Summary

**Decision**: Replace RMI with ALEX for dynamic workloads

**Why**:
- RMI designed for **static** data → O(n) full rebuild on writes
- ALEX designed for **dynamic** inserts/deletes → O(1) inserts, local splits
- Proven: 4.1x faster than B-tree, never slower (Microsoft Research)

**Target**: <5μs queries at 10M+ scale, no rebuild spikes

---

## ALEX Core Concepts (from Papers)

### 1. Gapped Arrays (Key Innovation)

**Problem with RMI**: Dense sorted_keys array requires full rebuild on insert

**ALEX Solution**: Gapped arrays with capacity for future inserts

```rust
// RMI (current)
sorted_keys: [1, 2, 3, 4, 5]  // Dense, no room for inserts

// ALEX (new)
gapped_keys: [1, None, 2, None, 3, None, 4, None, 5]  // Gaps for inserts
//              ↑ gap      ↑ gap      ↑ gap      ↑ gap
```

**Expansion Factor**: Node capacity = num_keys × (1 + expansion_factor)
- expansion_factor = 1.0 → 50% gaps (default)
- Higher = more gaps = fewer splits but more memory

### 2. Insert Algorithm

```
1. Model predicts approximate position in gapped array
2. Exponential search finds exact position (accounting for gaps)
3. If gap exists → insert O(1)
4. If no gap → shift elements OR trigger node split
```

**Exponential Search** (from paper):
```rust
fn find_insert_position(predicted_pos: usize, key: i64) -> usize {
    let mut bound = 1;

    // Search right
    while predicted_pos + bound < keys.len() {
        if keys[predicted_pos + bound].is_some()
            && keys[predicted_pos + bound].unwrap() >= key {
            break;
        }
        bound *= 2;
    }

    // Binary search in [predicted_pos, predicted_pos + bound]
    binary_search(predicted_pos, predicted_pos + bound, key)
}
```

**Why this is O(log(error))**: If model error is E positions, exponential search finds gap in O(log E) time.

### 3. Node Splitting

**When to split**: When node density exceeds threshold

```rust
const MAX_DENSITY: f64 = 0.8;  // 80% full

if node.num_keys() as f64 / node.capacity() as f64 > MAX_DENSITY {
    split_node(node);
}
```

**Split strategies** (from paper):
- **Sideways**: Split into 2 equal nodes (most common)
- **Downwards**: Insert level between current and children (rare, deep trees)

**After split**:
- Retrain models for the TWO new nodes only (~1-2ms)
- Update parent pointers
- **NO global rebuild**

### 4. Model Structure

**Two-level design** (simplified ALEX):
```
Root Models (static, retrained on major restructure)
    ↓ predict which leaf
Leaf Nodes (gapped arrays, local splits)
    ↓ predict position in array
```

**Linear models** (fast training):
```rust
struct LinearModel {
    slope: f64,
    intercept: f64,
}

fn predict(&self, key: i64) -> usize {
    (self.slope * key as f64 + self.intercept) as usize
}

fn train(keys: &[(i64, usize)]) -> Self {
    // Simple linear regression
    let n = keys.len() as f64;
    let sum_x = keys.iter().map(|(k, _)| *k as f64).sum::<f64>();
    let sum_y = keys.iter().map(|(_, p)| *p as f64).sum::<f64>();
    let sum_xy = keys.iter().map(|(k, p)| *k as f64 * *p as f64).sum::<f64>();
    let sum_x2 = keys.iter().map(|(k, _)| (*k as f64).powi(2)).sum::<f64>();

    let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
    let intercept = (sum_y - slope * sum_x) / n;

    Self { slope, intercept }
}
```

### 5. Adaptive Fanout (Advanced)

**ALEX optimization**: Dynamically adjust node fanout based on workload

```rust
struct CostModel {
    traversal_cost: f64,    // Cost to traverse one level
    model_cost: f64,        // Cost to evaluate model
    search_cost: f64,       // Cost of gap search
}

fn optimal_fanout(workload: &Workload) -> usize {
    // From paper: fanout that minimizes total cost
    // Simple version: fixed fanout of 64-128
    128
}
```

**For now**: Use fixed fanout (simpler), optimize later

---

## Implementation Architecture

### Phase 1: Core Structures (Day 1)

```rust
// File: src/alex/gapped_node.rs
pub struct GappedNode {
    /// Keys with gaps (None = empty slot)
    keys: Vec<Option<i64>>,

    /// Values aligned with keys
    values: Vec<Option<Vec<u8>>>,

    /// Linear model for position prediction
    model: LinearModel,

    /// Expansion factor (1.0 = 50% gaps)
    expansion_factor: f64,

    /// Current number of actual keys (not counting gaps)
    num_keys: usize,
}

impl GappedNode {
    pub fn new(capacity: usize, expansion_factor: f64) -> Self {
        Self {
            keys: vec![None; capacity],
            values: vec![None; capacity],
            model: LinearModel::default(),
            expansion_factor,
            num_keys: 0,
        }
    }

    pub fn insert(&mut self, key: i64, value: Vec<u8>) -> Result<()> {
        // 1. Model predicts position
        let predicted_pos = self.model.predict(key);

        // 2. Exponential search finds exact gap
        let insert_pos = self.find_insert_position(predicted_pos, key);

        // 3. Insert if gap exists
        if self.keys[insert_pos].is_none() {
            self.keys[insert_pos] = Some(key);
            self.values[insert_pos] = Some(value);
            self.num_keys += 1;
            Ok(())
        } else {
            // 4. Shift or trigger split
            self.shift_and_insert(insert_pos, key, value)
        }
    }

    pub fn density(&self) -> f64 {
        self.num_keys as f64 / self.keys.len() as f64
    }

    fn find_insert_position(&self, predicted_pos: usize, key: i64) -> usize {
        // Exponential search implementation
        todo!()
    }

    fn shift_and_insert(&mut self, pos: usize, key: i64, value: Vec<u8>) -> Result<()> {
        // Find nearest gap and shift elements
        todo!()
    }
}
```

```rust
// File: src/alex/linear_model.rs
#[derive(Debug, Clone)]
pub struct LinearModel {
    slope: f64,
    intercept: f64,
}

impl LinearModel {
    pub fn new() -> Self {
        Self {
            slope: 1.0,
            intercept: 0.0,
        }
    }

    pub fn train(&mut self, data: &[(i64, usize)]) {
        if data.is_empty() {
            return;
        }

        let n = data.len() as f64;
        let sum_x = data.iter().map(|(k, _)| *k as f64).sum::<f64>();
        let sum_y = data.iter().map(|(_, p)| *p as f64).sum::<f64>();
        let sum_xy = data.iter().map(|(k, p)| *k as f64 * *p as f64).sum::<f64>();
        let sum_x2 = data.iter().map(|(k, _)| (*k as f64).powi(2)).sum::<f64>();

        let denominator = n * sum_x2 - sum_x * sum_x;

        if denominator.abs() < 1e-10 {
            // All keys are the same
            self.slope = 0.0;
            self.intercept = sum_y / n;
        } else {
            self.slope = (n * sum_xy - sum_x * sum_y) / denominator;
            self.intercept = (sum_y - self.slope * sum_x) / n;
        }
    }

    pub fn predict(&self, key: i64) -> usize {
        let pos = self.slope * key as f64 + self.intercept;
        pos.max(0.0) as usize
    }
}
```

### Phase 2: Adaptive Tree (Day 2)

```rust
// File: src/alex/alex_tree.rs
pub struct AlexTree {
    root: AlexNode,
    max_node_size: usize,
    expansion_factor: f64,
}

enum AlexNode {
    Inner(InnerNode),
    Leaf(GappedNode),
}

struct InnerNode {
    model: LinearModel,
    children: Vec<AlexNode>,
}

impl AlexTree {
    pub fn insert(&mut self, key: i64, value: Vec<u8>) -> Result<()> {
        let split = self.insert_recursive(&mut self.root, key, value)?;

        if let Some((new_key, new_node)) = split {
            // Root split - increase tree height
            self.grow_root(new_key, new_node);
        }

        Ok(())
    }

    fn insert_recursive(
        &mut self,
        node: &mut AlexNode,
        key: i64,
        value: Vec<u8>
    ) -> Result<Option<(i64, AlexNode)>> {
        match node {
            AlexNode::Leaf(leaf) => {
                leaf.insert(key, value)?;

                if leaf.density() > MAX_DENSITY {
                    // Split leaf into two
                    let (split_key, new_leaf) = self.split_leaf(leaf)?;
                    Ok(Some((split_key, AlexNode::Leaf(new_leaf))))
                } else {
                    Ok(None)
                }
            }
            AlexNode::Inner(inner) => {
                // Predict child, recursively insert
                let child_idx = inner.model.predict(key).min(inner.children.len() - 1);
                self.insert_recursive(&mut inner.children[child_idx], key, value)
            }
        }
    }

    fn split_leaf(&mut self, leaf: &mut GappedNode) -> Result<(i64, GappedNode)> {
        // Collect all keys
        let mut pairs: Vec<(i64, Vec<u8>)> = leaf.keys.iter()
            .zip(leaf.values.iter())
            .filter_map(|(k, v)| {
                if let (Some(key), Some(value)) = (k, v) {
                    Some((*key, value.clone()))
                } else {
                    None
                }
            })
            .collect();

        pairs.sort_by_key(|(k, _)| *k);

        let split_point = pairs.len() / 2;
        let split_key = pairs[split_point].0;

        // Create two new nodes
        let capacity = (pairs.len() as f64 * (1.0 + self.expansion_factor)) as usize;
        let mut left = GappedNode::new(capacity, self.expansion_factor);
        let mut right = GappedNode::new(capacity, self.expansion_factor);

        // Distribute keys
        for (i, (key, value)) in pairs.into_iter().enumerate() {
            if i < split_point {
                left.insert(key, value)?;
            } else {
                right.insert(key, value)?;
            }
        }

        // Train models
        left.retrain()?;
        right.retrain()?;

        *leaf = left;
        Ok((split_key, right))
    }
}
```

### Phase 3: Integration with RedbStorage (Day 2-3)

```rust
// File: src/redb_storage.rs (modified)
pub struct RedbStorage {
    db: Database,
    alex_tree: AlexTree,  // Replace RecursiveModelIndex
    row_count: u64,
    cached_read_txn: Option<ReadTransaction>,
    value_cache: LruCache<i64, Vec<u8>>,
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

        // Insert into ALEX tree (no rebuild, just local inserts!)
        for (key, value) in entries {
            self.alex_tree.insert(key, value)?;
        }

        // No index_dirty flag needed - ALEX handles incrementally!
        self.cached_read_txn = None;
        self.value_cache.clear();

        Ok(())
    }

    pub fn point_query(&mut self, key: i64) -> Result<Option<Vec<u8>>> {
        // Check cache
        if let Some(value) = self.value_cache.get(&key) {
            return Ok(Some(value.clone()));
        }

        // ALEX tree lookup (no rebuild check needed!)
        if let Some(predicted_pos) = self.alex_tree.search(key) {
            // Get value from redb
            let read_txn = self.get_read_txn()?;
            let table = read_txn.open_table(DATA_TABLE)?;

            if let Some(value_guard) = table.get(key)? {
                let value = value_guard.value().to_vec();
                self.value_cache.put(key, value.clone());
                return Ok(Some(value));
            }
        }

        Ok(None)
    }
}
```

---

## Expected Performance Improvements

### Current (RMI)
```
10M inserts: 12.5s
10M queries (avg): 39μs
  - First query: 35ms rebuild
  - Queries 2-1000: ~3μs
  - Amortized: 35ms / 1000 = 35μs overhead
```

### With ALEX
```
10M inserts: ~10s (20% faster, no rebuild overhead)
10M queries (avg): 3-4μs
  - No rebuild spikes!
  - All queries: 3-4μs (cached transaction + ALEX lookup)
  - Node splits: ~1ms every 10K inserts (amortized ~0.1μs)
```

**Improvements**:
- **10x better query latency** (39μs → 3μs)
- **No 35ms spikes** (smooth performance)
- **Scales to 100M+** (no O(n) rebuilds)

---

## Testing Strategy

### Unit Tests
```rust
#[test]
fn test_gapped_node_insert() {
    let mut node = GappedNode::new(100, 1.0);
    node.insert(5, vec![1, 2, 3]).unwrap();
    assert_eq!(node.num_keys, 1);
    assert!(node.density() < 0.1);  // Should have lots of gaps
}

#[test]
fn test_node_split() {
    let mut node = GappedNode::new(10, 0.0);
    // Fill to capacity
    for i in 0..10 {
        node.insert(i, vec![i as u8]).unwrap();
    }
    assert!(node.density() > 0.8);
    // Next insert should trigger split
}
```

### Benchmarks
```rust
// Compare RMI vs ALEX at different scales
benchmark_alex_vs_rmi(1_000_000);
benchmark_alex_vs_rmi(10_000_000);
benchmark_alex_vs_rmi(100_000_000);
```

---

## Migration Plan

### Step 1: Parallel Implementation
- Keep RMI code in `src/index.rs`
- Add ALEX in `src/alex/` directory
- Feature flag: `--features alex`

### Step 2: A/B Testing
- Run both indexes on same data
- Compare:
  - Query latency
  - Memory usage
  - Insert throughput
  - Stability (no crashes)

### Step 3: Cutover
- Once ALEX proves superior:
  - Make ALEX default
  - Archive RMI to `internal/archive/`
  - Update docs

---

## Implementation Schedule

### Day 1 (Today)
- [x] Research ALEX papers ← Done
- [ ] Implement LinearModel
- [ ] Implement GappedNode
- [ ] Implement exponential search
- [ ] Unit tests for gapped arrays
- [ ] Commit: "feat: ALEX core structures (gapped arrays + linear models)"

### Day 2 (Tomorrow)
- [ ] Implement AlexTree
- [ ] Implement node splitting
- [ ] Implement adaptive retraining
- [ ] Integration tests
- [ ] Commit: "feat: ALEX adaptive tree with node splitting"

### Day 3 (Next)
- [ ] Integrate with RedbStorage
- [ ] Remove RMI rebuild logic
- [ ] Benchmark 1M, 10M, 100M
- [ ] Documentation
- [ ] Commit: "feat: replace RMI with ALEX - 10x query improvement"

---

## References

1. **ALEX Paper** (Microsoft Research 2020)
   - https://www.microsoft.com/en-us/research/uploads/prod/2020/04/MSRAlexTechnicalReportV2.pdf
   - Key insight: Gapped arrays + local splits = no global rebuilds

2. **LIPP Paper** (VLDB 2021)
   - "Updatable Learned Index with Precise Positions"
   - Extends tree structure to eliminate deviation

3. **PGM-Index**
   - Alternative approach with different tradeoffs
   - Good for reference

4. **Microsoft ALEX GitHub**
   - https://github.com/microsoft/ALEX
   - C++ implementation (we're doing Rust)

---

## Success Criteria

**Must achieve**:
- ✅ 10M queries: <5μs average (vs 39μs now)
- ✅ No rebuild spikes (smooth latency)
- ✅ Insert throughput: ≥ current (no regression)

**Stretch goals**:
- 100M queries: <10μs average
- Memory: ≤ 2x current (gapped arrays use more space)
- Concurrent: Thread-safe ALEX (future work)

---

## Risk Mitigation

**Risk 1**: ALEX is more complex than RMI
- **Mitigation**: Incremental implementation, frequent testing

**Risk 2**: Memory usage from gapped arrays
- **Mitigation**: Tunable expansion_factor, start conservative (0.5)

**Risk 3**: Implementation bugs
- **Mitigation**: Comprehensive unit tests, A/B testing against RMI

**Risk 4**: Worse performance than expected
- **Mitigation**: Keep RMI code, easy rollback with feature flag

---

## Next Steps

1. Create `src/alex/` directory
2. Implement `linear_model.rs`
3. Implement `gapped_node.rs`
4. Write tests as we go
5. Commit frequently

**Let's begin!**
