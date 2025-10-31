# Vector Index Algorithm Research

**Status**: ✅ **RESEARCH COMPLETE** (Technical + Competitive)
**Created**: October 22, 2025
**Completed**: October 31, 2025 (Competitive Intelligence Added)
**Goal**: Determine optimal vector indexing algorithm + validate market position
**Decision**: HNSW (hnsw_rs implementation) → Custom HNSW (Week 8) → Vector DB plan validated (Oct 31)

---

## Latest Research (October 31, 2025) ⭐

### Competitive Intelligence & Strategic Validation

**Document**: `../omen-org/strategy/COMPETITIVE_INTELLIGENCE_2025.md` (24K words)

**Scope**: Comprehensive analysis of all major vector database competitors and market positioning

**Competitors Analyzed**:
- **LanceDB** (YC-backed, $30M Series A): Multimodal, embedded, BUT NOT PostgreSQL compatible, NO transactions
- **Qdrant** (626 QPS leader): Embedded mode is "testing only" (not production-ready)
- **Pinecone** ($130M funding): Cloud-only, $500-2K/mo (validates willingness to pay)
- **pgvector** (PostgreSQL extension): Fails at 10M+ vectors (97x slower builds, 2.2x slower queries)
- **Deep Lake** (YC S18): Multimodal, ML training focus (different market)

**Critical Finding**:
> **NO competitor offers ALL of: PostgreSQL compatibility + embedded production + transactions + full-text search**

**Harvey AI Validation**:
> Uses BOTH LanceDB (performance) AND pgvector (PostgreSQL compatibility).
> Translation: Market needs ONE database that combines both. **That's us.**

**Market Trends Identified**:
1. **Hybrid Search Trending** (2024-2025):
   - Milvus 2.5 (Dec 2024): Hybrid search, 30x faster
   - Azure Cosmos DB (Jan 2025): Hybrid search added
   - pg_textsearch (Oct 2025): BM25 for PostgreSQL (2 days ago!)

2. **Edge/IoT Growth**: Edge AI market $14.5B → 20-30% CAGR (embedded is strength)

3. **HTAP Declared Dead**: "HTAP is Dead" (industry consensus June 2025) - don't pivot

**Strategic Decisions Validated**:
- ✅ Keep Vector DB Plan ($10.6B market, YC precedent, unique positioning)
- ❌ Reject HTAP Pivot (industry failed, unvalidated market, weak differentiation)
- ✅ Full-Text Search MANDATORY (trending industry-wide, competitive gap)
- ✅ PostgreSQL Compatibility is MOAT (73% of AI initiatives need this)

**YC Application**: Strong case (LanceDB/Deep Lake precedent, Harvey AI validates gap)

**See**: `../omen-org/funding/YC_W26_APPLICATION_PREP.md` for complete YC application prep

---

## Research Completed

### ⭐ Custom HNSW SOTA Research (Oct 30, 2025 - Week 8 Day 3)
**Document**: `ai/research/CUSTOM_HNSW_SOTA_RESEARCH_2025.md` (12,500 words, comprehensive)

**Scope**:
- Analyzed 4 competitors: Qdrant (Rust, Delta Encoding, GPU), Milvus (C++, AVX512), LanceDB (Rust), Weaviate (Go)
- SOTA algorithms: Extended RaBitQ (SIGMOD 2025, 3-9 bit quantization), Delta Encoding (30% memory reduction)
- Implementation techniques: Cache-line alignment, prefetching, arena allocators, graph reordering (BFS/DFS)
- Performance projections: 581 QPS → 1000+ QPS (10-week roadmap)

**Key Findings**:
- **ALL serious competitors use custom HNSW** (not libraries like hnsw_rs)
- **40% of query time in memory fetches** (NeurIPS 2022 paper)
- **Qdrant innovations**: Delta Encoding (30% memory), GPU indexing (10x build speed)
- **Milvus**: AVX512 20-30% faster than AVX2, query batching
- **Cache optimization techniques**: Software prefetching (_mm_prefetch), 64-byte alignment, BFS/DFS layout (1.15-1.18x speedup)
- **Allocation optimization**: Arena allocators (typed-arena), thread-local buffers

