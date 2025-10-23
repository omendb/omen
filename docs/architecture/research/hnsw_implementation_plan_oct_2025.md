# HNSW Implementation Plan (Tactical)

**Date**: October 22, 2025
**Timeline**: 7 days (Oct 23-29, 2025)
**Goal**: Production-ready HNSW for 1536D vectors with >95% recall, <10ms latency

---

## Day 1 (Oct 23): Setup & Basic Integration

### 1. Add hnsw_rs Dependency

**File**: `Cargo.toml`

```toml
[dependencies]
# Vector search - HNSW algorithm
hnsw_rs = "0.3"  # Latest version
```

**Build with SIMD** (x86_64 only):
```bash
cargo build --release --features "hnsw_rs/simdeez_f"
```

**Note**: On M3 Mac, omit SIMD feature (ARM doesn't support simdeez_f)

### 2. Refactor Vector Module Structure

**Current**: `src/vector.rs` (single file)
**Target**: Modular structure

**Files to create**:
```
src/vector/
├── mod.rs              # Module entry point
├── types.rs            # Vector type, distance functions
├── hnsw_index.rs       # HNSW wrapper (NEW)
└── store.rs            # VectorStore with HNSW (refactored)
```

**Step-by-step**:

1. Create `src/vector/` directory
2. Copy `src/vector.rs` → `src/vector/types.rs`
3. Move `VectorStore` → `src/vector/store.rs`
4. Create `src/vector/mod.rs`:
   ```rust
   pub mod types;
   pub mod hnsw_index;
   pub mod store;

   pub use types::Vector;
   pub use hnsw_index::HNSWIndex;
   pub use store::VectorStore;
   ```
5. Create `src/vector/hnsw_index.rs` (see implementation below)
6. Update `src/lib.rs`:
   ```rust
   pub mod vector;
   pub mod pca; // Keep for reference
   ```

### 3. Implement HNSW Wrapper

**File**: `src/vector/hnsw_index.rs`

**Key struct**:
```rust
use hnsw_rs::hnsw::Hnsw;
use hnsw_rs::dist::DistL2;
use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct HNSWIndex {
    /// HNSW index from hnsw_rs
    index: Hnsw<f32, DistL2>,

    /// Index parameters
    max_elements: usize,
    max_nb_connection: usize,  // M parameter
    ef_construction: usize,

    /// Runtime search parameter
    ef_search: usize,

    /// Vector dimensionality
    dimensions: usize,

    /// Number of vectors inserted
    num_vectors: usize,
}
```

**API**:
```rust
impl HNSWIndex {
    pub fn new(max_elements: usize, dimensions: usize) -> Self;
    pub fn insert(&mut self, vector: &[f32]) -> Result<usize>;
    pub fn search(&self, query: &[f32], k: usize) -> Result<Vec<(usize, f32)>>;
    pub fn set_ef_search(&mut self, ef_search: usize);
    pub fn len(&self) -> usize;
}
```

**Parameters** (initial values):
- `max_nb_connection`: 48 (M parameter for high-dimensional vectors)
- `ef_construction`: 200 (balanced quality/speed)
- `ef_search`: 100 (tunable at runtime)

### 4. Unit Tests

**File**: `src/vector/hnsw_index.rs` (tests module)

**Tests to write**:
1. `test_hnsw_insert`: Insert 1K vectors, verify count
2. `test_hnsw_search`: Search with k=10, verify results length
3. `test_hnsw_recall`: Compare with brute-force, check recall >80%
4. `test_hnsw_dimensions`: Test different dimensions (128, 768, 1536)

**Run tests**:
```bash
cargo test --lib vector::hnsw_index
```

**Success criteria**: All tests pass

---

## Day 2 (Oct 24): RocksDB Integration

### 1. HNSW Serialization

**Challenge**: hnsw_rs has `file_dump()` and `file_load()`, but we need byte arrays for RocksDB

**Approach**: Use temporary files
```rust
impl HNSWIndex {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let temp_file = tempfile::NamedTempFile::new()?;
        self.index.file_dump(temp_file.path())?;
        let bytes = std::fs::read(temp_file.path())?;
        Ok(bytes)
    }

    pub fn from_bytes(bytes: &[u8], dimensions: usize) -> Result<Self> {
        let temp_file = tempfile::NamedTempFile::new()?;
        std::fs::write(temp_file.path(), bytes)?;
        let index = Hnsw::<f32, DistL2>::file_load(temp_file.path())?;

        // Extract parameters from loaded index
        Ok(Self {
            index,
            dimensions,
            // ... extract other params
        })
    }
}
```

**Alternative**: Investigate if hnsw_rs supports bincode serialization (check source code)

### 2. Update VectorStore

**File**: `src/vector/store.rs`

**Changes**:
1. Remove simple HashMap index
2. Add HNSW index as optional field:
   ```rust
   pub struct VectorStore {
       vectors: Vec<Vector>,
       hnsw_index: Option<HNSWIndex>,
       dimensions: usize,
   }
   ```

3. Update `insert()`:
   ```rust
   pub fn insert(&mut self, vector: Vector) -> Result<usize> {
       let id = self.vectors.len();

       // Lazy initialize HNSW on first insert
       if self.hnsw_index.is_none() {
           self.hnsw_index = Some(HNSWIndex::new(1_000_000, self.dimensions));
       }

       // Insert into HNSW
       if let Some(ref mut index) = self.hnsw_index {
           index.insert(&vector.data)?;
       }

       self.vectors.push(vector);
       Ok(id)
   }
   ```

4. Update `knn_search()` to use HNSW:
   ```rust
   pub fn knn_search(&self, query: &Vector, k: usize) -> Result<Vec<(usize, f32)>> {
       if let Some(ref index) = self.hnsw_index {
           return index.search(&query.data, k);
       }

       // Fallback to brute-force if no index
       self.knn_search_brute_force(query, k)
   }
   ```

### 3. Integration Tests

**Tests**:
1. Insert 10K vectors, verify HNSW index is built
2. Search and verify recall >90%
3. Serialize HNSW to bytes, deserialize, verify search still works

**Run**:
```bash
cargo test --lib vector::store
```

---

## Day 3 (Oct 25): PostgreSQL Protocol Integration

**Goal**: Wire up HNSW to PostgreSQL distance operators

### 1. Distance Operators (Already Implemented)

**Check**: `src/sql_engine.rs` or postgres module

**Operators needed**:
- `<->`: L2 distance (Euclidean)
- `<#>`: Negative dot product (max inner product)
- `<=>`: Cosine distance

**Status**: Vector type already has these methods (`l2_distance`, `dot_product`, `cosine_distance`)

### 2. Query Planning

**File**: `src/sql_engine.rs`

**Pattern to detect**:
```sql
SELECT * FROM embeddings
ORDER BY embedding <-> '[0.1, 0.2, ...]'::vector
LIMIT 10;
```

**Logic**:
1. Detect `ORDER BY {vector_column} {distance_op} {vector_literal}`
2. Extract vector literal, parse to Vec<f32>
3. Call `VectorStore::knn_search(query, LIMIT)`
4. Return results

**Implementation**:
- May need to add special case in query planner
- For prototype, can intercept in execution phase

### 3. Testing

**Manual test** (via psql):
```sql
-- Connect to omendb on port 5433
psql -h localhost -p 5433 -U admin -d testdb

-- Create table with vector column (if not exists)
CREATE TABLE embeddings (
    id SERIAL PRIMARY KEY,
    embedding vector(1536)
);

-- Insert test vectors
INSERT INTO embeddings (embedding) VALUES ('[...]'::vector);

-- Query nearest neighbors
SELECT id, embedding <-> '[...]'::vector AS distance
FROM embeddings
ORDER BY distance
LIMIT 10;
```

**Success**: Query returns 10 results with distances

---

## Day 4 (Oct 26): INSERT Optimization

### 1. Benchmark Current INSERT Performance

**Test**: Insert 100K vectors, measure time

**Expected**:
- HNSW insert: O(M * ef_construction * log(N))
- For M=48, ef=200, N=100K: ~0.1-1ms per insert
- Total: 10-100 seconds for 100K vectors

### 2. Batch INSERT Optimization (If Needed)

**Approach 1**: Rebuild index after N inserts
```rust
impl VectorStore {
    pub fn insert_batch(&mut self, vectors: Vec<Vector>) -> Result<()> {
        // Add vectors to storage
        self.vectors.extend(vectors);

        // Rebuild HNSW index from scratch
        self.rebuild_index()?;
        Ok(())
    }

    fn rebuild_index(&mut self) -> Result<()> {
        let mut index = HNSWIndex::new(self.vectors.len(), self.dimensions);
        for vec in &self.vectors {
            index.insert(&vec.data)?;
        }
        self.hnsw_index = Some(index);
        Ok(())
    }
}
```

**Approach 2**: Parallel insert (if hnsw_rs supports it)
- Check if hnsw_rs has `insert_parallel()` method
- If yes, use for batch inserts

### 3. Benchmark After Optimization

**Target**: 100K vectors in <60 seconds

---

## Day 5 (Oct 27): Search Optimization

### 1. ef_search Tuning

**Test different values**:
- ef_search = 50, 100, 200, 500
- Measure: Recall@10, latency

**Expected**:
- ef=50: 85-90% recall, ~1ms latency
- ef=100: 90-95% recall, ~2ms latency
- ef=200: 95-98% recall, ~5ms latency
- ef=500: 98-99% recall, ~10ms latency

**Choose**: ef_search=100 as default (balance recall/latency)

### 2. Parallel Search (If Needed)

**Check**: Does hnsw_rs have `parallel_search()`?

**If yes**:
```rust
pub fn search_batch(&self, queries: &[Vec<f32>], k: usize) -> Result<Vec<Vec<(usize, f32)>>> {
    // Use parallel search
    self.index.parallel_search(queries, k, self.ef_search)
}
```

### 3. SIMD Validation

**Test**: Build with SIMD feature on Fedora (x86_64 with AVX2)

```bash
ssh nick@fedora
cd omendb-server
cargo build --release --features "hnsw_rs/simdeez_f"
```

**Benchmark**: Compare SIMD vs non-SIMD
- Expected speedup: 2-4x on distance calculations

---

## Day 6 (Oct 28): Benchmark (100K vectors)

### 1. Update Benchmark Code

**File**: `src/bin/benchmark_vector_prototype.rs`

**Rename to**: `src/bin/benchmark_hnsw.rs`

**Changes**:
1. Remove ALEX-specific code
2. Use `VectorStore::knn_search()` (now HNSW-backed)
3. Add parameter tuning tests

**New sections**:
```rust
fn benchmark_parameter_tuning(store: &VectorStore) {
    println!("\n=== Parameter Tuning ===");

    for ef_search in [50, 100, 200, 500] {
        // Set ef_search
        // Run benchmark
        // Report recall, latency
    }
}
```

### 2. Run Comprehensive Benchmark

**Command**:
```bash
cargo build --release
./target/release/benchmark_hnsw
```

**Metrics to collect**:
- **Memory**: Total bytes / num_vectors (target <200 bytes/vector)
- **Insert**: Time to insert 100K vectors (target <5 minutes)
- **Search**:
  - p50, p95, p99 latency (target p95 <10ms)
  - Recall@10 (target >95%)
  - Queries/second

### 3. Results Analysis

**Create**: `docs/architecture/research/hnsw_benchmark_results_oct_2025.md`

**Format**:
```markdown
# HNSW Benchmark Results (100K Vectors, 1536D)

## Setup
- Machine: Mac M3 Max, 128GB RAM
- Dataset: 100K random vectors, 1536 dimensions
- Parameters: M=48, ef_construction=200, ef_search=100

## Results

### Memory
- Total: X MB
- Per vector: Y bytes
- Target: <200 bytes/vector
- Status: ✅ PASS / ❌ FAIL

### Insert Performance
- Total time: X seconds
- Throughput: Y inserts/sec
- Target: <5 minutes (300s)
- Status: ✅ PASS / ❌ FAIL

### Search Performance
- p50 latency: X ms
- p95 latency: Y ms
- p99 latency: Z ms
- Target: p95 <10ms
- Status: ✅ PASS / ❌ FAIL

### Recall
- Recall@10: X%
- Target: >95%
- Status: ✅ PASS / ❌ FAIL

## Parameter Tuning

| ef_search | Recall@10 | p95 Latency |
|-----------|-----------|-------------|
| 50        | X%        | Y ms        |
| 100       | X%        | Y ms        |
| 200       | X%        | Y ms        |
| 500       | X%        | Y ms        |

## Conclusion
[Analysis of results, recommendations]
```

---

## Day 7 (Oct 29): Validation & Go/No-Go Decision

### 1. Final Validation

**Checklist**:
- ✅ Recall@10 > 95%
- ✅ p95 latency < 10ms
- ✅ Memory < 200 bytes/vector
- ✅ Insert 100K vectors < 5 minutes
- ✅ All unit tests pass
- ✅ Integration tests pass

### 2. Go/No-Go Decision

**SUCCESS** (all criteria met):
- ✅ Proceed to 1M-10M scale (Week 3)
- ✅ Update ai/STATUS.md: "HNSW validated, production-ready"
- ✅ Update ai/TODO.md: Move to Phase 2 (large-scale optimization)
- ✅ Update ai/DECISIONS.md: "HNSW decision confirmed, 95%+ recall achieved"

**TUNE** (recall 90-95%):
- 🔄 Increase M to 64
- 🔄 Increase ef_construction to 400
- 🔄 Increase ef_search to 200
- 🔄 Re-benchmark (1 day extension)

**INVESTIGATE** (recall <90%):
- ❌ Debug HNSW integration (extremely unlikely)
- ❌ Check for bugs in distance calculation
- ❌ Validate test methodology

### 3. Documentation Updates

**Files to update**:

1. `ai/STATUS.md`:
   ```markdown
   ## Week 2 Results (Oct 29) ✅ SUCCESS

   **HNSW Implementation Completed**:
   - [x] Research HNSW algorithm & Rust implementations
   - [x] Implement hnsw_rs wrapper
   - [x] Integrate with VectorStore
   - [x] Benchmark 100K vectors

   **Results**:
   - ✅ Memory: X bytes/vector (target <200 ✅)
   - ✅ Latency: Y ms p95 (target <10ms ✅)
   - ✅ Recall: Z% recall@10 (target >95% ✅)

   **Decision**: HNSW validated, proceed to large-scale optimization ✅
   ```

2. `ai/TODO.md`:
   - Mark Week 2 tasks as completed
   - Update Week 3 tasks (if proceeding to scale)

3. `ai/DECISIONS.md`:
   ```markdown
   ## 2025-10-29: HNSW Validation Success

   **Results**: HNSW achieves >95% recall, <10ms latency for 100K 1536D vectors

   **Parameters**:
   - M: 48 (high-dimensional embeddings)
   - ef_construction: 200
   - ef_search: 100

   **Next Steps**: Scale to 1M-10M vectors (Week 3-4)
   ```

### 4. Commit Work

```bash
git add -A
git commit -m "feat: implement HNSW for vector search

Week 2 implementation (Oct 23-29):
- Add hnsw_rs dependency with SIMD support
- Refactor vector module structure
- Implement HNSWIndex wrapper
- Integrate with VectorStore and RocksDB
- Benchmark 100K vectors: 95%+ recall, <10ms p95 latency

Results:
- Memory: X bytes/vector (vs target <200)
- Latency: Y ms p95 (vs target <10ms)
- Recall: Z% recall@10 (vs target >95%)

Status: Production-ready for 100K-1M scale

🤖 Generated with Claude Code
Co-Authored-By: Claude <noreply@anthropic.com>"
```

---

## Key Files Summary

### New Files
- `src/vector/mod.rs` - Module entry point
- `src/vector/types.rs` - Vector type (from vector.rs)
- `src/vector/hnsw_index.rs` - HNSW wrapper (NEW)
- `src/vector/store.rs` - VectorStore with HNSW
- `src/bin/benchmark_hnsw.rs` - HNSW benchmark
- `docs/architecture/research/hnsw_benchmark_results_oct_2025.md` - Results

### Modified Files
- `Cargo.toml` - Add hnsw_rs dependency
- `src/lib.rs` - Update module declarations
- `src/sql_engine.rs` - Wire up distance operators (Day 3)

### Deleted Files
- `src/vector.rs` - Refactored into vector/ module

---

## Risk Mitigation

### Risk 1: hnsw_rs API Compatibility Issues
**Mitigation**: Check crate docs first, read examples in GitHub repo

### Risk 2: Serialization Complexity
**Mitigation**: Use temp files for dump/load, test early (Day 2)

### Risk 3: Recall <95%
**Mitigation**: Tune M, ef_construction, ef_search (well-documented parameters)

### Risk 4: Latency >10ms
**Mitigation**: Enable SIMD on x86_64, tune ef_search down to 50-100

---

## Success Criteria (Final)

**Mandatory** (all must pass):
1. ✅ Recall@10 > 95%
2. ✅ p95 latency < 10ms
3. ✅ Memory < 200 bytes/vector (excluding vector data)
4. ✅ Insert 100K vectors < 5 minutes

**Nice-to-have**:
- 🎯 Recall@10 > 98%
- 🎯 p95 latency < 5ms
- 🎯 Memory < 100 bytes/vector
- 🎯 SIMD speedup 2-4x (x86_64 only)

---

**Status**: Ready for implementation (Day 1: Oct 23, 2025)
**Estimated Time**: 40-50 hours (7 days × 6-8 hours/day)
**Success Probability**: 95%+
