# ALEX Integration Plan - October 2025

## Current Architecture Analysis

### TableIndex (src/table_index.rs)

**Current design** (with RMI):
```rust
pub struct TableIndex {
    learned_index: RecursiveModelIndex,          // Predicts position in array
    key_to_position: Vec<(i64, usize)>,          // Sorted array of (key, row_pos)
    needs_retrain: bool,                         // Manual retrain flag
}
```

**Problems**:
1. **Duplicate data**: RMI stores keys internally, key_to_position stores same keys
2. **Manual coordination**: Must call retrain() every 1000 inserts
3. **Search complexity**: RMI.search() → predicted_idx → binary search window → fallback
4. **O(n) retraining**: Rebuilds entire RMI every 1000 inserts

### ConcurrentOmenDB (src/concurrent.rs)

**Current design**:
```rust
pub struct ConcurrentOmenDB {
    index: Arc<RwLock<RecursiveModelIndex>>,    // Thread-safe RMI
    storage: Arc<RwLock<ArrowStorage>>,
    // ...metrics
}
```

**Issue**: RwLock<RMI> not ideal for write-heavy workloads (rebuilds block all readers)

---

## ALEX Integration Strategy

### Phase 1: TableIndex Refactor

**New design** (with ALEX):
```rust
pub struct TableIndex {
    alex: AlexTree,  // Single source of truth for (key → position)
}
```

**Benefits**:
1. **No duplication**: ALEX stores (key, position) directly in gapped nodes
2. **Auto-retraining**: ALEX handles splits/retraining automatically
3. **Simpler search**: alex.get(key) → Option<position>
4. **O(1) inserts**: Gapped arrays enable fast inserts without global rebuilds

**API mapping**:

| TableIndex API | RMI Implementation | ALEX Implementation |
|----------------|-------------------|---------------------|
| `insert(key, pos)` | Append to array + periodic retrain | `alex.insert(key, pos_as_bytes)` |
| `search(key)` | RMI predict → binary search | `alex.get(key)` |
| `range_query(s,e)` | Binary search on sorted array | ALEX range scan (future) |
| `retrain()` | Rebuild RMI O(n) | No-op (ALEX auto-retrains) |

**Note on range_query**: Current implementation uses sorted array for range queries. ALEX doesn't have built-in range scan yet. Options:
1. Implement ALEX range scan (traverse leaves in order)
2. Fall back to collecting all keys, sorting, and scanning (temporary)
3. Maintain auxiliary sorted array just for ranges (hybrid approach)

**Decision for v1**: Implement ALEX range scan by traversing leaves.

### Phase 2: Concurrent Access

**Current**: `Arc<RwLock<RecursiveModelIndex>>`
**New**: `Arc<RwLock<AlexTree>>`

ALEX is better for concurrent writes because:
- Local node splits don't require global lock
- Reads can proceed during local operations
- Future: Lock-free ALEX variant with atomic operations

**v1 approach**: Simple drop-in replacement (RwLock<AlexTree>)
**v2 optimization**: Fine-grained locking per leaf node

### Phase 3: Value Encoding

ALEX stores `Vec<u8>` values, but we need to store `usize` positions.

**Encoding**:
```rust
fn encode_position(pos: usize) -> Vec<u8> {
    pos.to_le_bytes().to_vec()
}

fn decode_position(bytes: &[u8]) -> usize {
    usize::from_le_bytes(bytes.try_into().unwrap())
}
```

---

## Implementation Steps

### Step 1: Add ALEX range scan support

**File**: `src/alex/alex_tree.rs`

```rust
impl AlexTree {
    /// Range query - return all (key, value) in [start_key, end_key]
    pub fn range(&self, start_key: i64, end_key: i64) -> Result<Vec<(i64, Vec<u8>)>> {
        let mut results = Vec::new();

        // Find starting leaf
        let start_leaf = self.find_leaf_index(start_key);

        // Traverse leaves collecting keys in range
        for leaf in &self.leaves[start_leaf..] {
            for (k, v) in leaf.pairs() {
                if k > end_key {
                    return Ok(results);
                }
                if k >= start_key {
                    results.push((k, v));
                }
            }
        }

        Ok(results)
    }
}
```

### Step 2: Refactor TableIndex

**File**: `src/table_index.rs`