**10-Week Roadmap**:
- Weeks 9-10: Custom HNSW foundation (500-600 QPS baseline)
- Weeks 11-12: Cache optimization (650-700 QPS, beat Qdrant)
- Week 13: Allocation optimization (750-800 QPS)
- Week 14: AVX512 support (850-900 QPS, +46% cumulative)
- Weeks 15-17: Extended RaBitQ + Delta Encoding (900-950 QPS)
- Weeks 18-19: HNSW-IF billion-scale (1000+ QPS, +72% cumulative)

**Risk Assessment**: Medium (custom implementation complexity, but research-backed)

**Strategic Justification**: Week 8 profiling showed 76% allocations + 23.41% cache misses in hnsw_rs library (cannot optimize without custom implementation)

### Strategic Competitive Analysis (Oct 30, 2025)
**Documents**:
- `ai/research/STRATEGIC_COMPETITIVE_POSITIONING.md` (6400+ words, comprehensive)
- `ai/research/COMPETITIVE_ANALYSIS_VECTOR_DBS.md` (market landscape, testing strategy)
- `ai/research/OPTIMIZATION_STRATEGY.md` (engine-first optimization roadmap)
- `ai/research/CUSTOM_HNSW_DECISION.md` (custom vs library analysis, 10-15 week plan)

**Scope**:
- Analyzed 8 competitors (Qdrant, Milvus, Weaviate, LanceDB, ChromaDB, Pinecone, pgvector, pgvecto.rs)
- What each competitor uses (ALL use custom implementations, not libraries)
- PostgreSQL compatibility value analysis (ecosystem vs 5-10% overhead)
- Can we reach Qdrant performance? (YES, within 3-6 months)
- Can we reach billion scale? (YES, via HNSW-IF)
- SOTA features analysis (Extended RaBitQ, HNSW-IF, MN-RU)
- Custom HNSW effort estimates (10-15 weeks for complete implementation)

**Key Findings**:
- **CRITICAL**: SIMD available but NOT ENABLED (2-4x free win in 5 minutes)
- **All serious competitors use custom**: Qdrant (custom Rust + GPU), Milvus (custom C++ Knowhere + SIMD), Weaviate (custom Go), LanceDB (custom Rust)
- **hnsw_rs limitations**: No delete/update, no HNSW-IF, limited RaBitQ integration, performance ceiling at ~400-500 QPS
- **Performance projections**: Current 162 QPS → Week 1: 400-500 QPS (SIMD) → Week 10: 1000 QPS (custom + SOTA)
- **PostgreSQL compatibility**: 5-10% overhead, but massive ecosystem value (only embedded vector DB with pgvector compatibility)

**Decision**: Build custom HNSW for market leadership and SOTA features

### PCA-ALEX Moonshot Attempt (Oct 22, 2025 - 6.5 hours)
**Documents**:
- `docs/architecture/research/pca_alex_approach_oct_2025.md` (250 lines)
- `src/pca.rs` (323 lines, 99% complete, 7 tests)

**Result**: BLOCKED on ndarray-linalg backend configuration
**Decision**: Pivot to HNSW (proven approach with 95%+ success)

### HNSW Research & Planning (4 hours)
**Documents**:
- `docs/architecture/research/hnsw_implementation_oct_2025.md` (comprehensive research, 600+ lines)
- `docs/architecture/research/hnsw_implementation_plan_oct_2025.md` (tactical plan, 500+ lines)

**Key Findings**:
- **Algorithm**: HNSW (Malkov & Yashunin 2018) - industry standard
- **Production validation**: Qdrant, pgvecto.rs (20x faster than pgvector), Pinecone, Weaviate
- **Implementation**: hnsw_rs crate (SIMD, full parameter control, persistence)
- **Parameters**: M=48-64, ef_construction=200-400, ef_search=100-500
- **Expected results**: >95% recall, <10ms p95 latency, ~500 bytes/vector

**Decision**: Implement HNSW with hnsw_rs (7-day timeline, 95%+ success probability)

### State-of-the-Art Vector Search Survey (6 hours) - Oct 22, 2025
**Document**:
- `docs/architecture/research/sota_vector_search_algorithms_2024_2025.md` (comprehensive survey, 1300+ lines)

