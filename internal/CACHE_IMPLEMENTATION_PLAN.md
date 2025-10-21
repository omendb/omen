# Cache Layer Implementation Plan

**Date**: October 21, 2025
**Priority**: 🔥 PRIORITY 1 (HN insights validated)
**Timeline**: 2-3 weeks
**Goal**: Reduce RocksDB overhead 77% → 30%, achieve 2-3x speedup at 10M+

---

## Executive Summary

**Problem**: RocksDB disk I/O overhead is 77% (Oct 14 profiling)
**Root cause**: 80x in-memory vs disk performance gap (HN validated)
**Solution**: Large LRU cache (1-10GB) before RocksDB
**Expected outcome**: 2-3x speedup at 10M+ scale, RocksDB overhead → 30%

---

## Implementation Phases

### Phase 1: Design & Foundation (Week 1, Days 1-5)

**Day 1-2: Design cache architecture**

**Core design**:
```rust
// src/cache.rs - NEW FILE

use lru::LruCache;
use std::num::NonZeroUsize;
use std::sync::{Arc, RwLock};
use crate::value::Value;
use crate::row::Row;

/// Large LRU cache for hot data (1-10GB configurable)
pub struct RowCache {
    /// LRU cache: key -> row
    cache: Arc<RwLock<LruCache<Value, Row>>>,

    /// Cache statistics
    hits: Arc<std::sync::atomic::AtomicU64>,
    misses: Arc<std::sync::atomic::AtomicU64>,

    /// Configuration
    max_size: usize,  // Max entries (not bytes)
}

impl RowCache {
    pub fn new(max_size: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(
                LruCache::new(NonZeroUsize::new(max_size).unwrap())
            )),
            hits: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            misses: Arc::new(std::sync::atomic::AtomicU64::new(0)),
            max_size,
        }
    }

    /// Get from cache (fast path)
    pub fn get(&self, key: &Value) -> Option<Row> {
        let mut cache = self.cache.write().unwrap();
        if let Some(row) = cache.get(key) {
            self.hits.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Some(row.clone())
        } else {
            self.misses.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            None
        }
    }

    /// Insert into cache
    pub fn insert(&self, key: Value, row: Row) {
        let mut cache = self.cache.write().unwrap();
        cache.put(key, row);
    }

    /// Invalidate on update/delete
    pub fn invalidate(&self, key: &Value) {
        let mut cache = self.cache.write().unwrap();
        cache.pop(key);
    }

    /// Get cache statistics
    pub fn stats(&self) -> CacheStats {
        let hits = self.hits.load(std::sync::atomic::Ordering::Relaxed);
        let misses = self.misses.load(std::sync::atomic::Ordering::Relaxed);
        let total = hits + misses;
        let hit_rate = if total > 0 {
            (hits as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        CacheStats { hits, misses, hit_rate, size: self.max_size }
    }
}

pub struct CacheStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub size: usize,
}
```

**Cache sizing strategy**:
```rust
// Default: 100K entries (estimate ~1GB for 10KB rows)
// Configurable via environment variable or config file
fn default_cache_size() -> usize {
    std::env::var("OMENDB_CACHE_SIZE")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000)  // 100K entries default
}
```

**Dependencies to add**:
```toml
# Cargo.toml
[dependencies]
lru = "0.12"  # LRU cache implementation
```

**Day 3-4: Integrate cache into Table**

**Modify `src/table.rs`**:
```rust
use crate::cache::RowCache;

pub struct Table {
    // ... existing fields ...
    cache: Option<Arc<RowCache>>,  // Optional cache layer
}

impl Table {
    pub fn new_with_cache(/* ... */, cache_size: usize) -> Self {
        Self {
            // ... existing fields ...
            cache: Some(Arc::new(RowCache::new(cache_size))),
        }
    }

    /// Get row with cache (fast path)
    pub fn get(&self, primary_key: &Value) -> Result<Option<Row>> {
        // Check cache first (80x faster than disk)
        if let Some(cache) = &self.cache {
            if let Some(row) = cache.get(primary_key) {
                return Ok(Some(row));  // Cache hit!
            }
        }

        // Cache miss - hit RocksDB
        let row_opt = self.storage.get(primary_key)?;

        // Populate cache for next access
        if let (Some(cache), Some(ref row)) = (&self.cache, &row_opt) {
            cache.insert(primary_key.clone(), row.clone());
        }

        Ok(row_opt)
    }

    /// Update with cache invalidation
    pub fn update(&mut self, primary_key: &Value, updates: HashMap<String, Value>) -> Result<usize> {
        // Invalidate cache on update
        if let Some(cache) = &self.cache {
            cache.invalidate(primary_key);
        }

        // Existing update logic...
        // ...
    }

    /// Delete with cache invalidation
    pub fn delete(&mut self, primary_key: &Value) -> Result<usize> {
        // Invalidate cache on delete
        if let Some(cache) = &self.cache {
            cache.invalidate(primary_key);
        }

        // Existing delete logic...
        // ...
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> Option<CacheStats> {
        self.cache.as_ref().map(|c| c.stats())
    }
}
```

