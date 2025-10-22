# OmenDB Development Context

**Last Updated**: October 21, 2025 (Night)

## Quick Start for AI Agents

**→ First time?** Load these in order:
1. This file (CLAUDE.md) - Project overview
2. `ai/TODO.md` - Current tasks
3. `ai/STATUS.md` - Current state
4. `ai/DECISIONS.md` - Key architectural decisions (when needed)

**→ Continuing work?** Check `ai/TODO.md` and `ai/STATUS.md` first

**→ Need organization guidance?** Reference [agent-contexts/PRACTICES.md](https://github.com/nijaru/agent-contexts/blob/main/PRACTICES.md) for:
- File update patterns (edit vs append)
- Context management strategies
- Multi-session handoff protocols
- Anti-patterns to avoid

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

**Repository Structure**:
```
omendb/core/
├── CLAUDE.md              # This file - AI agent entry point
├── ai/                    # AI working context (start here!) ⭐ NEW
│   ├── TODO.md            # Current tasks and priorities
│   ├── STATUS.md          # Current state (distilled from STATUS_REPORT)
│   ├── DECISIONS.md       # Architectural decisions with rationale
│   └── RESEARCH.md        # Research index with key findings
├── src/
│   ├── alex/              # Multi-level ALEX implementation
│   ├── postgres/          # PostgreSQL wire protocol + auth
│   ├── mvcc/              # MVCC snapshot isolation ✅
│   ├── cache.rs           # LRU cache layer ✅
│   ├── sql_engine.rs      # SQL: UPDATE/DELETE/JOIN + user management ✅
│   ├── catalog.rs         # Table + user management ✅ NEW
│   ├── user_store.rs      # Persistent user storage ✅ NEW
│   └── table.rs           # Table storage + ALEX + cache
├── internal/              # Permanent project documentation
│   ├── STATUS_REPORT.md   # Detailed status (reference, not daily use)
│   ├── research/          # Detailed research findings (26 docs)
│   ├── business/          # Business strategy, customer acquisition
│   ├── technical/         # Technical guides, MVCC design
│   ├── phases/            # Phase planning docs
│   └── PHASE_*_COMPLETE.md # Historical completion reports
└── tests/                 # 468 tests (all passing) ✅ NEW
    ├── user_store_tests.rs (11 tests) ✅ NEW
    ├── auth_tests.rs (6 tests) ✅ NEW
    ├── user_management_sql_tests.rs (15 tests) ✅ NEW
    ├── catalog_user_management_tests.rs (8 tests) ✅ NEW
    ├── cache_integration_tests.rs (7 tests) ✅
    ├── update_delete_tests.rs (30 tests) ✅
    └── join_tests.rs (14 tests) ✅
```

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

**We follow the [agent-contexts](https://github.com/nijaru/agent-contexts) pattern with existing structure respected:**

### ai/ - Agent Working Context (Start Here!)
*Evolving, AI-optimized working context - load this first!*

- **ai/TODO.md** - Current tasks and priorities (edit in place)
- **ai/STATUS.md** - Current state, what worked/didn't (edit in place, distilled from STATUS_REPORT)
- **ai/DECISIONS.md** - Architectural decisions with rationale (append-only)
- **ai/RESEARCH.md** - Research index with key findings (hybrid: update summaries, append new)

**Load order for new agents**:
1. CLAUDE.md (this file) - Project overview
2. ai/TODO.md - What to work on
3. ai/STATUS.md - Current state
4. ai/DECISIONS.md (if making architectural changes)

**Update every session**:
- ✅ Mark tasks complete in ai/TODO.md
- ✅ Update ai/STATUS.md with current state (edit in place, not append)
- ✅ Append new decisions to ai/DECISIONS.md
- ✅ Add research findings to ai/RESEARCH.md

### internal/ - Permanent Project Documentation (Existing Structure)
*Reference material - permanent, detailed, not for daily agent use*

**Note**: New projects would use `docs/` instead. We're respecting OmenDB's existing `internal/` structure.

- **internal/STATUS_REPORT.md** - Detailed status (reference only, use ai/STATUS.md for daily work)
- **internal/research/** - Detailed research findings (26 docs, permanent reference)
- **internal/business/** - Business strategy, customer acquisition
- **internal/technical/** - Technical guides, MVCC design, roadmaps
- **internal/phases/** - Phase planning documents
- **internal/PHASE_*_COMPLETE.md** - Historical completion reports

### docs/ - User-Facing Documentation
*For end users and contributors*

- **ARCHITECTURE.md** - System architecture
- **CONTRIBUTING.md** - Code guidelines
- **README.md** - Public project overview

### Knowledge Graduation Flow

```
Active work → ai/TODO.md
           ↓ (completed)
         ai/STATUS.md (what worked/didn't, edit in place)
           ↓ (important decision)
         ai/DECISIONS.md (working decision log)
           ↓ (if significant milestone)
         internal/PHASE_*_COMPLETE.md (historical record)
           ↓ (if permanent reference)
         internal/STATUS_REPORT.md (detailed, permanent)

Research → ai/research/{topic}.md
        ↓ (if valuable/permanent)
      internal/research/{topic}.md
        ↓ (if outdated)
      ai/research/archive/
```

### Anti-Patterns to Avoid

❌ **Don't duplicate between ai/ and internal/**
- Permanent findings → `internal/`
- Working context → `ai/`
- Don't copy the same content to both

❌ **Don't treat internal/ as working context**
- internal/ = permanent reference library
- ai/ = evolving scratchpad
- Load ai/ first, reference internal/ as needed

❌ **Don't append to ai/STATUS.md**
- Edit in place to reflect current truth
- Historical details stay in internal/STATUS_REPORT.md

❌ **Don't bloat ai/ files**
- Keep concise and current (optimize for tokens)
- Move old research to ai/research/archive/
- Archive completed work to internal/

## Development Principles

**Testing**: Every feature requires tests
**Benchmarking**: Performance-critical changes need validation
**Documentation**: Update docs alongside code changes
**Conventions**: Follow existing patterns in codebase

---

*Last Updated: October 21, 2025*

**Documentation reorganized following [agent-contexts](https://github.com/nijaru/agent-contexts) best practices**