**Research scope**:
- DiskANN deep dive (production problems, why abandon for HNSW+)
- HNSW improvements 2024-2025 (MN-RU, BBQ, RaBitQ, dual-branch)
- Alternative algorithms (ScaNN SOAR, SPANN, NSG, CAGRA, learned indexes)
- Production vector DB analysis (Pinecone, Weaviate, Qdrant, Milvus, pgvector)
- 2024-2025 research papers (VLDB, SIGMOD, NeurIPS Big-ANN)
- Quantization techniques (Binary, Product, RaBitQ, BBQ)
- Benchmark results (ann-benchmarks.com, VIBE, Big-ANN 2023)
- Memory footprint analysis (GB per 1M vectors)
- Implementation complexity and timelines

**Key findings**:
- **HNSW + Binary Quantization** is the production standard (Pinecone, Weaviate, Qdrant)
- **DiskANN fails**: Immutability, I/O inefficiency, complex batching, NVMe requirements
- **RaBitQ (SIGMOD 2024)**: Theoretical error bounds, 3x faster than PQ, 95% memory reduction
- **BBQ (Elasticsearch 2024)**: 20-30x faster quantization, 2-5x faster queries vs PQ
- **ScaNN SOAR (Google 2024)**: Best indexing/query tradeoff, smallest memory footprint
- **Performance**: 10,000-44,000 QPS at 90-95% recall (VIBE, ann-benchmarks)
- **Memory**: HNSW+BQ uses ~15GB for 10M 1536D vectors (vs 170GB float32)
- **pgvector comparison**: HNSW 15x faster than IVFFlat (40.5 vs 2.6 QPS)

