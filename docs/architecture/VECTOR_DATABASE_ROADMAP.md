# OmenDB Vector Database Roadmap

**Date**: October 22, 2025
**Target**: PostgreSQL-Compatible Vector Database That Scales
**Timeline**: 24 weeks (6 months) to production-ready vector database
**Status**: Strategic pivot approved, prototyping begins immediately

---

## Executive Summary

**Strategic Decision**: Pivot from "Fast Embedded PostgreSQL" to "PostgreSQL-Compatible Vector Database"

**Market Opportunity**:
- Vector DB market: $1.6B (2023) → $10.6B (2032), 23.54% CAGR
- Clear pain point: pgvector doesn't scale, Pinecone expensive ($70-8K+/month)
- Gap: No PostgreSQL-compatible vector DB that scales efficiently

**OmenDB's Unique Fit**:
- ✅ Multi-level ALEX: Perfect for high-dimensional vector indexing
- ✅ Memory efficiency: 28x vs PostgreSQL (critical for 100M+ vectors)
- ✅ PostgreSQL wire protocol: Drop-in pgvector replacement
- ✅ MVCC + HTAP: Transactions + analytics (unique vs pure vector DBs)
- ✅ Linear scaling: Validated to 100M+ keys

**Target Market**: AI applications needing vector search + PostgreSQL compatibility

**Positioning**: "OmenDB: The PostgreSQL-compatible vector database that actually scales. Drop-in replacement for pgvector. 10x faster at 10M+ vectors. Self-host or cloud. Open source."

---

## Phase 1: Prototype & Validation (Weeks 1-2) 🚨 URGENT

**Goal**: Validate ALEX works for high-dimensional vectors OR pivot to HNSW

### Week 1: ALEX Vector Prototype

**Critical Tasks**:
- [ ] Research pgvector implementation (GitHub: pgvector/pgvector)
  - Understand vector data type implementation
  - Study distance operator implementations
  - Analyze index structures (IVFFlat, HNSW)
- [ ] Design vector(N) data type (dimensions 128-1536)
  - Variable-length vector storage
  - PostgreSQL wire protocol serialization
  - Type validation and error handling
- [ ] Prototype ALEX for 1536-dim vectors (OpenAI embedding size)
  - Adapt ALEX for high-dimensional keys
  - Test memory usage at 100K, 1M vectors
  - Measure query latency (target: <10ms)
- [ ] **Go/No-Go Decision**: If ALEX doesn't work → pivot to HNSW algorithm

**Success Criteria**:
- Memory: <2GB for 1M 1536-dim vectors
- Latency: <10ms p95 for k-NN queries (k=10)
- Accuracy: >90% recall@10 vs brute force

**Deliverable**: Technical validation report + decision to continue or pivot

### Week 2: Customer Validation

**Market Research**:
- [ ] Identify 50 companies using pgvector
  - Search GitHub repos with pgvector dependencies
  - Find LangChain, LlamaIndex users
  - Identify AI startups (YC companies, AI directories)
- [ ] Cold outreach: "We're building pgvector that scales to 100M vectors"
  - Target: 20 emails sent
  - Goal: 10 responses, 5 customer calls
- [ ] Customer interview questions:
  - Max vector count? Performance issues at what scale?
  - Current pgvector pain points?
  - Willing to pay? How much?
  - What features matter most?
- [ ] **Success Metric**: 5+ say "I would switch from pgvector if 10x faster"

**Deliverable**: Customer validation report + market opportunity confirmation

---

## Phase 2: Vector Foundation (Weeks 3-10)

**Goal**: pgvector-compatible vector database (1M-10M vector scale)

### Week 3-4: Vector Data Type

**Implementation**:
- [ ] Implement `vector(N)` data type (variable dimensions)
  - Storage format: `[f32; N]` with dimension metadata
  - PostgreSQL wire protocol: Binary + text format
  - Type coercion and validation
- [ ] Distance operators:
  - `<->` (L2 distance / Euclidean)
  - `<#>` (negative dot product for max inner product)
  - `<=>` (cosine distance)
- [ ] Vector functions:
  - `l2_distance(vector, vector)` → float
  - `inner_product(vector, vector)` → float
  - `cosine_distance(vector, vector)` → float
  - `l2_normalize(vector)` → vector
- [ ] Unit tests: 50+ tests for vector operations
- [ ] PostgreSQL wire protocol: Serialize/deserialize vector type

**Success Criteria**:
- [ ] Can create table with vector columns
- [ ] Can insert/select vectors via psql
- [ ] Distance operators return correct results
- [ ] 50+ vector tests passing

**Deliverable**: pgvector-compatible vector data type

### Week 5-6: ALEX Index for Vectors

**Implementation**:
- [ ] Adapt ALEX for high-dimensional keys
  - Dimension-aware linear models
  - Distance-based partitioning
