# OmenDB Development Context

**Last Updated**: October 21, 2025 (Night)

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

## Current Status

**Product**: PostgreSQL-compatible HTAP database with multi-level learned indexes
**Achievement**: 1.5-3x faster than SQLite (validated), scales to 100M+ rows
**Status**: Phase 2 Security Days 1-5 COMPLETE → Days 6-7 (SSL/TLS) next
**Stack**: Rust (Multi-level ALEX + DataFusion + PostgreSQL protocol + RocksDB + LRU cache + MVCC)
**Phase**: Security implementation (Days 6-10) → 0.1.0 in 7 weeks
**Priority**: 🔒 SSL/TLS for PostgreSQL wire protocol (Days 6-7)

## Technical Core

**Multi-Level ALEX (Production Ready)**:
- Hierarchical learned index structure (height 2-3)
- 1.5-3x faster than SQLite across all scales (1M-100M)
- 1.24μs query latency at 100M rows
- 1.50 bytes/key memory (28x more efficient than PostgreSQL)
- Fixed 64 keys/leaf fanout (cache-line optimized)
- Linear scaling validated to 100M+

**Market Position**:
- **vs SQLite**: 1.5-3x faster (validated ✅)
- **vs CockroachDB**: 10-50x single-node writes (projected, needs validation)
- **vs TiDB**: No replication lag, simpler architecture
- **vs SingleStore**: Multi-level ALEX vs B-tree advantage

## Architecture (Current - October 21, 2025 Late Evening)

```
Production Stack:
├── Protocol Layer: PostgreSQL wire protocol (port 5433)
├── SQL Layer: UPDATE/DELETE/JOIN support (Phase 3 Week 1-2) ✅
├── MVCC Layer: Snapshot isolation (Phase 1) ✅
├── Index Layer: Multi-level ALEX (3-level hierarchy)
├── Cache Layer: 1-10GB LRU cache (Day 1-5 complete) ✅ NEW
├── Storage Layer: RocksDB (LSM tree, HN validated) ✅
└── Recovery: 100% crash recovery success
```

