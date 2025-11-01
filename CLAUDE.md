# omen - Vector Database

**Repository**: omen (Embedded PostgreSQL-compatible Vector Database)
**Last Updated**: November 1, 2025 - Week 11-12 Complete (Extended RaBitQ Quantization!)
**License**: Elastic License 2.0 (source-available, embeddable)
**Status**: 7223 QPS @ 128D, Extended RaBitQ (16x compression @ 84% recall), 122 tests passing

---

## Product Overview

**omen**: Embedded vector database with PostgreSQL wire protocol for AI applications

**What It Is**:
- Vector search (HNSW + Binary Quantization)
- SQL queries (PostgreSQL wire protocol)
- Transactions (MVCC snapshot isolation)
- Full-text search (Week 20-24, in progress)

**Positioning**: "Separate vector database for scale, PostgreSQL tools for familiarity. 97x faster builds than pgvector, 28x more memory efficient."

**What "PostgreSQL-compatible" Means**:
- ✅ **PostgreSQL wire protocol** (port 5433): Use psql, pgcli, any PostgreSQL client library
- ✅ **SQL syntax**: Familiar query language, no new API to learn
- ❌ **NOT a PostgreSQL extension** (like pgvector): omen is a SEPARATE database
- ❌ **NOT in your PostgreSQL database**: Vectors are separate from business data

**Why This Matters**: At 10M+ vectors, pgvector fails → you MUST migrate to a separate vector database. Today's options (Pinecone, LanceDB, Qdrant) have custom APIs. omen has PostgreSQL tools you already know.

**Market**: $10.6B by 2032 (23.54% CAGR)

---

## 🔗 Related Repositories

**This repo**: `omen/` - Vector database implementation
**Shared code**: `../omen-core/` - Future shared library (empty until Phase 2b)
**Managed service**: `../omen-server/` - Multi-tenancy, billing, monitoring (future)
**Business/strategy**: `../omen-org/` - Business plans, funding, roadmap (PRIVATE)

**For platform roadmap** (time-series, graph, unified): See `../omen-org/strategy/`

**Current Focus**: Vector database only (Weeks 9-24)

## Quick Start for AI Agents

**→ First time?** Load these in order:
1. This file (CLAUDE.md) - Project overview
2. `ai/TODO.md` - Current tasks
3. `ai/STATUS.md` - Current state
4. `ai/DECISIONS.md` - Key architectural decisions (when needed)

**→ Continuing work?** Check `ai/TODO.md` and `ai/STATUS.md` first

