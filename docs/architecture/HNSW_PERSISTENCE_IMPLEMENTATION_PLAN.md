# HNSW Index Persistence Implementation Plan

**Date**: October 23, 2025
**Priority**: CRITICAL (blocks 100K+ scale)
**Timeline**: 2-3 days
**Goal**: 100K vectors: 96-122ms → <10ms queries

---

## Problem Statement

**Current Bottleneck** (discovered Week 5 Day 4):
- 10K vectors: 7-9ms queries ✅
- 100K vectors: 96-122ms queries ❌ (14x degradation)

**Root Cause**: Loading ALL rows from RocksDB (~85-90ms) for SQL predicate evaluation

**Solution**: Persisted HNSW index → Load only top-k*expansion candidates (~300-500 rows)

**Expected Improvement**: 96-122ms → 6-10ms (10-15x faster)

---

## Research Findings

### hnsw_rs v0.3 Serialization (hnswio module)

**Key Features**:
1. **Two-file format**:
   - `*.hnsw.graph` - Graph topology + point IDs (bincode serialization)
   - `*.hnsw.data` - Vectors + IDs (memory-mappable)

2. **API**:
   - `Hnsw::dump()` - Serialize index to files
   - `HnswIo::load_hnsw()` - Deserialize from files
   - `HnswIo::load_description()` - Read metadata without loading full index

3. **Requirements**:
   - Data type T must implement: `Serialize + Deserialize + Clone + Send + Sync`
   - Uses bincode v1.3 for serialization
   - Supports memory-mapped data file for efficient reloading

### Competitor Implementations

**Qdrant**:
- Write-ahead-log (WAL) for durability
- Memmap storage option (disk + page cache)
- On-disk flag for scalability

**Weaviate**:
- In-memory HNSW with WAL persistence
- PERSISTENCE_HNSW_MAX_LOG_SIZE parameter
- Full CRUD support

**Oracle Database**:
- Full checkpoints (serialized graph on disk)
- Same footprint as in-memory graph

**Apache Kvrocks**:
- RocksDB node_id → node_data mapping
- Block cache for in-memory performance
- Performance: 100μs array, 10ms RocksDB (at 2000 nodes/query)

---

## Implementation Options

### Option 1: File-Based Storage (RECOMMENDED ✅)

**Approach**: Use hnsw_rs built-in dump/load + file storage

**Pros**:
- Simple implementation (use hnsw_rs directly)
- Memory-mapped data file = efficient
- Proven format (bincode)
- Fast to implement (1-2 days)

**Cons**:
- Separate file management (not in RocksDB)
- Need to handle file paths, cleanup

**Storage Layout**:
```
data/
  vectors/
    table_{id}_hnsw.graph  # Graph structure
    table_{id}_hnsw.data   # Vector data (mmap)
```

**API**:
```rust
impl VectorStore {
    pub fn save_to_disk(&self, base_path: &str) -> Result<()>
    pub fn load_from_disk(base_path: &str) -> Result<Self>
}
```

### Option 2: RocksDB Column Family Storage

**Approach**: Store serialized graph + data in RocksDB column families

**Pros**:
- Unified storage (all data in RocksDB)
- Transactional consistency
- Backup/restore easier

**Cons**:
- More complex implementation (2-3 days)
- Can't use memory-mapped data file
- May be slower than file-based

**Storage Layout**:
```
RocksDB Column Families:
  cf_vector_index_graph: table_id → serialized graph
  cf_vector_index_data:  table_id → serialized vectors
```

### Option 3: Hybrid (File + RocksDB Metadata)

**Approach**: Files for data, RocksDB for metadata + pointers

**Pros**:
- Best of both worlds
- Efficient memory-mapping
- Metadata in RocksDB for consistency

**Cons**:
- Most complex (3-4 days)
- Two storage systems to manage

---

## Chosen Approach: Option 1 (File-Based)

**Rationale**:
1. **Fastest to implement**: 1-2 days (vs 2-4 days for alternatives)
2. **Proven format**: hnsw_rs built-in serialization
3. **Memory-efficient**: Memory-mapped data file
4. **Good enough**: Persistence is the goal, not perfect storage architecture
5. **Can iterate**: If needed, migrate to RocksDB later

---

## Implementation Plan

### Phase 1: Add Serialization to VectorStore (Day 1)

**Tasks**:
1. Add `f32` serialization support to Vector type
2. Implement `VectorStore::save_to_disk()`
3. Implement `VectorStore::load_from_disk()`
4. Add file path configuration
5. Unit tests (save + load roundtrip)

