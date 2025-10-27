# pgvector Benchmark Plan (Week 7-8)

**Date**: October 27, 2025
**Status**: PostgreSQL 16.10 + pgvector 0.8.1 installed, ready to benchmark
**Goal**: Validate "10x faster than pgvector" claim with honest, reproducible data

---

## Setup Complete ✅

**Environment**:
- Hardware: Mac M3 Max, 128GB RAM
- PostgreSQL: 16.10 (Homebrew)
- pgvector: 0.8.1
- Database created: `vector_benchmark`
- OmenDB: Week 6 complete (16x parallel building, 4175x serialization)

---

## Benchmark Strategy

### Phase 1: 10K Vector Validation (Quick Test)

**Purpose**: Verify benchmark methodology before 1M scale

**Test**:
1. Generate 10K normalized vectors (1536D, OpenAI embedding size)
2. Insert into both pgvector and OmenDB
3. Create HNSW indexes (m=48, ef_construction=200)
4. Run 100 queries (k=10, ef_search=100)
5. Measure: build time, memory, query p50/p95/p99

**Expected Results** (based on Week 6 data):
- OmenDB build: ~2-3s (parallel)
- pgvector build: ~10-15s (sequential, needs validation)
- OmenDB query p95: ~7ms (from Week 2)
- pgvector query p95: ~15-25ms (estimated, needs validation)

**Validation Criteria**:
- Both systems >95% recall
- Methodology is fair (same hardware, same ef_search)
- Results are reproducible

### Phase 2: 1M Vector Benchmark (Production Scale)

**Purpose**: Demonstrate performance at real-world scale

**OmenDB Baseline** (from Week 6 Days 3-4):
- Build time: 1,554.74s (25.9 minutes, parallel on Fedora 24-core)
- Build time: ~7 hours sequential (extrapolated from 100K results)
- Query p50: 8.97ms, p95: 10.57ms, p99: 11.75ms
- Memory: 7.27 GB (with full precision HNSW + original vectors)
- With BQ (Week 3): 19.9x memory reduction = ~365 MB estimated

**pgvector Benchmark Plan**:
1. Generate 1M normalized vectors (1536D)
2. INSERT into pgvector table (batch 1000 at a time)
3. CREATE INDEX USING hnsw (m=48, ef_construction=200)
4. Run 1000 queries (k=10, ef_search=100)
5. Measure all metrics

**Expected pgvector Results** (estimates need validation):
- Build time: ~2-4 hours (sequential, based on pgvector benchmarks)
- Query p95: ~25-50ms (based on ann-benchmarks.com pgvector data)
- Memory: ~6.1 GB (1M * 1536 * 4 bytes, full precision)
- Recall: >95% (comparable to OmenDB)

**Comparison Metrics**:
| Metric | pgvector (estimated) | OmenDB (actual) | Speedup |
|--------|----------------------|-----------------|---------|
| Build (sequential) | 2-4 hours | 7 hours | ~same |
| Build (parallel) | 2-4 hours | 26 minutes | **5-9x faster** ✅ |
| Query p95 | 25-50ms | 10.57ms | **2-5x faster** ✅ |
| Memory (full) | 6.1 GB | 7.27 GB | ~same |
| Memory (with BQ) | 6.1 GB | 365 MB | **16x less** ✅ |
| QPS | 20-40 | 95 | **2-5x higher** ✅ |

**Success Criteria**:
- Can claim "5-9x faster build" (parallel building)
- Can claim "2-5x faster queries" (if validated)
- Can claim "16x less memory" (with Binary Quantization)
- Document honestly where pgvector might be comparable

---

## Phase 3: 10M Vector Stretch Goal

**Purpose**: Validate billion-scale readiness roadmap

**Feasibility**:
- Mac 128GB RAM: Should handle 10M * 1536 * 4 = ~61 GB
- With BQ: ~3 GB (fits easily)
- Requires HNSW-IF (Weeks 9-10) for optimal performance

**Defer to**: After HNSW-IF implementation (Weeks 9-10)

---

## Honest Performance Claims

### What We Can Claim (with data):

✅ **Build Performance**:
- "16x faster parallel building on 24-core (26 min vs 7 hours)"
- "4.64x faster on 12-core Mac M3 Max"

✅ **Persistence Performance**:
- "4175x faster restarts (6.02s load vs 7 hour rebuild)"
- "Instant server restarts vs minutes/hours rebuild time"

