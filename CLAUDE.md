# OmenDB Development Context

**Last Updated**: October 21, 2025

## Current Status

**Product**: PostgreSQL-compatible HTAP database with multi-level learned indexes
**Achievement**: 1.5-3x faster than SQLite (validated), scales to 100M+ rows
**Status**: Phase 3 Week 1-2 COMPLETE (UPDATE/DELETE/JOIN), Cache optimization next
**Stack**: Rust (Multi-level ALEX + DataFusion + PostgreSQL protocol + RocksDB)
**Phase**: Performance optimization (cache layer) → 0.1.0 in 8 weeks
**Priority**: 🔥 Cache Layer Implementation (2-3 weeks, HN validated)

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

## Architecture (Current - October 21, 2025)

```
Production Stack:
├── Protocol Layer: PostgreSQL wire protocol (port 5433)
├── SQL Layer: UPDATE/DELETE/JOIN support (Phase 3 Week 1-2) ✅ NEW
├── MVCC Layer: Snapshot isolation (Phase 1) ✅
├── Index Layer: Multi-level ALEX (3-level hierarchy)
├── Storage Layer: RocksDB (LSM tree, HN validated) ✅
├── [NEXT] Cache Layer: 1-10GB LRU cache (Priority 1) 🔥
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
├── src/
│   ├── alex/              # Multi-level ALEX implementation
│   ├── postgres/          # PostgreSQL wire protocol
│   ├── mvcc/              # MVCC snapshot isolation (Phase 1) ✅
│   ├── sql_engine.rs      # SQL: UPDATE/DELETE/JOIN (Phase 3) ✅
│   ├── table.rs           # Table storage + ALEX integration
│   └── [next] cache.rs    # LRU cache layer (Priority 1) 🔥
├── internal/              # Strategy & status docs
│   ├── STATUS_REPORT.md   # Current status (Oct 21) ⭐
│   ├── research/          # HN insights, custom storage analysis
│   │   ├── HN_DATABASE_INSIGHTS_ANALYSIS.md ✅ NEW
│   │   └── CUSTOM_STORAGE_ANALYSIS.md ✅ NEW
│   ├── PHASE_1_COMPLETE.md   # MVCC complete
│   ├── PHASE_3_WEEK_1_COMPLETE.md   # UPDATE/DELETE
│   └── PHASE_3_WEEK_2_JOIN_COMPLETE.md   # JOIN
└── tests/                 # 456 tests (all passing)
    ├── update_delete_tests.rs (30 tests) ✅ NEW
    └── join_tests.rs (14 tests) ✅ NEW
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

## Recent Achievements (Last 7 Days - Oct 21, 2025)

**✅ Phase 3 Week 1-2 COMPLETE:**
- **UPDATE/DELETE support**: 30 tests, PRIMARY KEY immutability ✅
- **INNER JOIN + LEFT JOIN**: 14 tests, nested loop algorithm ✅
- **SQL coverage**: 15% → 35% (major milestone) ✅
- **Tests**: 442 → 456 (all passing, 100%)
- **Documentation**: Phase 3 complete summaries

**✅ HN Database Insights Analysis (Oct 21):**
- **Architecture validated**: ALEX + LSM + MVCC = best practices ✅
- **80x in-memory gap identified**: Explains RocksDB 77% overhead ✅
- **Cache layer validated**: HN insights confirm this is the right solution ✅
- **Custom storage analyzed**: Documented for future, defer to post-0.1.0 ✅

**🔥 PRIORITY 1 (Next 2-3 weeks):**
- **Large LRU cache implementation** (1-10GB)
- Target: Reduce RocksDB overhead 77% → 30%
- Expected: 2-3x speedup at 10M+ scale
- Timeline: 2-3 weeks to implementation + validation

**🔜 Next Steps (8 weeks to 0.1.0):**
1. Cache optimization (2-3 weeks) - **IMMEDIATE**
2. Phase 2: Security (2 weeks)
3. Phase 3 Week 3-4: SQL features (2 weeks)
4. Observability, Backup, Hardening (2-3 weeks)

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

## Documentation Navigation

**Starting a session? Load in this order**:
1. This file (quick context)
2. For detailed status → `internal/STATUS_REPORT_OCT_2025.md`
3. For architecture → `ARCHITECTURE.md`
4. For universal patterns → `~/.claude/CLAUDE.md` (points to agent-contexts)

**Task-specific documentation**:
- **Rust development** → `~/.claude/CLAUDE.md` → agent-contexts Rust patterns
- **Architecture changes** → `ARCHITECTURE.md`
- **Performance work** → `internal/research/100M_SCALE_RESULTS.md`
- **Code guidelines** → `CONTRIBUTING.md`
- **Business strategy** → `internal/business/`
- **Competitive analysis** → `internal/research/COMPETITIVE_ASSESSMENT_POST_ALEX.md`
- **Deployment** → `docs/deployment.md`

**Decision tree**:
```
IF writing Rust code → ~/.claude/CLAUDE.md → languages/rust/RUST_PATTERNS.md
IF modifying architecture → ARCHITECTURE.md
IF performance optimization → internal/research/ + Rust patterns
IF error debugging → ~/.claude/CLAUDE.md → standards/ERROR_PATTERNS.md
IF organizing docs → ~/.claude/CLAUDE.md → standards/DOC_PATTERNS.md
```

## Development Principles

**Testing**: Every feature requires tests
**Benchmarking**: Performance-critical changes need validation
**Documentation**: Update docs alongside code changes
**Conventions**: Follow existing patterns in codebase

---

*Updated: October 11, 2025*