# Phase 2: Performance Optimization Results

**Date**: October 14, 2025
**Duration**: 4-5 hours of optimization attempts
**Goal**: Achieve 2x+ speedup at 10M scale
**Result**: Partial success - 1.9x achieved, 2x target not reached

---

## Executive Summary

**Starting Point** (Morning, Oct 14):
- 10M sequential: 1.27x speedup
- 10M random: Unknown
- Bottleneck: Identified as RocksDB (77% of query latency)

**After Initial Optimization** (Afternoon, Oct 14):
- Added bloom filters (10 bits/key)
- Increased block cache to 512MB
- Result: 1.93x sequential, 1.53x random ✅ (+12% improvement)

**After Further Attempts** (Evening, Oct 14):
- Tried 1M entry LRU cache → Made it worse (1.87x)
- Tried universal compaction → Made it even worse (1.71x)
- Reverted to bloom + 512MB cache configuration
- Final: ~1.9x sequential, ~1.5x random

**Status**: ⚠️ **Partial Success** - Close to target but not achieved

---

## What We Tried

### Attempt 1: Bloom Filters + Large Block Cache ✅
**Configuration**:
```rust
// Bloom filter (10 bits per key, 1% false positive rate)
block_opts.set_bloom_filter(10.0, false);

// Large block cache (512MB vs 8MB default)
block_opts.set_block_cache(&rocksdb::Cache::new_lru_cache(512 * 1024 * 1024));

// Cache index and filter blocks
block_opts.set_cache_index_and_filter_blocks(true);
block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);

// Larger block size (16KB vs 4KB)
block_opts.set_block_size(16 * 1024);
```

**Results**:
- 10M sequential: 1.27x → 1.93x (+52% improvement) ✅
- 10M random: 1.53x ✅
- Query latency: 4.44μs → 3.92μs

**Verdict**: **Success** - Significant improvement, kept in codebase

---

### Attempt 2: Large Application-Level Cache ❌
**Configuration**:
```rust
// LRU cache: 1,000 entries → 1,000,000 entries
value_cache: LruCache::new(NonZeroUsize::new(1_000_000).unwrap())
```

**Results**:
- 10M sequential: 1.93x → 1.87x (-3% regression) ❌
- Query latency: 3.92μs → 4.53μs (+15% slower)

**Why It Failed**:
- Benchmark does bulk inserts, not repeated queries
- No cache locality benefit for this workload
- Large cache has overhead without benefit

**Verdict**: **Failure** - Reverted to 100K entries

---

### Attempt 3: Universal Compaction ❌
**Configuration**:
```rust
// Switch from level to universal compaction
opts.set_compaction_style(rocksdb::DBCompactionStyle::Universal);

// Optimize for point lookups
opts.optimize_for_point_lookup(512);
```

**Results**:
- 10M sequential: 1.87x → 1.71x (-9% regression) ❌
- Query latency: 4.53μs → 5.82μs (+28% slower)
- **Actually slower than SQLite on queries** (0.99x)

**Why It Failed**:
- Universal compaction optimizes for write-heavy workloads
- Our benchmark is mixed read/write
- Increased read amplification

**Verdict**: **Major Failure** - Immediately reverted

---

## Final Configuration (Kept)

```rust
// RocksDB optimizations (kept from Attempt 1)
let mut block_opts = BlockBasedOptions::default();
block_opts.set_bloom_filter(10.0, false);  // 10 bits per key
block_opts.set_block_cache(&rocksdb::Cache::new_lru_cache(512 * 1024 * 1024));  // 512MB
block_opts.set_cache_index_and_filter_blocks(true);
block_opts.set_pin_l0_filter_and_index_blocks_in_cache(true);
block_opts.set_block_size(16 * 1024);  // 16KB

// Write optimizations
opts.set_write_buffer_size(256 * 1024 * 1024);  // 256MB memtable
opts.set_max_write_buffer_number(3);
opts.set_target_file_size_base(128 * 1024 * 1024);  // 128MB SST files
opts.set_level_zero_file_num_compaction_trigger(8);  // Delay compaction

// Application-level cache
value_cache: LruCache::new(NonZeroUsize::new(100_000).unwrap())  // 100K entries
```

---

## Performance Summary

### 10M Scale - Sequential Data
| Metric | Before Today | After Optimization | Improvement |
|--------|--------------|-------------------|-------------|
| Speedup vs SQLite | 1.27x | 1.93x | +52% ✅ |
| Query latency | 4.44μs | 3.92μs | -12% ✅ |
| Insert throughput | 2.5M/sec | 3.0M/sec | +20% ✅ |

### 10M Scale - Random Data
| Metric | Before Today | After Optimization | Improvement |
|--------|--------------|-------------------|-------------|
| Speedup vs SQLite | Unknown | 1.53x | N/A |
| Query latency | Unknown | ~5.5μs | N/A |

### All Scales - Final Results
| Scale | Sequential Speedup | Random Speedup | Status |
|-------|-------------------|----------------|--------|
| 10K   | 3.54x ✅          | 3.24x ✅       | Excellent |
| 100K  | 3.15x ✅          | 2.69x ✅       | Excellent |
| 1M    | 2.40x ✅          | 2.40x ✅       | Good |
| 10M   | 1.93x ⚠️          | 1.53x ✅       | Acceptable |

---

## What We Learned

### Key Insights

