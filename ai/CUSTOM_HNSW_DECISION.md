# Custom HNSW Implementation Decision

**Date**: October 30, 2025
**Decision**: Build custom SOTA HNSW implementation

---

## Executive Summary

**Question**: Should we use hnsw_rs library or build custom HNSW?
**Answer**: **BUILD CUSTOM** (start Week 2-4)
**Confidence**: HIGH

**Quick Answers**:
- Do competitors use libraries? **NO** - ALL serious competitors use custom (Qdrant, Milvus, Weaviate, LanceDB)
- What's blocking hnsw_rs? **No delete/update, no HNSW-IF, no Extended RaBitQ deep integration**
- How much effort? **10-15 weeks total** (2-3 weeks for core, 8-12 weeks for SOTA features)
- Performance gain? **2-5x additional** (current 162 QPS → Week 1: 400-500 QPS → Week 10: 1000 QPS)
- Is it worth it? **YES** - Premium market positioning + SOTA features

**Critical Path**:
1. Week 1: Enable SIMD in hnsw_rs (quick wins, 2-4x improvement)
2. Week 2-4: Build custom HNSW core (match or beat hnsw_rs + SIMD)
3. Week 5-10: Add SOTA features (Extended RaBitQ, HNSW-IF, MN-RU)
4. Week 11-12: Production hardening

**Risk vs Reward**:
- Risk: MEDIUM (development time, bugs, maintenance)
- Reward: HIGH (market leadership, 2-5x performance, SOTA features)

---

## Table of Contents

