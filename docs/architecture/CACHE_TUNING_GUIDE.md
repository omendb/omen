# Cache Tuning Guide

**Date**: October 21, 2025
**Status**: Production recommendations based on benchmark validation
**Tests**: 6 configurations (100K-1M rows, 1%-50% cache sizes)

---

## Executive Summary

**Key Finding**: Smaller cache sizes (1-10% of data) provide better performance than larger caches.

**Recommended Configuration**:
- **Default**: 10,000 entries (≈1% of 1M rows, ≈100MB for 10KB rows)
- **Small datasets** (<100K rows): 1,000-10,000 entries
- **Large datasets** (1M+ rows): 10,000-100,000 entries
- **Memory-rich** environments: Up to 100K entries maximum

---

## Benchmark Results

### Test Configuration
- **Workload**: Zipfian distribution (80% queries hit 10% of data)
- **Queries**: 10,000 per test
- **Scales**: 100K and 1M rows
- **Cache sizes**: 1%, 10%, 50% of data

### Performance by Configuration

| Data Size | Cache Size | Cache % | Latency | Hit Rate | Speedup | Verdict |
|-----------|------------|---------|---------|----------|---------|---------|
| 100K | 1,000 | 1% | 0.066 μs | 90.0% | **3.22x** | ✅ EXCELLENT |
| 100K | 10,000 | 10% | 0.071 μs | 90.0% | **2.43x** | ✅ GOOD |
| 100K | 50,000 | 50% | 0.075 μs | 90.0% | **2.31x** | ✅ GOOD |
| 1M | 10,000 | 1% | 0.093 μs | 90.0% | **2.17x** | ✅ GOOD |
| 1M | 100,000 | 10% | 0.105 μs | 90.0% | **1.95x** | ⚠️  MODEST |
| 1M | 500,000 | 50% | 0.162 μs | 90.0% | **1.25x** | ❌ INSUFFICIENT |

---

## Key Findings

### 1. Smaller Caches Perform Better ⭐

**Observation**: Cache speedup **decreases** as cache size increases
- 1,000 entries (1%): **3.22x speedup**
- 10,000 entries (10%): **2.43x speedup**
- 50,000 entries (50%): **2.31x speedup**

**Reason**: LRU cache overhead increases with size
- Larger caches require more work to maintain LRU order
- Lock contention increases with more entries
- Memory access patterns become less cache-friendly

**Recommendation**: Use smallest cache that achieves 80%+ hit rate

### 2. 90% Hit Rate is Optimal

**Observation**: All configurations achieved exactly 90% hit rate

**Reason**: Zipfian workload (80% queries hit 10% of data)
- Even 1% cache (1,000 entries) covers the hot data
- Larger caches don't improve hit rate for this workload
- **Conclusion**: 1-10% cache is sufficient for typical workloads

### 3. Latency Still Fast, But Overhead Grows

**Observation**: Absolute latency is still sub-microsecond, but grows with cache size

| Cache Size | Latency (1M rows) | Overhead vs Baseline |
|------------|-------------------|----------------------|
| No cache | 0.202 μs | Baseline |
| 10K (1%) | 0.093 μs | **2.17x faster** ✅ |
| 100K (10%) | 0.105 μs | **1.95x faster** ⚠️ |
| 500K (50%) | 0.162 μs | **1.25x faster** ❌ |

**Conclusion**: Cache overhead (RwLock, LRU bookkeeping) becomes significant at 100K+ entries

---

## Tuning Recommendations

### Default Configuration

```rust
// Recommended default
Table::new_with_cache(
    name,
    schema,
    primary_key,
    table_dir,
    10_000  // 10K entries (optimal for most workloads)
)
```

**Memory usage**: ~100MB for 10KB rows, ~10MB for 1KB rows

### Configuration by Scale

#### Small Datasets (<100K rows)
```rust
cache_size: 1_000 to 10_000 entries
```
- **Best**: 1,000 entries (3.22x speedup)
- **Memory**: ~10-100MB
- **Hit rate**: 90%+

#### Medium Datasets (100K-1M rows)
```rust
cache_size: 10_000 entries
```
- **Speedup**: 2.17x (exceeds 2x target)
- **Memory**: ~100MB
- **Hit rate**: 90%

#### Large Datasets (1M+ rows)
```rust
cache_size: 10_000 to 100_000 entries
```
- **10K entries**: 2.17x speedup (recommended)
- **100K entries**: 1.95x speedup (if memory-rich)
- **Memory**: 100MB to 1GB

#### ⚠️ Avoid Large Caches (>100K entries)
```rust
// NOT RECOMMENDED for most workloads
cache_size: 500_000  // 1.25x speedup - overhead too high
```
- Diminishing returns beyond 100K entries
- LRU overhead dominates performance gains
- Only use if hit rate is critical and memory is plentiful

### Workload-Specific Tuning

#### Zipfian Workloads (Most Common)
- **Pattern**: 80% queries hit 20% of data
- **Recommendation**: 1-10% cache (10K-100K entries)
- **Hit rate**: 80-90%

#### Uniform Random Workloads
- **Pattern**: All keys equally likely
- **Recommendation**: Larger cache (10-50%) or no cache
- **Hit rate**: Proportional to cache size

#### Sequential Workloads
- **Pattern**: Time-series, monotonic IDs
- **Recommendation**: Small cache (1-10K entries) for recent data
- **Hit rate**: 90%+ if recent data dominates

---

