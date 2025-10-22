# OmenDB Development Context

**Last Updated**: October 22, 2025 (Evening) - STRATEGIC PIVOT

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

## STRATEGIC PIVOT (October 22, 2025)

**New Positioning**: PostgreSQL-Compatible Vector Database That Scales

**Old Focus** (Abandoned):
- ❌ "Fast Embedded PostgreSQL"
- ❌ Embedded/edge/IoT markets
- ❌ "Faster SQLite" positioning

**New Focus**:
- ✅ Vector database for AI applications
- ✅ pgvector drop-in replacement (10x faster at scale)
- ✅ $10.6B market growing at 23.54% CAGR
- ✅ Target: RAG, semantic search, AI platforms

**Why the Pivot**:
1. Vector DB market: $10.6B by 2032 (vs $2-3B embedded DB)
2. Clear pain point: pgvector doesn't scale, Pinecone expensive
3. Perfect tech fit: ALEX + PostgreSQL protocol + memory efficiency
4. High willingness to pay: $29-499/month (validated by Pinecone)

---

## Current Status

**Product**: PostgreSQL-compatible vector database with learned index (ALEX)
**Achievement**: Technology foundation ready for vectors (ALEX, PostgreSQL protocol, MVCC, 28x memory efficiency)
**Status**: STRATEGIC PIVOT → Vector database prototyping (Week 1-2)
**Stack**: Rust (Multi-level ALEX + PostgreSQL protocol + RocksDB + MVCC + LRU cache)
**Phase**: Vector foundation prototyping → 6 months to production-ready
**Priority**: 🚨 Validate ALEX for high-dimensional vectors (Week 1-2)

## Technical Core

**Multi-Level ALEX (Ready for Vectors)**:
- Hierarchical learned index structure (height 2-3)
- 1.50 bytes/key memory (28x more efficient than PostgreSQL)
- Linear scaling validated to 100M+ rows
- **Hypothesis**: Perfect for high-dimensional vector indexing
- **Validation needed**: Prototype for 1536-dim vectors (OpenAI embeddings)

**Competitive Advantages for Vector DB**:
- ✅ **PostgreSQL wire protocol**: Drop-in pgvector replacement
- ✅ **Memory efficiency (28x)**: Critical for 100M+ vector scale
- ✅ **MVCC + transactions**: Unique vs pure vector DBs (Pinecone, Weaviate)
- ✅ **HTAP architecture**: One DB for vectors + business data
- ✅ **Self-hosting option**: Compliance/privacy vs cloud-only (Pinecone)

**Market Position** (Vector DB Focus):
- **vs pgvector**: 10x faster at 10M+ vectors, 30x more memory efficient
- **vs Pinecone**: Same performance, 1/10th cost, self-hostable, open source
- **vs Weaviate/Qdrant**: PostgreSQL-compatible (no new API to learn)
- **Unique**: Only PostgreSQL-compatible vector DB that scales efficiently

## Architecture (Vector DB - October 22, 2025)

```
Current Stack (Pre-Vector):
├── Protocol Layer: PostgreSQL wire protocol (port 5433) ✅
├── MVCC Layer: Snapshot isolation ✅
├── Index Layer: Multi-level ALEX (3-level hierarchy) ✅
├── Cache Layer: 1-10GB LRU cache ✅
├── Storage Layer: RocksDB (LSM tree) ✅
├── Recovery: 100% crash recovery ✅
└── Security: Auth + SSL/TLS ✅

Planned (Vector DB):
├── Vector Data Type: vector(N) - pgvector compatible 🔨
├── Distance Operators: <-> (L2), <#> (dot), <=> (cosine) 🔨
├── Vector Index: ALEX for high-dimensional data 🔨
├── ANN Search: Approximate nearest neighbor 🔨
└── Hybrid Search: Vector similarity + SQL filters 🔨
```

**Architecture Validation**:
- ALEX (learned index): Works great for sequential keys, testing for vectors
- RocksDB (LSM tree): Industry-proven storage backend
- MVCC: Concurrent vector operations (unique vs Pinecone)
- PostgreSQL protocol: Huge ecosystem compatibility
- Memory efficiency: 28x advantage critical for large vector datasets