- [ ] Implement approximate nearest neighbor (ANN) search
  - k-NN query execution
  - Distance calculations optimized
- [ ] CREATE INDEX syntax:
  ```sql
  CREATE INDEX ON embeddings USING alex (embedding vector_l2_ops);
  ```
- [ ] Index build optimization
  - Batch training for ALEX models
  - Parallel index construction
- [ ] Query planning: Use ALEX index for vector similarity queries

**Success Criteria**:
- [ ] Can create vector index
- [ ] k-NN queries work correctly
- [ ] Index build completes in <60 seconds for 1M vectors
- [ ] Query latency <10ms for k=10

**Deliverable**: ALEX-based vector index

### Week 7-8: Benchmark vs pgvector (1M vectors)

**Benchmark Setup**:
- [ ] Environment: PostgreSQL 16 + pgvector vs OmenDB
- [ ] Dataset: 1M OpenAI embeddings (1536 dimensions)
- [ ] Queries:
  - Top-K nearest neighbors (K=10, 100, 1000)
  - Hybrid search (vector similarity + WHERE clauses)
  - Batch queries (1000 queries, measure p50/p95/p99)
- [ ] Metrics: Latency, throughput, memory usage, index build time

**Target Performance**:
- [ ] 10x faster queries than pgvector
- [ ] 5x less memory than pgvector
- [ ] Index build time competitive (<5 minutes for 1M vectors)

**Deliverable**: Benchmark report (GitHub, blog post) + performance validation

### Week 9-10: Integration & Testing

**Testing**:
- [ ] End-to-end tests: INSERT vectors, SELECT with distance ops
- [ ] MVCC tests: Concurrent vector inserts + queries
- [ ] Cache integration: LRU cache for hot vectors
- [ ] Crash recovery: WAL replay for vector data
- [ ] Total tests: 100+ vector-specific tests

**Documentation**:
- [ ] Vector data type reference
- [ ] Distance operator usage
- [ ] Index creation guide
- [ ] Example queries

**Deliverable**: Production-ready vector database (1M scale, 10x performance improvement)

---

## Phase 3: Scale & Performance (Weeks 11-16)

**Goal**: Production-ready at 10M-100M vector scale

### Week 11-12: Large-Scale Optimization

**Optimization Tasks**:
- [ ] Optimize ALEX for 10M+ vectors
  - Multi-level hierarchy tuning (3-4 levels)
  - Node splitting strategy (minimize rebalancing)
  - Memory pooling (reduce allocation overhead)
- [ ] Batch insert optimization:
  - Bulk vector loading (1M vectors in <60 seconds)
  - Parallel index building
  - Pre-sorting for sequential inserts
- [ ] Memory profiling:
  - Target: <2GB for 10M 1536-dim vectors
  - Compare: pgvector uses ~60GB for same dataset
  - 30x memory efficiency validation

**Success Criteria**:
- [ ] 10M vectors: <2GB memory
- [ ] Bulk insert: 1M vectors in <60 seconds
- [ ] Query latency: <50ms p95 for k=10

**Deliverable**: Large-scale optimization complete

### Week 13-14: Hybrid Search & Query Optimization

**Implementation**:
- [ ] Combine vector search + SQL filters:
  ```sql
  SELECT * FROM products
  WHERE category = 'electronics'
  ORDER BY embedding <-> '[...]'::vector
  LIMIT 10;
  ```
- [ ] Query planner: Decide ALEX vs sequential scan
  - Cost estimation for vector operations
  - Selectivity estimation
- [ ] Predicate pushdown (filter before vector search)
- [ ] Benchmark: Hybrid queries vs pure vector search

**Success Criteria**:
- [ ] Hybrid queries 5-10x faster than sequential scan + sort
- [ ] Query planner chooses correct strategy
- [ ] Predicate pushdown reduces vector comparisons

**Deliverable**: Hybrid search optimization

### Week 15-16: Benchmark vs Pinecone/Weaviate (10M vectors)

**Benchmark Setup**:
- [ ] Environment: Pinecone cloud, Weaviate self-hosted, OmenDB
- [ ] Dataset: 10M OpenAI embeddings (1536 dimensions)
- [ ] Queries:
  - Top-K nearest neighbors (K=10, 100, 1000)
  - Concurrent queries (100 queries/sec)
  - Hybrid search (vector + filters)
- [ ] Metrics: Latency (p50/p95/p99), throughput, cost

**Target Performance**:
- [ ] Latency: Match Pinecone (<50ms p95)
- [ ] Memory: 10x better than Pinecone
- [ ] Cost: 5-10x cheaper (due to memory efficiency)

**Deliverable**: "OmenDB vs Pinecone vs Weaviate" benchmark report

---

## Phase 4: Migration & Go-to-Market (Weeks 17-24)

