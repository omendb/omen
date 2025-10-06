# Optimization Roadmap: Data-Driven Strategy

**Date:** October 5, 2025
**Based on:** Profiling results from 100K-1M scale testing
**Status:** Ready to execute

---

## What Profiling Revealed

### Critical Finding: RocksDB is 77% of time at production scale

At 1M random keys (UUID workload - our target use case):
```
RocksDB:  1,243ms (77.1%) ← PRIMARY BOTTLENECK
ALEX:     332ms (20.6%)
Overhead: 37ms (2.3%)
Total:    1,612ms
```

**Current performance:** 2.0x faster than SQLite (1,612ms vs 3,260ms)
**Target performance:** 5-8x faster than SQLite (competitive positioning)

### Secondary Finding: ALEX searches are slow

```
Insert: 224 ns/key
Search: 2,257 ns/key (10x slower!)
```

This is an **immediate optimization opportunity** with proven ROI.

---

## Two-Phase Strategy

### Phase 1: Quick Wins (2-4 weeks)

Try proven optimizations with low implementation risk before committing 10+ weeks to custom storage.

**Target:** Get from 2x → 4-5x vs SQLite with RocksDB
**If successful:** Ship it, defer custom storage
**If stuck at 2-3x:** Proceed to Phase 2 (custom storage is proven necessary)

### Phase 2: Custom Storage (10-12 weeks)

Only proceed if Phase 1 doesn't hit 4-5x target.

**Target:** 5-8x vs SQLite via custom AlexStorage
**Confidence:** 80% (profiling proves RocksDB is 77% bottleneck)

---

## Phase 1: Quick Wins (2-4 Weeks)

### Week 1: SIMD Query Optimization

**Current state:**
- Exponential search in ALEX nodes: 2,257 ns/query
- 10x slower than inserts (224 ns)

**Optimization:**
```rust
// Current: Scalar exponential search
fn exponential_search(&self, key: i64) -> usize {
    let mut bound = 1;
    while bound < self.keys.len() && self.keys[bound] < key {
        bound *= 2;
    }
    binary_search(&self.keys[..bound], key)
}

// Optimized: SIMD exponential search
#[cfg(target_arch = "x86_64")]
fn exponential_search_simd(&self, key: i64) -> usize {
    use std::arch::x86_64::*;
    unsafe {
        let key_vec = _mm256_set1_epi64x(key);
        // Process 4 keys at once
        for chunk in self.keys.chunks_exact(4) {
            let keys_vec = _mm256_loadu_si256(chunk.as_ptr() as *const __m256i);
            let cmp = _mm256_cmpgt_epi64(key_vec, keys_vec);
            // ... SIMD comparison logic
        }
    }
}
```

**Expected improvement:** 2.3 μs → <1 μs (2-3x query speedup)
**Confidence:** 70% (SIMD is proven for this workload)
**Time:** 3-5 days

**Validation:**
```rust
// Benchmark before/after
cargo bench --bench simd_search
```

### Week 2-3: RocksDB Tuning

**Current bottleneck:** 1,243ms for 1M random writes (77% of time)

**Optimization 1: Larger memtable**
```rust
// Current
opts.set_write_buffer_size(64 * 1024 * 1024); // 64MB

// Optimized
opts.set_write_buffer_size(256 * 1024 * 1024); // 256MB
// Keeps more writes in memory before flush
// Reduces compaction frequency
```

**Optimization 2: Tune compaction**
```rust
opts.set_level_zero_file_num_compaction_trigger(8); // Default: 4
opts.set_max_bytes_for_level_base(512 * 1024 * 1024); // 512MB
opts.set_target_file_size_base(128 * 1024 * 1024); // 128MB
```

**Optimization 3: Disable compression for in-memory tier**
```rust
// CPU vs I/O tradeoff - we're CPU-bound on writes
opts.set_compression_type(rocksdb::DBCompressionType::None);
// OR use faster algorithm
opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
```

**Expected improvement:** 1,243ms → 800-1,000ms (1.2-1.5x write speedup)
**Confidence:** 60% (known techniques, but random writes are hard)
**Time:** 5-7 days

**Validation:**
```bash
cargo run --release --bin profile_benchmark
# Compare before/after at 1M random
```

### Week 4: Measure and Reassess