**Day 5: Testing**

**Create `tests/cache_tests.rs`**:
```rust
#[test]
fn test_cache_hit() {
    let cache = RowCache::new(1000);
    let key = Value::Int64(1);
    let row = Row::new(vec![Value::Int64(1), Value::Text("test".to_string())]);

    cache.insert(key.clone(), row.clone());

    let result = cache.get(&key);
    assert!(result.is_some());

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
    assert_eq!(stats.misses, 0);
    assert_eq!(stats.hit_rate, 100.0);
}

#[test]
fn test_cache_miss() {
    let cache = RowCache::new(1000);
    let key = Value::Int64(1);

    let result = cache.get(&key);
    assert!(result.is_none());

    let stats = cache.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.hit_rate, 0.0);
}

#[test]
fn test_cache_lru_eviction() {
    let cache = RowCache::new(2);  // Max 2 entries

    cache.insert(Value::Int64(1), Row::new(vec![Value::Int64(1)]));
    cache.insert(Value::Int64(2), Row::new(vec![Value::Int64(2)]));
    cache.insert(Value::Int64(3), Row::new(vec![Value::Int64(3)]));

    // Entry 1 should be evicted (LRU)
    assert!(cache.get(&Value::Int64(1)).is_none());
    assert!(cache.get(&Value::Int64(2)).is_some());
    assert!(cache.get(&Value::Int64(3)).is_some());
}

#[test]
fn test_table_with_cache() {
    let temp_dir = TempDir::new().unwrap();
    let catalog = Catalog::new(temp_dir.path().to_path_buf()).unwrap();

    // Create table with cache
    let schema = Arc::new(Schema::new(vec![
        Field::new("id", DataType::Int64, false),
        Field::new("name", DataType::Utf8, false),
    ]));

    let mut table = Table::new_with_cache(
        "users".to_string(),
        schema,
        temp_dir.path().to_path_buf(),
        1000,  // 1000 entry cache
    ).unwrap();

    // Insert row
    let row = Row::new(vec![Value::Int64(1), Value::Text("Alice".to_string())]);
    table.insert(row.clone()).unwrap();

    // First get: cache miss (cold)
    let result1 = table.get(&Value::Int64(1)).unwrap();
    assert!(result1.is_some());

    // Second get: cache hit (warm)
    let result2 = table.get(&Value::Int64(1)).unwrap();
    assert!(result2.is_some());

    // Check cache stats
    let stats = table.cache_stats().unwrap();
    assert_eq!(stats.hits, 1);  // Second get was cache hit
    assert_eq!(stats.misses, 1);  // First get was cache miss
}
```

---

### Phase 2: RocksDB Tuning (Week 2, Days 6-10)

**Day 6-7: Tune RocksDB parameters**

**Modify RocksDB options** (where Tables are created):
```rust
use rocksdb::{Options, DB};

fn create_rocksdb_with_tuning(path: &Path) -> Result<DB> {
    let mut options = Options::default();
    options.create_if_missing(true);

    // TUNING: Larger write buffer (reduce flushes)
    options.set_write_buffer_size(256 * 1024 * 1024);  // 256MB (was default ~64MB)

    // TUNING: Reduce compaction frequency
    options.set_level_zero_file_num_compaction_trigger(8);  // 8 files (was 4)

    // TUNING: Reduce background CPU usage
    options.set_max_background_jobs(2);  // 2 threads (was auto)

    // TUNING: Larger block cache (if not using our cache)
    // Note: We have our own cache now, so keep this moderate
    options.set_block_cache_size_mb(128);  // 128MB block cache

    // TUNING: Compression
    options.set_compression_type(rocksdb::DBCompressionType::Lz4);  // Fast compression

    DB::open(&options, path).map_err(|e| anyhow::anyhow!("RocksDB error: {}", e))
}
```

**Day 8-9: Compaction profiling**

**Add compaction metrics**:
```rust
// In benchmarking code, measure compaction overhead
fn profile_with_compaction_disabled() {
    let mut options = Options::default();
    options.set_disable_auto_compactions(true);  // Disable for baseline

    // Run benchmark...
    // Measure: How much faster without compaction?
}

fn profile_with_compaction_enabled() {
    let mut options = Options::default();
    // Default compaction settings

    // Run benchmark...
    // Measure: Overhead of compaction
}
```

**Document compaction impact**:
- Measure insert throughput with/without compaction
- Measure query latency with/without compaction
- Determine if compaction is a bottleneck
- Document trade-offs (storage size vs performance)

**Day 10: Testing and validation**

**Run existing benchmarks with tuning**:
```bash
cargo build --release
./target/release/benchmark_table_vs_sqlite 10000000
```

**Expected results**:
- Cache hit rate: 80-90% (for typical workloads)
- RocksDB overhead: 77% → 40-50% (intermediate milestone)
- Speedup: 1.5x-2x improvement over baseline

---

### Phase 3: Validation & Optimization (Week 3, Days 11-15)

**Day 11-12: Large-scale benchmarking**

