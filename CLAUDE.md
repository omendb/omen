# OmenDB Core Development Context

## 🎯 Strategy: Open Source Extension + Proprietary Database

**Date**: September 26, 2025
**Mission**: Build state-of-the-art learned database using latest research
**Business Model**: PostgreSQL extension (free tier) → Full database (paid SaaS)

## Repository Structure (CORRECT)

```
omendb/ (GitHub Organization)
├── pg-learned/        # PUBLIC - PostgreSQL extension (marketing/free tier)
├── website/           # PRIVATE - Marketing site at omendb.io
└── core/              # PRIVATE - THIS REPO - Learned DB development
```

### THIS Repository (core)
```
core/
├── src/               # Core learned index algorithms (Rust)
│   ├── linear.rs      # LinearIndex implementation
│   ├── rmi.rs         # RMI (Recursive Model Index)
│   └── lib.rs         # Library exports
├── learneddb/         # Standalone database foundation
│   ├── src/           # Database implementation
│   └── examples/      # Demo code
├── docs/              # Internal documentation
│   └── internal/      # Strategy, roadmap, research
└── website/           # IGNORE - moved to separate repo
```

## Key Facts to Remember

### Repositories
- **pg-learned**: Public PostgreSQL extension (NOT postgresql-extension)
- **website**: Separate private repo, deployed to Cloudflare Pages
- **core**: This repo - where we build the full learned database

### Development Strategy
1. **pg-learned extension**: Open source marketing tool (already exists)
2. **Learned database**: Build here in core using Rust + latest research
3. **Website**: Marketing site, evolves to SaaS platform later

## Key Documentation (Load in Order)

1. **`docs/internal/STATUS.md`** - Current performance metrics
2. **`docs/internal/BUSINESS.md`** - Market strategy and analysis
3. **`docs/internal/ARCHITECTURE.md`** - Technical design
4. **`docs/website/WEBSITE_STRATEGY.md`** - Launch strategy

## Development Workflow

### Build Commands
```bash
# Core library (learned indexes)
cargo build --release && cargo test

# Standalone database (in progress)
cd learneddb && cargo run --example demo

# PostgreSQL extension (separate repo)
cd ../pg-learned && cargo pgrx run

# Website (separate repo)
cd ../website && npm run build
```

### Testing Commands
```bash
# Performance validation
cargo run --bin benchmark --release

# Extension safety
# (Extension is stable - no crashes expected)

# Website deployment
cd apps/website && npm run preview
```

## Current Performance

### Achieved Results
- **LinearIndex**: 2-8x speedup vs BTreeMap
- **RMI**: 2-4x speedup (working implementation)
- **Range queries**: Up to 16x improvement
- **Bulk insertion**: 10K records in 19ms

### PostgreSQL Extension (pg-learned repo)
```sql
-- Functions in PUBLIC pg-learned repo:
SELECT learned_index_version();           -- Extension info
SELECT learned_index_benchmark(10000);    -- Performance demo
SELECT learned_index_info();              -- Educational content
```

### Learned Database Plan (THIS repo)
- Use learneddb/ as foundation
- Add persistence, query engine, wire protocol
- Incorporate latest research (RadixSpline, PGM-Index, etc.)
- Target: PostgreSQL wire compatible learned database

## Launch Strategy

### Phase 1: Market Validation
1. **Launch website** on omendb.io (GitHub Pages ready)
2. **Post on HackerNews** with blog post about 10x speedup
3. **Measure response** (target: 500+ GitHub stars)
4. **Collect early access signups** for DBaaS beta

### Phase 2: Product Development
If market validates (500+ stars):
1. **Build standalone database** with PostgreSQL wire protocol
2. **Enhance learned indexes** with state-of-the-art algorithms
3. **Launch DBaaS** (database-as-a-service)

### Phase 3: Scale
1. **Proprietary optimizations** (CXL memory, GPU acceleration)
2. **Enterprise features** (multi-region, compliance)
3. **VC funding** or continued bootstrapping

## Technical Approach

### Current: Proven Technology
- **RocksDB storage** (battle-tested)
- **Linear and RMI indexes** (research-backed)
- **Rust implementation** (memory safe, fast)

### Future: State-of-the-Art
- **CXL memory disaggregation** (100x memory capacity)
- **LSM-tree ML optimizations** (intelligent compaction)
- **GPU acceleration** (parallel model training)

## Market Position

### Competitive Advantages
1. **First to market** - no production learned databases exist
2. **PostgreSQL ecosystem** - 40% of all databases
3. **Proven performance** - demonstrable 2-10x speedup
4. **Solo developer viable** - focused scope, proven tech

### Target Market
- **Time-series databases** (financial, IoT, metrics)
- **Real-time analytics** (e-commerce, trading)
- **Any high-read workload** with ordered data

## Success Metrics

### Launch (Week 1)
- 100+ GitHub stars
- 50+ email signups
- 10+ PostgreSQL extension installs

### Validation (Month 1)
- 500+ GitHub stars
- 200+ email signups
- 5+ production use cases

### Scale Decision (Month 3)
- 1000+ GitHub stars
- 500+ email signups
- Clear demand → Build DBaaS

## Git Workflow

### Commit Guidelines
- **Format**: `type: description` (feat, fix, docs, perf)
- **Scope**: Component affected (website, extension, core)
- **Atomic**: One logical change per commit

### Documentation Updates
- **Always update** `docs/internal/STATUS.md` after performance changes
- **Keep current** all documentation with code changes
- **No dead docs** - delete outdated files immediately

## AI Agent Guidelines

### Context Loading
1. Read `docs/internal/` first for current state
2. Check `apps/website/` for launch readiness
3. Review recent commits for context

### Decision Making
- **Performance claims**: Always measure and verify
- **Code changes**: Preserve existing functionality
- **Documentation**: Update inline with changes

### Task Prioritization
1. **Launch blockers** - anything preventing omendb.io launch
2. **Performance regressions** - maintain 2-8x speedup
3. **User experience** - website, documentation, extension safety

## Contact & Business

**Developer**: Nick Russo (nijaru7@gmail.com)
**Domain**: omendb.io (ready for website)
**Strategy**: Validate market → Build DBaaS → Scale
**Timeline**: Launch now, iterate based on response

---

## Current State Summary

**What's Working**:
- Stable PostgreSQL extension (no crashes)
- Production website ready for launch
- Learned indexes showing 2-8x speedup
- Standalone database foundation

**What's Next**:
- Launch website and measure market response
- Enhance standalone database based on feedback
- Scale or pivot based on validation

**The Mission**: Build the first production learned database that's 10x faster than PostgreSQL, starting with a strong PostgreSQL extension and evolving to a full replacement.

*Last Updated: September 26, 2025 - Launch ready 🚀*