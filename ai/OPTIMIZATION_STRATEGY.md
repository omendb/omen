# Engine Optimization Strategy: Focus on Core Performance First

**Date**: October 30, 2025
**Strategic Pivot**: Optimize engine → Then benchmark competitors → Then build server
**Rationale**: No point comparing against Qdrant until we've optimized our engine

---

## Executive Summary

**Critical Finding**: We have SIMD support available but NOT ENABLED!
- Feature: `hnsw-simd` exists in Cargo.toml
- Status: NOT in default features
- **Impact: 2-4x speedup on distance calculations** (FREE WIN)

**Current State Assessment**:
- ✅ Binary Quantization: Implemented and tested (19.9x memory reduction)
- ✅ HNSW: Implemented via hnsw_rs 0.3
- ❌ SIMD: Available but not enabled (CRITICAL MISS)
- ❌ Profiling: Not done yet
- ❌ Extended RaBitQ: Not implemented (SOTA quantization from research)
- ❌ HNSW-IF: Not implemented (billion-scale from research)

**Strategic Recommendation**:
**OPTIMIZE ENGINE FIRST**, then compare against competitors

---

## Why Engine-First Approach is Correct

### Current vs Proposed Timeline

**WRONG Approach** (comparing too early):
```
Week 1: Benchmark Qdrant → Find we're 10x slower
Week 2: Profile & optimize → Find we had SIMD disabled
Week 3: Re-benchmark → Still finding issues
Week 4: More optimization → Wasted benchmarking time
```

**RIGHT Approach** (optimize first):
```
Week 1: Enable SIMD (2-4x) + Profile + Quick wins → 5-10x improvement
Week 2: Algorithmic improvements → Additional 2-3x
Week 3: Benchmark Qdrant → Competitive or better
Week 4: Fine-tune based on gaps
```

### Why This Matters

1. **Avoid premature comparison**: Don't benchmark with obvious optimizations missing
2. **Maximize learnings**: Profile AFTER low-hanging fruit picked
3. **Build confidence**: Compare when engine is at its best
4. **Save time**: One benchmark cycle vs multiple iterations

### Server Can Wait

**Engine readiness comes first**:
- Solid foundation before building server
- Can't sell slow server
- Server adds features, not performance

**Server is just a wrapper**:
- Multi-tenancy layer
- API endpoints
- Auth & billing
- **2-4 weeks of work AFTER engine is solid**

---

## Current Algorithm State

### What We Have ✅

**1. HNSW (Hierarchical Navigable Small World)**
- Implementation: hnsw_rs 0.3
- Status: Working, tested (142 tests passing)
- Performance: 31.05s for 100K vectors (3220 vec/sec build)
- **Optimization opportunity**: SIMD not enabled!

**2. Binary Quantization (BQ)**
- Implementation: src/quantization/ module
- Status: Implemented and tested (7 BQ tests)
- Memory reduction: 19.9x
- Recall: 33% baseline, 70% with reranking
- **Gap**: Not SOTA (Extended RaBitQ is better)

**3. Parallel Building**
- Implementation: Rayon-based batch insertion
- Status: Working (97x faster than pgvector)
- Speedup: 16.17x on Fedora 24-core
- **Already optimized**: Major win here

**4. Graph Serialization**
- Implementation: hnsw_rs file_dump
- Status: Working (4175x speedup)
- Performance: 9.92s to save 1M vectors
- **Already optimized**: Another major win

### What We're Missing ❌

**1. SIMD Distance Calculations** ⚠️ CRITICAL
- Available: hnsw-simd feature in Cargo.toml
- Status: NOT ENABLED
- Expected impact: 2-4x speedup on queries
- **Effort: 5 minutes** (add to default features)

**2. Extended RaBitQ** (SOTA Quantization)
- Research status: Documented in SOTA_ALGORITHMS_INVESTIGATION
- Current: Basic Binary Quantization
- Improvement: Better accuracy at same compression
- **Effort: 2-3 weeks**

