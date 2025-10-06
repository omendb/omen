# Performance Audit - October 2, 2025

**Context:** After honest benchmarking, OmenDB shows 2-4x average speedup vs SQLite, below our 10-50x target. This audit identifies optimization opportunities.

---

## Library Choices ✅ (Mostly Good)

### redb (Storage Layer)
**Current:** `redb = "2.1"`
**Assessment:** ✅ **KEEP** - Excellent choice
- Faster individual writes than rocksdb (920ms vs 2432ms)
- Pure Rust, memory-safe
- Stable file format
- **Not the bottleneck** - our usage patterns are the problem

### DataFusion (SQL Engine)
**Current:** `datafusion = "43"`
**Assessment:** ✅ **KEEP** - State of the art
- Apache Arrow-based query engine
- Used in production by InfluxDB, Ballista
- Best-in-class Rust SQL engine

### Arrow/Parquet (Columnar Storage)
**Current:** `arrow = "53"`, `parquet = "53"`
**Assessment:** ✅ **KEEP** - Industry standard

### Learned Index Implementation
**Current:** Custom `RecursiveModelIndex`
**Assessment:** ⚠️ **OPTIMIZE** - No better Rust libraries exist
- ❌ No mature Rust crates for PGM-index, ALEX, or advanced learned indexes
- ✅ RMI is the right algorithm (reference impl is also Rust)
- ⚠️ Our implementation has performance issues (see below)
- **Decision:** Keep custom implementation, but optimize it

---

## Critical Performance Issues 🔥

### 1. Full Index Rebuild on Every Batch Insert (CRITICAL)
**Location:** `src/redb_storage.rs:195`
```rust
pub fn insert_batch(&mut self, entries: Vec<(i64, Vec<u8>)>) -> Result<()> {
    // ... insert entries ...

    // ❌ CRITICAL: Rebuild entire index from scratch!
    self.rebuild_index()?;  // O(n log n) every batch!
    // ...
}
```

**Problem:**
- Every batch insert triggers `rebuild_index()`
- Reads ALL keys from disk
- Sorts entire dataset: O(n log n)
- Trains learned index on entire dataset
- For 1M rows, this happens EVERY batch

**Impact:** This is likely the #1 reason inserts are slow (0.47-2.43x vs SQLite)

**Fix options:**
1. **Incremental index updates** (complexity: medium)
   - Update learned index without full rebuild
   - Requires tracking dirty segments

2. **Deferred rebuild** (complexity: low, HIGH IMPACT)
   - Only rebuild when index is queried, not on every insert
   - Batch inserts become pure redb writes (much faster)
   - Trade-off: First query after insert will be slower

3. **Periodic rebuild** (complexity: low)
   - Rebuild every N inserts (e.g., 10K rows)
   - Use stale index between rebuilds

**Recommendation:** Start with #2 (deferred rebuild) for immediate 10-100x insert speedup

---

### 2. Vec Insert in Middle (O(n) per insert)
**Location:** `src/redb_storage.rs:156`
```rust
pub fn insert(&mut self, key: i64, value: &[u8]) -> Result<()> {
    // ...
    match self.sorted_keys.binary_search(&key) {
        Err(pos) => {
            // ❌ O(n) operation - shifts all elements after pos
            self.sorted_keys.insert(pos, key);
            self.learned_index.add_key(key);
        }
    }
}
```

**Problem:**
- `Vec::insert()` shifts all elements after insertion point
- For 1M keys, average insert is O(500K) operations
- This is why individual inserts are slow

**Impact:** Moderate for single inserts, but batch inserts use `insert_batch()` which rebuilds anyway

**Fix options:**
1. **Use append + sort** for batches (already doing this in rebuild)
2. **Use BTreeSet** for sorted_keys (O(log n) insert, but slower iteration)
3. **Don't maintain sorted_keys incrementally** - rebuild from disk when needed

**Recommendation:** If we fix #1 (deferred rebuild), this becomes less critical

---

