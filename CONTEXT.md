# Quick Context - omendb-server

**Load this file first when starting Claude Code in this repo**

---

## What This Repo Is

**omendb-server** - PostgreSQL-compatible vector database that scales

**Positioning**: "PostgreSQL-compatible vector database. Drop-in replacement for pgvector. 10x faster, 28x more memory efficient."

- **Product**: Cloud vector database + self-hosting
- **License**: Elastic License 2.0 (source-available, self-hostable)
- **Pricing**: Free, $29, $99/month + Enterprise (custom)
- **Market**: AI startups (primary 70%), Enterprise (secondary 30%)
- **Status**: Week 1 - Vector prototype + customer validation
- **Tech**: Rust + ALEX indexing + HNSW + PostgreSQL protocol + RocksDB
- **GitHub**: omendb/omendb-server

---

## Load These Files (In Order)

1. **This file** (CONTEXT.md) - Quick overview
2. **CLAUDE.md** - Full project context (product overview, architecture, roadmap)
3. **ai/TODO.md** - Current tasks
4. **ai/STATUS.md** - Current state
5. **ai/RESEARCH.md** - Vector algorithm research (when needed)
6. **ai/DECISIONS.md** - Architectural decisions (when needed)

---

## Current Priority (Week 1)

**Goal**: Validate ALEX for high-dimensional vectors

**Tasks**:
1. Research pgvector implementation (how they store/index vectors)
2. Design `vector(N)` data type in Rust
3. Prototype ALEX for 1536-dim vectors (100K-1M OpenAI embeddings)
4. Benchmark: memory usage, query latency, index build time
5. **Decision point**: Does ALEX work for vectors? Or pivot to HNSW-only?

**Customer validation** (parallel track):
- Find 50 companies using pgvector (GitHub search)
- Cold email: "Building pgvector that scales to 100M vectors"
- Target: 3-5 customer calls

---

## Key Commands

```bash
# Development
cargo build                      # Fast build
cargo test                       # 557 tests
cargo clippy                     # Lints

# Benchmarking
cargo build --release
./target/release/benchmark_vs_sqlite 10000000

# Servers
./target/release/postgres_server # Port 5433 (PostgreSQL wire protocol)
./target/release/rest_server     # Port 8080 (REST API)
```

---

## Repository Structure

```
omendb-server/
├── CONTEXT.md          # This file (load first!)
├── CLAUDE.md           # Full project context
├── ai/                 # Working context (TODO, STATUS, DECISIONS)
├── docs/               # Permanent documentation
├── src/                # Source code (557 tests passing)
│   ├── alex/           # Multi-level ALEX index (ready to adapt for vectors)
│   ├── postgres/       # PostgreSQL wire protocol
│   ├── mvcc/           # MVCC snapshot isolation (85 tests)
│   └── cache.rs        # LRU cache (90% hit rate)
└── tests/              # 557 tests (99.8% passing)
```

---

## Tech Stack

**Already Built** ✅:
- Multi-level ALEX index (100M+ rows validated)
- PostgreSQL wire protocol (pgvector compatibility foundation)
- MVCC snapshot isolation (85 tests)
- Auth + SSL/TLS (57 tests)
- LRU cache (2-3x speedup)
- RocksDB storage (LSM tree)

**To Build** 🔨 (Week 1-16):
- Vector data type `vector(N)`
- Distance operators (`<->`, `<#>`, `<=>`)
- ALEX/HNSW vector indexing
- Hybrid search (vector + SQL filters)

---

## Year 1 Focus

**omendb-server ONLY** (no omen-lite until Year 2+):
- Build cloud vector database first
- Self-hosting mode covers "embedded" use cases
- Validate product-market fit before expanding
- **Rationale**: Focus beats parallelization

---

## Performance Goals & Competitive Position

**vs pgvector**:
- 10x faster at 10M+ vectors
- 28x memory efficiency (<2GB for 10M 1536-dim vectors)
- Drop-in compatible (PostgreSQL wire protocol)

**vs Pinecone**:
- 90% cheaper ($99 vs $500/month for 10M vectors)
- Self-hostable (compliance-friendly)
- Source-available (Elastic License)
- PostgreSQL-compatible (no new API to learn)

---

## Development Workflow

1. **Check tasks**: `ai/TODO.md`
2. **Check status**: `ai/STATUS.md`
3. **Make changes**: Edit code, write tests
4. **Update docs**: Update `ai/TODO.md`, `ai/STATUS.md`, `ai/DECISIONS.md` (if architectural)
5. **Test**: `cargo test`
6. **Commit**: Frequent commits after each logical change

---

*Last Updated: October 22, 2025*
*Week 1 Priority: ALEX vector prototype + customer validation*
*Decision Point: End of Week 1 - Does ALEX work for vectors?*