**3. HNSW-IF** (Hybrid In-Memory/Disk)
- Research status: Documented, ready to implement
- Purpose: Billion-scale support
- Vespa-proven approach
- **Effort: 2-3 weeks**

**4. MN-RU** (Delete/Update Support)
- Research status: Documented, blocked by hnsw_rs
- Blocker: hnsw_rs doesn't support delete/update
- **Effort: Blocked** (need to fork hnsw_rs or wait)

---

## Optimization Priorities (Correct Order)

### Phase 1: Quick Algorithmic Wins (THIS WEEK) ⚠️ HIGHEST PRIORITY

**Goal**: Get largest wins with minimal effort BEFORE profiling

**1. Enable SIMD** ⭐⭐⭐ CRITICAL (5 minutes)
```toml
# Change Cargo.toml from:
default = []

# To:
default = ["hnsw-simd"]
```
- **Expected**: 2-4x query speedup
- **Effort**: 5 minutes
- **Risk**: None (well-tested feature)
- **Why first**: Free 2-4x win before we spend time profiling

**2. Enable LTO (Link-Time Optimization)** ⭐⭐ HIGH (1 minute)
```toml
[profile.release]
lto = "thin"  # or "fat" for maximum optimization
codegen-units = 1
```
- **Expected**: 5-15% improvement
- **Effort**: 1 minute
- **Risk**: None

**3. Enable CPU-Specific Optimizations** ⭐⭐ MEDIUM (1 minute)
```toml
[profile.release]
opt-level = 3
```
-
 **Expected**: 5-10% improvement
- **Effort**: 1 minute

**Total Phase 1 Impact**: 2-5x improvement in 10 minutes!

---

### Phase 2: Profiling & Targeted Optimization (WEEK 1-2)

**Now that we've picked low-hanging fruit, profile to find REAL bottlenecks**

**1. CPU Profiling** ⭐⭐⭐
```bash
cargo install flamegraph
cargo flamegraph --bin benchmark_pgvector_comparison --features hnsw-simd -- 100000
```
- Find hot paths
- Identify allocation sites
- Measure cache misses

**2. Memory Profiling** ⭐⭐
```bash
heaptrack ./target/release/benchmark_pgvector_comparison 100000
```
- Find memory leaks
- Identify allocation patterns
- Optimize memory layout

**3. Micro-benchmarks** ⭐⭐
```bash
cargo bench
```
- Distance calculations
- HNSW traversal
- Quantization operations

**Expected findings**:
- Distance calculations (likely hot path)
- HNSW graph traversal (cache misses)
- Vector allocations (temporary buffers)

**Optimizations based on profiling**:
- Reduce allocations (object pooling)
- Better memory layout (cache-friendly)
- SIMD for custom code (if not already covered)

**Total Phase 2 Impact**: Additional 2-3x improvement

---

### Phase 3: Algorithmic Improvements (WEEK 3-4)

**Implement SOTA algorithms from research**

**1. Extended RaBitQ** ⭐⭐⭐ (2-3 weeks)
- Implementation: Follow SIGMOD 2025 paper
- Benefit: Better accuracy at same memory footprint
- Replaces: Current Binary Quantization
- **Impact**: 10-20% better recall at same compression

**2. Filtered Search** ⭐⭐ (1 week)
- Implementation: Metadata filtering during HNSW traversal
- Target: <10% overhead (like Qdrant)
- **Impact**: Competitive feature parity

**3. Query Optimization** ⭐⭐ (1 week)
- Batch query processing
- Parallel query execution (Rayon)
- Result caching
- **Impact**: 2-3x throughput improvement

**Total Phase 3 Impact**: 2-4x additional improvement

---

### Phase 4: Scale Optimization (WEEK 5-8)

**Prepare for billion-scale**

