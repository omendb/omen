# OmenDB Documentation

**Standard OSS Structure** (agent-contexts v0.1.1)

**Current Product**: omendb-server - PostgreSQL-compatible vector database (2025-2026)

**Product Roadmap**: Multi-database platform
- Phase 1: Vector database (current)
- Phase 2: Time series database (2026-2027)
- Phase 3: Graph database (2027-2028)
- Unified: Multi-model platform with shared infrastructure (2028+)

**Positioning**: PostgreSQL-Compatible Vector Database That Scales

---

## Directory Organization

```
docs/
├── README.md            # This file
├── QUICKSTART.md        # Getting started guide
├── ARCHITECTURE.md      # System architecture overview
├── PERFORMANCE.md       # Performance benchmarks
├── SECURITY.md          # Security documentation
├── OPERATIONS_GUIDE.md  # Operations and deployment
├── architecture/        # Technical deep-dives
│   ├── research/        # Research findings
│   ├── MULTI_LEVEL_ALEX.md
│   ├── ROADMAP_0.1.0.md
│   ├── ROCKSDB_PERFORMANCE_ANALYSIS_OCT_22.md
│   └── STORAGE_ENGINE_TEST_VALIDATION_OCT_22.md
├── strategy/            # Business strategy (private repository only)
│   ├── COMPETITIVE_STRATEGY_OCT_2025.md
│   ├── CUSTOMER_ACQUISITION.md
│   └── VC_FUNDING_STRATEGY_OCT_2025.md
├── archive/             # Historical documentation
│   ├── phases/          # Phase planning docs
│   ├── PHASE_*_COMPLETE.md
│   └── old-internal-archive/
└── deployment.md        # Deployment guides
```

---

## Documentation Types

### User Documentation
- **QUICKSTART.md** - Get started in 5 minutes
- **ARCHITECTURE.md** - System design overview
- **PERFORMANCE.md** - Benchmarks and performance characteristics
- **SECURITY.md** - Security features and best practices

### Technical Documentation
- **architecture/** - Deep-dives into system design
- **architecture/research/** - Research findings (26+ documents)
  - HTAP replication strategies
  - Learned index validation
  - Competitor analysis
  - Performance benchmarking

### Strategy Documentation (Private)
- **strategy/** - Business strategy and competitive analysis
  - Competitive positioning
  - Customer acquisition
  - Funding strategy

### Historical Documentation
- **archive/** - Phase completion reports, old implementations
  - Kept for reference, not actively maintained

---

## For Contributors

See `../CLAUDE.md` for AI agent workflow and `../ai/` for current project status.

**AI working context**: Load `ai/TODO.md` and `ai/STATUS.md` first

**Permanent docs**: Reference `docs/architecture/` for technical background

---

## Documentation Standards

- User-facing docs in root: QUICKSTART.md, ARCHITECTURE.md, etc.
- Technical deep-dives in architecture/
- Keep docs up-to-date with code changes
- Archive old docs, don't delete (git history preserves anyway)

---

*Last Updated: October 27, 2025*
*Pattern: Standard OSS structure (PostgreSQL, MongoDB, DuckDB, CockroachDB)*
*Current Phase*: Vector database validation (142 tests, Phase 2 60% complete)
