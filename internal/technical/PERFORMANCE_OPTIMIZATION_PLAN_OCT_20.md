# Performance Optimization Plan - October 20, 2025

**Goal**: Achieve 2x speedup at 10M scale (currently 1.44x from Oct 14)
**Bottleneck**: RocksDB (77% of query latency)
**Strategy**: Incremental optimization with validation at each step

---

## Current Status (October 14, 2025 Baseline)

### Performance Metrics

| Scale | Speedup (Sequential) | Speedup (Random) | Status |
|-------|---------------------|------------------|--------|
| 10K   | 3.54x ✅            | 3.24x ✅         | Production-ready |
| 100K  | 3.15x ✅            | 2.69x ✅         | Production-ready |
| 1M    | 2.40x ✅            | 2.40x ✅         | Production-ready |
| 10M   | 1.44x ⚠️            | 1.53x ✅         | **Needs optimization** |

### Query Latency Breakdown (10M scale)

```
Component               Time      Percentage
─────────────────────────────────────────────
ALEX Index Lookup:      571ns     21%  ✅ Efficient
RocksDB Get:           2092ns     77%  ⚠️ BOTTLENECK
Overhead/Other:          58ns      2%  ✅ Negligible
─────────────────────────────────────────────
Total:                 2721ns    100%
```

**Key Finding**: ALEX is efficient (21%). RocksDB is the bottleneck (77%).

### Current Optimizations (Already Applied)

```rust
// src/rocks_storage.rs (lines 58-86)
✅ Bloom filters: 10 bits/key (skip SST files without key)
✅ Block cache: 512MB (cache hot data)
✅ Cache index/filter blocks (reduce read amplification)
✅ Pin L0 index/filter blocks (keep recent data hot)
✅ Block size: 16KB (better compression)
✅ Value cache: 100K entries LRU (in-memory hot data)
```

---

## Optimization Roadmap

### Phase 1: Validate Current Performance (Day 1)

**Objective**: Establish current baseline (may have changed since Oct 14)

**Tasks**:
1. Run `benchmark_vs_sqlite` at 10M scale (both sequential and random)
2. Run `profile_10m_queries` to get detailed breakdown
3. Compare against Oct 14 baseline (1.44x sequential, 1.53x random)

**Expected Outcome**: Know if we've regressed or maintained performance

**Benchmark Commands**:
```bash
# Build optimized binary
cargo build --release

# Sequential benchmark (10M rows)
./target/release/benchmark_vs_sqlite 10000000 sequential

# Random benchmark (10M rows)
./target/release/benchmark_vs_sqlite 10000000 random

# Detailed profiling
./target/release/profile_10m_queries
```

**Success Criteria**:
- Sequential: ≥1.44x speedup
- Random: ≥1.53x speedup
- If regressed, investigate changes since Oct 14

---

### Phase 2: Increase In-Memory Cache (Days 2-3)

**Objective**: Test if larger cache gets us to 2x speedup for hot workloads

**Current**: 100K entries (~10MB-50MB depending on value size)
**Proposed**: 500K-1M entries (~50MB-500MB)

**Rationale**: Many production workloads have locality (80/20 rule - 80% of queries hit 20% of data)

#### Implementation

**File**: `src/rocks_storage.rs`

**Change 1**: Increase cache size
```rust
// Line 134 - Current
value_cache: LruCache::new(NonZeroUsize::new(100_000).unwrap()),

// Proposed
value_cache: LruCache::new(NonZeroUsize::new(500_000).unwrap()),
```

**Change 2**: Add cache metrics
```rust
// Add to struct
cache_hits: AtomicU64,
cache_misses: AtomicU64,

// In get() method, track hits/misses
if let Some(cached_value) = self.value_cache.get(&key) {
    self.cache_hits.fetch_add(1, Ordering::Relaxed);
    return Ok(Some(cached_value.clone()));
} else {
    self.cache_misses.fetch_add(1, Ordering::Relaxed);
}

// Add method to get cache stats
pub fn cache_hit_rate(&self) -> f64 {
    let hits = self.cache_hits.load(Ordering::Relaxed);
    let misses = self.cache_misses.load(Ordering::Relaxed);
    if hits + misses == 0 {
        return 0.0;
    }
    hits as f64 / (hits + misses) as f64
}
```