**1. HNSW-IF Implementation** ⭐⭐⭐ (2-3 weeks)
- Hybrid in-memory/disk HNSW
- Keep hot layers in memory
- Store cold layers on disk
- **Impact**: 100M-1B vectors on single node

**2. 10M Validation** ⭐⭐ (1 week)
- Memory optimization
- Disk I/O optimization
- **Impact**: Validate scale claims

**Total Phase 4 Impact**: Billion-scale support

---

## Cumulative Expected Performance

### Current Baseline (100K vectors):
- Build: 31.05s (3220 vec/sec)
- Query: 6.16ms p95 (~162 QPS)

### After Phase 1 (Quick wins - THIS WEEK):
- Build: ~31s (same, parallel already optimized)
- Query: ~2-3ms p95 (~400-500 QPS)
- **Improvement: 2-5x queries**

### After Phase 2 (Profiling - WEEK 1-2):
- Build: ~25s (profile-guided optimization)
- Query: ~1.5ms p95 (~600-800 QPS)
- **Cumulative: 4-8x queries**

### After Phase 3 (Algorithms - WEEK 3-4):
- Build: ~20s (algorithmic improvements)
- Query: ~1ms p95 (~1000 QPS)
- **Cumulative: 6-10x queries**

### Competitive Position After Optimization:

| Metric | OmenDB (Current) | OmenDB (After Optimization) | Qdrant | Gap |
|--------|------------------|----------------------------|--------|-----|
| Build (100K) | 31.05s | ~20s | ? | Likely faster |
| Query p95 | 6.16ms (~162 QPS) | ~1ms (~1000 QPS) | 626 QPS @ 99.5% | Competitive |
| Build speedup | 97x vs pgvector | ~150x vs pgvector | ? | Leading |

**Verdict**: After optimization, we're competitive with Qdrant or better

---

## Why Not Profile First?

**Common wisdom**: "Profile first, optimize what matters"

**Why this is wrong here**:

1. **Obvious wins**: SIMD is sitting there disabled (2-4x for free)
2. **Noise reduction**: Profiling with SIMD disabled will show wrong bottlenecks
3. **Time waste**: Why spend hours profiling when we know SIMD is the problem?
4. **Confirmation**: After SIMD, profile will confirm what's REALLY slow

**Analogy**:
- Don't profile a car with the parking brake on
- First: Release the brake (enable SIMD)
- Then: Profile to find real issues

---

## What About HNSW+ Optimizations?

### hnsw_rs Library State

**Current**: hnsw_rs 0.3
- Maintained library
- Good performance
- SIMD support available
- Parallel building support

**Potential hnsw_rs Optimizations**:

**1. SIMD** ⭐⭐⭐
- Status: Available via feature flag
- **Action: Enable it** (Phase 1)

**2. Custom Distance Functions**
- Current: Uses built-in L2, cosine
- Opportunity: Write SIMD-optimized custom distances
- **Effort: 1-2 weeks** (if needed after profiling)

**3. Memory Layout**
- Current: Standard Vec<T> storage
- Opportunity: Cache-friendly layout, alignment
- **Effort: 2-3 weeks** (if profiling shows cache misses)

**4. Graph Traversal**
- Current: Standard HNSW algorithm
- Opportunity: Prefetching, branch prediction
- **Effort: 2-3 weeks** (advanced optimization)

**Recommendation**: Enable SIMD first, then profile to see if custom work is needed

---

## Major Algorithm Optimizations Available

### From SOTA Research (ai/research/SOTA_ALGORITHMS_INVESTIGATION_OCT2025.md)

**1. Extended RaBitQ** ⭐⭐⭐ (Weeks 11-12 in roadmap)
- Paper: SIGMOD 2025
- Current: Basic Binary Quantization (19.9x compression)
- Improvement: Arbitrary compression rates (4x-32x)
- Better accuracy at same memory
- **When**: After basic optimizations (Week 3-4)

