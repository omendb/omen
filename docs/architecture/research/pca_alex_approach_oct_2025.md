# PCA-ALEX Approach for High-Dimensional Vector Indexing

**Research Date**: October 22, 2025
**Status**: EXPERIMENTAL MOONSHOT
**Goal**: Achieve 10-30x memory efficiency vs HNSW with >90% recall@10

---

## Executive Summary

This document outlines an experimental approach combining PCA (Principal Component Analysis) dimensionality reduction with ALEX (Adaptive Learned Index) for high-dimensional vector search. **This is untested in production** and represents a high-risk, high-reward moonshot attempt.

**Key Insight**: Week 1 prototype showed simple 1D projection fails (5% recall). Academic research (LIDER) proves dimensionality reduction + learned indexes CAN work, but uses LSH not PCA. We're betting PCA is simpler and may work equally well.

**Risk Level**: HIGH (50-60% success rate estimated)
**Fallback**: HNSW (proven, 95%+ success rate)
**Timeline**: 3-4 weeks to full implementation + validation

---

## Research Context

### What We Know (Validated)

**Week 1 Prototype Results**:
- ✅ **Memory**: 6,146 bytes/vector (2-13 bytes overhead) - Excellent
- ✅ **Latency**: 0.58-5.73ms (17-22x speedup) - Excellent
- ❌ **Recall**: 5% recall@10 (target >90%) - Catastrophic failure

**Root Cause**: Simple projection (sum of first 4 dimensions) loses 99.7% of information, destroying nearest-neighbor relationships.

### Academic Validation

**LIDER Paper** (VLDB 2023, arXiv:2205.00970):
- **Method**: SK-LSH (locality-sensitive hashing) + RMI (learned index)
- **Results**: 1.2x faster than HNSW, high recall, memory efficient
- **Key Finding**: Dimensionality reduction + learned index IS viable

**2025 Survey** (arXiv:2403.06456):
- "ML-enhanced variants of traditional high-dimensional indexes"
- Confirms learned indexes struggle with high dimensions WITHOUT reduction
- Multiple approaches exist (LSH, PCA, quantization)

**Dimensionality Reduction Survey** (arXiv:2403.13491, Feb 2025):
- PCA still widely used for ANN search (classical but effective)
- Modern alternatives: deep learning autoencoders, transformer-based
- Trade-off: complexity vs. effectiveness

### Our Hypothesis: PCA-ALEX

**Why PCA instead of LSH**:
1. **Simpler**: Mature Rust library exists (linfa_reduction)
2. **Deterministic**: Same input always produces same output (LSH is probabilistic)
3. **Explainable**: Principal components have statistical meaning
4. **Fast**: O(D²) training, O(D×d) projection (D=1536, d=64)

**Why This Might Work**:
- Embeddings are clustered by semantic meaning (not random)
- PCA preserves maximum variance (= semantic similarity structure)
- ALEX learns models on PCA space (64D much easier than 1536D)
- Multi-level hierarchy: coarse-to-fine search

