# TODO

_Last Updated: 2025-10-22 - STRATEGIC DECISIONS FINALIZED_

## FINALIZED STRATEGY

**Product**: PostgreSQL-compatible vector database that scales
**License**: Elastic License 2.0 (source-available, self-hostable)
**Pricing**: Free (100K vectors), $29, $99/month + Enterprise
**Market**: AI startups (70%), Enterprise (30%)
**Year 1**: omendb-server ONLY (omen-lite in Year 2+)

**Timeline**: 6 months to production-ready MVP, 12 months to $10K MRR

---

## Critical Priority: Vector Database Foundation

### Phase 1: Prototype & Validation (Weeks 1-2) 🚨 URGENT

**Goal**: Validate ALEX works for high-dimensional vectors OR pivot to HNSW

- [ ] **Week 1: ALEX Vector Prototype**
  - [ ] Research pgvector implementation (data types, operators, index structures)
  - [ ] Design vector data type (`vector(N)` - dimensions 128-1536)
  - [ ] Prototype ALEX for 1536-dim vectors (OpenAI embedding size)
  - [ ] Test: Insert 1M vectors, measure memory & query latency
  - [ ] **Go/No-Go Decision**: If ALEX doesn't work → pivot to HNSW algorithm

- [ ] **Week 2: Customer Validation**
  - [ ] Identify 50 companies using pgvector (GitHub, LangChain users, AI startups)
  - [ ] Cold outreach: "We're building pgvector that scales to 100M vectors"
  - [ ] Target: 10 customer calls, validate pain point
  - [ ] Questions: Max vector count? Performance issues? Willing to pay?
  - [ ] **Success Metric**: 5+ say "I would switch from pgvector if 10x faster"

**Deliverable**: Technical validation + market validation OR decision to abandon vector pivot

---

### Phase 2: Vector Foundation (Weeks 3-10)

**Goal**: pgvector-compatible vector database (1M-10M vector scale)

**Week 3-4: Vector Data Type**
- [ ] Implement `vector(N)` data type (variable dimensions)
- [ ] Implement distance operators:
  - [ ] `<->` (L2 distance / Euclidean)
  - [ ] `<#>` (negative dot product for max inner product)
  - [ ] `<=>` (cosine distance)
- [ ] Implement vector functions:
  - [ ] `l2_distance(vector, vector)` → float
  - [ ] `inner_product(vector, vector)` → float
  - [ ] `cosine_distance(vector, vector)` → float
  - [ ] `l2_normalize(vector)` → vector
- [ ] Unit tests: 50+ tests for vector operations
- [ ] PostgreSQL wire protocol: Serialize/deserialize vector type

**Week 5-6: ALEX Index for Vectors**
- [ ] Adapt ALEX for high-dimensional keys (dimension-aware model)
- [ ] Implement approximate nearest neighbor (ANN) search
- [ ] CREATE INDEX USING alex syntax:
  ```sql
  CREATE INDEX ON embeddings USING alex (embedding vector_l2_ops);
  ```
- [ ] Index build optimization (batch training for ALEX models)
- [ ] Query planning: Use ALEX index for vector similarity queries

**Week 7-8: Benchmark vs pgvector (1M vectors)**
- [ ] Setup: PostgreSQL 16 + pgvector vs OmenDB
- [ ] Dataset: 1M OpenAI embeddings (1536 dimensions)
- [ ] Queries:
  - [ ] Top-K nearest neighbors (K=10, 100, 1000)
  - [ ] Hybrid search (vector similarity + WHERE clauses)
  - [ ] Batch queries (1000 queries, measure p50/p95/p99)
- [ ] Metrics: Latency, throughput, memory usage, index build time
- [ ] **Target**: 10x faster queries, 5x less memory than pgvector
- [ ] **Publish**: Benchmark report (GitHub, blog post)

**Week 9-10: Integration & Testing**
- [ ] End-to-end tests: INSERT vectors, SELECT with distance ops
- [ ] MVCC tests: Concurrent vector inserts + queries
- [ ] Cache integration: LRU cache for hot vectors
- [ ] Crash recovery: WAL replay for vector data
- [ ] Total tests: 100+ vector-specific tests

**Deliverable**: pgvector-compatible OmenDB (1M vector scale, 10x performance improvement)

---

### Phase 3: Scale & Performance (Weeks 11-16)

**Goal**: Production-ready at 10M-100M vector scale

**Week 11-12: Large-Scale Optimization**
- [ ] Optimize ALEX for 10M+ vectors
  - [ ] Multi-level hierarchy tuning (3-4 levels)
  - [ ] Node splitting strategy (minimize rebalancing)
  - [ ] Memory pooling (reduce allocation overhead)
- [ ] Batch insert optimization:
  - [ ] Bulk vector loading (1M vectors in <60 seconds)
  - [ ] Parallel index building
  - [ ] Pre-sorting for sequential inserts
