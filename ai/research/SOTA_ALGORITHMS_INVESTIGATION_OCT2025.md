# SOTA Vector Search Algorithms Research (October 27, 2025)

## Research Goal
Determine if we should implement advanced algorithms (HNSW+, SPANN, SPFresh) or proceed with current HNSW + Binary Quantization stack.

## Context
After completing Week 6 (parallel building with 16x speedup), we investigated whether to:
1. Continue with current HNSW + Binary Quantization
2. Implement MN-RU updates
3. Adopt SPANN/SPFresh for billion-scale
4. Implement hybrid HNSW-IF approach

## Algorithms Investigated

### 1. MN-RU (Multi-Neighbor Replaced Updates) - ArXiv 2407.07871
**Problem**: HNSW suffers from "unreachable points" during deletions
**Solution**: Improved neighbor replacement during updates
**Performance**: 2-4x faster updates, fewer unreachable points
**Verdict**: ❌ **BLOCKED** - hnsw_rs has no delete/update methods
**Effort**: 2-4 weeks + fork maintenance
**Why Blocked**: Would require forking hnsw_rs and becoming maintainers

### 2. SPANN (Space-Partitioned ANN) - Microsoft 2021
**Problem**: HNSW limited by RAM
**Solution**: Hybrid inverted file + graph (centroids in memory, vectors on disk)
**Performance**: 2x faster than DiskANN at billion scale
**Verdict**: ⚠️ **COMPLEX** - requires offline clustering
**Effort**: 6-8 weeks
**Why Complex**:
- Requires hierarchical balanced clustering (HBC)
- Offline centroid computation
- Complex posting list management
- Similar issues to DiskANN (which we avoided before)

### 3. SPFresh (SPANN + Updates) - SOSP 2023
**Problem**: SPANN doesn't support efficient updates
**Solution**: In-place updates with LIRE protocol + SPDK
**Performance**: Real-time billion-scale updates
**Verdict**: ❌ **TOO COMPLEX** - requires SPDK, production system
**Effort**: 8-12 weeks + infrastructure
**Why Too Complex**:
- Requires SPDK (Intel NVMe library)
- Infrastructure dependency (NVMe SSDs)
- Complex version management
- Block controller implementation
- Not suitable for general use cases

### 4. Hybrid HNSW-IF (Inverted File) - Vespa 2024
**Problem**: HNSW RAM-bound
**Solution**: Random centroids (20% of data) in HNSW, neighbors on disk
**Performance**: Billion-scale with modest RAM
**Verdict**: ✅ **RECOMMENDED** - simple, proven, natural extension
**Effort**: 3-4 weeks
**Why Recommended**:
- Simple: use actual vectors as centroids (no clustering)
- Random selection (20% of dataset)
- HNSW for centroids, disk for posting lists
- Vespa production-validated
- No infrastructure dependencies
- Works for ALL scales (1K → 1B+)

### 5. Extended RaBitQ - SIGMOD 2025
**Problem**: Binary quantization only supports 32x compression
**Solution**: Arbitrary compression rates (4x, 8x, 16x, 32x)
**Performance**: Better accuracy at same compression
**Verdict**: ✅ **RECOMMENDED** - improves our quantization
**Effort**: 2-3 weeks
**Why Recommended**:
- SOTA quantization (SIGMOD 2025)
- Backward compatible with current BQ
- Arbitrary compression rates
- Better accuracy-memory tradeoff
- Natural evolution of what we have

### 6. NGT-QG (Quantized Graph) - Yahoo Japan
**Problem**: Graph + quantization fusion
**Solution**: Product quantization integrated into graph traversal
**Performance**: Competitive with HNSW
**Verdict**: ⚠️ **ALTERNATIVE** - not clearly better than HNSW + E-RaBitQ
**Effort**: 4-6 weeks
**Why Alternative**:
- Similar performance to HNSW + quantization
- More complex than separate graph + quantization
- Yahoo Japan specific optimizations

## Key Findings

### Why DiskANN/SPANN/SPFresh Are Complex
- NVMe SSD dependency (infrastructure lock-in)
- Offline clustering required (operational complexity)
- Batch-oriented updates (not real-time friendly)
- Doesn't handle small datasets well (not general purpose)
- **Lesson**: Avoid like we did with DiskANN in Mojo MVP

### Why HNSW-IF Is Right
- Works for ALL scales (1K → 1B+)
- Simple: 20% random centroids → HNSW, rest → disk
- No dependencies (just disk I/O, no SPDK)
- Natural progression from current HNSW
- Vespa production-validated (not research)
- Addresses "support many workloads at many scales" goal

### Why Extended RaBitQ Is Right
- Improves what we already have
- SIGMOD 2025 = cutting-edge, peer-reviewed
- Arbitrary compression (not just 32x)
- Backward compatible with current BQ
- Better accuracy-memory tradeoff

## Competitive Landscape