1. [What Do Competitors Use?](#what-do-competitors-use)
2. [hnsw_rs Limitations](#hnsw_rs-limitations)
3. [SOTA Features Requiring Custom](#sota-features-requiring-custom)
4. [Implementation Effort](#implementation-effort)
5. [Migration Strategy](#migration-strategy)
6. [Performance Projections](#performance-projections)
7. [Risk Analysis](#risk-analysis)
8. [Decision Matrix](#decision-matrix)
9. [Implementation Roadmap](#implementation-roadmap)

---

## What Do Competitors Use?

### Competitor Implementation Analysis

| Database | Implementation | Key Features | Why Custom |
|----------|---------------|--------------|------------|
| **Qdrant** | Custom Rust | GPU (10x faster), delta encoding (30% memory), filtered search (<10% overhead) | Maximum performance control |
| **Milvus** | Custom C++ (Knowhere) | SIMD (20-30% gain), soft deletes, distributed | Enterprise features |
| **Weaviate** | Custom Go | GraphQL integration, hybrid search | Language integration |
| **LanceDB** | Custom Rust | Columnar format (Arrow), embedded | Architecture control |
| **pgvector** | Custom C | PostgreSQL integration | Deep integration |

**Finding**: ⚠️ **ZERO top competitors use libraries**

**Verdict**: Library-based approach (hnsw_rs) is NOT used by any serious competitor

---

## hnsw_rs Limitations

### What hnsw_rs Provides ✅

| Feature | Status | Notes |
|---------|--------|-------|
| HNSW algorithm | ✅ Working | Good baseline implementation |
| SIMD support | ✅ Available | Via simdeez_f feature (NOT enabled) |
| Parallel building | ✅ Working | Good performance |
| Graph serialization | ✅ Working | file_dump/from_file_dump |

### What hnsw_rs CANNOT Do ❌

| Missing Feature | Why Blocked | Impact | SOTA Algorithm Affected |
|----------------|-------------|--------|-------------------------|
| **Delete/update** | Immutable graph design | No production updates | MN-RU |
| **Hybrid memory/disk** | In-memory only | No billion-scale | HNSW-IF |
| **Deep quantization hooks** | Opaque distance calculations | Limited quantization | Extended RaBitQ |
| **GPU acceleration** | CPU-only | 10x slower indexing | Qdrant feature |
| **Filtered search** | No callback system | No metadata filtering | Competitive parity |
| **Custom memory layout** | Library-controlled | Can't optimize cache | Performance ceiling |

### Performance Ceiling

| Metric | With hnsw_rs (SIMD) | With Custom | Gap |
|--------|---------------------|-------------|-----|
| QPS | 300-500 | 1000-2000 | **2-5x** |
| SOTA features | BLOCKED | Possible | **Critical** |
| GPU support | No | Optional | **10x indexing** |
| Billion-scale | No | Yes (HNSW-IF) | **Critical** |

**Verdict**: hnsw_rs has hard performance and feature ceiling

---

## SOTA Features Requiring Custom

### Extended RaBitQ (SIGMOD 2025)

**Requirements**:
- Deep integration with HNSW distance calculations
- Custom quantization hooks during graph traversal
- Arbitrary compression rates (4x-32x)

**Why hnsw_rs can't do it**:
- Opaque distance calculations (no hooks)
- Would require complete fork and modification

**Impact**: 10-20% better recall at same memory

---

### HNSW-IF (Hybrid Memory/Disk)

**Requirements**:
- Layer-aware memory management
- Hot layers (top 2-3) in memory, cold on disk
- Custom prefetching and caching

**Why hnsw_rs can't do it**:
- Fully in-memory design (no disk support)
- No layer-level control

**Impact**: Billion-scale on single node

---

### MN-RU (Delete/Update Support)

**Requirements**:
- Delete nodes from graph
- Update node connections
- Maintain graph quality

**Why hnsw_rs can't do it**:
- No delete/update API (immutable design)
- Would require complete rewrite

**Impact**: Production-ready updates (no full rebuild)

---

### Filtered Search

**Requirements**:
- Metadata filtering during traversal
- Skip filtered-out nodes efficiently
- <10% overhead (Qdrant benchmark)

**Why hnsw_rs can't do it**:
- No filtering hooks or callback system

**Impact**: Competitive parity with Qdrant/Milvus

---

### GPU Acceleration

**Requirements**:
- CUDA/ROCm for distance calculations
- Batch processing on GPU
- Memory transfer optimization

**Why hnsw_rs can't do it**:
- CPU-only (no GPU support)

**Impact**: 10x faster indexing (Qdrant has this)

---

## Implementation Effort

### Phase Breakdown

| Phase | Scope | Effort | Benefit |
|-------|-------|--------|---------|
| **Phase 1: Core HNSW** | Basic algorithm, insert/search, L2/cosine, parallel building, serialization | 2-3 weeks | Full control, foundation for SOTA |
| **Phase 2: SIMD + Optimization** | AVX2/AVX-512/NEON, prefetching, memory layout | 1 week | 2-4x vs baseline |
| **Phase 3: Extended RaBitQ** | SIGMOD 2025 algorithm, HNSW integration, arbitrary compression | 2-3 weeks | SOTA quantization, 10-20% better recall |
| **Phase 4: HNSW-IF** | Hybrid memory/disk, layer-aware caching, efficient I/O | 2-3 weeks | Billion-scale support |
| **Phase 5: Advanced Features** | MN-RU, filtered search, GPU (optional) | 2-4 weeks | Full competitive parity |

**Total**: 10-15 weeks for complete SOTA implementation

---

## Migration Strategy

### Timeline

| Week | Phase | Actions | Goal |
|------|-------|---------|------|
| **Week 1** | Keep hnsw_rs | Enable SIMD, profile, optimize, quick wins (LTO, opt-level) | 2-5x improvement, establish baseline gap |
| **Week 2-4** | Build Custom Core | Implement basic HNSW, SIMD distances, parallel building, A/B test | Match or beat hnsw_rs + SIMD |
| **Week 5-10** | Add SOTA Features | Extended RaBitQ, HNSW-IF, filtered search, MN-RU | Differentiation vs competitors |
| **Week 11-12** | Production Hardening | Testing, edge cases, validation, documentation | Production-ready |

---

## Performance Projections

### Evolution Timeline

| Stage | Build Time | Query Latency | QPS | Features | When |
|-------|-----------|---------------|-----|----------|------|
| **Current** (hnsw_rs, no SIMD) | 31.05s | 6.16ms p95 | ~162 | Basic HNSW, BQ | Now |
| **+ SIMD** (hnsw_rs optimized) | ~31s | 2-3ms p95 | ~400-500 | + SIMD | Week 1 |
| **Custom Core** | ~20s | 1.5ms p95 | ~600-800 | + Custom, optimized | Week 4 |
| **+ SOTA** (Extended RaBitQ, HNSW-IF) | ~15s | 1ms p95 | ~1000 | + SOTA algorithms, billion-scale | Week 10 |
| **+ GPU** (optional) | ~3s | 0.5ms p95 | ~2000 | + 10x faster indexing | Week 12+ |

### vs Qdrant (Target)

| Metric | Current | Custom (Week 10) | Qdrant | Result |
|--------|---------|------------------|--------|--------|
| Build (100K) | 31.05s | ~15s | ? | Likely competitive |
| Query p95 | 6.16ms (162 QPS) | ~1ms (1000 QPS) | 626 QPS @ 99.5% | **Beating Qdrant** ⭐ |
| Recall | 97-100% | 97-100% (better with RaBitQ) | 99.5% | Competitive |
| Memory | Good | Better (Extended RaBitQ) | Good | Better ⭐ |
| Scale | 10M tested | 1B (HNSW-IF) | 1B+ | Competitive ⭐ |

**Verdict**: Custom implementation can match or beat Qdrant

---

## Risk Analysis

### Risks: Custom Implementation

| Risk | Severity | Mitigation | Impact |
|------|----------|------------|--------|
| Development time (10-15 weeks) | MEDIUM | Incremental (keep hnsw_rs until custom better) | Delayed competitive benchmarking |
| Bugs & edge cases | MEDIUM | Extensive testing, A/B vs hnsw_rs | Production stability |
| Maintenance burden | LOW-MEDIUM | Good documentation, comprehensive tests | Long-term engineering cost |

**Overall Risk**: MEDIUM (manageable with testing & incremental approach)

---

### Risks: Staying with hnsw_rs

| Risk | Severity | Impact |
|------|----------|--------|
| **Performance ceiling** | ⚠️⚠️⚠️ HIGH | Permanent competitive disadvantage |
| **SOTA features blocked** | ⚠️⚠️⚠️ HIGH | Feature parity impossible |
| **Dependency risk** | MEDIUM | Stuck with limitations if development stalls |
| **Market positioning** | ⚠️⚠️ HIGH | Can't claim "SOTA" or "fastest" |

**Overall Risk**: HIGH (permanent ceiling, critical features blocked)

---

### Risk Comparison

| Factor | Custom | hnsw_rs |
|--------|--------|---------|
| Short-term risk | MEDIUM | LOW |
| Long-term risk | LOW | HIGH ⚠️ |
| **Recommendation** | **BUILD CUSTOM** | Avoid |

**Verdict**: Custom implementation risks are manageable, hnsw_rs risks are permanent

---

## Decision Matrix

| Factor | hnsw_rs | Custom | Winner |
|--------|---------|--------|--------|
| Time to Market | ✅ Fast (now) | ❌ Slow (10-15 weeks) | hnsw_rs |
| **Performance Ceiling** | ❌ 400-500 QPS | ✅ 1000-2000 QPS | **Custom ⭐⭐⭐** |
| **SOTA Features** | ❌ Blocked | ✅ Possible | **Custom ⭐⭐⭐** |
| Maintenance | ✅ Low | ❌ High | hnsw_rs |
| **Competitive Position** | ❌ Good | ✅ Market Leader | **Custom ⭐⭐⭐** |
| Development Effort | ✅ 0 weeks | ❌ 10-15 weeks | hnsw_rs |
| **Long-term Value** | ❌ Limited | ✅ High | **Custom ⭐⭐⭐** |

**Score**: Custom wins on critical factors (performance, features, positioning)

**Decision**: BUILD CUSTOM

---

## Competitive Positioning Impact

### With hnsw_rs (Limited)
> "PostgreSQL-compatible vector database. 97x faster than pgvector. Competitive with leading vector databases."

**Ceiling**: "Competitive" positioning (middle of pack)

---

### With Custom HNSW (Ambitious)
> "PostgreSQL-compatible vector database with SOTA performance. Faster than leading vector databases. SOTA algorithms: Extended RaBitQ (SIGMOD 2025), HNSW-IF (billion-scale), MN-RU (updates)."

**Positioning**: Premium tier, feature leadership

---

### With Custom + GPU (Market Leader)
> "Fastest PostgreSQL-compatible vector database. 10x faster indexing with GPU acceleration. SOTA algorithms. Billion-scale support."

**Positioning**: Market leader, clear differentiation

---

**Marketing Impact**: Custom unlocks premium positioning and premium pricing

---

## Build vs Buy Analysis

| Approach | Pros | Cons | Best For | Long-term |
|----------|------|------|----------|-----------|
| **"Buy" (hnsw_rs)** | Fast to market, working today | Performance ceiling, no SOTA features | MVP, early validation | Dead end ❌ |
| **"Build" (Custom)** | SOTA features, max performance, market leader | 10-15 weeks, maintenance | Serious product, competitive advantage | Sustainable differentiation ✅ |

**Verdict**: BUILD (but use hnsw_rs for quick wins first)

---

## Implementation Roadmap

### Week 1: hnsw_rs Optimization ⚠️ IN PROGRESS
- [x] Decision to build custom (this doc)
- [ ] Enable SIMD (5 minutes)
- [ ] Profile with flamegraph
- [ ] Quick wins (allocations, LTO, opt-level)
- [ ] Benchmark vs Qdrant (establish gap)

**Goal**: 2-5x improvement, confirm custom is needed

---

### Week 2-3: Custom HNSW Core
- [ ] Basic algorithm (insert, search, build)
- [ ] SIMD distance (AVX2/AVX-512/NEON)
- [ ] Parallel building (Rayon)
- [ ] Graph serialization
- [ ] A/B test vs hnsw_rs

**Goal**: Match or beat hnsw_rs + SIMD

---

### Week 4: Validation & Switch
- [ ] Performance validation (match or beat hnsw_rs)
- [ ] Correctness testing (all 142 tests pass)
- [ ] Switch from hnsw_rs to custom
- [ ] Document decision

**Goal**: Custom becomes primary implementation

---

### Week 5-6: Extended RaBitQ
- [ ] Implement SIGMOD 2025 algorithm
- [ ] Integration with HNSW traversal
- [ ] Arbitrary compression rates (4x-32x)
- [ ] Validation vs basic BQ

**Goal**: SOTA quantization

---

### Week 7-8: HNSW-IF
- [ ] Hybrid memory/disk storage
- [ ] Layer-aware caching
- [ ] Efficient I/O
- [ ] 100M-1B validation

**Goal**: Billion-scale support

---

### Week 9-10: Advanced Features
- [ ] Filtered search (<10% overhead)
- [ ] MN-RU (delete/update)
- [ ] Production testing

**Goal**: Full competitive parity + unique features

---

### Week 11-12: Hardening & Launch
- [ ] Edge case handling
- [ ] Extensive testing (fuzz, property-based)
- [ ] Documentation
- [ ] Benchmark publication

**Goal**: Production-ready launch

---

## Conclusion

**Decision**: ✅ **BUILD CUSTOM HNSW IMPLEMENTATION**

**Why**:
1. **ALL serious competitors use custom** (Qdrant, Milvus, Weaviate, LanceDB, pgvector)
2. **hnsw_rs has hard ceiling** (no delete/update, no HNSW-IF, no Extended RaBitQ deep integration)
3. **SOTA features require deep integration** (can't bolt on to library)
4. **2-5x additional performance possible** (1000-2000 QPS vs 400-500 QPS)
5. **Premium market positioning** (SOTA vs "competitive")

**Timeline**:
- **Short-term** (Week 1): Use hnsw_rs + SIMD (quick wins, 2-4x)
- **Medium-term** (Week 2-4): Build custom core (match or beat hnsw_rs)
- **Long-term** (Week 5-10): Add SOTA features (Extended RaBitQ, HNSW-IF, MN-RU)

**Risk**: MEDIUM (development time, bugs, testing) - Mitigated by incremental approach
**Reward**: HIGH (market leadership, SOTA features, 2-5x performance, premium positioning)

**Next Step**: Enable SIMD in hnsw_rs (Week 1), then start custom implementation (Week 2)

---

**Last Updated**: October 30, 2025
**Status**: Decision made - build custom HNSW
**First Action**: Enable SIMD feature flag (5 minutes, in Cargo.toml)