**Success criteria:**
- SIMD queries: <1 μs (2-3x improvement)
- RocksDB tuning: <1,000ms for 1M random (1.5x improvement)
- Combined: ~4-5x vs SQLite

**If we hit 4-5x:** SHIP IT
- Document optimizations
- Update benchmarks
- Focus on production hardening
- Defer custom storage

**If stuck at 2-3x:** Custom storage is proven necessary (RocksDB fundamentally wrong for this workload)

---

## Phase 2: Custom Storage (10-12 Weeks)

**Only proceed if Phase 1 doesn't hit 4-5x target.**

### Weeks 5-6: Foundation

**Goal:** Basic mmap storage + integrated ALEX

```rust
pub struct AlexStorage {
    // Memory-mapped storage for values
    mmap: MmapMut,

    // ALEX tree (stores positions in mmap)
    alex: AlexTree<u64>, // u64 = offset in mmap

    // Free space management
    free_list: Vec<(u64, usize)>, // (offset, size)

    // Metadata
    next_offset: u64,
}

impl AlexStorage {
    pub fn insert(&mut self, key: i64, value: &[u8]) -> Result<()> {
        // 1. Allocate space in mmap
        let offset = self.allocate(value.len())?;

        // 2. Write value to mmap (zero-copy)
        self.mmap[offset..offset + value.len()].copy_from_slice(value);

        // 3. Insert offset into ALEX
        self.alex.insert(key, offset)?;

        Ok(())
    }

    pub fn get(&self, key: i64) -> Result<Option<&[u8]>> {
        // 1. Search ALEX for offset (2.3 μs → <1 μs with SIMD)
        let offset = self.alex.get(key)?;

        // 2. Return slice from mmap (zero-copy read)
        Ok(Some(&self.mmap[offset..offset + len]))
    }
}
```

**Expected:** 200-300ms for 1M random (3-5x improvement over RocksDB)
**Confidence:** 70% (eliminates 77% bottleneck)

### Weeks 7-8: Durability

**Goal:** WAL + crash recovery

```rust
pub struct WAL {
    file: File,
    buffer: Vec<u8>,
}

impl WAL {
    pub fn append(&mut self, key: i64, value: &[u8]) -> Result<()> {
        // Format: [checksum:4][key:8][len:4][value:N]
        let checksum = crc32fast::hash(value);
        self.buffer.extend_from_slice(&checksum.to_le_bytes());
        self.buffer.extend_from_slice(&key.to_be_bytes());
        self.buffer.extend_from_slice(&(value.len() as u32).to_le_bytes());
        self.buffer.extend_from_slice(value);

        // Flush when buffer reaches 1MB
        if self.buffer.len() >= 1024 * 1024 {
            self.flush()?;
        }
        Ok(())
    }

    pub fn flush(&mut self) -> Result<()> {
        self.file.write_all(&self.buffer)?;
        self.file.sync_data()?; // Durable
        self.buffer.clear();
        Ok(())
    }
}

impl AlexStorage {
    pub fn recover(&mut self) -> Result<()> {
        let wal = WAL::open(&self.wal_path)?;
        for entry in wal.entries() {
            self.insert(entry.key, &entry.value)?;
        }
        Ok(())
    }
}
```

**Expected:** Adds ~10% overhead (durability cost)
**Still faster than RocksDB:** 220-330ms vs 1,243ms

### Weeks 9-10: Compaction

**Goal:** Reclaim deleted space, defragment

```rust
impl AlexStorage {
    pub fn compact(&mut self) -> Result<()> {
        // 1. Create new mmap file
        let new_mmap = MmapMut::map_anon(self.mmap.len())?;

        // 2. Copy live values (skip deleted)
        let mut new_offset = 0;
        for (key, old_offset) in self.alex.iter() {
            let value = &self.mmap[old_offset..old_offset + len];
            new_mmap[new_offset..new_offset + len].copy_from_slice(value);

            // Update ALEX with new offset
            self.alex.update(key, new_offset)?;
            new_offset += len;
        }

        // 3. Swap mmap files
        std::mem::replace(&mut self.mmap, new_mmap);

        // 4. Clear free list (all space reclaimed)
        self.free_list.clear();

        Ok(())
    }
}
```

**Trigger:** When free space > 30% of file size
**Cost:** O(n) scan, but infrequent (like LSM compaction)

### Weeks 11-12: Optimization & Hardening