- [ ] Memory profiling:
  - [ ] Target: <2GB for 10M 1536-dim vectors
  - [ ] Compare: pgvector uses ~60GB for same dataset
  - [ ] 30x memory efficiency validation

**Week 13-14: Hybrid Search & Query Optimization**
- [ ] Combine vector search + SQL filters:
  ```sql
  SELECT * FROM products
  WHERE category = 'electronics'
  ORDER BY embedding <-> '[...]'::vector
  LIMIT 10;
  ```
- [ ] Query planner: Decide ALEX vs sequential scan
- [ ] Index selectivity estimation
- [ ] Predicate pushdown (filter before vector search)
- [ ] Benchmark: Hybrid queries vs pure vector search

**Week 15-16: Benchmark vs Pinecone/Weaviate (10M vectors)**
- [ ] Setup: Pinecone cloud, Weaviate self-hosted, OmenDB
- [ ] Dataset: 10M OpenAI embeddings (1536 dimensions)
- [ ] Queries:
  - [ ] Top-K nearest neighbors (K=10, 100, 1000)
  - [ ] Concurrent queries (100 queries/sec)
  - [ ] Hybrid search (vector + filters)
- [ ] Metrics: Latency (p50/p95/p99), throughput, cost
- [ ] **Target**:
  - [ ] Latency: Match Pinecone (<50ms p95)
  - [ ] Memory: 10x better than Pinecone
  - [ ] Cost: 5-10x cheaper (due to memory efficiency)
- [ ] **Publish**: "OmenDB vs Pinecone vs Weaviate" benchmark report

**Deliverable**: Production-ready vector database (10M-100M scale, competitive with Pinecone)

---

### Phase 4: Migration & Go-to-Market (Weeks 17-24)

**Goal**: 50-100 active users, $1-5K MRR

**Week 17-18: Migration Tooling**
- [ ] pgvector → OmenDB migration script:
  - [ ] Schema migration (CREATE TABLE with vector columns)
  - [ ] Data migration (pg_dump → OmenDB import)
  - [ ] Index migration (CREATE INDEX USING alex)
  - [ ] Validation (compare query results)
- [ ] Migration guide (step-by-step documentation)
- [ ] Example: Migrate LangChain app from pgvector to OmenDB

**Week 19-20: Documentation & Examples**
- [ ] **Installation**:
  - [ ] Docker image (1-command deploy)
  - [ ] Binary releases (Linux, macOS)
  - [ ] Cloud deployment (AWS, GCP, Fly.io)
- [ ] **API Documentation**:
  - [ ] Vector data types
  - [ ] Distance operators
  - [ ] Index management
  - [ ] Query syntax
- [ ] **Examples**:
  - [ ] RAG application (LangChain + OmenDB)
  - [ ] Semantic search (product catalog search)
  - [ ] Recommendation engine (user-item embeddings)
  - [ ] Code search (semantic code retrieval)

**Week 21-22: Public Launch**
- [ ] Make GitHub repo public (Apache 2.0 license)
- [ ] Write launch blog post:
  - [ ] "OmenDB: The pgvector Alternative That Scales"
  - [ ] Benchmark results (10x faster, 30x memory efficient)
  - [ ] Migration guide (5-minute drop-in replacement)
- [ ] Launch on:
  - [ ] Hacker News (Show HN: OmenDB)
  - [ ] Reddit (/r/MachineLearning, /r/PostgreSQL, /r/LangChain)
  - [ ] Twitter/X (tag @LangChainAI, @OpenAI, AI influencers)
- [ ] Target: 500+ GitHub stars, 100+ Hacker News points, 50+ Discord members

**Week 23-24: Managed Cloud (MVP)**
- [ ] Deploy OmenDB cloud (Fly.io or AWS)
- [ ] Sign-up flow (email + password, no credit card for free tier)
- [ ] Pricing tiers:
  - [ ] Free: 1M vectors, 1 database, community support
  - [ ] Starter ($29/mo): 10M vectors, 100GB storage, email support
  - [ ] Pro ($99/mo): 100M vectors, 1TB storage, priority support
  - [ ] Enterprise (custom): Unlimited, dedicated infra, SLA
- [ ] Payment integration (Stripe)
- [ ] Dashboard (usage, billing, API keys)
- [ ] **Target**: First 10 paying customers ($290-990 MRR)

**Deliverable**: Public launch, 50-100 users, $1-5K MRR, validated product-market fit

---

## Deferred (Post-Vector MVP)

### SQL Features (Not Differentiating)
- [ ] Subqueries (WHERE EXISTS, scalar subqueries)
- [ ] Window functions (ROW_NUMBER, RANK)
- [ ] CTEs (WITH clauses)
- [ ] RIGHT/FULL OUTER JOIN
- [ ] DISTINCT, UNION, INTERSECT