### 3. Inefficient Learned Index Retrain
**Location:** `src/index.rs:78`
```rust
pub fn retrain(&mut self) {
    info!("Retraining learned index");
    let mut data = self.data.clone();  // ❌ Clone entire dataset!
    self.train(data);
}
```

**Problem:**
- Clones entire dataset unnecessarily
- `train()` already takes ownership and sorts

**Impact:** Small (only affects add_key() path, not batch path)

**Fix:** Remove clone, restructure to avoid it

---

### 4. Hardcoded Error Bounds
**Location:** `src/redb_storage.rs:225`, `src/index.rs:234`
```rust
let window_size = 100; // ❌ Hardcoded, should use model's max_error

// In index.rs:
max_error = (max_error + 1).min(8).max(1); // ❌ Why cap at 8?
```

**Problem:**
- Not using learned index's actual prediction error
- Window size of 100 is arbitrary
- Capping max_error at 8 is aggressive and might miss keys

**Impact:** Moderate - could cause incorrect lookups or inefficient searches

**Fix:** Use `model.max_error` from learned index, remove arbitrary caps

---

### 5. No SIMD/Vectorization
**Location:** Throughout `src/index.rs`

**Problem:**
- Linear regression uses scalar operations
- No use of SIMD for bulk operations
- Could use `nalgebra` or `ndarray` with SIMD features

**Impact:** Small-moderate (training is fast, but could be faster)

**Fix options:**
1. Use `ndarray` with BLAS backend for linear regression
2. Use `packed_simd` for manual vectorization
3. Wait until SIMD becomes bottleneck

**Recommendation:** Low priority - focus on algorithmic fixes first

---

### 6. Sampling for Error Calculation
**Location:** `src/index.rs:227-234`
```rust
let sample_size = segment.len().min(100);
for (i, (key, _)) in segment.iter().take(sample_size).enumerate() {
    let predicted = (slope * (*key as f64) + intercept).round() as i64;
    let error = (predicted - i as i64).abs() as usize;
    max_error = max_error.max(error);
}
```

**Problem:**
- Only samples first 100 elements
- Might miss worst-case errors in the tail
- Then caps max_error at 8 (line 234)

**Impact:** Could cause lookup failures if actual errors > 8

**Fix:**
- Sample more strategically (e.g., first 50, last 50, middle 50)
- Remove arbitrary cap at 8
- Or calculate error for all elements (still O(n) but correct)

---

## Optimization Priority (High to Low)

### 🔥 P0: Deferred Index Rebuild (MUST DO)
**Impact:** 10-100x insert speedup
**Complexity:** Low
**Time:** 2-4 hours

**Change:**
```rust
pub fn insert_batch(&mut self, entries: Vec<(i64, Vec<u8>)>) -> Result<()> {
    // Insert to redb
    let write_txn = self.db.begin_write()?;
    {
        let mut table = write_txn.open_table(DATA_TABLE)?;
        for (key, value) in &entries {
            table.insert(*key, value.as_slice())?;
        }
    }
    write_txn.commit()?;

    // ✅ Mark index as dirty, don't rebuild immediately
    self.index_dirty = true;
    self.row_count += entries.len() as u64;

    // Save metadata
    self.save_metadata()?;
}

pub fn point_query(&self, key: i64) -> Result<Option<Vec<u8>>> {
    // ✅ Rebuild index if dirty (lazy rebuild)
    if self.index_dirty {
        self.rebuild_index()?;
    }

    // ... rest of query ...
}
```

**Expected results:**
- Insert speedup: 10-100x (no more O(n log n) on every batch)
- Query latency: Unchanged (rebuild happens once, amortized)
- First query after insert: Slower (one-time rebuild cost)

---

### 🔥 P1: Fix Error Bound Calculation
**Impact:** 2-5x query speedup (more accurate predictions)
**Complexity:** Low
**Time:** 1-2 hours

**Changes:**
1. Remove `max_error.min(8)` cap in `src/index.rs:234`
2. Calculate error on representative sample (not just first 100)
3. Use actual `model.max_error` in queries, not hardcoded 100

