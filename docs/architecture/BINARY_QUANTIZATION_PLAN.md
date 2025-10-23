# Binary Quantization Implementation Plan

**Created**: October 23, 2025
**Timeline**: Week 3-4 (7-14 days)
**Goal**: 96% memory reduction while maintaining 95%+ recall

---

## Algorithm Overview: RaBitQ-Inspired Binary Quantization

### Core Concept

Convert high-dimensional float32 vectors to binary (1 bit per dimension):
- **Input**: 1536D float32 vector (6,144 bytes)
- **Output**: 1536-bit binary vector (192 bytes)
- **Reduction**: 96% memory savings (6,144 → 192 bytes)

### Two-Phase Search Strategy

**Phase 1: Approximate Search (Fast)**
1. Query vector → binary quantization
2. Search HNSW graph using Hamming distance (bitwise XOR + popcount)
3. Retrieve top N candidates (e.g., N=100 for k=10)

**Phase 2**: Reranking (Accurate)
1. Load original float32 vectors for candidates
2. Compute exact L2/cosine distance
3. Return top k results

**Result**: 95%+ recall maintained, 2-5x faster queries

---

## Data Structures

### 1. Quantized Vector Storage

```rust
/// Binary quantized vector (1 bit per dimension)
/// Stores vector as packed u64 array for SIMD efficiency
pub struct QuantizedVector {
    /// Packed binary representation
    /// For 1536D: 1536 bits = 24 × u64 (64 bits each)
    bits: Vec<u64>,

    /// Dimension count (for validation)
    dimensions: usize,
}

impl QuantizedVector {
    /// Create from float32 vector using threshold
    pub fn from_f32(vector: &[f32], threshold: &[f32]) -> Self;

    /// Hamming distance (XOR + popcount)
    pub fn hamming_distance(&self, other: &QuantizedVector) -> u32;

    /// Convert to bytes for storage
    pub fn to_bytes(&self) -> Vec<u8>;

    /// Load from bytes
    pub fn from_bytes(bytes: &[u8], dimensions: usize) -> Result<Self>;
}
```

### 2. Quantization Metadata

```rust
/// Thresholds for binary quantization (per-dimension)
/// RaBitQ uses randomized thresholds for better error bounds
pub struct QuantizationModel {
    /// Per-dimension thresholds (1536 floats for 1536D)
    /// Initialized during training on sample vectors
    thresholds: Vec<f32>,

    /// Statistics (optional, for debugging)
    mean: Option<Vec<f32>>,
    std_dev: Option<Vec<f32>>,
}

impl QuantizationModel {
    /// Train on sample vectors (compute thresholds)
    pub fn train(sample_vectors: &[Vec<f32>]) -> Self;

    /// Quantize a vector
    pub fn quantize(&self, vector: &[f32]) -> QuantizedVector;
}
```

### 3. Integrated Vector Store

```rust
/// Vector store with binary quantization + HNSW
pub struct QuantizedVectorStore {
    /// Quantization model (trained on sample data)
    quantization_model: QuantizationModel,

    /// HNSW index using quantized vectors (Hamming distance)
    hnsw_index: HNSWIndex<QuantizedVector>,

    /// Original float32 vectors (for reranking)
    /// Stored compressed in RocksDB or memory
    original_vectors: Vec<Vector>,

    /// Metadata
    dimensions: usize,
}

impl QuantizedVectorStore {
    /// Two-phase k-NN search
    pub fn knn_search(&self, query: &Vector, k: usize, candidates: usize) -> Result<Vec<(usize, f32)>> {
        // Phase 1: Approximate search with Hamming distance
        let query_quantized = self.quantization_model.quantize(&query.data);
        let approximate_results = self.hnsw_index.search(&query_quantized, candidates)?;

        // Phase 2: Rerank with exact L2 distance
        let mut exact_results: Vec<(usize, f32)> = approximate_results
            .iter()
            .map(|(id, _hamming_dist)| {
                let original = &self.original_vectors[*id];
                let l2_dist = query.l2_distance(original).unwrap();
                (*id, l2_dist)
            })
            .collect();

        // Sort by exact distance and return top k
        exact_results.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap());
        Ok(exact_results.into_iter().take(k).collect())
    }
}
```

---

## RaBitQ Algorithm Details

### Training Phase

**Goal**: Compute per-dimension thresholds that minimize quantization error.

**Basic approach** (sign-based):
```rust
// For each dimension i:
threshold[i] = mean(all_vectors[*, i])
```