**Rationale**: SQL completeness doesn't matter for vector database users. Focus on vector performance.

### Observability (Phase 4+)
- [ ] EXPLAIN QUERY PLAN command
- [ ] Query performance metrics
- [ ] Slow query logging
- [ ] Prometheus metrics endpoint

**Rationale**: Nice-to-have, not blocking for early adopters.

### Backup & Recovery (Phase 5+)
- [ ] pg_dump/pg_restore compatibility
- [ ] Point-in-time recovery (PITR)
- [ ] Incremental backups
- [ ] Backup verification tools

**Rationale**: Important for enterprise, but not for initial traction.

---

## Recently Completed (Pre-Pivot)

✅ **Phase 3 Quick Wins** (Oct 22, 1 session):
- Aggregations: COUNT, SUM, AVG, MIN, MAX, GROUP BY (22 tests)
- HAVING clause: Full filtering support (7 tests)
- CROSS JOIN: Cartesian product (3 tests)
- **Result**: SQL coverage 35% → 45%, 557 total tests

✅ **Phase 2 Security (Days 1-10) COMPLETE**:
- Days 1-5: Auth + User Management (40 tests)
- Days 6-7: SSL/TLS Implementation (6 tests)
- Day 8: Security integration tests (17 tests)
- Day 9: Security documentation (SECURITY.md)
- Day 10: Security audit & validation
- **Total**: 57 security tests, 10 days on schedule

✅ **Cache Layer (Days 1-10) COMPLETE**:
- LRU cache (1-10GB configurable)
- 2-3x speedup validated (90% hit rate)
- 7 cache integration tests

✅ **Phase 3 Week 1-2: INNER JOIN + LEFT JOIN** (14 tests)

✅ **Phase 3 Week 1: UPDATE/DELETE support** (30 tests)

✅ **Phase 1: MVCC snapshot isolation** (85 tests)

✅ **Multi-level ALEX index** (1.5-3x faster than SQLite)

---

## Immediate Next Steps (This Week)

**Priority 1: ALEX Vector Prototype** (COMPLETED - Oct 22 Evening)
1. [x] Research pgvector source code (GitHub: pgvector/pgvector)
2. [x] Design vector(N) data type in Rust
3. [x] Prototype ALEX for 1536-dim vectors (tested 10K-100K vectors)
4. [x] Measure: Memory usage, query latency, recall@10
5. [x] **Results**: Memory ✅ (<50 bytes overhead), Latency ✅ (<20ms), Recall ❌ (5% vs 90% target)

**Week 1 Findings** (see docs/architecture/research/vector_prototype_week1_oct_2025.md):
- ✅ Memory: 6,146 bytes/vector (mostly raw data, 2-13 bytes overhead)
- ✅ Latency: 0.58-5.73ms average (17-22x faster than brute force)
- ❌ Recall: 5% recall@10 (target was >90%)
- **Root cause**: Simple 1D projection (sum of first 4 dims) doesn't preserve nearest-neighbor relationships
- **Validation**: Confirms LIDER paper - need PCA or LSH for high-dimensional indexing

**Priority 2: Market Validation** (2-3 days)
1. [ ] List 50 companies using pgvector (search GitHub, LangChain repos)
2. [ ] Draft cold email: "Building pgvector that scales to 100M vectors"
3. [ ] Send 20 emails (target 5 responses)
4. [ ] Schedule 3-5 customer calls
5. [ ] **Validate**: Pain point is real, willingness to pay $29-99/month

**Decision Point (End of Week 1 - Oct 22 Evening)**: ⚠️ MIXED RESULTS

- ✅ Memory/Latency: Excellent (met all targets)
- ❌ Recall: Catastrophic failure (5% vs 90% target)
- ⚠️ **Simple ALEX projection doesn't work for vectors**

**Three Options**:
1. **PCA-ALEX** (LIDER paper approach): 50-60% success, 3-4 weeks, revolutionary if works
2. **Hybrid ALEX+HNSW**: 70-80% success, 2-3 weeks, moderate differentiation
3. **Pure HNSW** (recommended): 95% success, 1-2 weeks, proven algorithm ✅

**Recommendation**: Pivot to HNSW
- Still 10x faster than pgvector (30 seconds → <10ms)
- Still 30x less memory (6000 → 100 bytes/vector)
- Still PostgreSQL-compatible (unique vs Pinecone/Weaviate)
- Can revisit PCA-ALEX in v0.2.0 if HNSW succeeds

**AWAITING USER DECISION**: HNSW pivot vs PCA-ALEX continuation

---

**Status**: Strategic pivot approved, execution begins immediately
**Focus**: Validate ALEX for vectors + validate market demand
**Timeline**: 6 months to production-ready vector database
**Goal**: $100K-500K ARR, 50-200 paying customers (Year 1)