**1. SIMD query optimization (if not done in Phase 1)**
- Apply to ALEX searches in AlexStorage
- Expected: 2-3x query speedup

**2. Batch insert optimization**
- Already have batch mode in ALEX
- Ensure AlexStorage leverages it

**3. Concurrency**
```rust
use parking_lot::RwLock;

pub struct AlexStorage {
    inner: Arc<RwLock<AlexStorageInner>>,
}

impl AlexStorage {
    pub fn get(&self, key: i64) -> Result<Option<Vec<u8>>> {
        let storage = self.inner.read();
        storage.get(key) // Read lock (concurrent reads)
    }

    pub fn insert(&self, key: i64, value: &[u8]) -> Result<()> {
        let mut storage = self.inner.write();
        storage.insert(key, value) // Write lock (exclusive)
    }
}
```

**4. Testing**
```bash
# Scale testing
cargo run --release --bin scale_test -- --keys 10000000

# Crash recovery
cargo test test_crash_recovery

# Concurrent access
cargo test test_concurrent_reads_writes
```

---

## Expected Performance Trajectory

| Milestone | Time | Performance vs SQLite | Confidence |
|-----------|------|----------------------|------------|
| **Current** | - | 2.0x | ✅ Measured |
| **SIMD queries** | Week 1 | 2.5-3x | 70% |
| **RocksDB tuning** | Week 3 | 3-4x | 60% |
| **Decision point** | Week 4 | - | - |
| **Custom foundation** | Week 6 | 3-5x | 70% |
| **+ Durability** | Week 8 | 3-5x | 70% |
| **+ Compaction** | Week 10 | 4-6x | 75% |
| **+ Optimization** | Week 12 | **5-8x** | 80% |

**Key decision:** Week 4 - if RocksDB optimizations get us to 4-5x, we can ship and defer custom storage. If stuck at 2-3x, profiling proves custom storage is necessary.

---

## Risk Mitigation

### Risk 1: SIMD doesn't improve queries

**Likelihood:** Low (30%)
**Impact:** Lost 1 week
**Mitigation:** Benchmark early, pivot if no improvement

### Risk 2: RocksDB tuning doesn't help

**Likelihood:** Medium (40%)
**Impact:** Lost 2 weeks, but proves custom storage needed
**Mitigation:** This IS the validation - if RocksDB can't be tuned, we have data to justify custom storage

### Risk 3: Custom storage hits unforeseen issues

**Likelihood:** Medium (30%)
**Impact:** Delays by 2-4 weeks
**Mitigation:**
- Build incrementally (mmap → WAL → compaction)
- Test at each milestone
- Have RocksDB fallback

### Risk 4: Custom storage doesn't achieve 5-8x

**Likelihood:** Low (20%)
**Impact:** Wasted 10-12 weeks
**Mitigation:** Profiling proves RocksDB is 77% bottleneck - eliminating it MUST improve performance. Worst case: 3-4x (still better than RocksDB optimizations)

---

## Success Metrics

### Phase 1 Success (Week 4)
- ✅ Query latency <1 μs (SIMD)
- ✅ Random 1M writes <1,000ms (RocksDB tuning)
- ✅ 4-5x vs SQLite total
- → **Ship it, defer custom storage**

### Phase 1 Failure (Week 4)
- ❌ Still 2-3x vs SQLite
- → **Proceed to custom storage (justified by profiling data)**

### Phase 2 Success (Week 12)
- ✅ Random 1M writes <600ms (custom storage)
- ✅ Query latency <1 μs (SIMD)
- ✅ 5-8x vs SQLite total
- ✅ Crash recovery works
- ✅ Compaction works
- → **Competitive positioning achieved**

---

## Recommendation

**Start with Phase 1 (Quick Wins):**

1. **Week 1:** SIMD query optimization (low risk, proven technique)
2. **Week 2-3:** RocksDB tuning (validates if storage layer is fixable)
3. **Week 4:** Measure and decide

**Rationale:**
- 2-4 weeks vs 10-12 weeks (5x faster to validate)
- If RocksDB can be tuned to 4-5x, we save 8 weeks
- If RocksDB can't be tuned, we have data-driven proof custom storage is necessary
- No wasted work: SIMD queries will apply to custom storage too

**Next action:** Implement SIMD exponential search for ALEX queries.

---

**Last Updated:** October 5, 2025
**Status:** Ready to execute Phase 1
**Next:** SIMD query optimization