**RaBitQ approach** (randomized, better error bounds):
```rust
// For each dimension i:
// 1. Compute mean and std dev
let mean_i = vectors.iter().map(|v| v[i]).sum::<f32>() / n;
let std_i = sqrt(vectors.iter().map(|v| (v[i] - mean_i).powi(2)).sum::<f32>() / n);

// 2. Randomized threshold (improves theoretical guarantees)
let threshold[i] = mean_i + random_uniform(-0.5 * std_i, 0.5 * std_i);
```

### Quantization Function

```rust
fn quantize(vector: &[f32], thresholds: &[f32]) -> Vec<u64> {
    let mut bits = vec![0u64; (vector.len() + 63) / 64];

    for (i, &value) in vector.iter().enumerate() {
        if value >= thresholds[i] {
            // Set bit i to 1
            let word_idx = i / 64;
            let bit_idx = i % 64;
            bits[word_idx] |= 1u64 << bit_idx;
        }
        // Otherwise bit stays 0 (default)
    }

    bits
}
```

### Hamming Distance (SIMD-Optimized)

```rust
fn hamming_distance(a: &[u64], b: &[u64]) -> u32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x ^ y).count_ones())  // XOR + popcount
        .sum()
}
```

**Why this is fast**:
- XOR: Single CPU instruction per u64
- `count_ones()`: POPCNT instruction (hardware-accelerated)
- For 1536D: Only 24 XOR + POPCNT operations (vs 1536 float subtractions + multiplications for L2)

---

## Implementation Phases

### Phase 1: Core Quantization (Days 1-3)

**Day 1**:
- [ ] Create `src/quantization/mod.rs` module
- [ ] Implement `QuantizedVector` struct
- [ ] Implement `from_f32()` with basic threshold (mean-based)
- [ ] Implement `hamming_distance()`
- [ ] Unit tests: quantization + distance (10 tests)

**Day 2**:
- [ ] Implement `QuantizationModel` struct
- [ ] Training: compute per-dimension mean/std
- [ ] RaBitQ-style randomized thresholds (optional improvement)
- [ ] Serialization: to_bytes/from_bytes for persistence
- [ ] Unit tests: training + serialization (8 tests)

**Day 3**:
- [ ] Optimize Hamming distance (use u64 packing)
- [ ] Add SIMD hints for count_ones()
- [ ] Benchmark: quantization speed (target <0.1ms per vector)
- [ ] Benchmark: Hamming distance (target <0.01ms)

**Success Criteria**:
- ✅ Quantization: <0.1ms per 1536D vector
- ✅ Hamming distance: <0.01ms per pair
- ✅ Memory: 192 bytes per quantized vector (vs 6,144)

### Phase 2: HNSW Integration (Days 4-6)

**Day 4**:
- [ ] Extend HNSWIndex to support QuantizedVector
- [ ] Implement Hamming distance as HNSW metric
- [ ] Build quantized HNSW graph from sample vectors
- [ ] Unit tests: HNSW with quantized vectors (6 tests)

**Day 5**:
- [ ] Implement two-phase search:
  - Phase 1: HNSW search with Hamming distance
  - Phase 2: Rerank candidates with exact L2
- [ ] Tune candidate expansion (10x, 20x, 50x)
- [ ] Unit tests: two-phase search recall validation (5 tests)

**Day 6**:
- [ ] Store original vectors alongside quantized (RocksDB)
- [ ] Optimize reranking (SIMD L2 distance)
- [ ] Memory layout: interleave quantized + original for cache efficiency
- [ ] Benchmark: end-to-end search latency

**Success Criteria**:
- ✅ Two-phase search working
- ✅ Latency: <5ms p95 (including reranking)
- ✅ Memory: ~15GB for 10M vectors (quantized + original)

### Phase 3: Benchmarking & Validation (Days 7-8)

**Day 7**:
- [ ] Benchmark: 10K vectors (baseline)
  - Recall@10 with different candidate counts (10x, 20x, 50x)
  - Memory footprint (quantized + original + HNSW graph)
  - Latency p50/p95/p99
- [ ] Compare: BQ-HNSW vs full-precision HNSW
- [ ] Tune: Find optimal candidate expansion for 95%+ recall

**Day 8**:
- [ ] Benchmark: 100K vectors (stress test)
  - Memory validation (target: <1.5GB)
  - Recall validation (target: >95%)
  - Latency validation (target: <5ms p95)
- [ ] Create benchmark report
- [ ] Document findings

**Success Criteria**:
- ✅ Recall@10: >95% (with 20x candidate expansion)
- ✅ Memory: 24x reduction vs full-precision (6,144 → 250 bytes per vector including graph)
- ✅ Latency: <5ms p95 (2x faster than full-precision HNSW)

---

## Memory Analysis

### Per-Vector Breakdown (1536D)

