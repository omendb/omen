# OmenDB Server Development Context

**Repository**: omendb-server (PostgreSQL-compatible Vector Database)
**Last Updated**: October 26, 2025 - Week 6 Days 1-2 Complete, Days 3-4 In Progress
**License**: Elastic License 2.0 (source-available, self-hostable)

## Product Overview

**omendb-server**: PostgreSQL-compatible vector database that scales

**Positioning**: "PostgreSQL-compatible vector database. Drop-in replacement for pgvector. 10x faster, 28x more memory efficient. Source-available. Self-hostable."

**Year 1 Focus** (2025-2026):
- Cloud-native deployment (managed service $29-99/month)
- Self-hosting mode (enterprises, compliance-driven)
- PostgreSQL wire protocol (drop-in pgvector compatibility)

**Future** (Year 2+):
- omen-lite (embedded variant) - shares 80% of codebase, different wire protocol
- Currently on hold to maintain focus

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

**Product**: PostgreSQL-compatible vector database (HNSW + Binary Quantization)
**Achievement**: Week 6 Days 1-2 COMPLETE - 100K graph serialization validated (3626x improvement!)
**Status**: Week 6 Days 3-4 IN PROGRESS → 1M scale validation running
**Stack**: Rust (HNSW + Binary Quantization + PostgreSQL protocol + RocksDB + MVCC)
**Phase**: Week 6/8 → Validate 1M scale → Optimize → Benchmarks vs pgvector
**Current**: Running 1M benchmark (~30 min build + save/load/query testing)

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
- **vs pgvector**: 10x faster at 10M+ vectors, 28x more memory efficient
- **vs Pinecone**: Same performance, 90% cheaper ($99 vs $500/mo), self-hostable, source-available
- **vs Weaviate/Qdrant**: PostgreSQL-compatible (no new API to learn)
- **Unique**: Only PostgreSQL-compatible vector DB that scales efficiently

---

## Licensing & Business Model

**License**: Elastic License 2.0 (source-available)

**What this means**:
- ✅ Free to use, modify, and self-host
- ✅ Source code publicly available (can verify PostgreSQL compatibility)
- ✅ Community can contribute (bug fixes, features)
- ✅ Enterprises can deploy on their infrastructure
- ❌ Cannot resell as managed service (protects cloud revenue)

**Revenue Model** (Hybrid: Flat + Caps):

| Tier | Price | Vectors | Queries/mo | Target Customer |
|------|-------|---------|------------|-----------------|
| **Developer** | **FREE** | 100K | 100K | Hobbyists, prototyping |
| **Starter** | **$29/mo** | 1M | 1M | Early startups, side projects |
| **Growth** | **$99/mo** | 10M | 10M | Production apps, scaling startups |
| **Enterprise** | **Custom** | Unlimited | Unlimited | Large deployments, compliance |

**Why this pricing wins**:
- **Predictable**: No surprise bills (vs Pinecone usage spikes)
- **Transparent**: Know your costs upfront
- **Competitive**: 90% cheaper than Pinecone at Growth tier

**Customer Focus (Year 1)**:
- **Primary (70%)**: AI startups (RAG, LangChain users, semantic search)
- **Secondary (30%)**: Enterprise (healthcare, finance, legal - compliance-driven)
- **Channel**: Self-serve (Free → Starter → Growth) + direct sales (Enterprise)

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
omendb-server/
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

## Product Roadmap & Code Strategy

**Year 1 Focus** (2025-2026): omendb-server ONLY
- Build cloud vector database first
- Validate product-market fit
- Self-hosting mode covers 95% of "embedded" use cases
- **Rationale**: Focus beats parallelization in early stage

**Year 2+**: Consider omen-lite (embedded variant)
- Extract to omen-lite IF demand exists
- Shares 80% of code (ALEX, vector ops, storage)
- Only difference: Wire protocol (embedded API vs PostgreSQL)
- 2-4 weeks of work (not 6 months)

**Code organization** (when extracting omen-lite):
- Extract shared code to **omendb-core** library (Apache 2.0):
  - `omendb-core/alex` - Multi-level ALEX index
  - `omendb-core/vector` - Vector types, distance functions
  - `omendb-core/mvcc` - MVCC snapshot isolation
  - `omendb-core/storage` - RocksDB abstractions
- Both products depend on omendb-core
- Standard Rust pattern: Build first, extract when stable

**GitHub Organization:**
- `omendb/omendb-server` - This repository (Elastic License 2.0)
- `omendb/omen-lite` - Embedded variant (Elastic License 2.0, Year 2+)
- `omendb/omendb-core` - Shared library (Apache 2.0, when extracted)
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

## Current Week 6 Progress (Oct 24-30)

### ✅ Days 1-2: Persisted HNSW Index (COMPLETE)
1. [✅] Implemented hnsw_rs serialization (dump/reload via hnswio module)
2. [✅] Added persistence to VectorStore (save/load graph + data)
3. [✅] Tested 100K vectors: **0.498s load (3626x faster than 1806s rebuild!)**
4. [✅] Auto-rebuild fallback implemented

**Actual Results** (100K vectors, 1536D):
- Build: 1806.43s (~30 minutes)
- Save: 0.699s (graph + data)
- Load: 0.498s (graph deserialization)
- **Improvement: 3626x faster than rebuild!**
- Query (before): 10.33ms avg (97 QPS)
- Query (after): 9.45ms avg (106 QPS) - 8.5% faster!
- Disk: 743.74 MB (127 MB graph + 616 MB data)
- **All pass/fail criteria exceeded ✅**

### 🔄 Days 3-4: 1M Scale Validation (IN PROGRESS)
5. [🔄] Running benchmark: 1M vectors (1536D), ~30 min build
6. [ ] Measure: Query latency (p50, p95, p99), memory usage, disk usage
7. [ ] Document: Scaling characteristics, any new bottlenecks
8. [ ] Validate: Linear scaling from 100K → 1M

**Expected Results** (1M vectors):
- Build: <30 minutes
- Save: <10s
- Load: <10s (vs ~5-10 hour rebuild)
- Query p95: <15ms
- Memory: <15GB
- Improvement: >50x vs rebuild

### Days 5-7: MN-RU Updates (Optional)
9. [ ] Research MN-RU algorithm (ArXiv 2407.07871)
10. [ ] Implement multi-neighbor replaced updates
11. [ ] Test insert/delete performance, graph quality
12. [ ] Benchmark mixed workload (50% reads, 50% writes)

**Success Criteria** (Week 6):
- ✅ 100K vectors <10ms p95 queries (achieved 9.45ms!)
- [ ] 1M vectors <15ms p95 queries (testing now)
- ✅ Persisted HNSW working (3626x improvement validated)

---

## Development Principles

**Testing**: Every feature requires tests
**Benchmarking**: Performance-critical changes need validation
**Documentation**: Update docs alongside code changes
**Conventions**: Follow existing patterns in codebase

**NEW Principle**: Build for vector DB market, defer non-vector features

---

*Last Updated: October 26, 2025 - Week 6 Days 1-2 COMPLETE (100K validated)*

**Product**: omendb-server - Cloud/server PostgreSQL-compatible vector database
**Companion**: omen-lite - Embedded vector database (separate repo, future)
**Market**: $10.6B vector DB market (23.54% CAGR)
**Timeline**: 6 months to production-ready, 12 months to $100K-500K ARR
**Next Milestone**: 1M scale validation (Week 6 Days 3-4, in progress)
**GitHub**: omendb/omendb-server
