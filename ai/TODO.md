# TODO

_Last Updated: 2025-10-23 - HNSW + BINARY QUANTIZATION VALIDATED_

## FINALIZED STRATEGY (Updated Oct 23)

**Product**: PostgreSQL-compatible vector database that scales
**Algorithm**: HNSW + Binary Quantization (industry standard, proven)
**License**: Elastic License 2.0 (source-available, self-hostable)
**Pricing**: Free (100K vectors), $29, $99/month + Enterprise
**Market**: AI startups (70%), Enterprise (30%)

**Timeline**: 8 weeks to production-ready MVP with quantization

---

## ✅ Week 1-2 Complete: Vector Search Validation

### Week 1: ALEX Vector Prototype (FAILED)
- ✅ Research pgvector implementation
- ✅ Design vector(N) data type
- ✅ Prototype ALEX for 1536D vectors
- ✅ Benchmark: Memory ✅, Latency ✅, Recall ❌ (5% vs 90% target)
- ✅ **Root cause**: 1D projection loses too much information

### Week 2 Day 1-2: HNSW Baseline (SUCCESS ✅)
- ✅ Integrated hnsw_rs crate
- ✅ HNSW wrapper with M=48, ef_construction=200
- ✅ Benchmark: **99.5% recall**, **6.63ms p95 latency** (< 10ms target)
- ✅ 14 tests passing (6 HNSW + 4 PCA + 4 vector types)
- ✅ **Verdict**: Production-ready HNSW baseline achieved

### Week 2 Day 2: PCA-ALEX Moonshot (FAILED)
- ✅ Custom PCA implementation (99.58% variance, 0.0738ms p95)
- ✅ PCA-ALEX integration (64D PCA → 1D ALEX key)
- ✅ Benchmark vs HNSW: **12.4% recall** (vs 99.5% HNSW)
- ✅ **Root cause**: Collapsing 64D to 1D ALEX key loses spatial information
- ✅ **Verdict**: ALEX not suitable for high-dimensional vectors

### Week 2 Day 2: SOTA Research (COMPLETE)
- ✅ Comprehensive research: DiskANN, HNSW+, quantization methods
- ✅ 32+ citations (academic papers, industry blogs, benchmarks)
- ✅ 1,300+ line research report: `docs/architecture/research/sota_vector_search_algorithms_2024_2025.md`
- ✅ **Key findings**:
  - DiskANN has immutability/batching issues (why we abandoned it)
  - HNSW + Binary Quantization is industry standard (Qdrant, Weaviate, Elasticsearch)
  - RaBitQ (SIGMOD 2024): 96% memory reduction, 3x faster than PQ
  - pgvector uses 30x more memory (no quantization support)

---

## 🚀 Phase 1: HNSW + Binary Quantization (Weeks 3-10)

**Goal**: Production-ready vector database with industry-leading memory efficiency

### ✅ Week 3: Binary Quantization Implementation (COMPLETE)

**Core Quantization:**
- ✅ Implement binary quantization (RaBitQ-style):
  - ✅ float32 → 1 bit per dimension = 96% memory reduction
  - ✅ Randomized threshold selection (theoretical error bounds)
  - ✅ Reranking with original vectors (92.7% recall achieved)
- ✅ Quantization training:
  - ✅ Sample-based threshold computation
  - ✅ Per-dimension quantization (better than global)
  - ✅ Validation: measure quantization error
- ✅ Integration with HNSW:
  - ✅ Store quantized vectors in HNSW graph
  - ✅ Store original vectors for reranking
  - ✅ Two-phase search: BQ candidates → exact L2 refinement

**Benchmarks:**
- ✅ Memory comparison:
  - Achieved: 19.9x reduction (3.08 MB vs 61.44 MB for 10K vectors)
  - Potential: 19.9x for 10M vectors (3 GB vs 61 GB)
- ✅ Recall validation:
  - Achieved: 92.7% recall@10 @ 5.6ms (150x expansion)
  - Achieved: 95.1% recall@10 @ 6.9ms (200x expansion)
- ✅ Latency validation:
  - Achieved: 5.58ms p95 @ 150x expansion
  - Target met: <7ms for 95% recall

**Success Criteria:**
- ✅ 92.7% recall @ 5.6ms (150x expansion) - Production-ready
- ✅ 95.1% recall @ 6.9ms (200x expansion) - Meets recall target
- ✅ 19.9x memory reduction demonstrated
- ✅ 12x faster index building (1,576 vs 133 vectors/sec)

### Week 4-5: PostgreSQL Vector Integration

