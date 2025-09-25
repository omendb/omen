# OmenDB - Learned Database Development Context

## 🎯 Current Focus: PostgreSQL Extension with Learned Indexes

**Pivot Date**: September 25, 2025
**Status**: ✅ Major breakthrough achieved - 7.89x speedup on Day 1
**Timeline**: Oct 7 go/no-go, Nov 10 YC deadline
**Success Probability**: 75-80% (upgraded from 30%)

## What We're Building

**The world's first production learned database** - PostgreSQL extension that uses machine learning models instead of B-trees for 10x faster lookups.

```sql
-- Traditional B-tree (200ns lookup)
CREATE INDEX users_id_idx ON users(id);

-- Our learned index (20ns lookup target)
CREATE INDEX users_id_learned ON users USING learned(id);
SELECT * FROM users WHERE id = 12345; -- 10x faster
```

## Current Status (September 25, 2025)

### ✅ Completed Milestones
- **LinearIndex implementation**: 7.89x speedup vs BTreeMap (100K keys)
- **PostgreSQL extension**: Built with pgrx, SQL functions working
- **Repository cleanup**: Professional structure, 90% smaller
- **Performance validation**: 3.3x-7.9x across all dataset sizes
- **Range queries**: Up to 16x speedup

### 🚧 Next Priorities
1. **PostgreSQL integration testing** (resolve pgrx linker issues)
2. **RMI implementation** for 10x performance target
3. **CREATE INDEX USING learned** syntax
4. **Performance benchmarks** vs PostgreSQL B-tree

## Repository Structure

```
omendb/core/
├── src/                     # Rust learned index implementation
│   ├── lib.rs              # Core traits and LinearIndex
│   ├── linear.rs           # Linear regression implementation
│   └── bin/benchmark.rs    # Performance benchmarks
├── pgrx-extension/         # PostgreSQL extension
│   ├── src/lib.rs          # SQL functions and integration
│   └── Cargo.toml          # Extension configuration
├── internal/               # Strategic documentation
│   ├── STATUS.md           # Current progress (update often)
│   ├── ARCHITECTURE.md     # Technical design
│   ├── BUSINESS.md         # Market analysis
│   └── BACKUP_PIVOTS.md    # Alternative strategies
└── external/               # Research papers and references
```

## Key Implementation Files

**Core Algorithm**: `src/linear.rs:LinearIndex` - Working 7.89x speedup
**PostgreSQL Extension**: `pgrx-extension/src/lib.rs` - SQL integration
**Benchmarks**: `src/bin/benchmark.rs` - Performance validation
**Current Status**: `internal/STATUS.md` - Keep updated after changes

## Development Guidelines

### 🔄 Commit & Documentation Rules
- **Commit frequently**: After each logical unit of work
- **Update STATUS.md**: After performance changes or milestones
- **Keep repo clean**: Delete temp files, test artifacts
- **No dead code**: Remove unused functions/files immediately
- **Performance claims**: Always verify with actual benchmarks

### 🧪 Testing Requirements
```bash
# Before any commit, ensure these pass:
cargo build --release                    # Must compile
cargo test                              # All tests pass
cargo run --bin benchmark --release     # Performance check
```

### 📊 Performance Standards
- **Minimum target**: 3x speedup vs BTreeMap (✅ achieved)
- **Stretch target**: 10x speedup (need RMI implementation)
- **PostgreSQL target**: 5-7x net speedup (accounting for overhead)
- **Benchmark format**: Always report exact numbers, not approximations

## PostgreSQL Extension

### Current Functions
```sql
SELECT hello_omendb();                           -- Test connection
SELECT create_learned_index('idx', 'table', 'col'); -- Create index
SELECT lookup_learned_index('idx', 42);         -- Lookup value
SELECT benchmark_learned_vs_btree(10000);       -- Performance test
```

### Next Steps
```sql
-- Target syntax (not yet implemented):
CREATE EXTENSION omendb_learned;
CREATE INDEX users_learned_idx ON users USING learned(id);
```

## Decision Framework

### Oct 7 Go/No-Go Criteria
- **Must achieve**: 5-10x demo with PostgreSQL extension
- **Must have**: CREATE INDEX syntax working
- **Nice to have**: RMI implementation showing 10x+

### If Learned DB Fails
Backup pivots documented in `internal/BACKUP_PIVOTS.md`:
1. **Multimodal Database** (strongest alternative)
2. **Inference Database**
3. **Time Series + Vector Hybrid**

## Current Challenges

### Technical Issues
- **pgrx linker errors**: PostgreSQL symbols not found (architecture mismatch)
- **Need RMI**: Linear model hits ~8x ceiling, need recursive models for 10x
- **CREATE INDEX**: Need proper PostgreSQL index AM integration

### Strategic Risks
- **Timeline pressure**: 12 days to go/no-go decision
- **Competition risk**: Google/others could enter market
- **Technical risk**: PostgreSQL overhead might limit performance gains

## Quick Commands

### Development
```bash
# Core development
cargo run --bin benchmark --release    # Test performance
cargo build && cargo test             # Validate code

# PostgreSQL extension
cd pgrx-extension && cargo build      # Build extension
cargo pgrx run pg14                   # Test with PostgreSQL

# Repository maintenance
git add -A && git commit -m "type: description"
git status && git log --oneline -10
```

### Performance Investigation
```bash
# Generate performance data
cargo run --bin benchmark --release > results.txt
grep "Speedup" results.txt

# Check code quality
cargo fmt && cargo clippy
find . -name "*.tmp" -delete  # Clean temp files
```

## Research Foundation

**Core Papers**: External/papers/ (to be organized)
- "The Case for Learned Index Structures" (Kraska et al., 2018)
- "From WiscKey to Bourbon" (Dai et al., 2020)
- "XIndex" (Tang et al., 2020)

**Competitive Analysis**: Zero production learned databases exist
**Market Timing**: Perfect - PostgreSQL ecosystem + AI momentum

## Success Metrics

| Metric | Current | Target | Status | Deadline |
|--------|---------|--------|--------|----------|
| Pure Rust Speedup | 7.89x | 3x | ✅ | Oct 7 |
| PostgreSQL Demo | 80% | Working | 🚧 | Sept 30 |
| CREATE INDEX Syntax | 0% | Working | 📅 | Oct 5 |
| RMI Implementation | 0% | 10x speedup | 📅 | Oct 7 |

## Contact & Context

**Developer**: Nick Russo (nijaru7@gmail.com)
**Previous Work**: Vector database (archived to engine-legacy/)
**Pivot Reason**: Learned DBs have zero competition vs 30+ vector DB competitors
**YC Application**: Nov 10, 2025

---

## AI Agent Instructions

**Load these docs first**: STATUS.md (current state), ARCHITECTURE.md (tech details)
**Update frequently**: STATUS.md after any performance changes
**Commit pattern**: "type: description" (feat, fix, perf, docs)
**Before suggesting**: Check if we've tried it (git log, docs)
**Performance claims**: Always measure, report exact numbers
**Keep focused**: Learned databases only, avoid vector DB tangents

**The One Thing**: Ship PostgreSQL extension showing 5-10x speedup by Oct 7 or pivot to backup strategy.

*Last Updated: September 25, 2025 - Day 1 success with 7.89x speedup ✅*