---

### ⚠️ P2: Remove Unnecessary Clones
**Impact:** Small
**Complexity:** Low
**Time:** 30 minutes

**Changes:**
1. Fix `retrain()` to avoid cloning dataset
2. Review for other unnecessary clones

---

### ⚠️ P3: Batch Operations in redb
**Impact:** 2-5x insert speedup
**Complexity:** Medium
**Time:** 2-4 hours

**Research:** Does redb have bulk loading APIs? Check documentation.

---

### 🔮 P4: SIMD Optimization (Future)
**Impact:** 2-3x training speedup
**Complexity:** High
**Time:** 1-2 days

**When:** After P0-P2 are done and we re-benchmark

---

## Expected Results After P0+P1

**Current (honest benchmark at 1M rows):**
- Sequential insert: 0.88x (slower than SQLite)
- Random insert: 2.43x
- Query: 3.48-4.93x

**Expected after P0 (deferred rebuild):**
- Sequential insert: **5-10x** (no rebuild per batch)
- Random insert: **10-20x** (no rebuild per batch)
- Query: 3.48-4.93x (unchanged, or better with P1)

**Expected after P0+P1 (better error bounds):**
- Sequential insert: **5-10x**
- Random insert: **10-20x**
- Query: **5-10x** (more accurate predictions = smaller search windows)

**Average speedup: 7-13x** (approaching YC target of 10-50x)

---

## Alternative: Consider PGM-Index (C++ FFI)

**Option:** Use PGM-index C++ library via FFI
**Source:** https://github.com/gvinciguerra/PGM-index

**Pros:**
- State-of-the-art algorithm (Piecewise Geometric Model)
- Proven performance: 83x less space than B-tree, same query time
- Supports dynamic updates

**Cons:**
- ❌ C++ FFI adds complexity
- ❌ Not "pure Rust" anymore
- ❌ Deployment complexity (need C++ compiler)
- ❌ Our RMI implementation with fixes should be comparable

**Recommendation:** ❌ **Don't do FFI unless algorithmic improvements fail**
- Prefer pure Rust for deployment simplicity
- Fix our RMI implementation first
- Re-evaluate if we still can't hit 10x+ after optimizations

---

## Algorithm Validation

**Question:** Is RMI the right algorithm?

**Answer:** ✅ Yes, for our use case

**Research findings:**
- RMI is the reference learned index (MIT SOSP 2018)
- ALEX is good for dynamic updates, but more complex
- PGM-index is also good, but C++ only
- Our use case (batch inserts + point queries) fits RMI perfectly

**Validation:**
- Query speedup (3.5-7.5x) proves learned indexes work
- Problem is insert performance, not query performance
- Insert problem is implementation (full rebuild), not algorithm

---

## Summary

**Library choices:** ✅ All good (redb, DataFusion, Arrow)
**Algorithm choice:** ✅ RMI is right, custom impl is reasonable
**Implementation issues:** ❌ Critical bugs in insert path

**Top 2 fixes to implement:**
1. **Deferred index rebuild** (P0) → 10-100x insert speedup
2. **Fix error bounds** (P1) → 2-5x query speedup

**Expected results:**
- Current: 2-4x average
- After fixes: **7-13x average**
- **Potential to hit 10-50x target** ✅

**Time to implement:** 4-8 hours total

**Next steps:**
1. Implement P0 (deferred rebuild)
2. Run honest benchmark again
3. Implement P1 (error bounds) if needed
4. Re-evaluate YC W25 decision

---

## References

- RMI Paper: Kraska et al., "The Case for Learned Index Structures" (SOSP 2018)
- PGM-Index: https://github.com/gvinciguerra/PGM-index
- ALEX: https://github.com/microsoft/ALEX
- redb benchmarks: https://www.redb.org/post/2023/06/16/1-0-stable-release/
- Survey: "A Survey of Learned Indexes for the Multi-dimensional Space" (2024)