**Benchmark suite**:
```bash
# 1M scale
./benchmark_table_vs_sqlite 1000000

# 10M scale
./benchmark_table_vs_sqlite 10000000

# 100M scale (if time permits)
./benchmark_table_vs_sqlite 100000000
```

**Measure**:
1. **Cache hit rate**: Target 80-90%
2. **RocksDB overhead**: Target <30%
3. **Speedup vs SQLite**: Target 2-3x at 10M
4. **Memory usage**: Ensure cache fits in available RAM
5. **Query latency**: p50, p95, p99

**Day 13-14: Optimization**

**Based on profiling, optimize**:

**If cache hit rate < 80%**:
- Increase cache size
- Implement smarter eviction (frequency-based)
- Add cache warming on startup

**If RocksDB overhead still > 30%**:
- Further tune compaction parameters
- Investigate RocksDB write stalls
- Consider bloom filters for faster lookups

**If memory usage too high**:
- Implement cache size limits by bytes (not just entry count)
- Add memory pressure detection
- Implement cache eviction under memory pressure

**Day 15: Documentation & final testing**

**Documentation to create**:
1. `internal/CACHE_IMPLEMENTATION_SUMMARY.md` - What we built
2. Update `ARCHITECTURE.md` with cache layer
3. Update `README.md` performance claims
4. Update `internal/STATUS_REPORT.md` with results

**Final tests**:
- Run full test suite: `cargo test`
- Validate no regressions
- Confirm 456 tests still passing
- Add new cache tests to suite

---

## Success Criteria

**Must have** (required for completion):
- [ ] Cache layer implemented and integrated
- [ ] RocksDB overhead reduced from 77% to <40%
- [ ] Speedup at 10M: 1.5x-2x improvement
- [ ] Cache hit rate: >70%
- [ ] All tests passing (456+)
- [ ] No performance regressions at small scale

**Should have** (target goals):
- [ ] RocksDB overhead <30%
- [ ] Speedup at 10M: 2-3x vs SQLite
- [ ] Cache hit rate: 80-90%
- [ ] Memory usage: Configurable, <2GB default
- [ ] Compaction profiling complete

**Nice to have** (stretch goals):
- [ ] Cache warming on startup
- [ ] Adaptive cache size based on workload
- [ ] Per-table cache statistics
- [ ] Prometheus metrics for cache

---

## Risk Mitigation

**Risk 1: Cache doesn't improve performance**
- Mitigation: Profile early (Day 5), abort if no improvement
- Fallback: Focus on RocksDB tuning only
- Decision point: Day 10

**Risk 2: Memory usage too high**
- Mitigation: Configurable cache size, start small (100K entries)
- Fallback: Reduce default cache size
- Monitor: Memory usage in benchmarks

**Risk 3: Cache invalidation bugs**
- Mitigation: Comprehensive tests, invalidate on all writes
- Fallback: Disable cache for write-heavy workloads
- Validation: Test update/delete invalidation

**Risk 4: Timeline slippage**
- Mitigation: Weekly checkpoints, adjust scope if needed
- Fallback: Ship cache only (defer RocksDB tuning)
- Decision points: Day 5, Day 10, Day 15

---

## Timeline Summary

| Phase | Days | Deliverable | Validation |
|-------|------|-------------|------------|
| **Week 1: Foundation** | 1-5 | Cache implementation + tests | 456+ tests passing |
| **Week 2: Tuning** | 6-10 | RocksDB optimization + profiling | 1.5-2x improvement |
| **Week 3: Validation** | 11-15 | Large-scale benchmarks + docs | 2-3x target achieved |

**Total**: 15 days (3 weeks)

---

## Post-Implementation

**If successful (2-3x achieved)**:
- Ship cache layer in 0.1.0 ✅
- Move to Phase 2 (Security) or Phase 3 Week 3 (SQL)
- Document cache best practices

**If partially successful (1.5-2x achieved)**:
- Ship cache layer ✅
- Investigate remaining bottlenecks
- Consider additional optimizations

**If unsuccessful (<1.5x improvement)**:
- Investigate why cache didn't help
- Profile to find true bottleneck
- Re-evaluate: Is storage the problem? Or something else?
- Consider: Custom storage (see CUSTOM_STORAGE_ANALYSIS.md)

---

## Reference Documents

**Related docs**:
- `internal/research/HN_DATABASE_INSIGHTS_ANALYSIS.md` - Validates cache approach
- `internal/research/CUSTOM_STORAGE_ANALYSIS.md` - Alternative path (future)
- `internal/STATUS_REPORT.md` - Current status
- `ARCHITECTURE.md` - System architecture

**HN insights that validate this approach**:
- "80x in-memory vs disk gap" - Cache addresses this
- "Sparse indices balance memory vs lookup" - ALEX does this
- "LSM trees power DynamoDB" - RocksDB is proven
- "Immutable records eliminate in-place updates" - MVCC does this

---

**Date**: October 21, 2025
**Status**: Plan complete, ready to implement
**Timeline**: 2-3 weeks (15 days)
**Expected outcome**: 2-3x speedup at 10M+, RocksDB overhead <30%
**Next**: Start Day 1 (Design cache architecture)
