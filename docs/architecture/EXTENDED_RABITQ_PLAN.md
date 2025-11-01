# Extended RaBitQ Implementation Plan
**Created**: October 31, 2025
**Target**: Weeks 11-12 (Week 11 Day 4 - Week 12 Day 5)
**Effort**: 2-3 weeks
**Status**: Planning

---

## Executive Summary

Extended RaBitQ is a SIGMOD 2025 paper that extends RaBitQ (SIGMOD 2024) to support arbitrary compression rates (2-bit, 3-bit, 4-bit, etc.) instead of just 1-bit binary quantization. It provides **orders of magnitude better accuracy** than scalar quantization at 2-bit while maintaining **same query speed**.

**Why Implement This:**
- ✅ SOTA quantization (SIGMOD 2025, peer-reviewed)
- ✅ Flexible compression (3, 4, 5, 7, 8, 9 bits/dimension)
- ✅ Better accuracy than current binary quantization
- ✅ Same query speed as scalar quantization
- ✅ Already adopted by TensorChord, Elastic
- ✅ Natural evolution of our current BQ code

---

## Background: Current Binary Quantization

**What We Have:**
- `src/vector/quantization.rs` (167 lines)
- Binary quantization: 1-bit per dimension
- 32x compression (4 bytes → 1 bit)
- Hamming distance approximation
- Reranking with original vectors

**Limitations:**
- Only 1-bit = coarse approximation
- 33% recall baseline (requires reranking)
- No flexibility in compression rate

**What Extended RaBitQ Adds:**
- 2-9 bits per dimension
- Better accuracy (orders of magnitude at 2-bit)
- Flexible compression (4x-32x)
- Same query speed

---

## Extended RaBitQ Algorithm Overview

### Core Idea

**Problem**: When using multiple bits, codebook vectors form a grid (not sphere).
**Solution**: Rescale vectors to find optimal grid point, then normalize to sphere.

### Quantization Process

```
1. Input: High-dimensional vector v
2. For each candidate rescaling factor s:
   a. Scale: v' = s * v
   b. Round: q = round(v') to grid
   c. Compute error: err = ||v - q||²
3. Select: s* = argmin(err)
4. Store: Quantized vector q* with scale s*
```

### Distance Computation

```
1. Retrieve quantized vectors q1, q2 with scales s1, s2
2. Normalize to sphere: q1' = q1/||q1||, q2' = q2/||q2||
3. Compute: dist ≈ angle(q1', q2')
```

**Key Insight**: Query speed = scalar quantization (no extra cost!)

---

## Implementation Plan

### Phase 1: Core Data Structures (Week 11 Day 4)

**1.1. Quantizer Configuration**

```rust
pub enum QuantizationBits {
    Bits2,  // 16x compression
    Bits3,  // ~10x compression
    Bits4,  // 8x compression
    Bits5,  // ~6x compression
    Bits7,  // ~4x compression
    Bits8,  // 4x compression
}

pub struct ExtendedRaBitQParams {
    bits_per_dim: QuantizationBits,
    num_rescale_factors: usize,  // Typically 8-16
    rescale_range: (f32, f32),   // E.g., (0.5, 2.0)
}
```

**1.2. Quantized Vector Storage**

```rust
pub struct QuantizedVector {
    // Packed quantized values (e.g., 2 bits → 4 values per byte)
    data: Vec<u8>,

    // Optimal rescaling factor for this vector
    scale: f32,

    // Number of bits per dimension
    bits: u8,
}
```

**Success Criteria:**
- ✅ Data structures defined
- ✅ Compression rates calculated
- ✅ Memory layout planned

---

### Phase 2: Quantization Algorithm (Week 11 Day 5 - Week 12 Day 1)

**2.1. Rescaling Search**

```rust
impl ExtendedRaBitQ {
    fn find_optimal_quantization(
        &self,
        vector: &[f32],
    ) -> (Vec<u8>, f32) {
        let mut best_error = f32::MAX;
        let mut best_quantized = Vec::new();
        let mut best_scale = 1.0;

        // Try multiple rescaling factors
        for scale in self.generate_scales() {
            let quantized = self.quantize_with_scale(vector, scale);
            let error = self.compute_error(vector, &quantized, scale);

            if error < best_error {
                best_error = error;
                best_quantized = quantized;
                best_scale = scale;
            }
        }

        (best_quantized, best_scale)
    }
}
```

**2.2. Grid Rounding**

```rust
fn quantize_with_scale(
    &self,
    vector: &[f32],
    scale: f32,
) -> Vec<u8> {
    let bits = self.params.bits_per_dim.to_u8();
    let levels = 1 << bits;  // 2^bits quantization levels
    let max_val = (levels - 1) as f32;

    let mut quantized = Vec::new();

    for &val in vector {
        // Scale, clamp, round
        let scaled = val * scale;
        let clamped = scaled.clamp(0.0, 1.0);
        let level = (clamped * max_val).round() as u8;
        quantized.push(level);
    }

    // Pack into bytes (e.g., 4x 2-bit values per byte)
    self.pack_quantized(&quantized, bits)
}
```