**Goal**: 50-100 active users, $1-5K MRR

### Week 17-18: Migration Tooling

**Implementation**:
- [ ] pgvector → OmenDB migration script:
  - Schema migration (CREATE TABLE with vector columns)
  - Data migration (pg_dump → OmenDB import)
  - Index migration (CREATE INDEX USING alex)
  - Validation (compare query results)
- [ ] Migration guide (step-by-step documentation)
- [ ] Example: Migrate LangChain app from pgvector to OmenDB

**Deliverable**: Migration tooling + documentation

### Week 19-20: Documentation & Examples

**Installation**:
- [ ] Docker image (1-command deploy)
- [ ] Binary releases (Linux, macOS)
- [ ] Cloud deployment (AWS, GCP, Fly.io)

**API Documentation**:
- [ ] Vector data types
- [ ] Distance operators
- [ ] Index management
- [ ] Query syntax

**Examples**:
- [ ] RAG application (LangChain + OmenDB)
- [ ] Semantic search (product catalog search)
- [ ] Recommendation engine (user-item embeddings)
- [ ] Code search (semantic code retrieval)

**Deliverable**: Complete documentation + examples

### Week 21-22: Public Launch

**Launch Plan**:
- [ ] Make GitHub repo public (Apache 2.0 license)
- [ ] Write launch blog post:
  - "OmenDB: The pgvector Alternative That Scales"
  - Benchmark results (10x faster, 30x memory efficient)
  - Migration guide (5-minute drop-in replacement)
- [ ] Launch on:
  - Hacker News (Show HN: OmenDB)
  - Reddit (/r/MachineLearning, /r/PostgreSQL, /r/LangChain)
  - Twitter/X (tag @LangChainAI, @OpenAI, AI influencers)
- [ ] Target: 500+ GitHub stars, 100+ Hacker News points, 50+ Discord members

**Deliverable**: Public launch + initial traction

### Week 23-24: Managed Cloud (MVP)

**Implementation**:
- [ ] Deploy OmenDB cloud (Fly.io or AWS)
- [ ] Sign-up flow (email + password, no credit card for free tier)
- [ ] Pricing tiers:
  - Free: 1M vectors, 1 database, community support
  - Starter ($29/mo): 10M vectors, 100GB storage, email support
  - Pro ($99/mo): 100M vectors, 1TB storage, priority support
  - Enterprise (custom): Unlimited, dedicated infra, SLA
- [ ] Payment integration (Stripe)
- [ ] Dashboard (usage, billing, API keys)
- [ ] **Target**: First 10 paying customers ($290-990 MRR)

**Deliverable**: Managed cloud MVP + first paying customers

---

## Success Criteria (6 Months)

**Technical**:
- ✅ 10x faster than pgvector (1M-10M vectors)
- ✅ <2GB memory for 10M 1536-dim vectors (30x better than pgvector)
- ✅ PostgreSQL-compatible (drop-in replacement)
- ✅ 100+ vector tests passing
- ✅ Crash recovery working

**Market**:
- ✅ 50-100 active users
- ✅ $1-5K MRR (10-50 paying customers)
- ✅ 500+ GitHub stars
- ✅ 10+ customer testimonials
- ✅ Benchmark report published

**Decision Point (End of Week 2)**:
- ✅ If ALEX prototype works + 3+ customer validations → Proceed with Phase 2
- ❌ If ALEX doesn't work → Pivot to HNSW algorithm
- ❌ If no customer interest → Reconsider vector market entirely

---

## Deferred (Post-Vector MVP)

### SQL Features (Not Differentiating)
- Subqueries (WHERE EXISTS, scalar subqueries)
- Window functions (ROW_NUMBER, RANK)
- CTEs (WITH clauses)
- RIGHT/FULL OUTER JOIN
- DISTINCT, UNION, INTERSECT

**Rationale**: SQL completeness doesn't matter for vector database users. Focus on vector performance.

### Observability (Phase 4+)
- EXPLAIN QUERY PLAN command
- Query performance metrics
- Slow query logging
- Prometheus metrics endpoint

**Rationale**: Nice-to-have, not blocking for early adopters.

### Backup & Recovery (Phase 5+)
- pg_dump/pg_restore compatibility
- Point-in-time recovery (PITR)
- Incremental backups
- Backup verification tools

**Rationale**: Important for enterprise, but not for initial traction.

---

## Competitive Positioning

| Feature | pgvector | Pinecone | Weaviate | OmenDB |
|---------|----------|----------|----------|---------|
| PostgreSQL compatible | ✅ | ❌ | ❌ | ✅ |
| Scales to 100M+ vectors | ❌ | ✅ | ✅ | ✅ |
| Self-hosting | ✅ | ❌ | ✅ | ✅ |
| Memory efficient | ❌ | ? | ❌ | ✅ (28x) |
| HTAP (transactions + analytics) | ✅ | ❌ | ❌ | ✅ |
| Pricing | Free | $70-8K+/mo | Free/Paid | $29-499/mo |

