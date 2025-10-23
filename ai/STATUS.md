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

**Day 0 (Oct 22 Evening): Research & Planning** - COMPLETED ✅
- [x] Research HNSW algorithm (Malkov & Yashunin 2018 paper)
- [x] Evaluate Rust implementations (instant-distance, hnsw_rs)
- [x] Choose hnsw_rs (SIMD, full parameter control, persistence)
- [x] Design HNSW index structure for 1536D vectors
- [x] Create comprehensive research document (250+ lines)
- [x] Create tactical implementation plan (7-day timeline)

**Research Findings**:
- **Algorithm**: Industry-proven (Qdrant, Pinecone, Weaviate, pgvecto.rs all use HNSW)
- **Parameters**: M=48-64, ef_construction=200-400, ef_search=100-500
- **Expected**: >95% recall, <10ms p95 latency, ~500 bytes/vector
- **Production validation**: pgvecto.rs is 20x faster than pgvector with HNSW

**HNSW Implementation Timeline**:
- [x] Day 0 (Oct 22): Research + planning (COMPLETED)
- [ ] Day 1 (Oct 23): Setup + basic integration (hnsw_rs dependency, wrapper)
- [ ] Day 2 (Oct 24): RocksDB integration (serialization, storage)
- [ ] Day 3 (Oct 25): PostgreSQL protocol (distance operators)
- [ ] Day 4 (Oct 26): INSERT optimization (batch, parallel)
- [ ] Day 5 (Oct 27): Search optimization (ef_search tuning, SIMD)
- [ ] Day 6 (Oct 28): Benchmark (100K vectors)
- [ ] Day 7 (Oct 29): Validation + go/no-go decision

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