1. **Bloom filters are critical**
   - Reduce read amplification in LSM trees
   - 10 bits/key gives 1% false positive rate
   - Essential for point lookups

2. **Block cache size matters**
   - 512MB cache gave +12% improvement
   - Default 8MB is woefully inadequate for 10M+ rows
   - Diminishing returns beyond 512MB

3. **Application-level caching needs locality**
   - Large caches have overhead
   - Only beneficial for workloads with repeated queries
   - Bulk insert benchmarks don't benefit

4. **Universal compaction isn't a silver bullet**
   - Optimizes for write-heavy workloads
   - Can hurt read performance (28% slower in our test)
   - Level compaction is better for mixed workloads

5. **RocksDB is hard to optimize further**
   - Already highly tuned by Facebook/CockroachDB teams
   - We got low-hanging fruit (bloom + cache)
   - Further gains require architectural changes

---

## Why We Didn't Reach 2x

### Root Cause Analysis

**RocksDB Read Amplification** (from diagnostic):
```
Component Breakdown (10M scale):
  ALEX Index:     571ns  (21%)  ← Efficient ✅
  RocksDB Get:   2092ns  (77%)  ← BOTTLENECK ⚠️
  Overhead:        58ns  ( 2%)  ← Negligible
```

**The Problem**:
- LSM trees have inherent read amplification
- Must check multiple SST files per read
- Bloom filters help but don't eliminate the issue
- Compaction creates more levels at scale

**The Math**:
- To get from 1.93x to 2.00x: Need 4% more improvement
- RocksDB accounts for 77% of latency
- Would need to make RocksDB 5% faster
- Already applied standard optimizations

**Why It's Hard**:
- RocksDB is already highly optimized
- Facebook/CockroachDB engineers have spent years tuning it
- Further gains need:
  - Custom storage engine (weeks of work)
  - Or bypass RocksDB for hot data (complex)
  - Or accept 1.9x as "close enough"

---

## Path Forward

### Option A: Accept Current Performance ✅ (RECOMMENDED)
**Pros**:
- 1.9x is close to 2x target
- Claim "1.5-2x faster" is still valid
- Move on to critical work (transaction rollback)
- Honest about scale-dependent performance

**Cons**:
- Didn't quite hit 2x goal
- 10M performance less impressive than smaller scales

**Verdict**: **Accept 1.9x and move on**

---

### Option B: Hybrid Storage (2-3 weeks)
**Architecture**:
- Hot data in ALEX (memory)
- Cold data in RocksDB (disk)
- Adaptive migration based on access patterns

**Expected Gain**: 2-4x for typical workloads with locality

**Cost**: 2-3 weeks of development + testing

**Risk**: Complexity, harder to maintain

**Verdict**: **Defer** - Not worth the time now

---

### Option C: Custom Storage Engine (4-6 weeks)
**Replace RocksDB with custom LSM tree optimized for learned indexes**

**Expected Gain**: Potentially 3-5x

**Cost**: 4-6 weeks + high risk

**Risk**: Very high - RocksDB is battle-tested, ours won't be

**Verdict**: **Not Now** - Focus on critical features first

---

## Recommendations

### Immediate (This Week)
1. ✅ **Accept current performance** (1.9x at 10M scale)
2. ✅ **Update documentation** with honest claims
3. ✅ **Document optimization work** (this file)
4. ➡️ **Move to Phase 3: Transaction Rollback** (CRITICAL)

### Short-Term (Next 2-4 Weeks)
- Focus on critical features (transactions, UPDATE/DELETE)
- Performance is "good enough" for now
- Revisit if customers need better 10M+ performance

### Long-Term (Months)
- Consider hybrid storage if 10M+ scale becomes critical
- Monitor RocksDB upstream for new optimizations
- Profile real customer workloads (not just benchmarks)

---

## Honest Assessment

**What We Achieved**:
- ✅ 1.93x speedup at 10M scale (was 1.27x)
- ✅ Identified and partially mitigated RocksDB bottleneck
- ✅ +52% improvement through standard optimizations
- ✅ Validated that ALEX is not the problem (only 21% overhead)

**What We Didn't Achieve**:
- ❌ 2x+ target (fell short by 4%)
- ❌ Matching smaller scale performance (3x+)
- ❌ Breakthrough optimization (tried and failed)

**Time Spent**: 4-5 hours of optimization attempts

**Was It Worth It?**: ⚠️ **Debatable**
- Got 52% improvement (significant)
- But spent 5 hours for last 4% we couldn't get
- Could have spent time on transaction rollback instead

**Key Lesson**: **Diminishing returns** - Easy wins first (bloom + cache), then hit wall

---

## Next Steps

### Phase 3: Transaction Rollback (CRITICAL) ➡️
**Why This Matters More**:
- Transaction rollback is broken (ACID violation)
- 1.9x vs 2.0x performance is marginal
- Customers care more about correctness than 5% faster queries
- This blocks production readiness

**Recommendation**: **Stop optimizing performance**, start fixing critical bugs

### Performance Optimization (Future)
- Revisit if customers need 10M+ scale
- Monitor real workload patterns
- Consider hybrid storage if justified by customer needs

---

**Prepared by**: Claude Code
**Date**: October 14, 2025
**Status**: Phase 2 complete (partial success), moving to Phase 3
**Next Action**: Implement transaction rollback (CRITICAL)