**OmenDB's Advantages**:
1. **PostgreSQL compatibility** (pgvector users can drop-in migrate)
2. **Memory efficiency** (28x vs PostgreSQL = cheaper at scale)
3. **HTAP** (one DB for vectors + business logic)
4. **Self-hosting + managed** (unlike Pinecone cloud-only)
5. **Open source** (avoid vendor lock-in)

---

## Target Customers

**Tier 1**: AI-first startups ($29-299/month)
- RAG applications (chatbots, search, Q&A)
- Code search, document search, semantic search
- Pain: pgvector too slow at 10M embeddings, Pinecone costs $2K/month

**Tier 2**: E-commerce + SaaS ($299-2K/month)
- Product recommendations, semantic search
- User analytics, customer support
- Pain: Need PostgreSQL for transactions + vector search, running two DBs

**Tier 3**: Enterprise AI ($2K-20K/month)
- Healthcare (patient similarity, drug discovery)
- Finance (fraud detection, trading signals)
- Legal (case law search, document similarity)
- Pain: Can't use cloud Pinecone (compliance), pgvector doesn't scale

**Tier 4**: AI Platform Companies ($20K+/month)
- LangChain, LlamaIndex (need vector backend)
- AI agent platforms, RAG-as-a-service
- Pain: Building on Pinecone = vendor lock-in

---

## Revenue Projections

**Year 1**: $100K-500K ARR (50-200 customers)
- Month 1-6: Build + launch ($0-5K MRR)
- Month 7-12: Early adopters + managed cloud ($8-40K MRR)

**Year 2**: $1M-3M ARR (enterprise adoption)
- Enterprise customers: 10-50 customers @ $2K-20K/mo
- SMB customers: 500-1,000 customers @ $29-299/mo

**Year 3**: $5M-15M ARR (scale, competitive with Pinecone)
- Market share: 5-10% of pgvector users migrate
- Enterprise: 50-100 customers @ $2K-20K/mo
- SMB: 1,000-3,000 customers @ $29-299/mo

---

## Risk Mitigation

**Technical Risks**:
- ⚠️ ALEX for high-dimensional vectors: Unproven (needs prototype Week 1-2)
  - Mitigation: Week 1-2 validation, pivot to HNSW if needed
- ⚠️ Performance at 100M vectors: Need to validate vs Pinecone benchmarks
  - Mitigation: Incremental benchmarking at 1M, 10M, 100M scales
- ⚠️ Memory overhead: Target <2GB for 10M vectors
  - Mitigation: Memory profiling + optimization in Week 11-12

**Market Risks**:
- ⚠️ Vector DB market crowding: Pinecone, Weaviate, Qdrant well-funded
  - Mitigation: Focus on PostgreSQL compatibility (unique advantage)
- ⚠️ pgvector improvements: If it gets 10x faster, reduces urgency
  - Mitigation: Move fast, launch in 6 months before pgvector catches up
- ⚠️ PostgreSQL adoption for AI: Need to validate demand
  - Mitigation: Week 2 customer validation, 50 interviews

---

## Immediate Next Steps (This Week)

**Priority 1: ALEX Vector Prototype** (2-3 days)
1. Research pgvector source code (GitHub: pgvector/pgvector)
2. Design vector(N) data type in Rust
3. Prototype ALEX for 1536-dim vectors (100K-1M vectors)
4. Measure: Memory usage, query latency, index build time
5. **Decision**: Continue if <2GB for 1M vectors, <10ms query latency

**Priority 2: Market Validation** (2-3 days)
1. List 50 companies using pgvector (search GitHub, LangChain repos)
2. Draft cold email: "Building pgvector that scales to 100M vectors"
3. Send 20 emails (target 5 responses)
4. Schedule 3-5 customer calls
5. **Validate**: Pain point is real, willingness to pay $29-99/month

**Decision Point (End of Week 1)**:
- ✅ If ALEX prototype works + 3+ customer validations → Proceed with Phase 2
- ❌ If ALEX doesn't work → Pivot to HNSW algorithm
- ❌ If no customer interest → Reconsider vector market entirely

---

**Status**: Roadmap approved, execution begins immediately
**Focus**: Validate ALEX for vectors + validate market demand
**Timeline**: 24 weeks (6 months) to production-ready vector database
**Goal**: $100K-500K ARR, 50-200 paying customers (Year 1)
**Positioning**: "The PostgreSQL-compatible vector database that actually scales"

---

*Created: October 22, 2025*
*Strategic pivot from embedded database to vector database market*
*Based on $10.6B market opportunity and perfect tech-market fit*