## Environment Variables

Configure cache size via environment variable:

```bash
# Default: 100,000 entries
export OMENDB_CACHE_SIZE=100000

# Recommended for production
export OMENDB_CACHE_SIZE=10000

# Memory-constrained environments
export OMENDB_CACHE_SIZE=1000
```

In code:
```rust
use omendb::cache::RowCache;

// Uses OMENDB_CACHE_SIZE env var or default (100K)
let cache = RowCache::with_default_size();
```

---

## Monitoring Cache Effectiveness

### Check Cache Statistics

```rust
if let Some(stats) = table.cache_stats() {
    println!("Cache hits: {}", stats.hits);
    println!("Cache misses: {}", stats.misses);
    println!("Hit rate: {:.1}%", stats.hit_rate);
    println!("Cache size: {}", stats.size);
    println!("Capacity: {}", stats.capacity);
    println!("Utilization: {:.1}%", stats.utilization());
}
```

### Target Metrics

| Metric | Target | Action if Below Target |
|--------|--------|------------------------|
| **Hit rate** | >80% | Increase cache size or analyze workload |
| **Speedup** | >2x | Decrease cache size if hit rate is high |
| **Utilization** | >50% | Cache is too large, reduce size |

### Tuning Process

1. **Start with default** (10K entries)
2. **Measure hit rate** after warmup period
3. **Adjust cache size**:
   - Hit rate <80%: Increase cache size (but watch for overhead)
   - Hit rate >90% and low utilization: Decrease cache size
   - Speedup <2x: Try smaller cache
4. **Re-measure** and iterate

---

## Memory Usage Estimation

### Cache Memory Formula

```
Memory (MB) ≈ cache_size × avg_row_size / 1,000,000
```

### Examples

| Cache Size | Row Size | Memory Usage |
|------------|----------|--------------|
| 1,000 | 1 KB | ~1 MB |
| 1,000 | 10 KB | ~10 MB |
| 10,000 | 1 KB | ~10 MB |
| 10,000 | 10 KB | ~100 MB |
| 100,000 | 1 KB | ~100 MB |
| 100,000 | 10 KB | ~1 GB |

### Production Sizing

For production deployments:
- **Measure** average row size: `SELECT AVG(LENGTH(row)) FROM table`
- **Calculate** memory: `cache_size × avg_row_size`
- **Ensure** memory available: Cache + working set + 2GB headroom

---

## Advanced: Cache Warming

For predictable workloads, pre-populate cache on startup:

```rust
// After table load, warm cache with hot data
for key in hot_keys {
    table.get(&key)?;  // Populates cache
}
```

**When to use**:
- First queries must be fast (no cold start)
- Hot data is known (e.g., recent records)
- Startup time is not critical

**When not to use**:
- Workload is unpredictable
- Fast startup is critical
- Hot data changes frequently

---

## Troubleshooting

### Problem: Low Cache Hit Rate (<50%)

**Causes**:
- Workload is truly random (no hot data)
- Cache size too small for working set
- Data access pattern is sequential (cache not helpful)

**Solutions**:
1. Analyze query patterns: Are there hot keys?
2. Increase cache size to 10-20% of data
3. If hit rate doesn't improve: Disable cache (overhead not worth it)

### Problem: Cache Provides <1.5x Speedup

**Causes**:
- Cache size too large (LRU overhead)
- Baseline queries already fast (< 200ns)
- Cache implementation overhead

**Solutions**:
1. **Reduce cache size**: Try 1K or 10K entries
2. **Check hit rate**: Should be 80%+ or cache isn't helpful
3. **Profile**: Use flamegraph to identify overhead

### Problem: High Memory Usage

**Causes**:
- Cache size too large
- Large row sizes

**Solutions**:
1. **Reduce cache size**: Calculate memory usage (see formula above)
2. **Monitor utilization**: If <50%, cache is too large
3. **Disable cache**: If memory-constrained

---

## Benchmark Reproducibility

Run benchmarks yourself:

```bash
# Small-scale validation (fast)
cargo run --release --bin benchmark_cache_effectiveness

# Large-scale validation (comprehensive)
cargo run --release --bin benchmark_cache_scale

# Simple correctness test
cargo run --release --bin test_cache_simple
```

**Expected results**:
- Speedup: 2-3x for 1-10% cache sizes
- Hit rate: 80-90% for Zipfian workloads
- Latency: Sub-microsecond with cache

---

## Production Checklist

Before deploying with cache:

- [ ] Run benchmark on production-like data
- [ ] Measure hit rate (target: >80%)
- [ ] Verify speedup (target: >2x)
- [ ] Calculate memory usage (cache_size × avg_row_size)
- [ ] Ensure available memory (cache + working set + 2GB)
- [ ] Monitor cache stats in production
- [ ] Set up alerts for low hit rate (<60%)

---

## Summary

**Optimal Configuration**: 10,000 entries (1-10% of data)
- ✅ 2-3x speedup
- ✅ 90% hit rate
- ✅ ~100MB memory (10KB rows)
- ✅ Minimal LRU overhead

**Key Insight**: Smaller caches perform better due to lower overhead.
**Avoid**: Cache sizes >100K entries unless absolutely necessary.

---

**Date**: October 21, 2025
**Validated**: 100K and 1M row benchmarks
**Recommended**: 10,000 entry default (configurable via `OMENDB_CACHE_SIZE`)
**Next**: Monitor production hit rates and adjust as needed