**Testing**:
```bash
# Test with different cache sizes
# 100K (baseline), 250K, 500K, 1M

# For each:
cargo build --release
./target/release/benchmark_vs_sqlite 10000000 sequential
./target/release/benchmark_vs_sqlite 10000000 random

# Check cache hit rate (add to profile_10m_queries.rs)
```

**Expected Outcome**:
- Cache hit rate: 60-80% for realistic workloads
- Speedup: 1.8-2.2x (if hit rate is high)
- Memory usage: 50-500MB additional

**Success Criteria**:
- ≥2x speedup at 10M scale OR
- Cache hit rate ≥70% with clear path to 2x

---

### Phase 3: RocksDB Compaction Tuning (Days 4-5)

**Objective**: Reduce RocksDB read amplification through compaction tuning

**Current**: Default level-style compaction

**Options to Test**:

#### Option 3A: Universal Compaction

**Rationale**: Better for read-heavy workloads, less write amplification

```rust
// src/rocks_storage.rs - Add after line 61
opts.set_compaction_style(rocksdb::DBCompactionStyle::Universal);

// Universal compaction options
opts.set_max_background_jobs(4);  // Parallel compaction
opts.set_target_file_size_base(256 * 1024 * 1024);  // 256MB files
```

**Expected**: 10-20% read latency reduction

#### Option 3B: Increase max_open_files

**Rationale**: Keep more SST files open, reduce open/close overhead

```rust
// Current: Default (1000)
opts.set_max_open_files(10000);  // Or -1 for unlimited
```

**Expected**: 5-10% read latency reduction

#### Option 3C: Direct I/O

**Rationale**: Bypass OS page cache (RocksDB has its own cache)

```rust
opts.set_use_direct_reads(true);
opts.set_use_direct_io_for_flush_and_compaction(true);
```

**Expected**: 5-15% improvement on systems with warm OS cache

**Testing Matrix**:
```bash
# Test combinations:
1. Baseline (current config)
2. Universal compaction
3. Universal + max_open_files = 10000
4. Universal + max_open_files + direct I/O
5. Best from above + 1GB block cache

# For each:
./target/release/benchmark_vs_sqlite 10000000 sequential
./target/release/benchmark_vs_sqlite 10000000 random
```

**Success Criteria**:
- ≥1.8x speedup (combined with Phase 2) OR
- Clear path to 2x with final tweaks

---

### Phase 4: Write Optimizations (Days 6-7)

**Objective**: Maintain write performance while optimizing reads

**Current Write Performance**: ~5-10K writes/sec at 10M scale

**Potential Improvements**:

#### Option 4A: Increase Write Buffer Size

```rust
// Current: Default (64MB)
opts.set_write_buffer_size(128 * 1024 * 1024);  // 128MB
opts.set_max_write_buffer_number(4);  // Allow 4 memtables
```

**Expected**: Reduce write stalls, maintain insert throughput

#### Option 4B: Batch Insert Optimization

**Check if batch inserts use WriteBatch efficiently**

```rust
// src/rocks_storage.rs - Line 243
// Already using WriteBatch ✅
let mut batch = WriteBatch::default();
for (key, value) in &entries {
    batch.put(key_bytes, value)?;
}
self.db.write(batch)?;
```

**Testing**:
```bash
# Measure insert throughput
./target/release/benchmark_vs_sqlite 10000000 sequential --inserts-only
```

**Success Criteria**:
- Maintain ≥5K inserts/sec at 10M scale
- No regression from current baseline

---

### Phase 5: Alternative Approaches (If Phases 1-4 Don't Reach 2x)

#### Option 5A: Hybrid Storage (2-3 weeks)

**Concept**: Hot data in custom storage, cold data in RocksDB

```
Architecture:
├── Hot Tier: Memory-mapped file (mmap) for recent 100K keys
├── Warm Tier: RocksDB block cache (512MB-1GB)
└── Cold Tier: RocksDB on disk
```

**Expected**: 3-5x for hot queries, 1.5x for cold queries

**Complexity**: High - need tiering logic, eviction policy

#### Option 5B: Custom Storage Layer (4-6 weeks)

**Concept**: Replace RocksDB entirely with custom ALEX-optimized storage

**References**: See `internal/archive/alexstorage/` for previous work

**Expected**: 3-10x potential speedup (based on ALEX efficiency)

**Complexity**: Very high - need WAL, crash recovery, compaction

**Risk**: High - lots of complexity, likely bugs

---

## Testing & Validation

### Benchmark Suite

