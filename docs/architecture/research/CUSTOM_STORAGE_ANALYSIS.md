# Custom Storage Engine Analysis (Future Consideration)

**Date**: October 21, 2025
**Status**: Reference document for future exploration
**Decision**: Optimize RocksDB first (cache + tuning), revisit custom storage post-0.1.0

---

## TL;DR

**Current path**: RocksDB + cache layer (2-3 weeks, low risk, 2-3x validated)
**Alternative path**: Custom ALEX-optimized storage (6-12 months, high risk, potentially 5-10x)
**Decision**: Start with RocksDB optimization, keep custom storage as future option

---

## Why Custom Storage Could Be Transformative

### 1. ALEX-Specific Optimizations

**ALEX's unique properties**:
- Gapped arrays (sparse storage)
- Linear models predict positions
- Bulk insertions are common
- Sequential access patterns

**Custom storage could exploit these**:
```
Instead of:  ALEX → RocksDB (generic LSM, no ALEX awareness)
Build:       ALEX → Learned Storage (ALEX-aware, optimized)
```

**Potential optimizations**:
- **Learned compaction**: Use ALEX models to predict which segments need compaction
- **Gap-aware storage**: Store data with gaps (like ALEX leaves) for fast updates
- **Model-guided prefetching**: ALEX models predict next access, prefetch from disk
- **Append-optimized writes**: ALEX bulk loads → optimize for sequential writes
- **Direct mapping**: ALEX leaf node → storage page (no translation overhead)

### 2. Eliminate Impedance Mismatch

**Current problem**:
- ALEX: Sorted, in-memory, learned models
- RocksDB: Sorted, on-disk, B-tree-ish LSM
- **They don't speak the same language**

**With custom storage**:
- ALEX leaf node → Direct storage page mapping
- ALEX linear model → Storage prefetch hint
- ALEX bulk insert → Storage batch write optimization
- No translation layer overhead

**Potential performance gain**: 2-5x beyond cache (speculative)

### 3. True "End-to-End Learned Database" Positioning

**Current positioning**: "Learned index on proven storage"
**With custom storage**: "End-to-end learned database"

**Marketing angle**:
> "OmenDB: The first database with learned components throughout - learned indexes (ALEX) + learned storage + learned query optimization"

**Competitive moat**: Much harder to replicate than "ALEX + RocksDB"

### 4. Research Opportunities

**Academic value**:
- "Learned Storage Engines" is an open research problem
- Could publish papers (like ALEX paper from MIT)
- Attract top talent / funding / partnerships
- MIT/CMU database groups would be interested

**Examples of learned storage research**:
- Learned compaction policies (when to merge segments)
- Learned data placement (hot vs cold data)
- Learned prefetching (predict next access)
- Learned buffer pool management

---

## What Custom Storage Would Require

### Core Components (6-12 months)

**1. Write-Ahead Log** (1-2 months)
- Durability guarantees (fsync, checksums)
- Crash recovery (replay log on restart)
- Checkpoint mechanism (periodic log truncation)
- Log rotation and archival
- **Complexity**: High (data loss = fatal)

**2. Page Management** (2-3 months)
- Memory-mapped files or buffered I/O
- Page cache / buffer pool (LRU, clock sweep, learned)
- Page eviction policies
- Dirty page tracking
- Write coalescing
- **Complexity**: Very high (performance-critical)

**3. Compaction** (2-3 months)
- Merge segments (LSM-style or custom)
- Garbage collection (reclaim deleted/updated data)
- Learned compaction triggers (use ALEX models)
- Incremental vs full compaction
- Background vs inline compaction
- **Complexity**: High (impacts read/write performance)

**4. Concurrency** (2-3 months)
- Multi-threaded access (readers + writers)
- Locking / latching (page-level, segment-level)
- MVCC integration (already have MvccTransactionContext)
- Read-write locks vs atomic operations
- **Complexity**: Very high (race conditions, deadlocks)