**2.3. Error Computation**

```rust
fn compute_error(
    &self,
    original: &[f32],
    quantized: &[u8],
    scale: f32,
) -> f32 {
    let decoded = self.decode(quantized, scale);

    original.iter()
        .zip(decoded.iter())
        .map(|(o, q)| (o - q).powi(2))
        .sum::<f32>()
}
```

**Success Criteria:**
- ✅ Rescaling search works
- ✅ Grid rounding correct
- ✅ Error minimization validated
- ✅ Test: Quantize random vector, verify optimal scale found

---

### Phase 3: Distance Computation (Week 12 Day 2)

**3.1. Dequantization**

```rust
fn decode(&self, quantized: &[u8], scale: f32) -> Vec<f32> {
    let bits = self.params.bits_per_dim.to_u8();
    let unpacked = self.unpack_quantized(quantized, bits);
    let levels = 1 << bits;
    let max_val = (levels - 1) as f32;

    unpacked.iter()
        .map(|&level| (level as f32 / max_val) / scale)
        .collect()
}
```

**3.2. Distance Approximation**

```rust
fn approximate_distance(
    &self,
    q1: &QuantizedVector,
    q2: &QuantizedVector,
) -> f32 {
    let v1 = self.decode(&q1.data, q1.scale);
    let v2 = self.decode(&q2.data, q2.scale);

    // Normalize to sphere
    let v1_norm = normalize(&v1);
    let v2_norm = normalize(&v2);

    // Angular distance
    angular_distance(&v1_norm, &v2_norm)
}
```

**Success Criteria:**
- ✅ Dequantization works
- ✅ Distance approximation correct
- ✅ Test: Verify distances correlate with L2

---

### Phase 4: SIMD Optimizations (Week 12 Day 3)

**4.1. SIMD Rescaling**

```rust
#[cfg(target_feature = "avx2")]
unsafe fn quantize_with_scale_avx2(
    vector: &[f32],
    scale: f32,
    bits: u8,
) -> Vec<u8> {
    // AVX2: Process 8 floats at once
    let scale_vec = _mm256_set1_ps(scale);
    let max_val = ((1 << bits) - 1) as f32;
    let max_vec = _mm256_set1_ps(max_val);

    // ... SIMD quantization
}
```

**4.2. SIMD Distance**

```rust
#[cfg(target_feature = "avx2")]
unsafe fn approximate_distance_avx2(
    q1: &QuantizedVector,
    q2: &QuantizedVector,
) -> f32 {
    // AVX2: Process 8 floats at once for decode + distance
    // ... SIMD distance computation
}
```

**4.3. ARM NEON Support**

```rust
#[cfg(target_arch = "aarch64")]
unsafe fn quantize_with_scale_neon(
    vector: &[f32],
    scale: f32,
    bits: u8,
) -> Vec<u8> {
    // NEON: Process 4 floats at once
    // ... NEON quantization
}
```

**Success Criteria:**
- ✅ AVX2 implementation works
- ✅ NEON implementation works
- ✅ SSE2 fallback works
- ✅ Runtime CPU detection
- ✅ Test: Verify same results as scalar

---

### Phase 5: Integration (Week 12 Day 4)

**5.1. HNSW Integration**

```rust
// Replace BinaryQuantization in HNSWIndex
pub struct HNSWIndex {
    // ...
    quantizer: Option<ExtendedRaBitQ>,
}

impl HNSWIndex {
    pub fn insert(&mut self, vector: Vec<f32>) -> Result<usize> {
        let id = self.next_id();

        // Store original vector
        self.storage.store_vector(id, vector.clone())?;

        // Quantize for fast search
        if let Some(ref quantizer) = self.quantizer {
            let (quantized, scale) = quantizer.quantize(&vector);
            self.storage.store_quantized(id, quantized, scale)?;
        }

        // Insert into graph (using quantized for distance)
        self.insert_into_graph(id)?;
        Ok(id)
    }
}
```

**5.2. Two-Phase Search**

```rust
pub fn search(
    &self,
    query: &[f32],
    k: usize,
    ef: usize,
) -> Result<Vec<Neighbor>> {
    // Phase 1: Fast search with quantized vectors
    let candidates = self.search_graph_quantized(query, ef * 3)?;

    // Phase 2: Rerank with original vectors
    let reranked = self.rerank(query, candidates, k)?;

    Ok(reranked)
}
```

**Success Criteria:**
- ✅ Quantizer integrated into HNSWIndex
- ✅ Two-phase search works
- ✅ Persistence updated (save/load quantized vectors)
- ✅ Test: Search results match expected recall

---

### Phase 6: Benchmarks & Validation (Week 12 Day 5)

**6.1. Accuracy Benchmarks**

```bash
# Test different compression rates
./benchmark_extended_rabitq --bits 2  # 16x compression
./benchmark_extended_rabitq --bits 3  # ~10x compression
./benchmark_extended_rabitq --bits 4  # 8x compression
./benchmark_extended_rabitq --bits 8  # 4x compression (baseline)
```