**Vector Data Type:**
- [ ] Implement `vector(N)` data type:
  - [ ] Variable dimensions (128-1536 supported)
  - [ ] Serialize/deserialize for PostgreSQL wire protocol
  - [ ] Input validation (dimension checking, NaN handling)
- [ ] Distance operators:
  - [ ] `<->` (L2 distance / Euclidean)
  - [ ] `<#>` (negative dot product for max inner product)
  - [ ] `<=>` (cosine distance)
- [ ] Vector functions:
  - [ ] `l2_distance(vector, vector)` → float
  - [ ] `inner_product(vector, vector)` → float
  - [ ] `cosine_distance(vector, vector)` → float
  - [ ] `l2_normalize(vector)` → vector

**Index Management:**
- [ ] `CREATE INDEX USING hnsw_bq` syntax:
  ```sql
  CREATE INDEX ON embeddings USING hnsw_bq (embedding vector_l2_ops);
  ```
- [ ] Index parameters:
  - [ ] M (connections per node): default 48
  - [ ] ef_construction (build-time search): default 200
  - [ ] ef_search (query-time search): default 100
- [ ] Query planning:
  - [ ] Use HNSW index for ORDER BY vector <-> query
  - [ ] Sequential scan for small tables
  - [ ] Cost estimation based on index size

**MVCC Integration:**
- [ ] Concurrent vector inserts (snapshot isolation)
- [ ] Index updates within transactions
- [ ] Crash recovery (WAL replay for vectors)

### Week 6-7: Optimization & Advanced Features

**MN-RU Update Algorithm** (July 2024 paper):
- [ ] Implement improved HNSW updates:
  - [ ] Fix "unreachable points" during deletions
  - [ ] Better insertion performance
  - [ ] Maintain recall during updates
- [ ] Benchmark update performance:
  - [ ] Insert throughput (vectors/sec)
  - [ ] Delete throughput
  - [ ] Mixed workload (50% insert, 50% query)

**Parallel Index Building:**
- [ ] Multi-threaded HNSW construction:
  - [ ] Batch inserts (10K-100K at once)
  - [ ] Parallel graph building
  - [ ] Target: 85% reduction in build time (research finding)
- [ ] Bulk loading optimization:
  - [ ] COPY FROM for vector data
  - [ ] Batch quantization training
  - [ ] Target: 1M vectors in <60 seconds

**Hybrid Search:**
- [ ] Combine vector similarity + SQL filters:
  ```sql
  SELECT * FROM products
  WHERE category = 'electronics' AND price < 100
  ORDER BY embedding <-> '[...]'::vector
  LIMIT 10;
  ```
- [ ] Query optimization:
  - [ ] Filter pushdown (reduce vector search space)
  - [ ] ALEX index for SQL predicates
  - [ ] Combined cost estimation

### Week 8-9: Benchmarks & Validation

**vs pgvector (1M vectors, 1536D):**
- [ ] Setup: PostgreSQL 16 + pgvector vs OmenDB
- [ ] Metrics:
  - [ ] Memory: Target 24x reduction (OmenDB: ~7GB vs pgvector: 170GB)
  - [ ] QPS: Target 10x faster (OmenDB: 400+ vs pgvector: 40)
  - [ ] Latency: Target <5ms p95 (pgvector: ~25ms)
  - [ ] Recall: Both >95%
- [ ] Publish: Benchmark blog post + GitHub

**vs Pinecone (10M vectors):**
- [ ] Setup: Pinecone cloud vs OmenDB self-hosted
- [ ] Metrics:
  - [ ] Latency: Match Pinecone (<10ms p95)
  - [ ] Memory: 10x better (BQ efficiency)
  - [ ] Cost: 1/10th (self-hosted vs cloud pricing)
- [ ] Publish: "OmenDB vs Pinecone" comparison

**Large-Scale Validation:**
- [ ] 10M vectors stress test
- [ ] 100M vectors feasibility (estimate memory/performance)
- [ ] Concurrent queries (100 QPS sustained)
- [ ] Write-heavy workload (inserts + queries)

**Success Criteria:**
- ✅ 10x faster than pgvector
- ✅ 24x memory efficient
- ✅ Matches Pinecone performance
- ✅ >95% recall maintained

---

## Phase 2: Production Hardening (Weeks 11-16)

### Week 11-12: Documentation & Examples

**Installation:**
- [ ] Docker image (1-command deploy)
- [ ] Binary releases (Linux x86_64, macOS arm64)
- [ ] Cloud deployment guides (AWS, GCP, Fly.io)

**API Documentation:**
- [ ] Vector data types and operators
- [ ] Index creation and tuning
- [ ] Query syntax and examples
- [ ] Performance tuning guide

