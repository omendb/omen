# OmenDB AI Agent Context

**Quick Load**: This file provides essential context for AI agents helping with OmenDB.

## What We're Building

**Product**: First production learned database system
**Innovation**: Replace B-trees (1979) with ML models (2025)
**Performance**: 10-100x faster lookups than traditional indexes
**Language**: Rust (not Mojo anymore - we pivoted Sept 25)
**Timeline**: YC application due Nov 10, 2025

## Current Status (Sept 25, 2025)

- **Just pivoted** from vector database to learned database
- **Starting** prototype implementation in Rust
- **Goal**: Working demo in 2 weeks
- **Blocker**: Need ML co-founder

## Technical Approach

```rust
// Traditional B-tree: O(log n) = 20 operations for 1M records
btree.get(key)  // 200ns, 20+ cache misses

// Learned index: O(1) = 2 operations regardless of size
learned.get(key)  // 20ns, 2 cache misses
```

**Key Insight**: Position = CDF(key) × N
- Learn data distribution with ML models
- Predict location instead of traversing

## Architecture Summary

1. **Recursive Model Index (RMI)**: Two-stage prediction
   - Root model: Predicts segment (neural net, 32KB)
   - Leaf models: Predicts position (linear regression)
   - Data pages: Sorted arrays with binary search

2. **Deployment Modes**:
   - PostgreSQL extension (priority 1)
   - Embedded library like SQLite (priority 2)
   - Standalone server (future)

3. **Update Strategy**:
   - Delta buffer for recent changes
   - Background retraining
   - Eventual consistency

## File Structure

```
internal/
├── ARCHITECTURE.md   # Technical design details
├── BUSINESS.md      # Strategy, market, pitch
├── ROADMAP.md       # Timeline, milestones
├── STATUS.md        # Current progress
└── CONTEXT.md       # This file (AI instructions)
```

## How to Help

### When Implementing Features
1. Check ARCHITECTURE.md for design
2. Follow Rust best practices
3. Optimize for cache efficiency
4. Benchmark everything

### When Answering Questions
1. We're building learned indexes, NOT vector search
2. Target is 10x performance, not incremental
3. PostgreSQL extension first, then embedded
4. YC deadline is Nov 10 (45 days)

### Code Style
```rust
// Use clear names
pub struct RecursiveModelIndex<K, V> { }

// Document complex algorithms
/// Predicts data location using learned CDF
fn predict_position(key: f64) -> usize { }

// Benchmark everything
#[bench]
fn bench_lookup(b: &mut Bencher) { }
```

## Current Priorities

### This Week (Sept 25-Oct 1)
1. Basic RMI implementation
2. Linear models only
3. Prove 5x speedup
4. PostgreSQL skeleton

### This Month (October)
1. Full RMI with error bounds
2. PostgreSQL extension working
3. Demo video for YC
4. Find ML co-founder

## Important Context

### What Changed (Sept 25)
- **From**: Vector database in Mojo (30+ competitors)
- **To**: Learned database in Rust (0 competitors)
- **Why**: Greenfield opportunity, better economics

### Technical Decisions Made
- **Language**: Rust (not Mojo - too immature)
- **First Model**: Linear regression (simple, fast)
- **Storage**: Memory-mapped files
- **Updates**: Delta buffer approach

### Open Questions
1. Optimal retraining frequency?
2. Handling adversarial patterns?
3. Multi-dimensional indexes?
4. GPU worth it for inference?

## Key Metrics

### Performance Targets
- Point lookup: <20ns (vs 200ns B-tree)
- Memory usage: <2% of data size
- Training time: <100ms
- Correctness: 100%

### Business Targets
- GitHub stars: 100+ by Nov 1
- Working demo: Oct 15
- YC application: Nov 1
- First user: Nov 30

## Common Commands

```bash
# Build and test
cargo build --release
cargo test
cargo bench

# PostgreSQL extension
cargo pgrx run
cargo pgrx test

# Benchmarking
cargo run --release --bin benchmark

# Quick demo
cargo run --example demo
```

## Research References

Essential papers:
1. "The Case for Learned Index Structures" (2018)
2. "SOSD: A Benchmark for Learned Indexes" (2021)
3. "RadixSpline: A Single-Pass Learned Index" (2020)

## Don't Forget

1. **We pivoted** - Old vector DB code is archived
2. **YC deadline** - Nov 10, not Oct 15
3. **Performance** - 10x minimum, not incremental
4. **Simplicity** - PostgreSQL extension for adoption
5. **Competition** - We have none (yet)

## For Quick Context

If you need to understand one thing:
> We're replacing 45-year-old B-trees with ML models that learn your data distribution and predict where records are in 1-2 CPU cycles instead of 20+ tree traversals.

---

*Load ARCHITECTURE.md for technical details, BUSINESS.md for strategy, ROADMAP.md for timeline.*