# Engine Optimization Strategy

**Date**: October 30, 2025
**Strategic Pivot**: Optimize engine → Then benchmark competitors → Then build server

---

## Executive Summary

**Critical Finding**: SIMD available but NOT ENABLED (2-4x free win)

**Quick Answers**:
- Should we profile first? **NO** - Enable SIMD first (obvious 2-4x win)
- Should we benchmark Qdrant now? **NO** - Optimize engine first (avoid premature comparison)
- When to build server? **AFTER** engine is solid (2-4 weeks after optimization)
- Expected improvement? **6-10x cumulative** (Week 1-4)

**Critical Actions** (THIS WEEK):
1. Enable SIMD (5 min, 2-4x improvement) ⚠️ HIGHEST PRIORITY
2. Enable LTO (1 min, 5-15% improvement)
3. Enable opt-level=3 (1 min, 5-10% improvement)
4. Profile (after above, identify real bottlenecks)

**Performance Projections**:
- Current: 162 QPS (6.16ms p95)
- Week 1: 400-500 QPS (SIMD + quick wins)
- Week 4: 1000 QPS (profiling + algorithms)
- Qdrant: 626 QPS @ 99.5% recall (we'll be competitive)

---

## Table of Contents

1. [Current Algorithm State](#current-algorithm-state)
2. [What's Missing](#whats-missing)
3. [Optimization Phases](#optimization-phases)
4. [Performance Projections](#performance-projections)
5. [Why Engine-First Approach](#why-engine-first-approach)
6. [Action Plan](#action-plan)

---

## Current Algorithm State

### What We Have ✅

| Component | Status | Performance | Optimization Status |
|-----------|--------|-------------|---------------------|
| HNSW | ✅ Working (hnsw_rs 0.3) | 31.05s build (3220 vec/sec) | ⚠️ SIMD not enabled |
| Binary Quantization | ✅ Working (7 tests) | 19.9x memory reduction | ⚠️ Not SOTA (need Extended RaBitQ) |
| Parallel Building | ✅ Optimized | 97x vs pgvector, 16x speedup | ✅ Already optimized |
| Graph Serialization | ✅ Optimized | 4175x speedup (9.92s for 1M) | ✅ Already optimized |

**Tests**: 142 passing (101 Phase 1 + 41 Phase 2)

---

## What's Missing

### Critical Issues ⚠️

| Issue | Impact | Effort | Priority |
|-------|--------|--------|----------|
| **SIMD disabled** | 2-4x query speedup | 5 minutes | ⚠️⚠️⚠️ CRITICAL |
| LTO not enabled | 5-15% improvement | 1 minute | HIGH |
| opt-level not 3 | 5-10% improvement | 1 minute | HIGH |
| No profiling done | Unknown bottlenecks | 4 hours | HIGH |

### SOTA Algorithms Not Implemented

| Algorithm | Purpose | Effort | When |
|-----------|---------|--------|------|
| Extended RaBitQ | SOTA quantization | 2-3 weeks | Week 3-4 |
| HNSW-IF | Billion-scale | 2-3 weeks | Week 5-8 |
| MN-RU | Delete/update | BLOCKED | Future (need fork) |
| Filtered search | Metadata filtering | 1 week | Week 3-4 |

---

## Optimization Phases

### Phase 1: Quick Wins (THIS WEEK) ⚠️ HIGHEST PRIORITY

**Goal**: Get largest wins BEFORE profiling

**Actions**:

1. **Enable SIMD** (5 minutes)
   ```toml
   # Cargo.toml
   default = ["hnsw-simd"]
   ```
   - Expected: 2-4x query speedup
   - Risk: None

2. **Enable LTO** (1 minute)
   ```toml
   [profile.release]
   lto = "thin"
   codegen-units = 1
   ```
   - Expected: 5-15% improvement
   - Risk: None

3. **Enable CPU Optimizations** (1 minute)
   ```toml
   [profile.release]
   opt-level = 3
   ```
   - Expected: 5-10% improvement
   - Risk: None

**Total Phase 1 Impact**: 2-5x improvement in 10 minutes

---

### Phase 2: Profiling & Targeted Optimization (WEEK 1-2)

**Goal**: Find real bottlenecks AFTER picking low-hanging fruit

**Tools**:
- CPU: `cargo flamegraph --features hnsw-simd`
- Memory: `heaptrack ./target/release/benchmark`
- Micro: `cargo bench`

**Expected Findings**:
- Distance calculations (hot path)
- HNSW graph traversal (cache misses)
- Vector allocations (temporary buffers)

**Optimizations**:
- Reduce allocations (object pooling)
- Better memory layout (cache-friendly)
- Custom SIMD (if needed)

**Total Phase 2 Impact**: Additional 2-3x improvement

---

### Phase 3: Algorithmic Improvements (WEEK 3-4)

**Goal**: Implement SOTA algorithms from research

| Improvement | Impact | Effort |
|-------------|--------|--------|
| Extended RaBitQ | 10-20% better recall at same compression | 2-3 weeks |
| Filtered search | Competitive parity with Qdrant | 1 week |
| Query optimization | 2-3x throughput (batch/parallel) | 1 week |

**Total Phase 3 Impact**: 2-4x additional improvement

---

### Phase 4: Scale Optimization (WEEK 5-8)

**Goal**: Prepare for billion-scale

| Feature | Impact | Effort |
|---------|--------|--------|
| HNSW-IF | 100M-1B vectors on single node | 2-3 weeks |
| 10M validation | Validate scale claims | 1 week |

**Total Phase 4 Impact**: Billion-scale support

---

## Performance Projections

### Current Baseline (100K vectors)
- Build: 31.05s (3220 vec/sec)
- Query: 6.16ms p95 (~162 QPS)

### After Phase 1 (Quick wins - THIS WEEK)
- Build: ~31s (parallel already optimized)
- Query: ~2-3ms p95 (~400-500 QPS)
- **Improvement: 2-5x queries**

### After Phase 2 (Profiling - WEEK 1-2)
- Build: ~25s (profile-guided optimization)
- Query: ~1.5ms p95 (~600-800 QPS)
- **Cumulative: 4-8x queries**

### After Phase 3 (Algorithms - WEEK 3-4)
- Build: ~20s (algorithmic improvements)
- Query: ~1ms p95 (~1000 QPS)
- **Cumulative: 6-10x queries**

### Competitive Position After Optimization

| Metric | Current | After Optimization | Qdrant | Result |
|--------|---------|-------------------|--------|--------|
| Build (100K) | 31.05s | ~20s | ? | Likely faster |
| Query p95 | 6.16ms (162 QPS) | ~1ms (1000 QPS) | 626 QPS @ 99.5% | Competitive or better |
| Build speedup | 97x vs pgvector | ~150x vs pgvector | ? | Market leading |

**Verdict**: After optimization, competitive with Qdrant or better

---

## Why Engine-First Approach

### Timeline Comparison

**WRONG Approach** (compare too early):
```
Week 1: Benchmark Qdrant → Find we're 10x slower
Week 2: Profile & optimize → Find SIMD was disabled
Week 3: Re-benchmark → Still finding issues
Week 4: More optimization → Wasted benchmarking cycles
```

**RIGHT Approach** (optimize first):
```
Week 1: Enable SIMD + Profile + Quick wins → 5-10x improvement
Week 2: Algorithmic improvements → Additional 2-3x
Week 3: Benchmark Qdrant → Competitive or better
Week 4: Fine-tune based on gaps
```

### Why This Matters

| Reason | Explanation |
|--------|-------------|
| Avoid premature comparison | Don't benchmark with obvious optimizations missing |
| Maximize learnings | Profile AFTER low-hanging fruit picked |
| Build confidence | Compare when engine at its best |
| Save time | One benchmark cycle vs multiple iterations |

### Why Not Profile First?

**Common wisdom**: "Profile first, optimize what matters"

**Why this is WRONG here**:
1. **Obvious wins**: SIMD disabled (2-4x for free)
2. **Noise reduction**: Profiling with SIMD disabled shows wrong bottlenecks
3. **Time waste**: Why profile when we know SIMD is the problem?
4. **Confirmation**: After SIMD, profile shows what's REALLY slow

**Analogy**: Don't profile a car with parking brake on

---

## Server Development

**When**: AFTER engine is solid (not before)

**Why**:
- Can't sell slow server
- Solid foundation first
- Server adds features, not performance

**Server is just a wrapper**:
- Multi-tenancy layer
- API endpoints
- Auth & billing
- **Effort: 2-4 weeks** (after engine ready)

---

## Action Plan

### TODAY (30 minutes)

**1. Enable SIMD** (5 minutes):
```bash
# Edit Cargo.toml
default = ["hnsw-simd"]

# Rebuild and test
cargo test --release
```

**2. Enable LTO** (1 minute):
```toml
[profile.release]
lto = "thin"
codegen-units = 1
opt-level = 3
```

**3. Benchmark Before/After** (20 minutes):
```bash
# Before SIMD
time ./target/release/benchmark_pgvector_comparison 10000

# After SIMD
cargo build --release
time ./target/release/benchmark_pgvector_comparison 10000
```

**Expected**: 2-4x query improvement in 30 minutes

---

### THIS WEEK (2-3 days)

**4. Profile with SIMD enabled** (4 hours):
```bash
cargo install flamegraph
cargo flamegraph --bin benchmark_pgvector_comparison --features hnsw-simd -- 100000
```

**5. Implement top 3 optimizations** (1-2 days):
- Reduce allocations
- Object pooling
- Better memory layout

**6. Re-benchmark** (1 hour):
```bash
cargo test --release
./target/release/benchmark_pgvector_comparison 100000
```

**Expected**: 5-10x cumulative improvement by end of week

---

## Success Metrics

### Minimum Success (Week 1)
- ✅ SIMD enabled: 2-4x query improvement
- ✅ Profiling complete: Bottlenecks identified
- ✅ Quick wins implemented: 5-10x cumulative

### Target Success (Week 2-4)
- ✅ Algorithmic improvements: Extended RaBitQ, filtered search
- ✅ Query performance: ~1ms p95 (~1000 QPS)
- ✅ Competitive with Qdrant (within 50%)

### Stretch Success (Week 5-8)
- ✅ HNSW-IF: Billion-scale support
- ✅ Match or beat Qdrant
- ✅ Market-leading PostgreSQL-compatible vector DB

---

## Competitive Benchmarking Timeline

### OLD Plan (Wrong)
- Week 1: Benchmark Qdrant
- Week 2-4: Optimize based on findings
- Week 5: Re-benchmark

### NEW Plan (Correct)
- **Week 1**: Enable SIMD + Profile + Quick wins → 2-5x improvement
- **Week 2**: Targeted optimizations → 4-8x cumulative
- **Week 3-4**: Algorithmic improvements → 6-10x cumulative
- **Week 5**: Benchmark Qdrant (NOW we're ready)
- **Week 6**: Fine-tune based on gaps

**Rationale**: Compare when engine is optimized, not before

---

## Major Algorithm Optimizations Available

**From SOTA Research**: ai/research/SOTA_ALGORITHMS_INVESTIGATION_OCT2025.md

| Algorithm | Status | When | Impact |
|-----------|--------|------|--------|
| **Extended RaBitQ** | Ready to implement | Week 3-4 | Better accuracy at same compression |
| **HNSW-IF** | Ready to implement | Week 5-8 | Billion-scale (Vespa-proven) |
| **MN-RU** | BLOCKED | Future | Delete/update (hnsw_rs doesn't support) |
| **NGT-QG** | Alternative | Maybe never | Not clearly better than Extended RaBitQ |

**Priority Order**:
1. Enable SIMD (this week, 5 minutes) ⚠️
2. Profile & optimize (week 1-2)
3. Extended RaBitQ (week 3-4, SOTA quantization)
4. Filtered search (week 3-4, competitive parity)
5. HNSW-IF (week 5-8, billion-scale)
6. MN-RU (future, when possible)

---

## Conclusion

**Strategic Pivot**: ✅ **OPTIMIZE ENGINE FIRST, THEN COMPARE**

**Quick Wins Available**:
1. ⚠️ **SIMD** (5 minutes, 2-4x) - DO THIS NOW
2. LTO (1 minute, 5-15%)
3. CPU optimizations (1 minute, 5-10%)

**Why This Approach**:
- Don't waste time benchmarking unoptimized code
- Pick low-hanging fruit before profiling
- Build confidence with solid foundation
- Compare when engine at its best

**Timeline**:
- Week 1: 5-10x improvement (quick wins + profiling)
- Week 2-4: 6-10x cumulative (algorithms)
- Week 5: Benchmark Qdrant (competitive or better)
- Week 6-8: Fine-tune, scale validation

**Server Development**: Wait until engine solid (2-4 weeks after)

---

**Next Step**: Enable SIMD (5 minutes), then profile

**Last Updated**: October 30, 2025
**Status**: Ready to execute - SIMD first!
