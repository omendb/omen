# Custom HNSW Implementation Decision

**Date**: October 30, 2025
**Decision**: Build custom SOTA HNSW implementation
**Rationale**: All serious competitors use custom implementations for maximum performance and SOTA features

---

## Executive Summary

**Recommendation**: **Build custom HNSW implementation** (Phase 1: Start Week 2-3)

**Why**:
1. hnsw_rs is limiting (no delete/update, limited customization)
2. ALL serious competitors use custom implementations
3. SOTA features (Extended RaBitQ, MN-RU, HNSW-IF) require deep integration
4. Performance ceiling: Custom gives us 2-5x additional headroom

**Timeline**:
- Short-term (Week 1): Use hnsw_rs with SIMD enabled
- Medium-term (Week 2-4): Build custom HNSW core
- Long-term (Week 5+): Add SOTA features (Extended RaBitQ, HNSW-IF, MN-RU)

---

## Competitor Analysis: What Do They Use?

### Qdrant (Performance Leader)

**Implementation**: Custom Rust HNSW
- **NOT** using any library
- Full custom implementation
- GPU-accelerated (10x faster indexing)
- Delta encoding (30% memory reduction)
- Advanced filtering (<10% overhead)
- **Source**: https://qdrant.tech/blog/qdrant-1.13.x/

**Why custom**: Maximum performance control, GPU integration, custom optimizations

### Milvus (Scale Leader)

**Implementation**: Knowhere (Custom C++ library)
- Wraps FAISS/hnswlib but heavily modified
- Custom SIMD (SSE, AVX, AVX2, AVX512)
- 20-30% performance improvement from SIMD
- Custom bitset mechanism (soft deletes)
- Not using vanilla hnswlib
- **Source**: https://github.com/milvus-io/knowhere

**Why custom**: SIMD control, soft deletes, enterprise features

### Weaviate

**Implementation**: Custom Go HNSW
- Custom implementation in Go
- GraphQL integration
- Hybrid search support
- **Why custom**: Language integration, feature control

### LanceDB

**Implementation**: Custom Rust
- Built from ground up in Rust
- Columnar format (Arrow/Parquet)
- **Why custom**: Embedded architecture, performance

### Summary

**ALL serious competitors use custom implementations**

| Database | Implementation | Reason |
|----------|---------------|--------|
| Qdrant | Custom Rust | GPU, delta encoding, max performance |
| Milvus | Custom C++ (Knowhere) | SIMD, soft deletes, enterprise |
| Weaviate | Custom Go | Language integration, features |
| LanceDB | Custom Rust | Embedded, columnar format |
| pgvector | Custom C | PostgreSQL integration |

**Verdict**: Library-based approach (like our current hnsw_rs) is NOT used by any top competitor

---

## hnsw_rs Limitations

### Current State

**What hnsw_rs provides** ✅:
- HNSW implementation
- SIMD support (simdeez_f feature)
- Parallel building
- Graph serialization
- Good performance