**Code Changes**:
```rust
// src/vector/store.rs

use hnsw_rs::hnswio::*;
use std::path::Path;

impl<'a> VectorStore<'a> {
    /// Save HNSW index to disk
    pub fn save_to_disk(&self, base_path: &str) -> Result<()> {
        if let Some(ref index) = self.hnsw_index {
            // Convert Vector to Vec<f32> for serialization
            let data: Vec<(&Vec<f32>, usize)> = self.vectors
                .iter()
                .enumerate()
                .map(|(id, v)| (&v.data, id))
                .collect();

            // Dump to files
            let graph_path = format!("{}.hnsw.graph", base_path);
            let data_path = format!("{}.hnsw.data", base_path);

            index.file_dump(&graph_path, &data_path)?;
        }
        Ok(())
    }

    /// Load HNSW index from disk
    pub fn load_from_disk(base_path: &str, dimensions: usize) -> Result<Self> {
        let graph_path = format!("{}.hnsw.graph", base_path);
        let data_path = format!("{}.hnsw.data", base_path);

        if !Path::new(&graph_path).exists() {
            anyhow::bail!("HNSW graph file not found: {}", graph_path);
        }

        // Load description
        let description = load_description(&graph_path)?;

        // Load HNSW
        let hnsw_loaded = HnswIo::load_hnsw::<f32, DistL2>(
            &graph_path,
            &data_path
        )?;

        // Reconstruct VectorStore
        let mut vectors = Vec::new();
        for (data, _id) in hnsw_loaded.get_data() {
            vectors.push(Vector::new(data.clone()));
        }

        Ok(Self {
            vectors,
            hnsw_index: Some(hnsw_loaded.into_hnsw()),
            dimensions,
        })
    }
}
```

### Phase 2: Integrate with Table Storage (Day 1-2)

**Tasks**:
1. Add persistence config to Table
2. Auto-save on index build
3. Auto-load on first query
4. Handle missing index (rebuild)

**Code Changes**:
```rust
// src/table.rs

impl Table {
    /// Get or build vector index (with persistence)
    pub fn get_or_build_vector_index(&mut self, column: &str) -> Result<Arc<VectorStore>> {
        let cache_key = format!("vector_index:{}", column);

        // Check cache
        if let Some(cached) = self.cache.get(&cache_key) {
            return Ok(Arc::clone(cached));
        }

        // Try to load from disk
        let index_path = format!("data/vectors/{}_{}", self.name, column);
        if let Ok(store) = VectorStore::load_from_disk(&index_path, 1536) {
            eprintln!("✅ Loaded HNSW index from disk: {}", index_path);
            let arc_store = Arc::new(store);
            self.cache.insert(cache_key, Arc::clone(&arc_store));
            return Ok(arc_store);
        }

        // Build from scratch
        eprintln!("🔨 Building HNSW index for column '{}'...", column);
        let store = self.build_vector_index(column)?;

        // Save to disk
        store.save_to_disk(&index_path)?;
        eprintln!("✅ Saved HNSW index to disk: {}", index_path);

        let arc_store = Arc::new(store);
        self.cache.insert(cache_key, Arc::clone(&arc_store));
        Ok(arc_store)
    }
}
```

### Phase 3: Testing & Validation (Day 2)

**Tests**:
1. Unit test: save + load roundtrip (10K vectors)
2. Integration test: query after restart (100K vectors)
3. Benchmark: 100K vectors before/after persistence
4. Stress test: Multiple save/load cycles

**Validation Criteria**:
- ✅ Save/load preserves recall (99.5%)
- ✅ 100K vectors: <10ms p95 queries (vs 96-122ms before)
- ✅ Index survives restart (no rebuild)
- ✅ Memory usage stable (<500MB for 100K vectors)

### Phase 4: 1M Scale Validation (Day 3-4)

**Tasks**:
1. Insert 1M vectors (1536D)
2. Measure: Query latency, memory, build time
3. Save to disk, measure file size
4. Restart, load from disk, measure load time
5. Query performance after reload

**Expected Results**:
- Build time: 5-10 minutes
- File size: 6-12GB (1M × 1536 × 4 bytes ≈ 6GB + graph overhead)
- Load time: 10-30 seconds
- Query latency: <15ms p95
- Memory usage: 8-15GB

---

## Success Criteria

### Day 2 Complete:
- ✅ File-based persistence implemented
- ✅ Save/load working (10K vectors tested)
- ✅ 100K vectors: 96-122ms → <10ms queries
- ✅ Auto-save on build, auto-load on query

### Day 3-4 Complete:
- ✅ 1M vectors validated
- ✅ Query latency: <15ms p95
- ✅ Load time: <30 seconds
- ✅ Memory usage: <15GB

---

## Rollback Plan

If file-based persistence fails (unlikely):
1. **Fallback**: Keep in-memory only, document 50K limit
2. **Alternative**: Implement RocksDB storage (Option 2)
3. **Timeline**: +1-2 days for RocksDB approach

---

## Next Steps After Persistence

**Week 6 Days 3-4** (assuming Day 1-2 success):
1. Validate 1M scale (see Phase 4)
2. Document scaling characteristics
3. Identify any new bottlenecks

**Week 6 Days 5-7**:
1. MN-RU updates (production write performance)
2. Parallel building (10M+ scale)
3. Benchmark mixed workload

**Week 7-8**:
1. Benchmark vs pgvector (1M, 10M vectors)
2. Document "10x faster" claim
3. Prepare for launch

---

**END PLAN**

*Ready to implement. Starting with Phase 1 (VectorStore serialization).*