**Recommendation for OmenDB**:
✅ **Ship HNSW + Binary Quantization in 2 months**
- Proven: 100M+ vectors, 95%+ recall, <10ms latency
- Memory efficient: 95% reduction vs float32
- Fast implementation: Use hnswlib or hnsw_rs
- Clear differentiation vs pgvector: Quantization support (pgvector doesn't have)

⚠️ **Defer ALEX for vectors to Phase 2**
- Learned indexes (LIDER/LISA) haven't proven superior to HNSW
- High risk, unproven for 1536D vectors
- Better to ship proven tech, acquire customers, then innovate

**Sources**: 32+ citations (ArXiv, SIGMOD/VLDB 2024, industry blogs, benchmarks)

**Rationale**:
1. Time pressure: Need go/no-go by Oct 29 (Week 2)
2. Risk management: HNSW is 95%+ proven, PCA-ALEX was 40-50% moonshot
3. Value preserved: HNSW still delivers 10x faster than pgvector
4. Can retry PCA-ALEX: Documented for v0.2.0 optimization if HNSW succeeds

**See**: `ai/DECISIONS.md` (Oct 22 entries) for full rationale

---

## Original Research Objectives (Historical)

**Primary Question**: What vector indexing algorithm should we use for omendb-server and omen-lite?

**Secondary Questions**:
1. What's SOTA (state-of-the-art) in 2025?
2. Why is pgvector slow despite using HNSW?
3. Can learned indexes work for high-dimensional vectors?
4. What's our competitive differentiation?

---

## Research Documents (TO CREATE)

### 1. vector_index_algorithms_2025.md

**Comprehensive survey of SOTA algorithms**

**Cover**:
- HNSW (Hierarchical Navigable Small World)
- DiskANN (Microsoft Research)
- ScaNN (Google)
- Faiss (Facebook/Meta)
- Learned indexes (LIDER paper, etc.)
- Emerging algorithms (2024-2025 papers)

**For each algorithm**:
- Description and how it works
- Performance characteristics (recall@K, latency, memory, index build time)
- Production usage (which companies use it?)
- Pros and cons
- Rust library availability
- Implementation complexity (1-2 weeks? 3-4 weeks? Build from scratch?)

**Benchmark data**:
- Find published benchmarks comparing algorithms
- Note dataset size, dimensionality, hardware
- Be skeptical of claims without data

**Output**: Comprehensive comparison table

---

### 2. pgvector_performance_analysis.md

**Why is pgvector slow?**

**Key fact to explore**: pgvector uses HNSW (since v0.5.0 in 2023)

**Questions**:
- If pgvector has HNSW, why 13-hour index builds? (from GitHub issues)
- Is it PostgreSQL overhead? (WAL, TOAST, VACUUM, row-based storage)
- Is it HNSW implementation quality?
- Is it configuration? (HNSW parameters: M, ef_construction, ef_search)

**Research**:
- Read pgvector source code (GitHub: pgvector/pgvector)
- Find standalone HNSW benchmarks (hnswlib, instant-distance, etc.)
- Compare: standalone HNSW vs pgvector HNSW
- Isolate: PostgreSQL overhead vs algorithm

**Thesis to validate**: "RocksDB + HNSW will be 10x faster than PostgreSQL + HNSW"

**Output**:
- Root cause analysis (is it PostgreSQL? algorithm? both?)
- Data to support "10x faster" claim
- Confidence level in our approach

---

### 3. vector_index_decision.md

**Final algorithm choice with full justification**

**Cover**:
- Chosen algorithm (HNSW? DiskANN? PCA-ALEX? Something else?)
- Full rationale (why this over alternatives?)
- Risk assessment:
  - Technical risk (implementation complexity, unknowns)
  - Market risk (differentiation, competitive advantage)
  - Timeline risk (can we ship in time?)
- Implementation plan:
  - Library choice (specific Rust crate) OR build from scratch?
  - Timeline (realistic: 1 week? 2 weeks? 3-4 weeks?)
  - Milestones (what to deliver by when?)
  - Tests needed (recall@K, latency, memory)
- Fallback plan:
  - If chosen algorithm fails after 1-2 weeks, what next?
  - Secondary choice ready to implement
- Confidence level: **MUST be >80% to proceed**

**Output**:
- Clear decision with justification
- Implementation roadmap
- Risk mitigation plan

---

## Research Process

**Step 1: Survey SOTA (4-6 hours)**
- Read latest papers (arXiv, Google Scholar)
- Read production system docs (Pinecone, Weaviate, Qdrant, Milvus)
- Find benchmark comparisons
- List all viable algorithms

**Step 2: Deep dive on top 3-4 algorithms (4-6 hours)**
- How do they work technically?
- Performance data (recall, latency, memory)
- Implementation complexity
- Rust library availability
- Production track record

**Step 3: Analyze pgvector (2-4 hours)**
- Read pgvector source code
- Find HNSW implementation details
- Compare standalone HNSW benchmarks
- Understand PostgreSQL overhead

**Step 4: Make decision (2-3 hours)**
- Weigh tradeoffs
- Assess risk
- Choose algorithm
- Write implementation plan
- Identify fallback

**Total time**: 1-2 days (don't rush this!)

---

## Research Guidelines

**Be thorough**:
- Don't pick an algorithm based on a single blog post
- Find multiple sources, verify claims
- Look for actual benchmark data, not marketing

**Be skeptical**:
- "Revolutionary new algorithm" = probably research-stage
- "10x faster" = prove it with data
- Academic papers = may not work in production

**Be practical**:
- Can we implement in 1-2 weeks? (if not, is it worth it?)
- Is there a mature Rust library? (if not, do we build from scratch?)
- Do we have fallback if it fails?

**Be honest**:
- If HNSW is the boring but correct choice, that's OK
- If we need differentiation, be clear about the tradeoffs
- Don't pick an algorithm just because it's "novel"

---

## Success Criteria

**Research is complete when**:
- ✅ All 3 documents written (comprehensive, not rushed)
- ✅ Algorithm chosen with >80% confidence
- ✅ Implementation plan clear (library, timeline, milestones)
- ✅ Fallback plan identified
- ✅ Updated `omendb-server/ai/DECISIONS.md` and `omen-lite/ai/DECISIONS.md`

**Then**: Proceed to Week 2 implementation with confidence

---

## Notes

**History context** (why we're doing this research):
- Tried Mojo paper algorithm → didn't work (paper was only reference)
- Pivoted: DiskANN → HNSW+ → Rust learned DB → current state
- ALEX prototype: 5% recall (simple projection doesn't work for high-dim vectors)
- Lesson: Stop pivoting, do research first

**Key insights from Week 1**:
- pgvector IS slow (13-hour index builds, 60GB memory, 30s queries)
- pgvector USES HNSW (since v0.5.0, 2023)
- ALEX works great for 1D (primary keys), fails for high-dim vectors
- Market wants: PostgreSQL compatibility + scale

**What we're optimizing for**:
1. **Correctness**: >95% recall@10 (production-ready)
2. **Performance**: <10ms p95 latency, <2GB for 10M 1536-dim vectors
3. **Time-to-market**: 1-2 weeks implementation (not 6 months)
4. **Risk**: >80% confidence it will work

---

*Created: October 22, 2025*
*Start research: Next session*
*Complete by: October 24, 2025 (1-2 days, thorough)*