**Architecture Validation (HN Insights)**:
- ALEX (sparse learned index): Validated by DB fundamentals ✅
- RocksDB (LSM tree): Industry-proven (DynamoDB, Cassandra) ✅
- MVCC (immutable records): Best practice (append-only) ✅
- Cache layer: Addresses 80x in-memory gap (HN validated) ✅

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
│   │   ├── ROADMAP_0.1.0.md
│   │   ├── ROCKSDB_PERFORMANCE_ANALYSIS_OCT_22.md
│   │   └── STORAGE_ENGINE_TEST_VALIDATION_OCT_22.md
│   ├── strategy/          # Business strategy (private repo only)
│   │   ├── COMPETITIVE_STRATEGY_OCT_2025.md
│   │   └── CUSTOMER_ACQUISITION.md
│   └── archive/           # Historical documentation
│       ├── phases/        # Phase planning docs
│       └── PHASE_*_COMPLETE.md
├── ai/                    # AI working context ⭐
│   ├── TODO.md            # Current tasks (edit in place)
│   ├── STATUS.md          # Current state (edit in place)
│   ├── DECISIONS.md       # Working decision log (append-only)
│   └── RESEARCH.md        # Research index (hybrid)
├── src/                   # Source code
│   ├── alex/              # Multi-level ALEX implementation
│   ├── postgres/          # PostgreSQL wire protocol + auth
│   ├── mvcc/              # MVCC snapshot isolation ✅
│   ├── cache.rs           # LRU cache layer ✅
│   ├── sql_engine.rs      # SQL: UPDATE/DELETE/JOIN + user mgmt ✅
│   ├── catalog.rs         # Table + user management ✅
│   ├── user_store.rs      # Persistent user storage ✅
│   └── table.rs           # Table storage + ALEX + cache
└── tests/                 # 520 tests (99.8% passing) ✅
```

**Pattern**: Standard OSS database structure (like PostgreSQL, MongoDB, DuckDB, CockroachDB)
- **docs/** — All permanent documentation (user guides, architecture, research)
- **ai/** — AI working context (tasks, status, decisions, research notes)

## Validated Competitive Advantages

1. **1.5-3x Faster**: Validated vs SQLite at 1M-100M scale ✅
2. **28x Memory Efficient**: 1.50 bytes/key vs PostgreSQL's 42 bytes/key ✅
3. **Linear Scaling**: Multi-level ALEX scales to 100M+ ✅
4. **PostgreSQL Compatible**: Wire protocol complete, drop-in ready ✅
5. **Production Durability**: 100% crash recovery success ✅

## Validated Performance (October 14, 2025)

**Full System (RocksDB + ALEX) - Honest Benchmarks:**

| Scale | Speedup (Sequential) | Speedup (Random) | Status |
|-------|---------------------|------------------|--------|
| 10K   | 3.54x ✅            | 3.24x ✅         | Production-ready |
| 100K  | 3.15x ✅            | 2.69x ✅         | Production-ready |
| 1M    | 2.40x ✅            | 2.40x ✅         | Production-ready |
| 10M   | 1.93x ⚠️            | 1.53x ✅         | Optimization ongoing |

**ALEX Isolated (for reference):**
- 1-10M: 628ns, 25M: 1.1μs, 50M: 984ns, 100M: 1.24μs (memory: 1.50 bytes/key)

**Key Findings (Oct 14):**
- Small/medium scale (10K-1M): 2.4-3.5x faster ✅ Excellent
- Large scale (10M): 1.9x faster ⚠️ Needs optimization
- Bottleneck identified: RocksDB (77%), not ALEX (21%)
- Path forward: Large cache + tuning (2-3 weeks to 2x target)

## Recent Achievements (Oct 21, 2025)

**✅ Phase 2 Security Days 1-5 COMPLETE:** ⭐ NEW
- **Day 1**: UserStore with RocksDB persistence (11 tests) ✅
- **Day 2**: OmenDbAuthSource integration (6 tests) ✅
- **Day 3-4**: SQL user management - CREATE/DROP/ALTER USER (15 tests) ✅
- **Day 5**: Catalog integration with default admin user (8 tests) ✅
- **Total**: 40/40 security tests passing, persistent authentication system
- **Timeline**: On schedule (5 days), Days 6-10 remaining

**✅ Cache Layer Days 1-10 COMPLETE:**
- **LRU cache**: 1-10GB configurable, 2-3x speedup validated ✅
- **90% hit rate** with Zipfian workloads ✅
- **Optimal cache size**: 1-10% of data (not 50%) ✅
- **Tests**: 7 cache integration tests passing ✅

**✅ Phase 3 Week 1-2 COMPLETE:**
- **UPDATE/DELETE support**: 30 tests, PRIMARY KEY immutability ✅
- **INNER JOIN + LEFT JOIN**: 14 tests, nested loop algorithm ✅
- **SQL coverage**: 15% → 35% ✅

**✅ Phase 1 MVCC COMPLETE:**
- **Snapshot isolation**: Production-ready, 85 tests (62 unit + 23 integration) ✅
- **7% ahead of schedule**: 14 days vs planned 15 ✅

**🔒 CURRENT PRIORITY (Days 6-10):**
- **Days 6-7**: SSL/TLS for PostgreSQL wire protocol - **NEXT**
- **Day 8**: Security integration tests (target: 50+ total tests)
- **Day 9**: Security documentation (SECURITY.md, deployment guides)
- **Day 10**: Final validation & security audit

**🔜 Next Steps (7 weeks to 0.1.0):**
1. Phase 2 Security Days 6-10 (5 days) - **IN PROGRESS**
2. Phase 3 Week 3-4: SQL features (aggregations, subqueries) - 2 weeks
3. Observability, Backup, Hardening - 3-4 weeks

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

**Hardware Details**:
- Fedora: i9-13900KF (8P + 16E cores), 32GB DDR5, RTX 4090, NVMe SSD
- Mac: M3 Max (~16 cores), 128GB RAM, Tailscale network

**Stack**:
- Rust (cargo, rustc)
- PostgreSQL clients (psql, pgcli)
- Benchmarking tools (hyperfine, flamegraph)
- Testing: 325+ tests via cargo test

## Common Commands

**Development:**
```bash
cargo build                      # Fast, unoptimized
cargo test                       # All tests
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

## Documentation Organization

**Standard OSS Pattern** (agent-contexts v0.1.1):

### Quick Reference

| Directory | Purpose | Update Pattern |
|-----------|---------|----------------|
| **ai/** | AI working context | Frequent, evolving (edit in place) |
| **docs/** | All permanent documentation | Versioned, deliberate |

### Workflow for AI Agents

**Load on session start**:
1. CLAUDE.md (this file) → Project overview
2. ai/TODO.md → Current tasks
3. ai/STATUS.md → Current state
4. ai/DECISIONS.md → Architectural context (if needed)

**Update every session**:
- ✅ ai/TODO.md (mark complete, add new tasks)
- ✅ ai/STATUS.md (edit in place, current truth)
- ✅ ai/DECISIONS.md (append new decisions)
- ✅ ai/RESEARCH.md (add findings)

**Graduate to docs/ when**:
- Research complete and valuable for contributors
- Technical deep-dive worth preserving
- Architecture decisions worth documenting

### Key Principles

✅ **ai/** = Working scratchpad (concise, current, <2K words per file)
✅ **docs/** = Permanent knowledge (detailed, versioned, no size limit)
✅ **Edit in place**: ai/STATUS.md, ai/TODO.md
✅ **Append-only**: ai/DECISIONS.md

❌ **Don't** duplicate content between ai/ and docs/
❌ **Don't** append to ai/STATUS.md (edit in place for current truth)
❌ **Don't** bloat ai/ files (archive old content to docs/archive/)

## Development Principles

**Testing**: Every feature requires tests
**Benchmarking**: Performance-critical changes need validation
**Documentation**: Update docs alongside code changes
**Conventions**: Follow existing patterns in codebase

---

*Last Updated: October 22, 2025*

**Documentation**: Standard OSS structure (agent-contexts v0.1.1) - docs/ + ai/ only
**Cleanup**: Migrated from `internal/` to standard `docs/` structure (PostgreSQL, MongoDB, DuckDB pattern)