**What hnsw_rs CANNOT do** ❌:
1. **No delete/update support** (blocks MN-RU algorithm)
2. **No hybrid memory/disk** (blocks HNSW-IF for billion-scale)
3. **Limited quantization integration** (Extended RaBitQ needs deep hooks)
4. **No GPU acceleration** (Qdrant has this)
5. **Limited customization** (can't modify core algorithm)
6. **No filtered search optimization** (Qdrant: <10% overhead)

### Performance Ceiling

**With hnsw_rs** (after enabling SIMD):
- Estimated: 300-500 QPS
- Limited by library's design decisions
- Can't implement SOTA features

**With Custom HNSW**:
- Potential: 1000-2000 QPS (Qdrant-level)
- Full control over memory layout
- SOTA algorithms (Extended RaBitQ, HNSW-IF, MN-RU)
- GPU acceleration (optional)

**Gap**: 2-5x additional performance possible with custom

---

## SOTA Features Requiring Custom Implementation

### 1. Extended RaBitQ (SIGMOD 2025)

**What it needs**:
- Deep integration with HNSW distance calculations
- Custom quantization hooks during graph traversal
- Arbitrary compression rates (4x-32x)

**Why hnsw_rs can't do it**:
- Library's distance calculation is opaque
- No hooks for custom quantization
- Would need to fork and heavily modify

**Impact**: Better accuracy at same memory (10-20% improvement)

### 2. HNSW-IF (Hybrid Memory/Disk)

**What it needs**:
- Layer-aware memory management
- Keep hot layers (top 2-3) in memory
- Store cold layers on disk with efficient I/O
- Custom prefetching and caching

**Why hnsw_rs can't do it**:
- Fully in-memory design
- No disk-backed storage support
- No layer-level control

**Impact**: Billion-scale on single node

### 3. MN-RU (Delete/Update Support)

**What it needs**:
- Delete node from graph
- Update node connections
- Maintain graph quality

**Why hnsw_rs can't do it**:
- No delete/update API
- Immutable graph design
- Would require complete rewrite

**Impact**: Production-ready (updates without full rebuild)

### 4. Filtered Search

**What it needs**:
- Metadata filtering during traversal
- Skip filtered-out nodes efficiently
- <10% overhead (Qdrant benchmark)

**Why hnsw_rs can't do it**:
- No filtering hooks
- Would need callback system

**Impact**: Competitive parity with Qdrant/Milvus

### 5. GPU Acceleration

**What it needs**:
- CUDA/ROCm for distance calculations
- Batch processing on GPU
- Memory transfer optimization

**Why hnsw_rs can't do it**:
- CPU-only
- No GPU support

**Impact**: 10x faster indexing (Qdrant has this)

---

## Custom Implementation Effort

### Phase 1: Core HNSW (2-3 weeks)

**Scope**:
- Basic HNSW algorithm (insert, search)
- L2 and cosine distance
- Parallel building (Rayon)
- Graph serialization

**Effort**: 2-3 weeks
**Risk**: Medium (well-understood algorithm)
**Benefit**: Full control, foundation for SOTA features

### Phase 2: SIMD + Optimization (1 week)

**Scope**:
- SIMD distance calculations (AVX2/AVX-512/NEON)
- Prefetching and cache optimization
- Memory layout optimization

**Effort**: 1 week
**Benefit**: 2-4x performance vs baseline

### Phase 3: Extended RaBitQ (2-3 weeks)

**Scope**:
- Implement SIGMOD 2025 algorithm
- Integration with HNSW traversal
- Arbitrary compression rates

**Effort**: 2-3 weeks
**Benefit**: SOTA quantization, 10-20% better recall

### Phase 4: HNSW-IF (2-3 weeks)

**Scope**:
- Hybrid memory/disk storage
- Layer-aware caching
- Efficient I/O

**Effort**: 2-3 weeks
**Benefit**: Billion-scale support

### Phase 5: Advanced Features (2-4 weeks)

**Scope**:
- MN-RU (delete/update)
- Filtered search (<10% overhead)
- GPU acceleration (optional)

**Effort**: 2-4 weeks
**Benefit**: Full competitive parity + unique features

**Total**: 10-15 weeks for complete custom implementation

---

## Migration Strategy

### Phase 1: Keep hnsw_rs (Week 1)

**Actions**:
- Enable SIMD (5 minutes)
- Profile and optimize
- Quick wins (LTO, opt-level)

**Why**: Get 2-5x improvement while planning custom

**Benchmark against Qdrant**: Establish baseline gap

### Phase 2: Build Custom Core (Week 2-4)

**Actions**:
- Implement basic HNSW (insert, search)
- SIMD distance calculations
- Parallel building with Rayon
- A/B test against hnsw_rs

**Why**: Prove custom is better before full migration

**Target**: Match or beat hnsw_rs + SIMD

### Phase 3: Add SOTA Features (Week 5-10)

**Actions**:
- Extended RaBitQ
- HNSW-IF
- Filtered search
- MN-RU

**Why**: Differentiation vs competitors

**Target**: Qdrant-level performance + unique features

### Phase 4: Production Hardening (Week 11-12)

**Actions**:
- Extensive testing
- Edge case handling
- Performance validation
- Documentation

**Why**: Production-ready

---

## Performance Projections

### Current (hnsw_rs, no SIMD):
- Build: 31.05s (3220 vec/sec)
- Query: 6.16ms p95 (~162 QPS)

### After hnsw_rs + SIMD (Week 1):
- Build: ~31s (same, parallel already optimized)
- Query: ~2-3ms p95 (~400-500 QPS)
- **Improvement: 2-3x**

### After Custom HNSW Core (Week 4):
- Build: ~20s (optimized parallel)
- Query: ~1.5ms p95 (~600-800 QPS)
- **Improvement: 4-5x vs current**

### After SOTA Features (Week 10):
- Build: ~15s (Extended RaBitQ, optimized)
- Query: ~1ms p95 (~1000 QPS)
- Extended RaBitQ: 10-20% better recall
- HNSW-IF: Billion-scale support
- **Improvement: 6-10x vs current**

### After GPU Acceleration (Optional, Week 12+):
- Build: ~3s (GPU-accelerated, 10x faster)
- Query: ~0.5ms p95 (~2000 QPS)
- **Improvement: 10-15x vs current**

### vs Qdrant (Target):

| Metric | hnsw_rs (current) | Custom (Week 10) | Qdrant | Gap |
|--------|-------------------|------------------|--------|-----|
| Build (100K) | 31.05s | ~15s | ? | Likely competitive |
| Query p95 | 6.16ms (~162 QPS) | ~1ms (~1000 QPS) | 626 QPS @ 99.5% | **Beating Qdrant** |
| Recall | 97-100% | 97-100% (better with RaBitQ) | 99.5% | Competitive |
| Memory | Good | Better (Extended RaBitQ) | Good | Better |
| Scale | 10M | 1B (HNSW-IF) | 1B+ | Competitive |

**Verdict**: Custom implementation can match or beat Qdrant

---

## Risk Analysis

### Risks of Custom Implementation

**1. Development Time** ⚠️
- Risk: 10-15 weeks of work
- Mitigation: Incremental (keep hnsw_rs until custom is better)
- Impact: Delayed competitive benchmarking

**2. Bugs & Edge Cases** ⚠️
- Risk: HNSW is complex, easy to introduce bugs
- Mitigation: Extensive testing, A/B against hnsw_rs
- Impact: Production stability

**3. Maintenance Burden** ⚠️
- Risk: Must maintain custom code
- Mitigation: Good documentation, tests
- Impact: Long-term engineering cost

### Risks of Staying with hnsw_rs

**1. Performance Ceiling** ⚠️⚠️⚠️
- Risk: Can't reach Qdrant-level performance
- Impact: Permanent competitive disadvantage

**2. SOTA Features Blocked** ⚠️⚠️⚠️
- Risk: Can't implement Extended RaBitQ, HNSW-IF, MN-RU
- Impact: Feature parity impossible

**3. Dependency Risk** ⚠️
- Risk: hnsw_rs development may stall
- Impact: Stuck with limitations

**4. Market Positioning** ⚠️⚠️
- Risk: Can't claim "SOTA" or "fastest"
- Impact: Marketing disadvantage

### Risk Assessment

**Custom implementation risks**: Medium (manageable with testing)
**hnsw_rs limitation risks**: HIGH (permanent ceiling)

**Verdict**: Custom implementation is worth the risk

---

## Competitive Positioning with Custom

### With hnsw_rs (Limited):
> "PostgreSQL-compatible vector database. 97x faster than pgvector. Competitive with leading vector databases."

### With Custom HNSW (Ambitious):
> "PostgreSQL-compatible vector database with SOTA performance. Faster than leading vector databases. SOTA algorithms: Extended RaBitQ (SIGMOD 2025), HNSW-IF (billion-scale), MN-RU (updates)."

### With Custom + GPU (Market Leader):
> "Fastest PostgreSQL-compatible vector database. 10x faster indexing with GPU acceleration. SOTA algorithms. Billion-scale support."

**Marketing Impact**: Custom unlocks premium positioning

---

## Decision Matrix

| Factor | hnsw_rs | Custom | Winner |
|--------|---------|--------|--------|
| **Time to Market** | Fast (now) | Slow (10-15 weeks) | hnsw_rs |
| **Performance Ceiling** | 400-500 QPS | 1000-2000 QPS | Custom ⭐⭐⭐ |
| **SOTA Features** | Blocked | Possible | Custom ⭐⭐⭐ |
| **Maintenance** | Low | High | hnsw_rs |
| **Competitive Position** | Good | Market Leader | Custom ⭐⭐⭐ |
| **Development Effort** | 0 weeks | 10-15 weeks | hnsw_rs |
| **Long-term Value** | Limited | High | Custom ⭐⭐⭐ |

**Score**: Custom wins on critical factors (performance, features, positioning)

---

## Recommendation

### Phase 1 (Week 1): hnsw_rs + SIMD ✅ IMMEDIATE
- **Action**: Enable SIMD, profile, optimize
- **Goal**: 2-5x improvement, understand ceiling
- **Benchmark**: Compare against Qdrant
- **Decision point**: Confirm gap requires custom

### Phase 2 (Week 2-4): Build Custom Core ✅ HIGH PRIORITY
- **Action**: Implement basic HNSW from scratch
- **Goal**: Match or beat hnsw_rs + SIMD
- **Risk mitigation**: A/B test, keep hnsw_rs as fallback

### Phase 3 (Week 5-10): SOTA Features ✅ DIFFERENTIATION
- **Action**: Extended RaBitQ, HNSW-IF, filtered search
- **Goal**: Beat Qdrant, billion-scale support
- **Marketing**: "SOTA PostgreSQL-compatible vector DB"

### Phase 4 (Week 11+): Production Hardening
- **Action**: Testing, docs, validation
- **Goal**: Production-ready custom implementation
- **Launch**: Replace hnsw_rs completely

---

## Build vs Buy Analysis

### "Buy" (Use hnsw_rs):
- **Pro**: Fast to market, working today
- **Con**: Performance ceiling, no SOTA features
- **Best for**: MVP, early validation
- **Long-term**: Dead end

### "Build" (Custom HNSW):
- **Pro**: SOTA features, max performance, market leader
- **Con**: 10-15 weeks, maintenance burden
- **Best for**: Serious product, long-term competitive advantage
- **Long-term**: Sustainable differentiation

**Verdict**: Build (but use hnsw_rs for quick wins first)

---

## Implementation Roadmap

### Week 1: hnsw_rs Optimization
- [x] Enable SIMD
- [ ] Profile with flamegraph
- [ ] Quick wins (allocations, LTO)
- [ ] Benchmark vs Qdrant
- **Decision**: Confirm custom is needed

### Week 2-3: Custom HNSW Core
- [ ] Basic algorithm (insert, search)
- [ ] SIMD distance (AVX2/AVX-512/NEON)
- [ ] Parallel building (Rayon)
- [ ] A/B test vs hnsw_rs

### Week 4: Validation & Switch
- [ ] Performance validation
- [ ] Correctness testing
- [ ] Switch from hnsw_rs to custom
- [ ] Document decision

### Week 5-6: Extended RaBitQ
- [ ] Implement SIGMOD 2025 algorithm
- [ ] Integration with HNSW
- [ ] Validation vs basic BQ

### Week 7-8: HNSW-IF
- [ ] Hybrid memory/disk storage
- [ ] Layer-aware caching
- [ ] 100M-1B validation

### Week 9-10: Advanced Features
- [ ] Filtered search (<10% overhead)
- [ ] MN-RU (delete/update)
- [ ] Production testing

### Week 11-12: Hardening & Launch
- [ ] Edge cases
- [ ] Extensive testing
- [ ] Documentation
- [ ] Benchmark publication

---

## Conclusion

**Decision**: **Build custom HNSW implementation**

**Why**:
1. ALL serious competitors use custom (Qdrant, Milvus, Weaviate, LanceDB)
2. hnsw_rs has hard ceiling (no delete/update, no HNSW-IF, limited RaBitQ)
3. SOTA features require deep integration
4. 2-5x additional performance possible
5. Premium market positioning

**Timeline**:
- Short-term (Week 1): Use hnsw_rs + SIMD (quick wins)
- Medium-term (Week 2-4): Build custom core
- Long-term (Week 5-10): Add SOTA features

**Risk**: Medium (development time, bugs)
**Reward**: HIGH (market leadership, SOTA features, 2-5x performance)

**Next Step**: Enable SIMD in hnsw_rs (Week 1), then start custom implementation (Week 2)

---

**Last Updated**: October 30, 2025
**Status**: Decision made - build custom HNSW
**First Action**: Enable SIMD feature flag (5 minutes)
