# Vector Index Algorithm Research

**Status**: 🚨 **RESEARCH IN PROGRESS**
**Created**: October 22, 2025
**Goal**: Determine optimal vector indexing algorithm before Week 2 implementation

---

## Research Objectives

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