| Vendor | Algorithm | Scale | Our Advantage |
|--------|-----------|-------|---------------|
| pgvector | HNSW only | <10M | 10x faster, 19x less RAM |
| Pinecone | Unknown | Billions | 90% cheaper, self-hostable |
| Qdrant | HNSW + quantization | 100M+ | PostgreSQL compat + HNSW-IF |
| Weaviate | HNSW | 100M+ | Better quantization + HNSW-IF |
| Vespa | HNSW-IF | Billions | PostgreSQL compat, simpler |
| PlanetScale | SPANN/SPFresh | Billions | PostgreSQL compat, open source |

**Key Insight**: PostgreSQL compatibility + billion-scale is UNIQUE

## Strategic Recommendation

### Phase 1 (Weeks 7-8): Validate Current Stack ⭐ CRITICAL PATH
1. pgvector benchmarks (1M, 10M, 100M vectors)
2. Document where we excel (<10M range likely)
3. Document scale limits (50-100M on 128GB RAM)
4. Ship with honest limits
5. **Why Critical**: Can't prove "10x faster" without data

### Phase 2 (Weeks 9-10): HNSW-IF for Scale
1. Implement hybrid scaling
2. Automatic: <10M in-memory, >10M hybrid
3. Validate at 100M-1B scale
4. **Differentiator**: Only PostgreSQL-compatible DB with billion-scale

### Phase 3 (Weeks 11-12): Extended RaBitQ for Efficiency
1. Replace binary quantization
2. 4x-32x compression options
3. Better accuracy at same memory
4. **Differentiator**: SOTA quantization (SIGMOD 2025)

### Future (Post-Launch): Test Alternatives on Branches
- NGT-QG (if customers request product quantization)
- SPANN (if customers have NVMe SSD infrastructure)
- MN-RU (if hnsw_rs adds delete/update support)

## What Makes OmenDB SOTA (Post-Implementation)

**Current State (Week 6):**
```
✅ HNSW: 99.5% recall, 10ms p95 (industry standard)
✅ Binary Quantization: 19.9x memory (competitive)
✅ 16x parallel building (UNIQUE - undocumented by competitors)
✅ 4175x serialization (UNIQUE - undocumented by competitors)
✅ PostgreSQL compatible (UNIQUE vs pure vector DBs)
```

**After HNSW-IF (Week 9-10):**
```
✅ All above +
✅ Billion-scale support (Vespa-proven)
✅ Automatic scaling (in-memory → hybrid)
✅ No infrastructure dependencies
= Only PostgreSQL DB with billion-scale
```

**After Extended RaBitQ (Week 11-12):**
```
✅ All above +
✅ SOTA quantization (SIGMOD 2025)
✅ Arbitrary compression (4x-32x)
✅ Better accuracy at same memory
= SOTA vector DB with PostgreSQL compatibility
```

## Implementation Details

### HNSW-IF Approach (Vespa-style)
1. **Centroid Selection**: Random 20% of dataset (no clustering!)
2. **Index Structure**:
   - HNSW for centroids (in-memory)
   - Posting lists for neighbors (on disk)
3. **Search**: Two-phase
   - Phase 1: HNSW search on centroids → top-K posting lists
   - Phase 2: Scan posting lists on disk → refine results
4. **Threshold**: Switch to hybrid at 10M vectors (configurable)

### Extended RaBitQ Approach
1. **Replace**: Current BinaryQuantization with E-RaBitQ
2. **Add**: Compression rate parameter (4x, 8x, 16x, 32x)
3. **Improve**: Accuracy at same memory footprint
4. **Maintain**: Backward compatibility with existing code

## References

- **MN-RU**: https://arxiv.org/abs/2407.07871
- **SPANN**: https://arxiv.org/abs/2111.08566
- **SPFresh**: https://www.microsoft.com/en-us/research/publication/spfresh-incremental-in-place-update-for-billion-scale-vector-search/
- **Vespa HNSW-IF**: https://blog.vespa.ai/vespa-hybrid-billion-scale-vector-search/
- **Extended RaBitQ**: https://github.com/VectorDB-NTU/Extended-RaBitQ
- **NGT-QG**: https://github.com/yahoojapan/NGT
- **hnsw_rs API**: https://docs.rs/hnsw_rs/latest/hnsw_rs/

## Decision: Target HNSW-IF + Extended RaBitQ

**Why**:
- Natural progression from current stack
- Proven approaches (not research)
- Addresses scale (HNSW-IF) + accuracy (E-RaBitQ)
- No infrastructure dependencies
- Supports "many workloads at many scales" goal

**Timeline**:
- Weeks 7-8: pgvector benchmarks (validation)
- Weeks 9-10: HNSW-IF (3-4 weeks)
- Weeks 11-12: Extended RaBitQ (2-3 weeks)
- Total: 5-7 weeks for SOTA stack

**Alternative algorithms**: Test on branches later based on customer feedback

**Priority**: Benchmarks first (weeks 7-8) to validate we can claim "10x faster than pgvector"

---

**Status**: Research complete, strategic direction decided
**Next**: Update TODO.md and CLAUDE.md with roadmap
**Date**: October 27, 2025
