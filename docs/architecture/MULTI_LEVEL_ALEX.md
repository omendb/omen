# Multi-Level ALEX Design Document

**Goal**: Fix performance degradation at 50M+ rows by implementing a hierarchical tree structure
**Target**: Maintain 2x+ speedup vs SQLite at 100M+ rows
**Timeline**: 2-4 weeks

---

## Problem Analysis

### Current Single-Level Architecture (Bottleneck)

```rust
pub struct AlexTree {
    leaves: Vec<GappedNode>,    // Direct array of leaf nodes
    split_keys: Vec<i64>,       // Keys for binary search routing
}
```

**At 50M scale:**
- 2.8M leaves = 2.8M split_keys
- split_keys array = 22MB (exceeds L3 cache)
- Binary search = log₂(2.8M) = 21 comparisons
- Each comparison = cache miss = 100ns
- Total overhead = 2.1μs just for routing

### Solution: Multi-Level Architecture

```rust
pub struct MultiLevelAlexTree {
    root: InnerNode,                    // Root of inner node tree
    leaves: Vec<GappedNode>,           // Leaf nodes (unchanged)
    height: usize,                      // Tree height (typically 2-3)
}

pub struct InnerNode {
    model: LinearModel,                 // Learned model for child routing
    children: InnerNodeChildren,        // Either inner nodes or leaf indices
    split_keys: Vec<i64>,              // Boundaries between children
    num_keys: usize,                    // Total keys in subtree
}

pub enum InnerNodeChildren {
    Inner(Vec<Box<InnerNode>>),        // Inner node children
    Leaves(Vec<usize>),                // Leaf node indices
}
```

---

## Design Decisions

### 1. Node Fanout
**Decision**: 256-1024 children per inner node
**Rationale**:
- 256 children = 2KB split_keys (fits in L1 cache)
- 1024 children = 8KB split_keys (fits in L2 cache)
- Adaptive based on level and data distribution

### 2. Tree Height
**Decision**: Adaptive, typically 2-3 levels
**Rationale**:
- 2 levels: Up to 1M leaves (256 × 4096)
- 3 levels: Up to 256M leaves (256 × 256 × 4096)
- Most workloads fit in 2 levels

### 3. Model Training
**Decision**: Train models during bulk build, retrain on splits
**Rationale**:
- Bulk build: Sort data, partition, train models recursively
- Incremental: Retrain only affected path on splits
- Maintains model accuracy while minimizing overhead

### 4. Split Strategy
**Decision**: Split when node exceeds fanout threshold
**Rationale**:
- Inner nodes: Split at median when > MAX_FANOUT
- Propagate splits up the tree as needed
- Rebalancing only on significant skew

---

## Implementation Plan

### Phase 1: Inner Node Structure (2 days)

```rust
impl InnerNode {
    /// Create new inner node with model and children
    pub fn new(level: usize, keys: &[(i64, usize)]) -> Self {
        // Train linear model on key distribution
        let model = LinearModel::train(keys);

        // Determine fanout based on level and key count
        let fanout = Self::calculate_fanout(level, keys.len());

        // Partition keys into child ranges
        let partitions = Self::partition_keys(keys, fanout);

        // Build child nodes recursively
        let children = if level > 1 {
            Self::build_inner_children(level - 1, partitions)
        } else {
            Self::build_leaf_children(partitions)
        };

        // Extract split keys
        let split_keys = Self::extract_split_keys(&partitions);

        Self {
            model,
            children,
            split_keys,
            num_keys: keys.len(),
        }
    }

    /// Route query to appropriate child
    pub fn route(&self, key: i64) -> usize {
        // Use model for initial prediction
        let predicted = self.model.predict(key);
        let predicted_child = predicted.min(self.children.len() - 1);

        // Binary search correction if needed
        if self.needs_correction(key, predicted_child) {
            self.binary_search_child(key)
        } else {
            predicted_child
        }
    }
}
```

### Phase 2: Tree Building (2 days)

```rust
impl MultiLevelAlexTree {
    /// Build tree from sorted data
    pub fn bulk_build(data: Vec<(i64, Vec<u8>)>) -> Result<Self> {
        // Determine tree height based on data size
        let height = Self::calculate_height(data.len());

        // Build leaf nodes first (bottom-up)
        let (leaves, leaf_keys) = Self::build_leaves(data)?;

        // Build inner node tree
        let root = if height > 1 {
            InnerNode::new(height - 1, &leaf_keys)
        } else {
            // Single level - create simple root
            Self::create_simple_root(&leaves)
        };

        Ok(Self {
            root,
            leaves,
            height,
        })
    }
}
```

### Phase 3: Query Operations (1 day)