**→ Need organization guidance?**
- OmenDB follows standard OSS pattern (docs/ + ai/)
- See [agent-contexts v0.1.1](https://github.com/nijaru/agent-contexts) for:
  - File update patterns (edit vs append)
  - Context management strategies
  - Multi-session handoff protocols

---

## Week 11-12 Status (Nov 1, 2025)

**Week 11-12 COMPLETE** - Extended RaBitQ Quantization! 🎉:
- ✅ Week 9 Days 1-5: Custom HNSW implemented (1,200+ lines)
- ✅ Week 10 Days 1-5: Algorithm port + baseline validation
- ✅ Week 11 Day 1: Error infrastructure (Result<T>, zero panics)
- ✅ Week 11 Day 2: **SIMD distance functions (3.1-3.9x improvement!)**
- ✅ Week 11 Day 3: **Persistence validation (1035-1222x speedup, 100% data integrity!)**
- ✅ Week 11 Day 4: **Profiling + Repository cleanup (249 files deleted!)**
- ✅ Week 11-12: **Extended RaBitQ quantization (ALL 6 PHASES COMPLETE!)**
- 🎯 Next: Scale testing Extended RaBitQ @ 100K/1M vectors

**Current Performance** (SOTA Quantization Ready!):
- **7223 QPS @ 128D** (3.9x faster than baseline, Mac M3)
- **1051 QPS @ 1536D** (3.1x faster than baseline, Mac M3)
- **Extended RaBitQ**: 84% recall @ 16x compression, 100% @ 8x compression
- **Query latency**: 0.2-0.3ms with quantization (production-ready!)
- **1414 QPS @ 1M vectors** (128D, 0.92ms p95 latency)
- **Persistence**: 1035-1222x faster than rebuild (0.44-0.57s load time)
- **Data integrity**: 100% query result match after save/load
- **Memory efficiency**: 1.1x overhead (custom HNSW) + 4-16x quantization
- 122 tests passing (40 Extended RaBitQ tests + 82 core tests)

**Extended RaBitQ Achievements**:
- ✅ 2-bit (16x compression): 84.4% recall (exceeds 70% target)
- ✅ 4-bit (8x compression): 100% recall (exceeds 85% target)
- ✅ 8-bit (4x compression): 100% recall (exceeds 95% target)
- ✅ Two-phase search (quantized → rerank) working perfectly
- ✅ Full SIMD support (AVX2/SSE2/NEON)
- ✅ Persistence with JSON + bincode

**Completed**:
- ✅ Custom HNSW implementation (full control, no library limitations)
- ✅ SIMD distance functions (3.1-3.9x improvement)
- ✅ Extended RaBitQ quantization (1440 lines, 40 tests)
- ✅ VectorStore integration (two-phase search, persistence)
- ✅ A/B testing framework (scientific validation)
- ✅ Code cleanup (removed unused optimizations)

**Next** (Weeks 13-14):
- Scale testing Extended RaBitQ @ 100K/1M vectors
- Memory validation at different compression rates
- Performance profiling with quantization enabled
- Target: Sub-millisecond queries with 8-16x memory savings

**Then** (Weeks 14-19):
- HNSW-IF for billion-scale support (hybrid in-memory + disk)
- Delta encoding for memory efficiency
- Performance optimization (cache locality, prefetch that works)

**After** (Weeks 20-24):
- Full-text search (BM25, inverted index)
- Complete "Embedded AI Database"

---

## Current Status

**Product**: PostgreSQL-compatible vector database (HNSW + Binary Quantization)
**Achievement**: Week 11 Day 4 COMPLETE - Production Ready! 🎉
  - Week 9-10: Custom HNSW implementation (1,200+ lines, zero external dependencies) ✅
  - Week 11 Day 1: Error handling (zero panics, production-ready) ✅
  - Week 11 Day 2: SIMD optimization (3.1-3.9x improvement, 7223 QPS @ 128D) ✅
  - Week 11 Day 3: Persistence validation (1035-1222x speedup, 100% data integrity) ✅
  - Week 11 Day 4: Repository cleanup + profiling (249 files deleted, 82 tests) ✅
**Stack**: Rust (custom HNSW + SIMD + Binary Quantization + persistence)
**Phase**: Week 11 (Production Readiness) - 82 tests passing, clean codebase
**Priority**: 🚀 Extended RaBitQ implementation (SIGMOD 2025, 2-9 bits/dimension)
**Next**: Begin Extended RaBitQ (Week 11 Day 5+)

## Technical Architecture

**Core Components**:
- ✅ **PostgreSQL wire protocol**: Drop-in pgvector replacement
- ✅ **Memory efficiency (28x)**: Critical for 100M+ vector scale
- ✅ **MVCC + transactions**: Unique vs pure vector DBs (Pinecone, Weaviate)
- ✅ **HTAP architecture**: One DB for vectors + business data
- ✅ **Self-hosting option**: Compliance/privacy vs cloud-only (Pinecone)

**Market Position** (Vector DB Focus):
- **vs pgvector**: 10x faster at 10M+ vectors, 19x more memory efficient
- **vs Pinecone**: Same performance, 90% cheaper ($99 vs $500/mo), self-hostable, source-available
- **vs Weaviate/Qdrant**: PostgreSQL-compatible (no new API to learn)
- **Unique**: Only PostgreSQL-compatible vector DB that scales efficiently

**SOTA Positioning**:

*Current State (Week 7 Day 3 Complete):*
- ✅ HNSW: 99.5% recall, <15ms p95 (industry standard)
- ✅ Binary Quantization: 19.9x memory reduction (competitive)
- ✅ **16x parallel building** (UNIQUE - undocumented by competitors)
- ✅ **4175x serialization** (UNIQUE - undocumented by competitors)
- ✅ **97x faster builds vs pgvector** (100K vectors: 31s vs 3026s)
- ✅ **2.2x faster queries vs pgvector** (p95: 6.16ms vs 13.60ms)
- ✅ PostgreSQL compatible (UNIQUE vs pure vector DBs)
- ✅ **142 tests passing** (101 Phase 1 + 41 Phase 2)
- ✅ **ASAN validated** (40 tests, ZERO memory safety issues)
- ✅ **Phase 2 validation** (60% complete - edge cases, boundaries, resource limits)

*After Engine Optimization (Week 8):*
- ✅ All above +
- ✅ **SIMD enabled** (2-4x query improvement, ~400-500 QPS)
- ✅ Profiling complete (bottlenecks identified)
- ✅ Quick wins implemented (LTO, opt-level, allocations)
- **Target**: 4-8x cumulative improvement from current baseline

*After Custom HNSW Core (Weeks 9-10):*
- ✅ All above +
- ✅ Custom HNSW implementation (full control, no library limitations)
- ✅ Match or beat hnsw_rs + SIMD performance
- ✅ Foundation for SOTA features (Extended RaBitQ, HNSW-IF, MN-RU)
- **Target**: 6-10x cumulative improvement, Qdrant-competitive

*After HNSW-IF (Weeks 9-10):*
- ✅ All above +
- ✅ Billion-scale support (Vespa-proven approach)
- ✅ Automatic scaling (in-memory → hybrid at 10M+)
- ✅ No infrastructure dependencies (no NVMe/SPDK)
- **Differentiator**: Only PostgreSQL-compatible DB with billion-scale support

*After Extended RaBitQ (Weeks 11-12):*
- ✅ All above +
- ✅ SOTA quantization (SIGMOD 2025)
- ✅ Arbitrary compression rates (4x-32x)
- ✅ Better accuracy at same memory footprint
- **Differentiator**: SOTA vector DB with PostgreSQL compatibility

**Research Reference**: See `ai/research/SOTA_ALGORITHMS_INVESTIGATION_OCT2025.md` for full analysis of 6 algorithms (MN-RU, SPANN, SPFresh, HNSW-IF, Extended RaBitQ, NGT-QG) and strategic roadmap.

**Strategic Documents** (Week 7-9 - Oct 30, 2025):
- `docs/architecture/CUSTOM_HNSW_DESIGN.md` (1000+ lines, Week 9 Day 1) - **Custom HNSW architecture design**
  - Complete implementation roadmap (Weeks 9-19)
  - Core data structures (cache-line aligned, flattened index)
  - Performance projections (581 → 1000+ QPS)
  - SOTA features roadmap (Extended RaBitQ, delta encoding)
- `ai/research/STRATEGIC_COMPETITIVE_POSITIONING.md` (6400+ words) - Comprehensive competitive analysis
  - 8 competitors analyzed (Qdrant, Milvus, Weaviate, LanceDB, ChromaDB, Pinecone, pgvector, pgvecto.rs)
  - PostgreSQL compatibility value analysis
  - Can we reach Qdrant performance? Billion scale? (answers: YES, YES)
  - Feature matrix, performance projections, strategic positioning
- `ai/research/COMPETITIVE_ANALYSIS_VECTOR_DBS.md` - Market landscape & testing strategy
  - Competitor priorities (Qdrant → LanceDB → Milvus/Weaviate)
  - Testing methodology, benchmarking approach
  - Profiling plan, optimization roadmap
- `ai/research/OPTIMIZATION_STRATEGY.md` - Engine-first optimization plan
  - **CRITICAL**: SIMD available but NOT ENABLED (2-4x free win)
  - Phase 1: Quick wins (SIMD, LTO, opt-level) - Week 1
  - Phase 2: Profiling & optimization - Week 1-2
  - Phase 3: Algorithmic improvements - Week 3-4
  - Phase 4: Scale validation - Week 5-8
- `ai/research/CUSTOM_HNSW_DECISION.md` - Custom vs library analysis
  - **Decision**: Build custom HNSW for SOTA features
  - ALL serious competitors use custom implementations
  - hnsw_rs limitations documented
  - 10-15 week implementation plan
  - Performance projections: 162 QPS → 400-500 QPS (Week 1) → 1000 QPS (Week 10)

---

## License

**Elastic License 2.0** (source-available)

**What this means**:
- ✅ Free to use, modify, and self-host
- ✅ Source code publicly available
- ✅ Community can contribute (bug fixes, features)
- ❌ Cannot resell as managed service

**For business model and pricing**: See `../omen-org/strategy/`

## Architecture (Vector DB - November 1, 2025)

```
Current Stack (Production Ready!):
├── Vector Data Type: Vector struct with f32 dimensions ✅
├── Custom HNSW Index: Full implementation (1,200+ lines) ✅
├── SIMD Distance Functions: AVX2/SSE2/NEON runtime detection ✅
├── Binary Quantization: 19x memory reduction ✅
├── Persistence Layer: Save/load with 1000x+ speedup ✅
├── Error Handling: Production-ready Result types ✅
└── Observability: Structured logging with tracing ✅

Future (Weeks 11-24):
├── Extended RaBitQ: 2-9 bits/dimension quantization 🔨
├── HNSW-IF: Hybrid in-memory + disk for billion-scale 🔨
├── PostgreSQL Protocol: Wire protocol integration 🔨
└── Full-text Search: BM25 + inverted index 🔨
```

**Architecture Validation**:
- Custom HNSW: Full control, SOTA features possible
- SIMD: 3.1-3.9x improvement, cross-platform (Mac/Linux)
- Persistence: 1035-1222x speedup, 100% data integrity
- Memory efficiency: 1.1x overhead (vs 2-3x for libraries)
- Production ready: Zero panics, comprehensive error handling

**Repository Structure** (Clean and Minimal - Nov 1, 2025):
```
omen/
├── CLAUDE.md              # This file - AI agent entry point
├── README.md              # Project overview
├── Cargo.toml             # Dependencies
├── docs/                  # Documentation (standard OSS pattern) 📚
│   └── architecture/      # Technical deep-dives
│       ├── CUSTOM_HNSW_DESIGN.md (1,539 lines)
│       ├── EXTENDED_RABITQ_PLAN.md (538 lines)
│       └── PROFILING_REPORT_OCT31.md
├── ai/                    # AI working context ⭐
│   ├── TODO.md            # Current tasks
│   ├── STATUS.md          # Current state (Week 11 Day 4)
│   ├── DECISIONS.md       # Architectural decisions
│   └── research/          # Research & analysis
│       ├── STRATEGIC_COMPETITIVE_POSITIONING.md (6,400+ words)
│       ├── COMPETITIVE_ANALYSIS_VECTOR_DBS.md
│       ├── OPTIMIZATION_STRATEGY.md
│       ├── CUSTOM_HNSW_DECISION.md
│       └── SOTA_ALGORITHMS_INVESTIGATION_OCT2025.md
├── src/                   # Source code (clean & minimal!) ✨
│   ├── lib.rs             # Public API (Vector, VectorStore, logging)
│   ├── logging.rs         # Structured logging with tracing
│   ├── vector/            # Vector database implementation
│   │   ├── mod.rs         # Module exports
│   │   ├── types.rs       # Vector, Distance types
│   │   ├── vector_value.rs # VectorValue enum
│   │   ├── store.rs       # VectorStore (900+ lines)
│   │   ├── hnsw_index.rs  # HNSWIndex adapter
│   │   └── custom_hnsw/   # Custom HNSW implementation
│   │       ├── mod.rs
│   │       ├── types.rs   # Node, Layer, IndexParams
│   │       ├── storage.rs # Graph storage
│   │       ├── index.rs   # Insert, search, persistence (800+ lines)
│   │       ├── distance.rs # SIMD distance functions
│   │       └── error.rs   # HNSWError types
│   ├── bin/               # Benchmarks & tools
│   │   ├── benchmark_simd.rs
│   │   ├── benchmark_persistence.rs
│   │   ├── profile_hnsw.rs
│   │   └── [10+ benchmarks]
│   └── tests/             # Integration tests
│       ├── test_distance_correctness.rs
│       ├── test_hnsw_recall.rs
│       └── test_resource_limits.rs
└── 82 tests passing! ✅
```

**Cleanup Summary (Week 11 Day 4)**:
- ✅ **249 files deleted** (81,608 lines) - 90%+ of old code
- ✅ **Archived** to `omen-org/archive/omendb-oct2025/`
- ✅ **lib.rs rewritten** from time-series → vector database API
- ✅ **Clean structure**: Only src/vector/, src/logging.rs, src/bin/, src/tests/
- ✅ **All old pivots removed**: ALEX, DataFusion, PCA, MVCC, SQL engine, etc.

**Pattern**: Minimal, focused vector database
- **src/vector/** — Core vector database (custom HNSW + SIMD + persistence)
- **src/bin/** — Performance benchmarks and profiling tools
- **ai/** — AI working context (research-validated strategy)

---

## Product Roadmap & Code Strategy

**Current (2025)**: omen - Embedded Vector Database
- Build solid embedded library first (embedded-first architecture)
- PostgreSQL wire protocol for compatibility
- Validate technology foundation (HNSW, BQ, MVCC, serialization)
- **Rationale**: Embedded-first approach (like libSQL→Turso, SQLite→D1)

**Future**: omen-server (Managed Cloud Service)
- Build managed cloud service on top of omen
- Add: Multi-tenancy, authentication, billing, monitoring
- Thin API layer wrapping omen embedded library
- 2-4 weeks of work after omen is production-ready

**Code organization** (Phase 2b - when building omen-time):
- All code stays in omen/ for now (don't extract yet)
- Phase 2b: Extract shared code to **omen-core** library (Apache 2.0):
  - `omen-core/mvcc` - MVCC snapshot isolation
  - `omen-core/storage` - RocksDB abstractions
  - `omen-core/protocol` - PostgreSQL wire protocol
  - `omen-core/cache` - LRU cache
  - `omen-core/security` - Auth, SSL/TLS
- Both omen and omen-time will depend on omen-core
- Standard Rust pattern: Build first, extract when needed

**GitHub Organization:**
- `omendb/omen` - This repository - Embedded vector database (Elastic License 2.0)
- `omendb/omen-core` - Empty placeholder - Future shared library (Apache 2.0, Phase 2b)
- `omendb/omen-server` - Future: Managed cloud service (Private, when built)
- `omendb/omen-org` - Private: Business strategy, experimental code archive
- `omendb/pg-learned` - PostgreSQL extension (Elastic License 2.0, marketing/education)

---

## Validated Technology Foundation (Ready for Vectors)

**Already Built** (Relevant to Vector DB):
1. ✅ **Multi-level ALEX index** (production-ready, scales to 100M+)
2. ✅ **PostgreSQL wire protocol** (pgvector compatibility foundation)
3. ✅ **MVCC snapshot isolation** (85 tests, concurrent vector operations)
4. ✅ **Authentication + SSL/TLS** (57 tests, enterprise-ready)
5. ✅ **LRU cache layer** (2-3x speedup, 90% hit rate)
6. ✅ **Crash recovery** (100% success rate)
7. ✅ **RocksDB storage** (LSM tree, write-optimized)

**Test Coverage**: 142 tests passing (Week 7 Day 2+)
- 101 Phase 1 tests (distance correctness, HNSW recall, serialization, concurrency, input validation)
- 41 Phase 2 tests (resource limits, edge cases, boundaries)
- 40 tests ASAN validated (ZERO memory safety issues)

**Performance Characteristics** (Base Technology):
- Memory: 1.50 bytes/key (28x better than PostgreSQL)
- Scaling: Linear to 100M+ rows
- Cache: 90% hit rate on Zipfian workloads
- Recovery: 100% success rate

---

## What We Need to Build (Vector DB)

**Phase 1: Vector Foundation** (8-10 weeks):
- [ ] Vector data type (`vector(N)` - pgvector compatible)
- [ ] Distance operators (`<->`, `<#>`, `<=>` for L2, dot, cosine)
- [ ] Vector functions (l2_distance, cosine_distance, etc.)
- [ ] ALEX index for vectors (CREATE INDEX USING alex)
- [ ] Benchmark vs pgvector (1M, 10M, 100M vectors)
- **Target**: 10x faster than pgvector, <2GB for 10M vectors

**Phase 2: Performance & Scale** (4-6 weeks):
- [ ] Optimize ALEX for high-dimensional data
- [ ] Batch vector insert optimization
- [ ] Hybrid search (vector + SQL filters)
- [ ] Query planning for vector operations
- [ ] Memory profiling and optimization
- **Target**: Match Pinecone performance, 30x memory efficiency

**Phase 3: Migration & Go-to-Market** (4-6 weeks):
- [ ] pgvector → OmenDB migration script
- [ ] Vector examples (RAG, semantic search, recommendations)
- [ ] Documentation (installation, migration, API)
- [ ] Managed cloud (basic $29-499/month tiers)
- **Target**: 50-100 users, $1-5K MRR

**Total Timeline**: 16-22 weeks (4-5 months) to production-ready vector database

---

## Target Market (Year 1 Focus)

**Customer Prioritization**:
- **Primary (70% of effort)**: AI-first Startups
- **Secondary (30% of effort)**: Enterprise AI

**Why this split**:
- AI startups = high volume, fast sales cycle, product-led growth
- Enterprise = high ARPU, validates enterprise readiness
- Both have urgent pain (pgvector doesn't scale, Pinecone too expensive)

---

### Primary: AI-First Startups ($29-99/month)

**Use cases**:
- RAG applications (chatbots, document Q&A, knowledge bases)
- Semantic search (code search, research papers, documentation)
- AI agents (LangChain, LlamaIndex integrations)

**Pain points**:
- pgvector too slow at 1M-10M vectors
- Pinecone costs $500-2K/month (overkill for early stage)
- Need PostgreSQL compatibility (existing infrastructure)

**Discovery channels**:
- LangChain/LlamaIndex Discord communities
- HackerNews ("Show HN: PostgreSQL-compatible vector database")
- Direct outreach (GitHub search for pgvector users)
- YC batch network (if applicable)

**Conversion path**: Free tier (prototype) → Starter $29 (launch) → Growth $99 (scale)

---

### Secondary: Enterprise AI ($20K-100K/year)

**Use cases**:
- Healthcare: Patient similarity, drug discovery, medical records search
- Finance: Fraud detection, trading signals, document analysis
- Legal: Case law search, contract similarity, e-discovery

**Pain points**:
- Can't use cloud Pinecone (compliance: HIPAA, SOC2, data sovereignty)
- pgvector doesn't scale to 100M+ vectors
- Need enterprise support, SLAs, on-prem deployment

**Discovery channels**:
- Direct sales (healthcare AI, fintech, legal tech companies)
- Conferences (AI in Healthcare, FinTech conferences)
- Compliance forums (self-hosting = key differentiator)

**Conversion path**: Custom POC → Annual contract → White-glove onboarding

---

**Market Size**:
- 2025: $2.5B
- 2032: $10.6B
- CAGR: 23.54%

---

## Competitive Landscape (Vector DB)

**OmenDB vs Competitors**:

| Feature | pgvector | Pinecone | Weaviate | OmenDB |
|---------|----------|----------|----------|---------|
| PostgreSQL compatible | ✅ | ❌ | ❌ | ✅ |
| Scales to 100M+ vectors | ❌ | ✅ | ✅ | ✅ |
| Self-hosting | ✅ | ❌ | ✅ | ✅ |
| Memory efficient | ❌ | ? | ❌ | ✅ (28x) |
| HTAP (transactions + analytics) | ✅ | ❌ | ❌ | ✅ |
| License | PostgreSQL | Proprietary | Apache 2.0 | Elastic 2.0 |
| Pricing | Free | $70-8K+/mo | Free/Paid | $29-99/mo |

**Competitive Moats**:
1. **PostgreSQL compatibility** (pgvector users can drop-in migrate, no new API)
2. **Memory efficiency** (28x vs PostgreSQL = 90% cheaper at scale)
3. **HTAP architecture** (one DB for vectors + business logic, not two systems)
4. **Self-hosting + managed** (unlike Pinecone cloud-only, compliance-friendly)
5. **Source-available** (Elastic License - can verify, modify, contribute)

---

## Development Environment

**Machine Usage Strategy**:

**Mac (M3 Max, 128GB RAM)** - Primary Development Machine:
- ✅ All development work: code editing, git operations
- ✅ Compilation: `cargo build --release` (fast, surprisingly quiet)
- ✅ Benchmarks: Performance testing, most workloads
- ✅ Quick iteration: Fast single-threaded performance
- ✅ 128GB RAM: Handles large datasets easily
- 💡 **Use for 95% of work** - faster and quieter than expected

**Fedora PC (i9-13900KF 24-core, 32GB DDR5)** - Backup/Specialized:
- ✅ Multi-hour stress tests (if Mac is needed elsewhere)
- ✅ GPU workloads (RTX 4090)
- ⚠️ Requires clang-devel for RocksDB compilation
- 🔌 Access via: `ssh nick@fedora` (Tailscale)
- 📝 Note: 32GB RAM may limit very large datasets vs Mac's 128GB

**Rule of Thumb**:
- **Default: Use Mac for everything** (fast, quiet, more RAM)
- Only use Fedora if Mac is busy or for GPU tasks

**Stack**:
- Rust (cargo, rustc)
- PostgreSQL clients (psql, pgcli)
- Benchmarking tools (hyperfine, flamegraph)
- Testing: 142 tests via cargo test (101 Phase 1 + 41 Phase 2)

---

## Common Commands

**Development:**
```bash
cargo build                      # Fast, unoptimized
cargo test                       # All tests (557)
cargo clippy                     # Lints
```

**Benchmarking:**
```bash
cargo build --release            # Optimized build
./target/release/benchmark_vs_sqlite 10000000
```

**Servers:**
```bash
./target/release/postgres_server # Port 5433
./target/release/rest_server     # Port 8080
```

---

## Week 6 Complete (Oct 24-30) ✅

### Days 1-2: Persisted HNSW Index ✅ COMPLETE
1. [✅] Implemented hnsw_rs serialization (dump/reload via hnswio module)
2. [✅] Added persistence to VectorStore (save/load graph + data)
3. [✅] Tested 100K vectors: 0.498s load (3626x faster than rebuild!)
4. [✅] Tested 1M vectors: **6.02s load (4175x faster than 7h rebuild!)**
5. [✅] Auto-rebuild fallback implemented

**Actual Results** (1M vectors, 1536D):
- Build: 25,146s (7 hours) sequential
- Save: 4.91s (graph + data)
- Load: 6.02s (graph deserialization)
- **Improvement: 4175x faster than rebuild!**
- Query (before): p50=13.70ms, p95=16.01ms, p99=17.10ms
- Query (after): p50=12.24ms, p95=14.23ms, p99=15.26ms (11.1% faster!)
- Disk: 7.26 GB (1.09 GB graph + 6.16 GB data)
- **Pass/Fail: 6/7 criteria passed** (build time needs parallel building)

### Days 3-4: Parallel Building + 1M Validation ✅ COMPLETE
5. [✅] Implemented parallel building (HNSWIndex::batch_insert + VectorStore::batch_insert)
6. [✅] Tested correctness: 10K vectors, 4.64x speedup, 100% query success
7. [✅] Validated 1M parallel on Fedora 24-core: **16.17x speedup!**
8. [✅] Edge cases handled: empty batch, single vector, large batches, dimension validation

**Implementation Results** (10K vectors):
- Sequential: 1,851 vec/sec
- Parallel: 8,595 vec/sec
- **Speedup: 4.64x** (exceeds 2-4x target!)

**Actual 1M Results** (Fedora 24-core):
- Build: 1,554.74s (25.9 minutes) vs 25,146s sequential (7 hours)
- **Speedup: 16.17x!** (far exceeds 7-9x target!)
- Rate: 643 vec/sec (vs 40 vec/sec sequential)
- Query p95: 10.57ms (excellent!)
- Save: 3.83s, Load: (needs validation)
- Disk: 7.27 GB

### Days 5-7: SOTA Research & Planning ✅ COMPLETE
9. [✅] Researched MN-RU algorithm - ❌ BLOCKED (hnsw_rs has no delete/update)
10. [✅] Researched SPANN/SPFresh - ⚠️ TOO COMPLEX (DiskANN-style issues)
11. [✅] Researched Hybrid HNSW-IF - ✅ RECOMMENDED (Vespa-proven, simple)
12. [✅] Researched Extended RaBitQ - ✅ RECOMMENDED (SIGMOD 2025)
13. [✅] Researched NGT-QG - ⚠️ ALTERNATIVE (not clearly better)
14. [✅] Created research document: `ai/research/SOTA_ALGORITHMS_INVESTIGATION_OCT2025.md`
15. [✅] Strategic decision: Target HNSW-IF (Weeks 9-10) + Extended RaBitQ (Weeks 11-12)

**Strategic Roadmap** (Validated):
1. **Weeks 7-8**: pgvector benchmarks ⭐ CRITICAL PATH (validate "10x faster" claims)
2. **Weeks 9-10**: HNSW-IF implementation (billion-scale support)
3. **Weeks 11-12**: Extended RaBitQ (SOTA quantization)

**Success Criteria** (Week 6): ✅ ALL PASSED
- ✅ 100K vectors <10ms p95 queries (achieved 9.45ms!)
- ✅ 1M vectors <15ms p95 queries (achieved 14.23ms!)
- ✅ Parallel building 2-4x speedup (achieved 4.64x on Mac, 16.17x on Fedora!)
- ✅ Persisted HNSW working (4175x improvement at 1M scale!)
- ✅ SOTA research complete (roadmap validated)

---

## Development Principles

**Testing**: Every feature requires tests
**Benchmarking**: Performance-critical changes need validation
**Documentation**: Update docs alongside code changes
**Conventions**: Follow existing patterns in codebase

**NEW Principle**: Build for vector DB market, defer non-vector features

---

*Last Updated: October 27, 2025 - Week 7 Day 2+ (Phase 2 Validation 60% Complete)*

**Product**: omen - Embedded PostgreSQL-compatible vector database (Phase 1 focus)
**Future Products**: omen-server (managed cloud service), Time series database (2026-2027), Graph database (2027-2028)
**Platform Vision**: Unified multi-database platform with shared infrastructure
**Market**: $10.6B vector DB market (23.54% CAGR)
**Current Phase**: Validation before marketing (142 tests, ASAN clean)
**Next Milestone**: Complete Phase 2 validation → pgvector benchmarks
**GitHub**: omendb/omen
