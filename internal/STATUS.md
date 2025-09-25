# OmenDB Current Status

## 🎯 FOCUS: PostgreSQL Extension with Learned Indexes

**Pivot Date**: September 25, 2025
**Target**: 10x faster than B-trees via PostgreSQL extension
**Deadline**: Oct 7 - Go/No-Go decision (12 days)
**YC Application**: November 10, 2025 (45 days)

---

**Last Updated**: September 25, 2025 (16:00 PST)

## Today's Progress ✅

### Completed (Sept 25)
- ✅ Strategic pivot from vector DB to learned indexes
- ✅ Merged pivot branch to main (committed to new direction)
- ✅ Cleaned up old branches (codex, mojo-25.6)
- ✅ Documentation consolidated to 5 core files
- ✅ Research papers organized (external/papers/)
- ✅ Clear monetization strategy defined

### Reality Check 🎯

**Honest Success Probability**: 30-40%
- **10x performance**: Possible but challenging with PG overhead
- **More likely**: 3-5x initially (still valuable!)
- **Timeline**: Aggressive but achievable if we start NOW

**Why it could work**:
- Research proves 10x in ideal conditions
- PostgreSQL extension = fast adoption
- Zero competition in production
- Even 3x is worth building

**Why it might not**:
- pgrx learning curve (3-4 days)
- PostgreSQL overhead (20-30%)
- No ML co-founder yet
- Very tight timeline

## Next 24 Hours (Critical)

### Must Do (Sept 26)
1. [ ] Create Rust project with pgrx
2. [ ] Implement simplest possible linear model
3. [ ] Benchmark against BTreeMap
4. [ ] Achieve 3x or re-evaluate

### Commands to Run
```bash
# Create project
cargo new omendb-learned --lib
cd omendb-learned
cargo add pgrx ndarray criterion

# Simple linear index
# Just 100 lines of code
# If not 3x faster by end of day, pivot
```

## The Hard Truth

**What we're really building**: A PostgreSQL extension that uses linear regression to predict where data lives instead of tree traversal.

**Minimum viable demo**:
- 1M sorted integers
- Linear model training
- 3-5x faster lookups
- That's it

**If we can't achieve 3x in 48 hours**, learned indexes aren't the answer.

## Decision Points

### Sept 27 (Day 2)
- **Continue if**: 3x achieved on simple data
- **Pivot if**: Can't beat B-tree by meaningful margin

### Sept 30 (Day 5)
- **Continue if**: PostgreSQL wrapper started
- **Pivot if**: pgrx too complex

### Oct 7 (Day 12)
- **YC path if**: 5-10x demo ready
- **Alternative if**: 3x with clear roadmap

## Current Blockers

1. **No code written yet** (starting today)
2. **No co-founder** (post on HN today)
3. **Unknown pgrx complexity** (will know by day 5)

## Metrics That Matter

| Metric | Current | Target | Deadline |
|--------|---------|--------|----------|
| Lookup Speed | 0 (no code) | 40ns | Oct 7 |
| vs B-tree | N/A | 5-10x | Oct 7 |
| Lines of Code | 0 | <1000 | Sept 30 |
| GitHub Stars | 0 | 50+ | Oct 15 |

## The One Thing

**By end of tomorrow (Sept 26), we must have a working linear index that's 3x faster than BTreeMap, or we pivot.**

Everything else is secondary.

---

*"Stop planning. Start coding. 12 days to prove it works."*