**Examples:**
- [ ] RAG application (LangChain + OmenDB)
- [ ] Semantic search (product catalog)
- [ ] Recommendation engine (user-item embeddings)
- [ ] Code search (semantic code retrieval)

### Week 13-14: Migration Tools

**pgvector → OmenDB Migration:**
- [ ] Schema migration script:
  - [ ] Detect vector columns in PostgreSQL
  - [ ] Generate CREATE TABLE for OmenDB
  - [ ] Convert HNSW indexes to hnsw_bq
- [ ] Data migration:
  - [ ] pg_dump → OmenDB import
  - [ ] Batch vector loading
  - [ ] Index building
- [ ] Validation:
  - [ ] Compare query results (pgvector vs OmenDB)
  - [ ] Ensure >99% query accuracy
- [ ] Migration guide (step-by-step docs)

**Example Migration:**
```bash
# 1. Export from pgvector
pg_dump -t embeddings mydb > embeddings.sql

# 2. Import to OmenDB
omendb import embeddings.sql

# 3. Build quantized index
omendb -c "CREATE INDEX ON embeddings USING hnsw_bq (vector);"

# 4. Validate
omendb -c "SELECT COUNT(*) FROM embeddings;"
```

### Week 15-16: Public Launch

**Pre-Launch:**
- [ ] GitHub repo cleanup (docs, examples, CI)
- [ ] Performance benchmark video/demo
- [ ] Landing page (omendb.com)
- [ ] Discord community setup

**Launch Content:**
- [ ] Blog post: "OmenDB: The pgvector Alternative That Scales"
  - [ ] Benchmark results (10x faster, 24x memory)
  - [ ] Why we built it (pgvector limitations)
  - [ ] Technical deep-dive (HNSW + BQ)
- [ ] HackerNews post (Show HN)
- [ ] Reddit posts (/r/MachineLearning, /r/PostgreSQL, /r/LangChain)
- [ ] Twitter/X threads

**Launch Targets:**
- [ ] 500+ GitHub stars (Week 1)
- [ ] 100+ HackerNews points
- [ ] 50+ Discord members
- [ ] 10+ customer calls scheduled

---

## Phase 3: Managed Cloud (Weeks 17-24)

### Week 17-20: Cloud Infrastructure

**Backend:**
- [ ] Multi-tenant architecture
- [ ] Database provisioning (Fly.io machines)
- [ ] Connection pooling (pgBouncer)
- [ ] Monitoring (Prometheus + Grafana)

**Frontend:**
- [ ] Sign-up flow (email + password)
- [ ] Dashboard (usage, databases, API keys)
- [ ] Billing (Stripe integration)
- [ ] Documentation portal

**Pricing:**
- [ ] Free: 100K vectors, 1 database, community support
- [ ] Starter ($29/mo): 10M vectors, 10 databases, email support
- [ ] Pro ($99/mo): 100M vectors, 50 databases, priority support
- [ ] Enterprise (custom): Unlimited, dedicated, SLA

### Week 21-24: Customer Acquisition

**Outbound:**
- [ ] 100 cold emails to pgvector users
- [ ] 20 customer calls (validate pain, pricing)
- [ ] 10 pilot customers (free/discounted)

**Content Marketing:**
- [ ] Weekly blog posts (vector DB tips, RAG tutorials)
- [ ] LangChain integration guide
- [ ] OpenAI embedding best practices
- [ ] Pinecone migration case studies

**Target Metrics:**
- [ ] 50-100 active users (free + paid)
- [ ] 10-20 paying customers
- [ ] $290-$1,980 MRR
- [ ] Product-market fit validation

---

## Deferred (Post-MVP)

**Advanced Quantization (Phase 4):**
- [ ] Product Quantization (32x compression)
- [ ] Scalar Quantization (4-bit, 8-bit)
- [ ] Extended-RaBitQ (SIGMOD 2025)

**Distributed (Phase 5):**
- [ ] Sharding for 100M+ vectors
- [ ] Query routing across nodes
- [ ] Replication for HA

**SQL Features (Low Priority):**
- [ ] Subqueries, window functions, CTEs
- [ ] Advanced JOINs (RIGHT, FULL OUTER)
- [ ] DISTINCT, UNION, INTERSECT

**Observability (Phase 6):**
- [ ] EXPLAIN QUERY PLAN
- [ ] Slow query logging
- [ ] Index quality metrics

---

## Recently Completed

✅ **Week 2 Day 1-2: HNSW Baseline** (Oct 22-23):
- HNSW integration (99.5% recall, 6.63ms p95)
- 14 tests passing
- Production-ready baseline