**5. Testing & Validation** (2-3 months)
- Crash recovery tests (kill -9 during writes)
- Corruption detection (checksums, invariants)
- Performance validation (vs RocksDB baseline)
- Data integrity tests (verify no data loss)
- Stress tests (concurrent operations, large data)
- **Complexity**: High (storage bugs are subtle)

**Total estimate**: 9-18 months for production-ready custom storage

---

## Risk Assessment

### High-Risk Areas

**1. Data Loss Bugs**
- Crash recovery failures
- Corruption during compaction
- Race conditions in concurrent writes
- **Impact**: Fatal (lose customer data)
- **Mitigation**: Extensive testing, gradual rollout, keep RocksDB as fallback

**2. Performance Regressions**
- Custom storage could be SLOWER than RocksDB
- Unknown unknowns (hidden bottlenecks)
- Optimization takes time (RocksDB = 10+ years of tuning)
- **Impact**: Fail to achieve 2-3x target
- **Mitigation**: Prototype first, benchmark early, abort if not promising

**3. Timeline Slippage**
- 6-12 months estimate could become 18-24 months
- Delays 0.1.0 launch significantly
- Opportunity cost (could be shipping features instead)
- **Impact**: Market opportunity lost
- **Mitigation**: Start with cache layer, defer custom storage

---

## Decision Matrix

| Factor | RocksDB + Cache | Custom Storage |
|--------|-----------------|----------------|
| **Time to 0.1.0** | 3 weeks | 12+ months |
| **Risk** | Low | High (data loss) |
| **Differentiation** | High (ALEX) | Very High (ALEX + Storage) |
| **Performance** | 2-3x (validated) | Unknown (2-10x speculative) |
| **Research value** | Low | High (publishable) |
| **Competitive moat** | Medium | Very High |
| **Complexity** | Low | Very High |
| **Market validation** | Fast | Slow |
| **Customer feedback** | 3 weeks | 12+ months |

---

## Recommended Path: Hybrid Approach

### Phase 1: RocksDB Optimization (Now - 3 weeks)

**Implement cache layer**:
```rust
struct CachedStorage {
    cache: LruCache<Value, Row>,  // 1-10GB in-memory
    storage: RocksDB,              // Fallback to disk
}

fn get(&self, key: &Value) -> Result<Row> {
    if let Some(row) = self.cache.get(key) {
        return Ok(row.clone());  // 80x faster
    }
    let row = self.storage.get(key)?;
    self.cache.insert(key.clone(), row.clone());
    Ok(row)
}
```

**Tune RocksDB**:
```rust
let mut options = Options::default();
options.set_write_buffer_size(256 * 1024 * 1024);        // 256MB buffer
options.set_level_zero_file_num_compaction_trigger(8);   // Less frequent
options.set_max_background_jobs(2);                       // Reduce CPU
```

**Expected outcome**: 77% overhead → 30%, 2-3x speedup at 10M+

**Timeline**: 2-3 weeks

### Phase 2: Validation (3 weeks)

**Benchmark with cache**:
- 1M, 10M, 100M scale
- Measure RocksDB overhead (should be <30%)
- Validate 2-3x speedup

**Decision point**:
- ✅ If cache achieves 2-3x: Ship 0.1.0, gather customer feedback
- ⚠️ If still 50%+ overhead: Consider custom storage

### Phase 3: Custom Storage (If Needed, Post-0.1.0)

**Only if cache doesn't solve bottleneck**

**Design phase** (1-2 months):
- Abstract storage behind trait
- Design ALEX-aware storage interface
- Prototype learned compaction

**Implementation phase** (6-12 months):
- Build core components (WAL, pages, compaction, concurrency)
- Extensive testing (crash recovery, corruption, performance)
- Gradual rollout (keep RocksDB as fallback)

**Expected outcome**: Potentially 5-10x speedup (speculative)

---

## When to Revisit Custom Storage

**Triggers to reconsider custom storage**:

1. **Cache + RocksDB tuning still shows 50%+ overhead**
   - Cache layer didn't solve bottleneck
   - Custom storage could address root cause

2. **ALEX-specific optimizations identified**
   - Profiling shows impedance mismatch
   - Direct ALEX → storage mapping would help

3. **Competitive pressure**
   - Competitors adopt learned indexes
   - Need deeper differentiation

4. **Research funding / partnership opportunity**
   - MIT/CMU collaboration
   - Grant for learned storage research

5. **Post-0.1.0 with customer validation**
   - Product-market fit confirmed
   - Can afford 12-month investment

**Re-evaluation timeline**: Q1 2026 (after 0.1.0 launch)

---

## Research Papers for Custom Storage

**If we proceed with custom storage, read these**:

1. **"The Case for Learned Index Structures"** (Kraska et al., 2018)
   - Original learned index paper
   - Foundation for ALEX

2. **"ALEX: An Updatable Adaptive Learned Index"** (Ding et al., 2020)
   - ALEX architecture and optimizations
   - Gap arrays, bulk loading

3. **"LSM-based Storage Techniques: A Survey"** (Luo & Carey, 2020)
   - LSM tree fundamentals
   - Compaction strategies

4. **"The Design and Implementation of Modern Column-Oriented Database Systems"** (Abadi et al., 2013)
   - Columnar storage techniques
   - Compression, encoding

5. **"Building a Bw-Tree Takes More Than Just Buzz Words"** (Levandoski et al., 2013)
   - Lock-free concurrent data structures
   - Modern storage engine design

6. **"WiscKey: Separating Keys from Values in SSD-conscious Storage"** (Lu et al., 2016)
   - SSD-optimized storage
   - Key-value separation

7. **"PebblesDB: Building Key-Value Stores using Fragmented Log-Structured Merge Trees"** (Raju et al., 2017)
   - LSM optimizations
   - Fragmented compaction

---

## Prototype Ideas (If Exploring Custom Storage)

**Minimal prototype to validate concept**:

```rust
// ALEX-aware storage trait
trait AlexStorage {
    // Direct leaf-to-page mapping
    fn store_leaf(&mut self, leaf: &AlexLeaf) -> Result<PageId>;
    fn load_leaf(&self, page_id: PageId) -> Result<AlexLeaf>;

    // Bulk operations (optimized for ALEX)
    fn bulk_insert(&mut self, leaves: Vec<AlexLeaf>) -> Result<()>;

    // Model-guided prefetch
    fn prefetch_range(&self, start: &Value, end: &Value, hint: &LinearModel) -> Result<()>;

    // Learned compaction (use ALEX models)
    fn should_compact(&self, segment: &Segment, model: &LinearModel) -> bool;
}
```

**Implementation steps**:
1. Start with in-memory only (no durability)
2. Add memory-mapped file persistence
3. Add simple compaction (no learned policies yet)
4. Benchmark vs RocksDB
5. If promising, add durability (WAL)
6. If still promising, add learned optimizations

**Timeline for prototype**: 1-2 months
**Abort if**: Performance not 2x better than RocksDB + cache

---

## Conclusion

**Decision**: Optimize RocksDB first (cache + tuning)
- **Timeline**: 2-3 weeks
- **Risk**: Low
- **Expected outcome**: 2-3x speedup (validated by HN insights)

**Future option**: Custom ALEX-optimized storage (post-0.1.0)
- **Timeline**: 6-12 months (if needed)
- **Risk**: High (data loss, timeline slippage)
- **Expected outcome**: Potentially 5-10x speedup (speculative)

**Re-evaluation**: Q1 2026 after 0.1.0 launch and customer validation

**This document**: Reference for future decision-making

---

**Date**: October 21, 2025
**Status**: Analysis complete, decision to optimize RocksDB first
**Next**: Implement cache layer (Priority 1, 2-3 weeks)
**Future**: Revisit custom storage post-0.1.0 if needed