**Metrics:**
- Recall@10 at different compression rates
- Distance correlation with ground truth
- Reranking improvement

**6.2. Performance Benchmarks**

```bash
# Compare vs binary quantization
./benchmark_quantization_comparison
```

**Metrics:**
- Quantization time (build overhead)
- Query latency (should be same as BQ)
- Memory usage at different compression rates

**6.3. Scale Validation**

```bash
# Test at 100K, 1M scale
./benchmark_extended_rabitq_scale
```

**Success Criteria:**
- ✅ 2-bit: Recall > 70% (vs 33% for 1-bit)
- ✅ 4-bit: Recall > 85%
- ✅ 8-bit: Recall > 95%
- ✅ Query speed: Same as binary quantization
- ✅ Quantization overhead: <10% of total build time

---

## File Structure

```
src/vector/
├── quantization/
│   ├── mod.rs                    # Public API
│   ├── extended_rabitq.rs        # Core algorithm
│   ├── simd_quantization.rs      # SIMD optimizations
│   ├── error.rs                  # Error types
│   └── packing.rs                # Bit packing utilities
│
src/bin/
├── benchmark_extended_rabitq.rs  # Accuracy benchmark
├── benchmark_quantization_comparison.rs  # vs BQ
└── benchmark_extended_rabitq_scale.rs    # Scale test

tests/
└── test_extended_rabitq.rs       # Unit tests
```

---

## Technical Challenges

### Challenge 1: Rescaling Search Efficiency

**Problem**: Testing 8-16 rescaling factors per vector is expensive.
**Solution**:
- Use logarithmic spacing for scale factors
- Cache error computations
- SIMD parallel evaluation of multiple scales

### Challenge 2: Bit Packing

**Problem**: Packing 2-9 bits per dimension efficiently.
**Solution**:
- Specialized packing for common cases (2, 4, 8 bits)
- General bit-packing for 3, 5, 7, 9 bits
- SIMD bit manipulation where possible

### Challenge 3: SIMD Across Architectures

**Problem**: AVX2 (x86), NEON (ARM), SSE2 (fallback).
**Solution**:
- Runtime CPU detection (like simd_distance.rs)
- Separate implementations per architecture
- Thorough testing on both platforms

### Challenge 4: Backward Compatibility

**Problem**: Existing code uses binary quantization.
**Solution**:
- Keep BinaryQuantization as special case (1-bit)
- Unified QuantizationMethod enum
- Migration path: load old BQ, save as Extended RaBitQ

---

## Success Criteria

### Phase 1 Complete When:
- ✅ Data structures defined and tested
- ✅ Memory layout documented
- ✅ Compression calculations validated

### Phase 2 Complete When:
- ✅ Quantization algorithm works
- ✅ Optimal scale search validated
- ✅ Error minimization correct

### Phase 3 Complete When:
- ✅ Distance computation works
- ✅ Correlates with ground truth
- ✅ Test suite passing

### Phase 4 Complete When:
- ✅ SIMD implementations work (AVX2, NEON, SSE2)
- ✅ Same results as scalar code
- ✅ Performance improvement measured

### Phase 5 Complete When:
- ✅ Integrated into HNSWIndex
- ✅ Two-phase search works
- ✅ Persistence works
- ✅ All existing tests pass

### Phase 6 Complete When:
- ✅ Accuracy benchmarks show 2x+ improvement at 2-bit
- ✅ Query speed matches binary quantization
- ✅ Memory usage validated
- ✅ Works at 100K, 1M scale

---

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Quantization too slow | High | SIMD optimization, cache scales |
| SIMD bugs | Medium | Extensive testing, scalar fallback |
| Memory overhead | Medium | Efficient bit packing |
| Integration breaks existing code | Low | Backward compatible design |

---

## References

- **Paper**: [Extended RaBitQ: Practical and Asymptotically Optimal Quantization](https://dl.acm.org/doi/10.1145/3725413)
- **ArXiv**: https://arxiv.org/pdf/2409.09913
- **GitHub**: https://github.com/VectorDB-NTU/Extended-RaBitQ
- **Blog**: https://dev.to/gaoj0017/extended-rabitq-an-optimized-scalar-quantization-method-83m
- **Original RaBitQ**: https://dl.acm.org/doi/10.1145/3654970

---

## Timeline Summary

| Phase | Days | Description |
|-------|------|-------------|
| 1. Data Structures | 1 | Define quantization config, storage |
| 2. Core Algorithm | 2 | Rescaling, grid rounding, error |
| 3. Distance Computation | 1 | Dequantization, distance approx |
| 4. SIMD Optimizations | 1 | AVX2, NEON, SSE2 |
| 5. Integration | 1 | HNSW integration, persistence |
| 6. Benchmarks | 1 | Accuracy, performance, scale |
| **Total** | **7 days** | **Week 11 Day 4 - Week 12 Day 5** |

---

**Status**: Ready to implement (research complete)
**Next Step**: Begin Phase 1 (Data Structures)