✅ **Week 2 Day 2: PCA-ALEX Experiment** (Oct 23):
- Custom PCA (99.58% variance)
- PCA-ALEX integration (12.4% recall - failed)
- Validated ALEX not suitable for high-D vectors

✅ **Week 2 Day 2: SOTA Research** (Oct 23):
- 1,300+ line research report
- 32+ citations (papers, blogs, benchmarks)
- Validated HNSW + BQ as optimal approach

✅ **Phase 3 Quick Wins** (Oct 22):
- Aggregations, HAVING, CROSS JOIN
- 557 total tests

✅ **Phase 2 Security** (Complete):
- Auth + SSL/TLS (57 tests)

✅ **Cache Layer** (Complete):
- LRU cache (2-3x speedup)

✅ **Phase 1 MVCC** (Complete):
- Snapshot isolation (85 tests)

---

## 🔥 CRITICAL PATH: Week 6 (Oct 24-30)

**Status**: ✅ Days 1-2 COMPLETE - 100K validated with 3626x improvement!

### Days 1-2: Persisted HNSW Index ⭐ COMPLETE ✅
1. [✅] Research hnsw_rs serialization (hnswio module, dump/reload)
   - Completed: 300+ line research doc
   - Solution: Box::leak for 'static lifetime, nb_layer = 16
2. [✅] Implement graph persistence:
   - [✅] Add from_file_dump() to HNSWIndex
   - [✅] Update save_to_disk() to use file_dump()
   - [✅] Update load_from_disk() with fast path
   - [✅] Fixed knn_search to check HNSW for data
3. [✅] Test 100K vectors:
   - [✅] 1K vectors: 0.002s load (75x faster than rebuild)
   - [✅] 100K vectors: 0.498s load (3626x faster than rebuild!)
   - All pass/fail criteria exceeded
4. [✅] Auto-rebuild fallback implemented (in load_from_disk)

**Test Results (1K vectors)**:
- Build: 0.17s
- Save: 0.002s (graph + data)
- Load: 0.002s (deserialize)
- Query accuracy: 100% (5/5 top results match)

**Actual Results (100K vectors)** - VALIDATED ✅:
- Build: 1806.43s (~30 minutes)
- Save: 0.699s (graph + data)
- Load: 0.498s (deserialize)
- **Improvement: 3626x faster than rebuild!**
- Query (before): 10.33ms avg (97 QPS)
- Query (after): 9.45ms avg (106 QPS) - 8.5% faster!
- Disk: 743.74 MB (127 MB graph + 616 MB data)

**Success Criteria**: ✅ ALL PASSED - Exceeded all targets!

### Days 3-4: 1M Scale Validation
5. [ ] Insert 1M vectors (1536D)
6. [ ] Measure performance:
   - [ ] Query latency (p50, p95, p99)
   - [ ] Memory usage (target <15GB with quantization)
   - [ ] Build time (target <10 minutes)
7. [ ] Document scaling characteristics
8. [ ] Identify any new bottlenecks

**Success Criteria**: 1M vectors <15ms p95 queries

### Days 5-7: MN-RU Updates (Optional if time)
9. [ ] Research MN-RU algorithm (ArXiv 2407.07871)
10. [ ] Implement multi-neighbor replaced updates
11. [ ] Test insert/delete performance
12. [ ] Benchmark mixed workload

**Success Criteria**: Production-ready write performance

---

## Strategic Decisions

**✅ HNSW + Binary Quantization** (Industry Standard)
- Proven: Used by Qdrant, Weaviate, Elasticsearch
- Fast: 10,000-40,000 QPS at 95% recall
- Memory Efficient: 96% reduction with BQ
- Real-time Updates: Better than DiskANN

**❌ ALEX for Vectors** (Experimental, Not Production-Ready)
- Week 1: 5% recall (1D projection)
- Week 2: 12.4% recall (PCA + 1D key)
- Verdict: Keep ALEX for SQL indexing only

**❌ DiskANN** (Immutability Issues)
- Requires batch updates
- NVMe SSD dependency
- Operational complexity
- Already tried and abandoned

**✅ Focus on HTAP Hybrid Search** (Unique Advantage)
- Vector similarity + SQL filters in one query
- Nobody else has this (Pinecone no SQL, pgvector doesn't scale)
- Leverage existing ALEX + MVCC work

---

**Status**: Week 2 complete, optimal plan validated, ready for BQ implementation
**Focus**: Ship HNSW + BQ in 8 weeks → Acquire customers → Iterate
**Timeline**: 6 months to production-ready, 12 months to $10K MRR
**Goal**: 50-100 users, $1-5K MRR by Month 6