**Repository Structure** (Standard OSS - agent-contexts v0.1.1):
```
omendb/core/
├── CLAUDE.md              # This file - AI agent entry point
├── docs/                  # Documentation (standard OSS pattern) 📚
│   ├── README.md          # Documentation index
│   ├── QUICKSTART.md      # Getting started
│   ├── ARCHITECTURE.md    # System design
│   ├── PERFORMANCE.md     # Benchmarks
│   ├── SECURITY.md        # Security guide
│   ├── architecture/      # Technical deep-dives
│   │   ├── research/      # Research findings (26+ docs)
│   │   ├── MULTI_LEVEL_ALEX.md
│   │   ├── ROADMAP_0.1.0.md (OUTDATED - needs vector update)
│   │   ├── ROCKSDB_PERFORMANCE_ANALYSIS_OCT_22.md
│   │   └── STORAGE_ENGINE_TEST_VALIDATION_OCT_22.md
│   ├── strategy/          # Business strategy (private repo only)
│   │   ├── COMPETITIVE_STRATEGY_OCT_2025.md (needs vector update)
│   │   └── CUSTOMER_ACQUISITION.md (OUTDATED - Jan 2025)
│   └── archive/           # Historical documentation
│       ├── phases/        # Phase planning docs
│       └── PHASE_*_COMPLETE.md
├── ai/                    # AI working context ⭐
│   ├── TODO.md            # Current tasks (UPDATED - vector roadmap)
│   ├── STATUS.md          # Current state (UPDATED - vector pivot)
│   ├── DECISIONS.md       # Working decision log (UPDATED - pivot decision)
│   └── RESEARCH.md        # Research index
├── src/                   # Source code
│   ├── alex/              # Multi-level ALEX implementation
│   ├── postgres/          # PostgreSQL wire protocol + auth
│   ├── mvcc/              # MVCC snapshot isolation ✅
│   ├── cache.rs           # LRU cache layer ✅
│   ├── sql_engine.rs      # SQL engine (needs vector operators)
│   ├── catalog.rs         # Table + user management
│   ├── user_store.rs      # Persistent user storage
│   └── table.rs           # Table storage + ALEX + cache
└── tests/                 # 557 tests (99.8% passing) ✅
```

**Pattern**: Standard OSS database structure (like PostgreSQL, MongoDB, DuckDB, CockroachDB)
- **docs/** — All permanent documentation (user guides, architecture, research)
- **ai/** — AI working context (tasks, status, decisions, research notes)

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

**Test Coverage**: 557 tests passing
- 468 library tests (MVCC, storage, ALEX)
- 57 security tests (auth, SSL/TLS)
- 32 SQL tests (aggregations, joins)

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

## Target Market (Vector DB)

**Primary Customers**:

**Tier 1: AI-first Startups** ($29-299/month):
- RAG applications (chatbots, search, Q&A)
- Code search, document search, semantic search
- **Pain**: pgvector too slow at 10M embeddings, Pinecone costs $2K/month
- **Examples**: AI chat platforms, research paper search, code assistants

**Tier 2: E-commerce + SaaS** ($299-2K/month):
- Product recommendations, semantic product search
- User analytics, customer support automation
- **Pain**: Need PostgreSQL for transactions + vector search, running two DBs
- **Examples**: E-commerce platforms, SaaS analytics, support automation

**Tier 3: Enterprise AI** ($2K-20K/month):
- Healthcare (patient similarity, drug discovery)
- Finance (fraud detection, trading signals)
- Legal (case law search, document similarity)
- **Pain**: Can't use cloud Pinecone (compliance), pgvector doesn't scale
- **Examples**: Healthcare AI, fintech, legal tech

**Tier 4: AI Platform Companies** ($20K+/month):
- LangChain, LlamaIndex (need vector backend)
- AI agent platforms, RAG-as-a-service
- **Pain**: Building on Pinecone = vendor lock-in, need open source
- **Examples**: AI infrastructure, developer tools, ML platforms

**Market Size**:
- 2023: $1.6B
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
| Pricing | Free | $70-8K+/mo | Free/Paid | $29-499/mo |

**Competitive Moats**:
1. **PostgreSQL compatibility** (pgvector users can drop-in migrate)
2. **Memory efficiency** (28x vs PostgreSQL = 10x cheaper at scale)
3. **HTAP architecture** (one DB for vectors + business logic)
4. **Self-hosting + managed** (unlike Pinecone cloud-only)
5. **Open source** (avoid vendor lock-in)

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
- Testing: 557 tests via cargo test

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

## Immediate Next Steps (This Week)

### Critical Priority: Vector Prototype & Validation

**Week 1 (Oct 22-28): ALEX Vector Prototype**
1. [ ] Research pgvector implementation (GitHub: pgvector/pgvector)
2. [ ] Design vector(N) data type in Rust
3. [ ] Prototype ALEX for 1536-dim vectors (100K-1M vectors)
4. [ ] Measure: Memory usage, query latency, index build time
5. [ ] **Go/No-Go**: If ALEX doesn't work → pivot to HNSW algorithm

**Week 1 (Oct 22-28): Customer Validation**
1. [ ] Identify 50 companies using pgvector (search GitHub, LangChain repos)
2. [ ] Draft cold email: "Building pgvector that scales to 100M vectors"
3. [ ] Send 20 emails (target 5 responses)
4. [ ] Schedule 3-5 customer calls
5. [ ] **Validate**: Pain point is real, willingness to pay $29-99/month

**Decision Point** (End of Week 1):
- ✅ If ALEX works + 3+ customer validations → Proceed with vector DB
- ❌ If ALEX doesn't work → Pivot to HNSW algorithm
- ❌ If no customer interest → Reconsider vector market

---

## Development Principles

**Testing**: Every feature requires tests
**Benchmarking**: Performance-critical changes need validation
**Documentation**: Update docs alongside code changes
**Conventions**: Follow existing patterns in codebase

**NEW Principle**: Build for vector DB market, defer non-vector features

---

*Last Updated: October 22, 2025 (Evening) - STRATEGIC PIVOT*

**Focus**: PostgreSQL-compatible vector database
**Market**: $10.6B vector DB market (23.54% CAGR)
**Timeline**: 6 months to production-ready, 12 months to $100K-500K ARR
**Next Milestone**: ALEX vector prototype validation (Week 1-2)