```rust
impl MultiLevelAlexTree {
    /// Find key using hierarchical traversal
    pub fn get(&self, key: i64) -> Result<Option<Vec<u8>>> {
        // Traverse inner nodes
        let leaf_idx = self.traverse_to_leaf(key)?;

        // Query leaf node
        self.leaves[leaf_idx].get(key)
    }

    /// Traverse from root to leaf
    fn traverse_to_leaf(&self, key: i64) -> Result<usize> {
        let mut current = &self.root;

        // Traverse inner levels
        for level in (1..self.height).rev() {
            let child_idx = current.route(key);

            match &current.children {
                InnerNodeChildren::Inner(children) => {
                    current = &children[child_idx];
                }
                InnerNodeChildren::Leaves(indices) => {
                    return Ok(indices[child_idx]);
                }
            }
        }

        // Final routing to leaf
        match &current.children {
            InnerNodeChildren::Leaves(indices) => {
                let child_idx = current.route(key);
                Ok(indices[child_idx])
            }
            _ => unreachable!("Bottom level must have leaf children"),
        }
    }
}
```

### Phase 4: Insert & Split (2 days)

```rust
impl MultiLevelAlexTree {
    /// Insert with potential cascading splits
    pub fn insert(&mut self, key: i64, value: Vec<u8>) -> Result<()> {
        // Find path to leaf
        let path = self.find_path(key)?;
        let leaf_idx = *path.last().unwrap();

        // Try insert into leaf
        if !self.leaves[leaf_idx].insert(key, value)? {
            // Leaf needs split
            self.split_leaf(leaf_idx, &path)?;
        }

        Ok(())
    }

    /// Split leaf and propagate up tree
    fn split_leaf(&mut self, leaf_idx: usize, path: &[usize]) -> Result<()> {
        // Split the leaf
        let (split_key, new_leaf) = self.leaves[leaf_idx].split()?;
        let new_leaf_idx = self.leaves.len();
        self.leaves.push(new_leaf);

        // Propagate split up the tree
        self.propagate_split(path, split_key, new_leaf_idx)?;

        Ok(())
    }
}
```

---

## Performance Projections

### Query Performance

**Current (Single-Level)**:
- 50M scale: 17μs (2.8M leaves)
- Bottleneck: 21 binary search comparisons on 22MB array

**Multi-Level Projection**:
- Level 1: Route through 256 inner nodes (2KB, L1 cache) = 0.5μs
- Level 2: Route through 256 children (2KB, L1 cache) = 0.5μs
- Leaf: Exponential search + scan = 1μs
- **Total: ~2μs** (8.5x improvement!)

### Memory Layout

**Current**:
- 2.8M leaves × 8 bytes = 22MB split_keys

**Multi-Level**:
- Level 1: 256 nodes × 8 bytes = 2KB
- Level 2: 256 × 256 nodes × 8 bytes = 512KB
- Total inner nodes: ~514KB (fits in L3!)

### Scalability

| Scale | Single-Level | Multi-Level | Speedup |
|-------|--------------|-------------|---------|
| 10M | 555K leaves, 4.4MB | 2-level, 100KB inner | Same |
| 50M | 2.8M leaves, 22MB | 2-level, 514KB inner | **8x** |
| 100M | 5.6M leaves, 44MB | 2-level, 1MB inner | **10x** |
| 1B | 56M leaves, 440MB | 3-level, 10MB inner | **20x** |

---

## Testing Plan

### Correctness Tests
1. Build tree with 1K, 10K, 100K, 1M keys
2. Verify all keys findable
3. Test insert with splits
4. Verify tree structure invariants

### Performance Tests
1. Benchmark at 10M (should maintain current performance)
2. Benchmark at 50M (target: 2x faster queries)
3. Benchmark at 100M (target: 2x+ vs SQLite)
4. Profile cache misses (should be 90% lower)

### Stress Tests
1. Random insert pattern
2. Sequential insert pattern
3. Skewed insert pattern
4. Mixed read/write workload

---

## Risk Mitigation

### Risk 1: Complex Implementation
**Mitigation**: Start with 2-level fixed height, generalize later

### Risk 2: Rebalancing Overhead
**Mitigation**: Allow imbalance up to 2x, lazy rebalancing

### Risk 3: Model Accuracy
**Mitigation**: Fallback to binary search, retrain on major skew

---

## Success Criteria

1. **Queries at 50M**: <5μs (current: 17μs)
2. **Queries at 100M**: <6μs
3. **Inserts**: Same performance as current
4. **Memory**: Inner nodes fit in L3 cache
5. **Benchmarks**: 2x+ faster than SQLite at 100M

---

## Implementation Timeline

- Day 1-2: Inner node structure
- Day 3-4: Tree building
- Day 5: Query operations
- Day 6-7: Insert & split
- Day 8-9: Testing & benchmarking
- Day 10: Documentation & cleanup

Total: ~2 weeks

---

**Created**: October 2025
**Status**: Design complete, ready for implementation