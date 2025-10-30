# Competitive Analysis: OmenDB vs Dedicated Vector Databases

**Date**: October 30, 2025
**Purpose**: Competitive positioning and testing priorities

---

## Executive Summary

**Quick Answers**:
- Have we tested vs dedicated vector DBs? **NO** (only vs pgvector so far)
- Who should we benchmark first? **Qdrant** (Rust performance leader, easiest to test)
- Current vs pgvector? **97x faster builds, 2.2x faster queries**
- Estimated vs Qdrant? **4-13x slower QPS** (fixable with SIMD + optimization)
- What's our unique advantage? **PostgreSQL compatibility** (no other embedded vector DB has this)

**Testing Priority**:
1. **Week 1**: Qdrant (performance leader, Rust-based)
2. **Week 2**: LanceDB (embedded competitor)
3. **Week 3**: Milvus/Weaviate (enterprise scale)

**Critical Action**: Profile OmenDB → Enable SIMD → Then benchmark Qdrant

---

## Table of Contents

1. [Competitive Landscape](#competitive-landscape)
2. [OmenDB Current State](#omendb-current-state)
3. [Competitive Gaps Analysis](#competitive-gaps-analysis)
4. [Testing Strategy](#testing-strategy)
5. [Profiling Plan](#profiling-plan)
6. [Optimization Roadmap](#optimization-roadmap)
7. [Competitive Positioning](#competitive-positioning)

---

## Competitive Landscape

### Market Leaders (2024-2025)

| Database | Stars | Language | Strength | Query Performance | Target Use Case |
|----------|-------|----------|----------|-------------------|-----------------|
| **Milvus** | ~25k | Go/Python | Enterprise scale | Sub-2ms, highest QPS | Distributed, billions of vectors |
| **Qdrant** | ~9k | Rust | Speed + filtering | 2200 QPS max, 626 @ 99.5% | Performance-critical apps |
| **Weaviate** | ~8k | Go | GraphQL API | Sub-2ms latency | Hybrid search, GraphQL users |
| **ChromaDB** | ~6k | Python | Simplicity | Good for prototypes | RAG, prototyping |
| **LanceDB** | ~3k | Rust | Embedded/serverless | Rust performance | Embedded, serverless |
| **pgvector** | ~4k | C | PostgreSQL | 33 vec/sec build (slow) | PostgreSQL users |

### Performance Leader Detail

**Qdrant** (Rust-based, our primary benchmark target):
| Metric | Value | Notes |
|--------|-------|-------|
| QPS (max) | 2200 | Peak throughput |
| QPS @ 99.5% recall | 626 | 1M vectors |
| Latency | Lowest across all scenarios | - |
| Filtering overhead | <10% | Metadata filtering |
| Implementation | Custom Rust | Not using library |

**Milvus** (Scale leader):
- QPS: Highest overall
- Latency: Sub-2ms
- Scale: Billions of vectors (distributed)
- Implementation: Custom C++ (Knowhere library)

---

## OmenDB Current State

### What We Know (100K vectors, 1536D, M=16, ef_construction=64)

| Metric | OmenDB | pgvector | Advantage |
|--------|--------|----------|-----------|
| Build speed | 31.05s (3220 vec/sec) | 3026.27s (33 vec/sec) | **97x faster** ⭐ |
| Query p95 | 6.16ms (~162 QPS) | 13.60ms (~73 QPS) | **2.2x faster** |
| Tests | 142 passing | - | ✅ Validated |
| ASAN | ZERO issues | - | ✅ Memory safe |

---

### What We DON'T Know (Critical Gaps)

| Unknown | Why It Matters | Priority |
|---------|----------------|----------|
| Build speed vs Qdrant/Milvus | Are we competitive at indexing? | HIGH |
| QPS under load | Max throughput with parallel queries? | ⚠️ CRITICAL |
| Latency at scale (1M, 10M, 100M) | Do we degrade? | HIGH |
| Filtered search | Metadata filtering performance | MEDIUM |
| Memory efficiency | RAM usage vs competitors | MEDIUM |
| Profiling data | Where are bottlenecks? | ⚠️ CRITICAL |

---

## Competitive Gaps Analysis

### Where We're Strong ✅

| Strength | Status | Competitor Comparison |
|----------|--------|-----------------------|
| Build speed | 3220 vec/sec (97x vs pgvector) | Likely competitive with Qdrant |
| PostgreSQL compatibility | ✅ Unique | No other embedded vector DB has this |
| Embedded deployment | ✅ No infrastructure | Like LanceDB, unlike Qdrant/Milvus |
| Rust implementation | ✅ Memory safe + fast | Like Qdrant, LanceDB |
| Parallel building | 16x speedup | UNIQUE (undocumented by competitors) |
| Graph serialization | 4175x speedup | UNIQUE (undocumented by competitors) |

---

### Where We Need Validation ⚠️

| Area | Current | Qdrant | Estimated Gap | Fixable? |
|------|---------|--------|---------------|----------|
| **Query QPS** | ~162 (from p95 latency) | 2200 max, 626 @ 99.5% | **4-13x slower** | ✅ YES (SIMD + optimization) |
| Parallel queries | Not tested | Validated | Unknown | ✅ YES (Rayon) |
| Filtered search | Not implemented | <10% overhead | Unknown | ✅ YES (implementation needed) |
| Scale (10M+) | 1M tested only | 1B+ supported | Unknown | ✅ YES (HNSW-IF) |

---

### Where We're Behind ❌

| Gap | Impact | Timeline to Fix |
|-----|--------|----------------|
| Distributed deployment | Can't scale horizontally | 6+ months (Phase 4+) |
| Cloud-native features | No multi-tenancy, sharding | 3-6 months |
| Ecosystem | No client libraries | 3-6 months |
| Maturity | Years vs our weeks | Ongoing |

**Strategy**: Accept these gaps, focus on embedded + PostgreSQL compatibility differentiators

---

## Testing Strategy

### Phase 1: Qdrant (Week 1) ⚠️ PRIORITY

**Why Qdrant First**:
| Reason | Benefit |
|--------|---------|
| Rust-based | Direct language comparison |
| Performance leader | Hardest benchmark (if we match Qdrant, we beat others) |
| Easy deployment | `docker run -p 6333:6333 qdrant/qdrant` |
| Good documentation | Clear benchmarking methodology |

**Test Setup** (100K vectors, 1536D):
- [x] Build time
- [ ] Single query latency (p50, p95, p99)
- [ ] QPS under load (1, 10, 100 parallel clients)
- [ ] Memory usage
- [ ] Disk usage

**Expected Outcome**:
- Build: Qdrant likely faster (or we're competitive)
- Query latency: Competitive (6.16ms vs Qdrant's optimized Rust)
- QPS: Qdrant ahead (2200 vs ~162) - this is what we need to fix

---

### Phase 2: LanceDB (Week 2)

**Why LanceDB Second**:
| Reason | Benefit |
|--------|---------|
| Rust embedded architecture | Direct embedded comparison |
| Similar use case | Both targeting embedded deployments |
| Growing adoption | Relevant competitor |

**Benchmarks**: Same as Qdrant

---

### Phase 3: Milvus/Weaviate (Week 3)

**Why Later**:
- More complex setup (distributed systems)
- Different use case (enterprise vs embedded)
- Less relevant for initial positioning

---

### Phase 4: Pinecone (Week 4+)

**Why Last**:
- Proprietary cloud (hard to benchmark fairly)
- Network latency confounds results
- Different target customer

---

## Profiling Plan

### Tools

| Tool | Purpose | Command |
|------|---------|---------|
| **flamegraph** | CPU profiling | `cargo flamegraph --bin benchmark -- 100000` |
| **heaptrack** | Memory profiling | `heaptrack ./target/release/benchmark 100000` |
| **perf** | Perf analysis | `perf record -g ./target/release/benchmark 100000` |
| **criterion** | Micro-benchmarks | Distance calculations, HNSW traversal |

---

### Expected Bottlenecks

| Bottleneck | Likelihood | Fix | Expected Impact |
|------------|------------|-----|----------------|
| **Distance calculations** | HIGH | SIMD (AVX2/AVX-512) | 2-4x speedup |
| HNSW traversal | MEDIUM | Cache optimization, prefetching | 10-20% speedup |
| Memory allocations | MEDIUM | Object pooling, reuse buffers | 10-20% speedup |
| Serialization | LOW | Already optimized (4175x) | N/A |

---

## Optimization Roadmap

### Phase 1: Quick Wins (Week 1) ⚠️ CRITICAL

| Optimization | Effort | Expected Impact | Priority |
|--------------|--------|----------------|----------|
| **Enable SIMD** | 5 minutes | 2-4x query speedup | ⚠️⚠️⚠️ HIGHEST |
| Enable LTO | 1 minute | 5-15% improvement | HIGH |
| Enable opt-level=3 | 1 minute | 5-10% improvement | HIGH |
| Reduce allocations | 1-2 days | 10-20% improvement | MEDIUM |

**Total Phase 1 Impact**: 2-5x improvement

---

### Phase 2: Medium Effort (Week 2)

| Optimization | Effort | Expected Impact |
|--------------|--------|----------------|
| Cache optimization | 2-3 days | 10-30% improvement |
| Query batching | 2-3 days | 20-50% throughput |
| Async I/O | 3-5 days | If disk bottleneck |

---

### Phase 3: Long-Term (Week 3-8)

| Optimization | Effort | Expected Impact |
|--------------|--------|----------------|
| GPU acceleration | 2-4 weeks | 10x indexing (Qdrant has this) |
| Custom HNSW | 10-15 weeks | 2-5x additional (market leadership) |
| SOTA algorithms | 4-8 weeks | Billion-scale, SOTA quantization |

---

## Competitive Positioning

### Our Unique Strengths

| Strength | Importance | Competitor Status |
|----------|------------|-------------------|
| **PostgreSQL compatibility** | ⭐⭐⭐ CRITICAL | NONE (unique to us) |
| **97x faster builds** | ⭐⭐⭐ HIGH | UNIQUE parallel construction |
| Embedded + Server modes | ⭐⭐ MEDIUM | LanceDB has embedded only |
| Source-available (Elastic 2.0) | ⭐⭐ MEDIUM | Some are closed-source |

---

### Positioning Statements

**If competitive with Qdrant (Week 4+)**:
> "PostgreSQL-compatible vector database. Drop-in pgvector replacement. Qdrant-class performance with PostgreSQL compatibility. 97x faster builds, [Nx faster queries]."

**If faster than pgvector but slower than Qdrant (Current)**:
> "PostgreSQL-compatible vector database. 97x faster than pgvector. PostgreSQL ecosystem compatibility that pure vector DBs can't match. Perfect for teams already using Postgres."

**Honest positioning principle**:
- Don't claim "fastest" without benchmarks
- Lead with unique strength (PostgreSQL compatibility)
- Be honest about tradeoffs (embedded vs distributed)

---

## Testing Checklist

### Immediate (Week 1) ⚠️ PRIORITY

- [ ] Profile OmenDB (flamegraph CPU)
- [ ] Profile OmenDB (heaptrack memory)
- [ ] Enable SIMD (5 minutes, 2-4x win)
- [ ] Benchmark Qdrant (100K, same params)
- [ ] Parallel query testing (10/100/1000 clients)
- [ ] Document bottlenecks
- [ ] Implement 1-2 quick optimizations
- [ ] Re-benchmark after optimizations

---

### Short-Term (Week 2-3)

- [ ] LanceDB comparison (embedded)
- [ ] 1M benchmark vs Qdrant
- [ ] 10M benchmark (memory limits)
- [ ] Filtered search (metadata)
- [ ] Binary Quantization comparison

---

### Long-Term (Week 4+)

- [ ] Milvus (enterprise scale)
- [ ] Weaviate (hybrid search)
- [ ] 100M+ benchmarks
- [ ] Distributed deployment tests

---

## Success Criteria

### Minimum Success (Acceptable)
- Within 2x of Qdrant query latency
- Competitive build speed (already 97x vs pgvector)
- Clear PostgreSQL compatibility advantage

### Target Success (Ideal)
- Match or beat Qdrant single-query latency
- Within 50% of Qdrant QPS
- Unique features (parallel builds, serialization, PostgreSQL compat)

### Stretch Success (Market Leader)
- Beat Qdrant on queries
- 1000+ QPS (custom HNSW + SIMD)
- SOTA features (Extended RaBitQ, HNSW-IF)

---

## Risk Mitigation

**If significantly slower than Qdrant**:
1. Focus on PostgreSQL compatibility as primary differentiator
2. Target users who need Postgres ecosystem (not pure vector DB users)
3. Roadmap aggressive optimizations (SIMD → custom HNSW → GPU)
4. Be honest about performance vs pure vector DBs
5. Emphasize embedded deployment simplicity

**Strategy**: PostgreSQL compatibility is defensible moat even if performance lags

---

## Next Steps (Week 1)

### Day 1: Profiling
- [ ] Run flamegraph (2 hours)
- [ ] Run heaptrack (2 hours)
- [ ] Analyze bottlenecks (2 hours)
- [ ] Document findings (1 hour)

### Day 2: Quick Optimizations
- [ ] Enable SIMD (5 minutes)
- [ ] Enable LTO + opt-level (2 minutes)
- [ ] Rebuild and test (30 minutes)
- [ ] Re-benchmark (2 hours)

### Day 3: Qdrant Benchmark
- [ ] Set up Qdrant Docker (1 hour)
- [ ] Run identical benchmark (2 hours)
- [ ] Compare results (2 hours)
- [ ] Update competitive positioning (2 hours)

**Timeline**: 3 days for initial competitive validation

---

**Last Updated**: October 30, 2025
**Status**: Ready to execute - Profile → SIMD → Qdrant benchmark
**Next Action**: Enable SIMD feature flag (5 minutes, in Cargo.toml)