**2. HNSW-IF** ⭐⭐⭐ (Weeks 9-10 in roadmap)
- Paper: Vespa (proven in production)
- Current: In-memory only
- Improvement: Hybrid memory/disk, billion-scale
- Keep hot layers in memory, cold on disk
- **When**: After 10M validation (Week 5-8)

**3. MN-RU** ⚠️ (Blocked)
- Paper: State-of-the-art delete/update
- Blocker: hnsw_rs doesn't support delete/update
- **When**: After forking hnsw_rs or upstream support

**4. NGT-QG** (Alternative, not priority)
- Yahoo's quantization + graph optimization
- Not clearly better than Extended RaBitQ
- **When**: Maybe never (Extended RaBitQ is better)

### Priority Order:

1. **Enable SIMD** (this week, 5 minutes)
2. **Profile & optimize** (week 1-2)
3. **Extended RaBitQ** (week 3-4, SOTA quantization)
4. **Filtered search** (week 3-4, competitive parity)
5. **HNSW-IF** (week 5-8, billion-scale)
6. **MN-RU** (future, when possible)

---

## Competitive Benchmarking Timeline

### OLD Plan (Wrong):
- Week 1: Benchmark Qdrant
- Week 2-4: Optimize based on findings
- Week 5: Re-benchmark

### NEW Plan (Correct):
- **Week 1**: Enable SIMD + Profile + Quick wins → 2-5x improvement
- **Week 2**: Targeted optimizations from profiling → 4-8x cumulative
- **Week 3-4**: Algorithmic improvements → 6-10x cumulative
- **Week 5**: Benchmark Qdrant (NOW we're ready)
- **Week 6**: Fine-tune based on any remaining gaps

**Rationale**: Compare when engine is optimized, not before

---

## Action Plan (Immediate)

### TODAY (30 minutes):

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

# Document improvement
```

**Expected**: 2-4x query improvement in 30 minutes

### THIS WEEK (2-3 days):

**4. Profile with SIMD enabled** (4 hours):
```bash
cargo install flamegraph
cargo flamegraph --bin benchmark_pgvector_comparison --features hnsw-simd -- 100000
```

**5. Implement top 3 optimizations from profiling** (1-2 days):
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

### Minimum Success (Week 1):
- SIMD enabled: 2-4x query improvement ✅
- Profiling complete: Bottlenecks identified ✅
- Quick wins implemented: 5-10x cumulative ✅

### Target Success (Week 2-4):
- Algorithmic improvements: Extended RaBitQ, filtered search
- Query performance: ~1ms p95 (~1000 QPS)
- Competitive with Qdrant (within 50%)

### Stretch Success (Week 5-8):
- HNSW-IF: Billion-scale support
- Match or beat Qdrant
- Market-leading PostgreSQL-compatible vector DB

---

## Conclusion

**Strategic Pivot**: ✅ **OPTIMIZE ENGINE FIRST, THEN COMPARE**

**Quick Wins Available**:
1. ⚠️ **SIMD** (5 minutes, 2-4x improvement) - DO THIS NOW
2. LTO (1 minute, 5-15% improvement)
3. CPU optimizations (1 minute, 5-10% improvement)

**Why This Approach**:
- Don't waste time benchmarking unoptimized code
- Pick low-hanging fruit before profiling
- Build confidence with solid foundation
- Compare when engine is at its best

**Timeline**:
- Week 1: 5-10x improvement (quick wins + profiling)
- Week 2-4: 6-10x cumulative (algorithms)
- Week 5: Benchmark Qdrant (competitive or better)
- Week 6-8: Fine-tune, scale validation

**Server Development**:
- Wait until engine is solid
- 2-4 weeks of work after engine ready
- Just a wrapper (multi-tenancy, API, auth)

---

**Next Step**: Enable SIMD (5 minutes), then profile

**Last Updated**: October 30, 2025
**Status**: Ready to execute - SIMD first!