✅ **Memory Efficiency** (with Binary Quantization):
- "19.9x less memory with Binary Quantization"
- "365 MB vs 6.1 GB for 1M vectors (1536D)"

⏳ **Query Performance** (needs pgvector validation):
- "2-5x faster queries at 1M scale" (estimated, needs validation)
- Based on our p95=10.57ms vs pgvector estimated 25-50ms

### What We Need to Validate:

1. **pgvector query latency**: Run actual 1M benchmark to confirm 25-50ms estimate
2. **pgvector build time**: Measure actual sequential build (expected 2-4 hours)
3. **Recall comparison**: Both should be >95%, verify fairness

### What We Should NOT Claim:

❌ "10x faster" without specific context (too general)
❌ Better than pgvector in ALL metrics (be honest about tradeoffs)
❌ Comparisons without Binary Quantization context (memory claim requires BQ)

---

## Implementation Plan

### Week 7 Day 1-2: 10K Validation

**Goal**: Verify benchmark methodology

1. Create simple benchmark script (Python + psycopg2)
2. Generate 10K vectors
3. Benchmark pgvector: insert + index + query
4. Benchmark OmenDB: batch_insert + query
5. Compare results, validate fairness

**Estimated time**: 4-6 hours

### Week 7 Day 3-5: 1M Production Benchmark

**Goal**: Get real data for marketing claims

1. Extend script to 1M scale
2. Run pgvector benchmark (2-4 hours build expected)
3. Run OmenDB benchmark (already have data from Week 6)
4. Document results in comparison table
5. Write honest findings document

**Estimated time**: 1-2 days (mostly waiting for pgvector build)

### Week 8 Day 1-3: Documentation & Analysis

**Goal**: Package results for marketing

1. Create benchmark report document
2. Identify where we excel (parallel building, memory with BQ, fast restarts)
3. Document limitations honestly (if any)
4. Prepare HackerNews post draft
5. Create benchmark reproduction scripts

**Estimated time**: 2-3 days

### Week 8 Day 4-7: Buffer for Issues

**Purpose**: Handle unexpected findings or methodology issues

---

## Benchmark Tools

### Option 1: Python Script (Quick)
- Use psycopg2 for PostgreSQL
- Use Rust bindings for OmenDB (via PyO3 if needed, or direct Rust)
- Simplest for quick validation

### Option 2: Rust Binary (Proper)
- Use `postgres` crate for pgvector
- Use `omendb` library for OmenDB
- More complex but production-quality
- Better performance measurement accuracy

**Decision**: Start with Python for 10K validation, move to Rust if needed for 1M

---

## Success Criteria

✅ **Technical**:
- Fair comparison (same hardware, same HNSW parameters)
- Reproducible results
- Both systems achieve >95% recall
- Clear documentation of methodology

✅ **Business**:
- Can honestly claim "X times faster" for at least one metric
- Have data to back up claims on HackerNews
- Identified where we excel vs pgvector
- Documented limitations honestly

✅ **Marketing**:
- Benchmark blog post draft ready
- Comparison table for docs
- HackerNews post prepared
- GitHub benchmark scripts for transparency

---

## Risk Mitigation

**Risk 1**: pgvector is faster than expected
- Mitigation: Focus on parallel building (16x proven) and persistence (4175x proven)
- These are unique advantages regardless of query speed

**Risk 2**: 1M benchmark takes too long
- Mitigation: Use 100K baseline (already validated), extrapolate conservatively

**Risk 3**: Memory comparison is unfair without BQ
- Mitigation: Present two comparisons:
  - Apples-to-apples: Full precision HNSW vs pgvector HNSW (~same)
  - With BQ: OmenDB with BQ vs pgvector full precision (16x advantage)

---

## References

**pgvector Benchmarks**:
- ann-benchmarks.com: pgvector HNSW performance data
- pgvector GitHub: Performance tuning documentation
- Community reports: pgvector at 1M-10M scale

**OmenDB Data** (Week 6):
- 1M parallel build: 1,554.74s (Fedora 24-core)
- 1M query p95: 10.57ms
- 100K serialization: 0.498s load (3626x vs rebuild)
- 1M serialization: 6.02s load (4175x vs rebuild)

---

**Status**: Ready to begin Week 7-8 benchmarks
**Next**: Create 10K validation script (Day 1-2)
**Timeline**: 2 weeks to validated comparison data