```rust
use crate::alex::AlexTree;
use crate::value::Value;
use anyhow::Result;

pub struct TableIndex {
    alex: AlexTree,
}

impl TableIndex {
    pub fn new(_capacity: usize) -> Self {
        Self {
            alex: AlexTree::new(),  // ALEX auto-sizes
        }
    }

    pub fn insert(&mut self, key: &Value, position: usize) -> Result<()> {
        let key_i64 = key.to_i64()?;
        let value = position.to_le_bytes().to_vec();
        self.alex.insert(key_i64, value)
    }

    pub fn search(&self, key: &Value) -> Result<Option<usize>> {
        let key_i64 = key.to_i64()?;

        match self.alex.get(key_i64)? {
            Some(bytes) => {
                let pos = usize::from_le_bytes(bytes.as_slice().try_into()?);
                Ok(Some(pos))
            }
            None => Ok(None),
        }
    }

    pub fn range_query(&self, start: &Value, end: &Value) -> Result<Vec<usize>> {
        let start_i64 = start.to_i64()?;
        let end_i64 = end.to_i64()?;

        let results = self.alex.range(start_i64, end_i64)?;

        results.into_iter()
            .map(|(_, bytes)| {
                usize::from_le_bytes(bytes.as_slice().try_into()
                    .map_err(|e| anyhow::anyhow!("Invalid position encoding: {}", e))?)
            })
            .collect()
    }

    pub fn len(&self) -> usize {
        self.alex.len()
    }

    pub fn is_empty(&self) -> bool {
        self.alex.is_empty()
    }

    pub fn retrain(&mut self) {
        // No-op: ALEX handles retraining automatically
    }
}
```

### Step 3: Update ConcurrentOmenDB

**File**: `src/concurrent.rs`

```rust
use crate::alex::AlexTree;  // Replace RecursiveModelIndex import

pub struct ConcurrentOmenDB {
    index: Arc<RwLock<AlexTree>>,  // Changed from RecursiveModelIndex
    storage: Arc<RwLock<ArrowStorage>>,
    // ...rest unchanged
}

impl ConcurrentOmenDB {
    pub fn new(expected_size: usize) -> Self {
        Self {
            index: Arc::new(RwLock::new(AlexTree::new())),  // ALEX auto-sizes
            // ...rest unchanged
        }
    }

    pub fn insert(&self, timestamp: i64, value: f64, series_id: i64) -> Result<()> {
        // ...storage write unchanged...

        {
            let mut index = self.write_index()?;
            // Changed: ALEX stores (key, position) not just key
            let position = /* get from storage */;
            index.insert(timestamp, position.to_le_bytes().to_vec())?;
        }

        Ok(())
    }

    pub fn search(&self, key: i64) -> Result<Option<usize>> {
        let index = self.read_index()?;

        match index.get(key)? {
            Some(bytes) => {
                let pos = usize::from_le_bytes(bytes.as_slice().try_into()?);
                Ok(Some(pos))
            }
            None => Ok(None),
        }
    }
}
```

### Step 4: Testing Strategy

**Test coverage**:
1. ✅ Unit tests in table_index.rs (already exist, should pass)
2. ✅ Unit tests in concurrent.rs (already exist, should pass)
3. ✅ Integration tests (scale_tests.rs, stress_tests.rs)
4. 🆕 Performance regression tests (compare RMI vs ALEX)

**Test command**:
```bash
cargo test table_index
cargo test concurrent
cargo test --lib
```

---

## Migration Checklist

- [ ] Add ALEX range scan (alex_tree.rs)
- [ ] Test ALEX range scan with 1K keys
- [ ] Refactor TableIndex to use ALEX
- [ ] Run TableIndex unit tests (5 tests)
- [ ] Update ConcurrentOmenDB to use ALEX
- [ ] Run concurrent unit tests (3 tests)
- [ ] Update redb_storage.rs if needed
- [ ] Run full test suite
- [ ] Performance validation (10M scale)
- [ ] Update docs
- [ ] Commit migration

---

## Rollback Plan

If ALEX integration causes issues:
1. Git revert to last RMI commit
2. Keep ALEX code in separate module
3. Add feature flag: `--features alex-index`
4. Parallel testing before full migration

---

## Expected Performance Impact

**Before (RMI)**:
- 1M scale: 4.9μs queries
- 10M scale: 40.5μs queries (8x degradation)
- Rebuild cost: O(n) every 1000 inserts

**After (ALEX)**:
- 1M scale: ~2μs queries (2.4x improvement)
- 10M scale: ~5.5μs queries (7.4x improvement)
- No rebuild spikes

**Memory**:
- RMI: 2 copies of keys (RMI + sorted array)
- ALEX: 1.5x data (50% gaps with expansion_factor=1.0)
- Net: Similar memory usage, better locality

---

**Status**: Ready to implement
**Risk level**: Low (extensive test coverage, proven benchmarks)
**Timeline**: 2-3 hours for full migration + testing