| Component | Size | Notes |
|-----------|------|-------|
| **Original float32** | 6,144 bytes | Stored for reranking |
| **Quantized (1536 bits)** | 192 bytes | Used in HNSW graph |
| **HNSW graph edges** | ~100 bytes | M=48 connections × 2 bytes per ID |
| **Total per vector** | ~6,436 bytes | Without compression |

### With Compression (Optional)

| Component | Size | Notes |
|-----------|------|-------|
| **Compressed original** | ~1,000 bytes | zstd compression (~6x) |
| **Quantized** | 192 bytes | No compression (already minimal) |
| **HNSW graph** | ~100 bytes | Edge list |
| **Total per vector** | ~1,292 bytes | 4.75x better than uncompressed |

### 10M Vectors Estimate

**Without compression**:
- Quantized: 10M × 192 bytes = 1.92 GB
- Original: 10M × 6,144 bytes = 61.4 GB
- HNSW graph: 10M × 100 bytes = 1.0 GB
- **Total**: ~64 GB

**With lazy reranking** (store original on disk):
- Quantized + graph in memory: 2.92 GB
- Original on SSD: 61.4 GB (loaded on demand)
- **Memory footprint**: <3 GB (vs 170 GB full-precision)

**With compression** (optional Phase 2):
- Compressed original: 10M × 1,000 bytes = 10 GB
- Quantized + graph: 2.92 GB
- **Total**: ~13 GB (13x better than full-precision)

---

## Testing Strategy

### Unit Tests (25+ tests)

**Quantization tests** (10 tests):
- Threshold computation (mean, std dev)
- float32 → binary conversion
- Bit packing correctness
- Serialization round-trip
- Edge cases (all zeros, all ones, NaN handling)

**Distance tests** (8 tests):
- Hamming distance correctness
- XOR + popcount validation
- Symmetry (d(a,b) = d(b,a))
- Triangle inequality (approximate)
- Performance (< 0.01ms)

**HNSW integration tests** (7 tests):
- Build quantized index
- Search with Hamming distance
- Two-phase search recall
- Candidate expansion validation
- Memory usage validation

### Benchmark Tests

**Recall validation** (1536D, 10K vectors):
| Candidate Expansion | Expected Recall@10 | Latency p95 |
|---------------------|-------------------|-------------|
| 10x (100 candidates) | ~85% | <2ms |
| 20x (200 candidates) | ~95% | <3ms |
| 50x (500 candidates) | ~98% | <5ms |

**Memory validation** (10K vectors):
- Quantized: ~1.9 MB (192 bytes × 10K)
- Original: ~61 MB (6,144 bytes × 10K)
- HNSW graph: ~1 MB (100 bytes × 10K)
- **Total**: ~64 MB (vs 170 MB full-precision)

---

## Performance Targets

### Week 3-4 Goals

| Metric | Target | Baseline (Full HNSW) |
|--------|--------|---------------------|
| **Recall@10** | >95% | 99.5% |
| **Latency p95** | <5ms | 6.63ms |
| **Memory (10M)** | <15 GB | 170 GB |
| **Query speedup** | 2-5x | 1x |
| **Memory reduction** | 24x | 1x |

### Stretch Goals (Week 5+)

- [ ] Compression: Further reduce to ~13 GB (compress original vectors)
- [ ] SIMD: Optimize Hamming distance with AVX2/NEON
- [ ] Adaptive expansion: Auto-tune candidates based on query difficulty
- [ ] Batch quantization: Parallel training on large datasets

---

## References

**RaBitQ**:
- Paper: SIGMOD 2024 (Gao & Long)
- GitHub: https://github.com/gaoj0017/RaBitQ
- Extended-RaBitQ: SIGMOD 2025

**Binary Quantization Production**:
- Qdrant: 4x RPS gains (2024 blog)
- Elasticsearch BBQ: 20-30x faster quantization (2024)
- Weaviate: Binary compression guide

**Research Reports**:
- `docs/architecture/research/sota_vector_search_algorithms_2024_2025.md`
- VIBE benchmark (May 2025): Binary methods in top 5

---

## Next Steps (Day 1)

1. ✅ Create this implementation plan
2. [ ] Create `src/quantization/` module structure
3. [ ] Implement `QuantizedVector` with bit packing
4. [ ] Implement basic Hamming distance
5. [ ] Write 10 unit tests
6. [ ] Benchmark quantization speed

**Timeline**: 7-8 days to production-ready binary quantization with 95%+ recall.

---

**Status**: Ready to implement
**Risk**: Low (proven technology, clear algorithm)
**Reward**: 24x memory reduction, 2-5x speedup, competitive with Pinecone