**1. Point Query Performance**:
```bash
./target/release/profile_10m_queries
```

**2. Full System Benchmark**:
```bash
./target/release/benchmark_vs_sqlite 10000000 sequential
./target/release/benchmark_vs_sqlite 10000000 random
```

**3. Write Performance**:
```bash
./target/release/benchmark_vs_sqlite 10000000 sequential --inserts-only
```

**4. Mixed Workload (YCSB-style)**:
```bash
./target/release/ycsb_benchmark --workload a --size 10000000
```

### Success Metrics

**Primary Goal**: 2x speedup at 10M scale
- Sequential queries: ≥2.0x vs SQLite
- Random queries: ≥2.0x vs SQLite

**Secondary Goals**:
- Cache hit rate: ≥70% for realistic workloads
- Write throughput: ≥5K inserts/sec (no regression)
- Memory usage: <1GB additional (reasonable for 10M rows)

### Regression Testing

**Before each optimization**:
1. Benchmark current state
2. Record query latency breakdown
3. Check cache hit rate

**After each optimization**:
1. Re-run all benchmarks
2. Compare against baseline
3. Check for regressions in other areas

---

## Implementation Schedule

### Week 1: Quick Wins (Days 1-5)

**Day 1**: Validate current performance
- Run benchmarks, establish baseline
- Compare to Oct 14 results
- Deliverable: Baseline report

**Day 2**: Implement cache size tuning
- Test 250K, 500K, 1M cache sizes
- Measure cache hit rates
- Deliverable: Cache size recommendation

**Day 3**: Add cache metrics
- Implement cache_hits/cache_misses tracking
- Add to benchmark output
- Deliverable: Cache statistics in benchmarks

**Day 4**: RocksDB compaction tuning
- Test universal compaction
- Test max_open_files tuning
- Deliverable: Optimal compaction config

**Day 5**: Validate combined optimizations
- Benchmark with all optimizations
- Check if 2x target reached
- Deliverable: Performance report

### Week 2: Advanced Optimizations (If Needed)

**Day 6-7**: Direct I/O and write optimizations
**Day 8-9**: Profile and eliminate remaining bottlenecks
**Day 10**: Final validation and documentation

---

## Expected Outcomes

### Conservative Estimate

**Phase 2 (Large cache)**: +20% → 1.73x speedup
**Phase 3 (Compaction)**: +15% → 1.99x speedup
**Combined**: **~2.0x speedup at 10M scale** ✅

### Optimistic Estimate

**Phase 2 (Large cache)**: +30% → 1.87x speedup
**Phase 3 (Compaction)**: +20% → 2.24x speedup
**Combined**: **~2.2x speedup at 10M scale** ✅✅

### If Still Short of Target

**Fallback**: Hybrid storage (Option 5A)
- Expected: 2.5-3x speedup
- Timeline: +2-3 weeks
- Complexity: Medium

---

## Risk Mitigation

### Risk 1: Cache Doesn't Help (Low Locality Workload)

**Mitigation**:
- Test with realistic workload patterns (YCSB)
- Implement tiered storage if needed
- RocksDB tuning still helps cold queries

### Risk 2: Compaction Hurts Write Performance

**Mitigation**:
- Benchmark writes before/after
- Tune write buffer size to compensate
- Keep separate configs for read vs write workloads

### Risk 3: Optimizations Conflict

**Mitigation**:
- Test each optimization independently
- Use git branches for each experiment
- Document which combinations work

---

## Deliverables

### Code Changes
- `src/rocks_storage.rs` - RocksDB configuration tuning
- `src/bin/profile_10m_queries.rs` - Enhanced with cache metrics
- New benchmark configurations

### Documentation
- Performance report with before/after metrics
- Configuration recommendations for different workloads
- Benchmark methodology documentation

### Metrics
- Query latency breakdown (ALEX vs RocksDB vs overhead)
- Cache hit rates
- Write throughput validation
- Memory usage analysis

---

## Next Action

**Start with Phase 1: Validate Current Performance**

Run benchmarks to establish current baseline:
```bash
cargo build --release
./target/release/benchmark_vs_sqlite 10000000 sequential
./target/release/benchmark_vs_sqlite 10000000 random
./target/release/profile_10m_queries
```

This will tell us:
1. Have we regressed since Oct 14?
2. What's the current query latency breakdown?
3. Where should we focus optimization efforts?

**Estimated time**: 30-60 minutes for benchmark runs