**Why This Might Fail**:
- PCA is linear, embeddings may have non-linear structure
- No academic validation for PCA+ALEX combo (we're inventing this)
- 1536D → 64D may lose too much information (need 90%+ variance preserved)

---

## Technical Approach

### Architecture: 3-Level PCA-ALEX

```
Input: 1536-dimensional embedding vector
  ↓
PCA Projection: 1536D → 64D (preserve 90%+ variance)
  ↓
Level 1 (Root): Linear model on first 16 PCA components
  Predicts: Which of 64 Level-2 nodes to traverse
  ↓
Level 2 (Internal): 64 linear models on next 32 PCA components
  Predicts: Which of 4096 Level-3 buckets to traverse
  ↓
Level 3 (Leaf): 4096 buckets, ~2,500 vectors each (10M total)
  Searches: Brute-force L2 distance in original 1536D space
  Returns: Top-K nearest neighbors
```

### Key Design Decisions

**1. PCA Target Dimensionality**

**Options**:
- 32D: Fast, may lose too much info (80-85% variance)
- 64D: Balanced, target 90-95% variance
- 128D: Safe, 95-98% variance, slower

**Choice**: Start with 64D
- Rationale: 64D gives 40x reduction (1536→64), likely 90%+ variance
- Fallback: If recall <80%, try 128D

**2. PCA Training Strategy**

**Training Set**: Random sample of 100K vectors
- Rationale: PCA training is O(n×D²), 100K is fast (<10 seconds)
- Retraining: Every 1M new vectors (10-30 seconds)

**Incremental PCA**: Use online PCA for updates
- Library: linfa_reduction supports incremental mode
- Benefit: No full retraining needed

**3. ALEX Model Complexity**

**Level 1**: Simple linear model (16 inputs → 64 outputs)
- Model: y = w₀ + w₁×x₁ + ... + w₁₆×x₁₆
- Training: Least squares regression on 100K samples

**Level 2**: 64 independent linear models (32 inputs → 64 outputs each)
- Partitioned by Level 1 prediction
- Each model trained on ~1,500 vectors (faster than root)

**Level 3**: No models, just hash buckets
- Direct mapping: bucket_id = model_prediction(pca_vector)
- Bucket size: 1000-5000 vectors (tunable)

**4. Search Strategy**

**Single-bucket search** (optimistic):
- Search only the predicted bucket
- Fastest, but recall may suffer

**Multi-bucket search** (recommended):
- Search top-3 predicted buckets (model uncertainty)
- 3x work, but likely 95%+ recall
- Still <20ms (3 × 2,500 vectors × 0.002ms/vector)

**Adaptive search** (fallback):
- Start with 1 bucket, expand if <K results
- Dynamic based on query difficulty

**5. Distance Computation**

**PCA space vs Original space**:
- ALEX searches in 64D PCA space (fast)
- Final ranking in 1536D original space (accurate)
- Hybrid: Coarse filter (PCA) + Fine ranking (original)

**Why this works**:
- PCA preserves distances approximately (variance = distance structure)
- Top-100 candidates in PCA space likely contain true top-10
- Brute-force 100 vectors in 1536D is still <1ms

---

## Implementation Plan

### Phase 1: PCA Integration (Days 1-2)

**Tasks**:
1. Add linfa dependencies to Cargo.toml:
   ```toml
   linfa = "0.7"
   linfa-reduction = "0.7"
   ndarray = "0.15"
   ndarray-linalg = "0.16"
   ```

2. Implement PCA module (src/pca.rs):
   - Train PCA on sample vectors
   - Project vectors to 64D
   - Serialize/deserialize PCA model (save to RocksDB)

3. Test PCA variance preservation:
   - Train on 100K vectors
   - Measure explained variance (target >90%)
   - Benchmark projection time (target <10μs/vector)

**Success Criteria**:
- ✅ PCA library compiles and runs
- ✅ Can project 1536D → 64D vectors
- ✅ 90%+ variance preserved
- ✅ <10μs projection time

### Phase 2: ALEX for PCA Vectors (Days 3-5)

**Tasks**:
1. Adapt existing ALEX (src/alex/) for 64D PCA vectors:
   - Modify key type: i64 → Vec<f32, 64>
   - Update linear models: 1D → 16D/32D
   - Implement training on PCA space

2. Build 3-level hierarchy:
   - Level 1: Root model (16 PCA dims → 64 nodes)
   - Level 2: 64 internal models (32 PCA dims → 64 buckets each)
   - Level 3: 4096 leaf buckets (raw vectors)

3. Implement bucket search:
   - Multi-bucket search (top-3 buckets)
   - Brute-force L2 distance in original 1536D space
   - Return top-K results

**Success Criteria**:
- ✅ Can insert 100K PCA-projected vectors
- ✅ Can query nearest neighbors (correctness TBD)
- ✅ 3-level hierarchy builds successfully

### Phase 3: Benchmark & Validation (Days 6-7)

**Dataset**: 100K OpenAI embeddings (1536 dimensions)

**Metrics**:
1. **Memory**:
   - PCA model: ~1536×64×4 bytes = 393KB (negligible)
   - ALEX overhead: Target <50 bytes/vector
   - Total: Target <2MB for 100K vectors

2. **Latency**:
   - PCA projection: <10μs
   - ALEX search: <5ms (target)
   - Brute-force ranking: <1ms (100 candidates)
   - Total: <10ms p95 (competitive with HNSW)

3. **Recall@10**:
   - Ground truth: Brute-force search in original space
   - PCA-ALEX: Multi-bucket search
   - Target: >90% recall@10

**Benchmarking Code**:
```rust
// Pseudo-code
for query_vector in test_queries {
    // Ground truth
    let true_neighbors = brute_force_search(all_vectors, query_vector, k=10);

    // PCA-ALEX
    let pca_query = pca.project(query_vector);
    let candidate_buckets = alex.search_top_k_buckets(pca_query, k=3);
    let candidates = get_vectors_from_buckets(candidate_buckets);
    let pred_neighbors = rerank_by_l2_distance(candidates, query_vector, k=10);

    // Measure recall
    let recall = count_overlap(true_neighbors, pred_neighbors) / 10.0;
}
```

**Success Criteria**:
- ✅ Memory: <100 bytes/vector (5x better than HNSW)
- ✅ Latency: <20ms p95 (acceptable vs HNSW's <10ms)
- ✅ Recall@10: >90% (production-ready)

### Phase 4: Go/No-Go Decision (Day 7)

**Proceed with PCA-ALEX if**:
- Recall@10 >90% ✅ Revolutionary! Continue to 1M scale
- Memory <100 bytes/vector ✅ Clear advantage vs HNSW
- Latency <20ms p95 ✅ Acceptable for production

**Adjust and retry if**:
- Recall@10: 80-90% → Increase PCA dims to 128D, re-benchmark
- Memory: 100-200 bytes/vector → Optimize bucket size, re-benchmark
- Latency: 20-50ms → Reduce buckets searched, trade recall for speed

**Pivot to HNSW if**:
- Recall@10 <80% ❌ PCA loses too much information
- Memory >200 bytes/vector ❌ No advantage vs HNSW
- Latency >50ms p95 ❌ Too slow for production
- Can't find tuning that balances all three metrics

---

## Risk Assessment

### Technical Risks

**1. PCA Information Loss** ⚠️ HIGH RISK
- **Problem**: 1536D → 64D may destroy nearest-neighbor structure
- **Probability**: 40-50% (unknown, no validation)
- **Impact**: Catastrophic (recall <80% = unusable)
- **Mitigation**:
  - Measure variance preservation (target >90%)
  - Fall back to 128D if 64D fails
  - Ultimate fallback: HNSW

**2. Model Accuracy in PCA Space** ⚠️ MEDIUM RISK
- **Problem**: Linear models may not partition PCA embeddings well
- **Probability**: 30-40%
- **Impact**: High (low recall, many buckets searched)
- **Mitigation**:
  - Multi-bucket search (top-3 buckets)
  - Adaptive bucket expansion

**3. Query Latency** ⚠️ MEDIUM RISK
- **Problem**: Brute-force search in multiple buckets may be slow
- **Probability**: 20-30%
- **Impact**: Medium (still faster than pgvector, but not as fast as HNSW)
- **Mitigation**:
  - SIMD for L2 distance computation
  - Reduce bucket size (1000 instead of 2500 vectors)

**4. Training Complexity** ⚠️ LOW RISK
- **Problem**: PCA + multi-level models = complex training
- **Probability**: 10-20%
- **Impact**: Low (offline cost, not query-time)
- **Mitigation**:
  - Batch training (overnight for 10M vectors)
  - Incremental PCA for updates

### Market Risks

**1. Time Investment** ⚠️ MEDIUM RISK
- **Problem**: 3-4 weeks to validate, may fail
- **Probability**: 50-60% (fails → 3 weeks lost)
- **Impact**: Medium (delays go-to-market by 1 month)
- **Mitigation**:
  - Clear go/no-go at Day 7 (1 week, not 3)
  - Fast pivot to HNSW (1 week implementation)
  - Total: 2-4 weeks vs 1-2 weeks for HNSW-first

**2. Differentiation** ⚠️ LOW RISK
- **Problem**: If PCA-ALEX fails, lose "revolutionary" narrative
- **Probability**: 50-60%
- **Impact**: Low (PostgreSQL compatibility still unique)
- **Mitigation**:
  - HNSW still 10x better than pgvector
  - Focus on PostgreSQL + scale narrative

### Success Probability

**Overall Confidence**: 40-50% PCA-ALEX works at >90% recall

**Best Case** (40-50% probability):
- Recall >90%, memory <50 bytes/vector, latency <10ms
- Revolutionary positioning: "First learned index vector DB"
- 10-30x memory advantage vs HNSW
- Hard-to-replicate moat

**Expected Case** (30-40% probability):
- Recall 80-90%, memory 50-100 bytes/vector, latency 10-20ms
- Good but not revolutionary
- 2-5x memory advantage vs HNSW
- Consider hybrid ALEX+HNSW

**Worst Case** (20-30% probability):
- Recall <80%, memory >100 bytes/vector, latency >20ms
- PCA-ALEX doesn't work
- Pivot to HNSW (3 weeks lost, but product still viable)

---

## Alternative: LIDER Approach (SK-LSH + RMI)

**What LIDER Actually Uses**:
- SK-LSH (SortingKeys Locality-Sensitive Hashing)
- Converts high-dim vectors → 1D sortable keys
- RMI (Recursive Model Index) on 1D keys
- 1.2x faster than HNSW (validated in paper)

**Why We're Not Doing This**:
1. **Complexity**: SK-LSH has no Rust library, need to implement from scratch
2. **Time**: 4-6 weeks to implement + validate (vs 3-4 for PCA-ALEX)
3. **Uncertainty**: LIDER paper is dense passage retrieval (768D), not 1536D OpenAI embeddings
4. **PCA is simpler**: Mature library, well-understood, deterministic

**When to Consider SK-LSH**:
- If PCA-ALEX fails (recall <80%)
- If we have 2-3 months to implement (not rushing)
- If HNSW also has issues (unlikely)

---

## Competitive Implications

### If PCA-ALEX Works (>90% recall)

**Differentiation**:
1. ✅ **World's first learned index vector DB** (truly novel)
2. ✅ **10-30x memory efficiency** (vs Pinecone, Weaviate, Qdrant)
3. ✅ **PostgreSQL compatibility** (unique vs all competitors)
4. ✅ **Self-hostable** (vs Pinecone cloud-only)

**Positioning**:
> "OmenDB: The only PostgreSQL-compatible vector database with learned indexes.
> 10x faster than pgvector, 30x more memory efficient, 90% cheaper than Pinecone."

**Marketing**:
- Hacker News: "Show HN: First production learned index for vector search"
- Academic ML/AI community: Novel contribution
- Cost-conscious startups: "100M vectors in <2GB RAM"

### If PCA-ALEX Fails (<80% recall) → HNSW Fallback

**Differentiation** (still strong):
1. ✅ **PostgreSQL compatibility** (unique vs Pinecone/Weaviate)
2. ✅ **30x memory vs pgvector** (HNSW 100 bytes vs pgvector 6000 bytes)
3. ✅ **MVCC + ACID** (unique vs pure vector DBs)
4. ✅ **Self-hostable** (vs Pinecone)

**Positioning**:
> "OmenDB: PostgreSQL-compatible vector database that actually scales.
> Drop-in replacement for pgvector. 10x faster, 30x less memory."

**Marketing**:
- PostgreSQL community: "pgvector but production-ready"
- Enterprises: Compliance-friendly, self-hosted
- Cost-conscious: "Same features, 1/10th Pinecone cost"

---

## Conclusion

**PCA-ALEX is a high-risk, high-reward moonshot** with 40-50% estimated success rate. The upside (10-30x memory efficiency, unique positioning) justifies 1-week investigation. Clear go/no-go criteria ensure we pivot to HNSW quickly if it fails.

**Recommendation**: PROCEED WITH PCA-ALEX PROTOTYPE
- Week 1 (Day 1-7): Implement and benchmark 100K vectors
- Decision point: Day 7 (proceed, adjust, or pivot to HNSW)
- Worst case: 1 week lost, still have proven HNSW fallback
- Best case: Revolutionary learned index vector database

**Risk-Adjusted Strategy**:
- 50% chance: PCA-ALEX works → unique technical moat
- 50% chance: PCA-ALEX fails → HNSW still 10x better than pgvector
- 100% chance: Ship production-ready vector database in 4-6 months

---

**Research Date**: October 22, 2025
**Next Steps**: Begin Phase 1 (PCA integration) immediately
**Decision Point**: Day 7 (October 29, 2025)
**Fallback**: HNSW implementation (1-2 weeks, proven approach)

---

## References

1. **LIDER Paper**: Wang et al., "LIDER: An Efficient High-dimensional Learned Index for Large-scale Dense Passage Retrieval", VLDB 2023, arXiv:2205.00970

2. **Learned Indexes Survey**: "A Survey of Learned Indexes for the Multi-dimensional Space", arXiv:2403.06456v1, March 2024

3. **Dimensionality Reduction Survey**: "Dimensionality-Reduction Techniques for Approximate Nearest Neighbor Search: A Survey and Evaluation", arXiv:2403.13491, February 2025

4. **Linfa PCA Documentation**: https://rust-ml.github.io/linfa/rustdocs/linfa_reduction/struct.Pca.html

5. **ALEX Paper**: Ding et al., "ALEX: An Updatable Adaptive Learned Index", SIGMOD 2020, arXiv:1905.08898
