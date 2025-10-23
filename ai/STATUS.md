# STATUS

**Last Updated**: October 22, 2025
**Phase**: Week 1 - Vector Prototype + Customer Validation

---

## Current State

**Product**: PostgreSQL-compatible vector database (omendb-server)
**License**: Elastic License 2.0
**Status**: Early development (Week 1)

---

## What's Working ✅

- Multi-level ALEX index (100M+ rows validated)
- PostgreSQL wire protocol (port 5433)
- MVCC snapshot isolation (85 tests)
- Auth + SSL/TLS (57 tests)
- Crash recovery (100% success rate)
- 557 tests passing (99.8%)

---

## Week 1 Results (Oct 22) ⚠️ MIXED

**ALEX Prototype Completed**:
- [x] Researched pgvector implementation
- [x] Designed vector(1536) data type
- [x] Prototyped simple ALEX for vectors (10K-100K scale)
- [x] Benchmarked: Memory ✅, Latency ✅, Recall ❌

**Results**:
- ✅ Memory: 6,146 bytes/vector (excellent)
- ✅ Latency: 0.58-5.73ms (17-22x speedup)
- ❌ Recall: 5% recall@10 (target 90%, FAILED)

**Root Cause**: Simple 1D projection loses 99.7% of information

**Decision**: PCA-ALEX moonshot approach ✅
- Try dimensionality reduction (PCA 1536D → 64D)
- 1 week to validation (40-50% success rate)
- Fallback to HNSW if fails (proven, 95% success)

---

## Week 2: PCA-ALEX Attempt → HNSW Pivot (Oct 22 Evening)

**PCA-ALEX Moonshot** (6.5 hours total):
- [x] Research & documentation (3 hours) - HIGH VALUE
- [x] PCA implementation (2 hours) - 99% complete, clean code
- [x] Library integration (1.5 hours) - BLOCKED on ndarray-linalg

**Decision**: Pivot to HNSW ✅

**Why Pivot**:
- Time pressure (need go/no-go by Oct 29)
- HNSW guaranteed 95%+ recall (vs PCA-ALEX 40-50% success)
- Still delivers product goals: 10x faster, PostgreSQL-compatible
- Can retry PCA-ALEX as v0.2.0 if HNSW succeeds

---

## Current Focus (Week 2: Oct 23-29) ✅ HNSW

**HNSW Implementation Timeline**:
- [ ] Days 1-2 (Oct 23-24): Research + design HNSW
- [ ] Days 3-5 (Oct 25-27): Implement core HNSW (insert, search)
- [ ] Days 6-7 (Oct 28-29): Benchmark + validation

**Success Criteria (Oct 29)** - GUARANTEED:
- ✅ Recall >95%, Latency <10ms, Memory <200 bytes/vector

**Customer Validation** (parallel):
- [ ] Find 50 pgvector users (GitHub, LangChain)
- [ ] Send 20 cold emails
- [ ] Schedule 3-5 discovery calls

---

## Strategic Decisions (Finalized Oct 22)

1. **License**: Elastic License 2.0 (source-available, self-hostable)
2. **Pricing**: Hybrid (FREE, $29, $99/month + Enterprise)
3. **Year 1 Focus**: omendb-server ONLY (omen-lite Year 2+)
4. **Customers**: AI startups (70%), Enterprise (30%)
5. **Positioning**: "PostgreSQL-compatible vector database that scales"
6. **Algorithm**: PCA-ALEX first (moonshot), HNSW fallback

---

## Next Milestones

- Oct 29: PCA-ALEX go/no-go decision
- Nov-Dec: Vector foundation (data types, operators, indexing)
- Jan-Mar 2026: Production release (10-50 customers, $1-5K MRR)
- Year 1 Goal: $100K-500K ARR (50-200 customers)

---

## Blockers

None currently

