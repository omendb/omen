# OmenDB AI Agent Context

**One Line**: Building PostgreSQL extension with learned indexes (ML replaces B-trees) for 10x faster lookups.

## Current Focus (Sept 25, 2025)

**What**: PostgreSQL extension ONLY (not standalone DB)
**Language**: Rust with pgrx
**Algorithm**: Linear RMI (Recursive Model Index)
**Timeline**: Oct 7 go/no-go, Nov 10 YC deadline
**Status**: Just pivoted, no code yet

## The Technical Approach

```rust
// B-tree: O(log n) with 20+ cache misses
btree.get(key) // 200ns

// Learned: O(1) with 2 cache misses
learned.predict_position(key) // 20ns goal, 40ns acceptable
```

**Key Insight**: Learn CDF of data, predict position directly

## Files You Need

```
internal/
├── ARCHITECTURE.md  # Technical design
├── STATUS.md        # Current progress
├── ROADMAP.md       # Deadlines
└── MONETIZATION.md  # Business model

external/papers/     # Research papers
external/learned-systems/ # Reference code
```

## Critical Success Metrics

- **Must achieve**: 10x faster than B-tree
- **Acceptable**: 5x with clear path to 10x
- **Deadline**: Oct 7 for go/no-go decision

## Implementation Priority

1. **Linear model on sorted array** (today)
2. **Benchmark vs BTreeMap** (tomorrow)
3. **PostgreSQL wrapper** (if 5x achieved)
4. **Demo video** (if 10x achieved)

## Code Pattern

```rust
// Start dead simple
struct LinearIndex {
    slope: f64,
    intercept: f64,
    data: Vec<(i64, Vec<u8>)>,
}

impl LinearIndex {
    fn lookup(&self, key: i64) -> Option<Vec<u8>> {
        let pos = (self.slope * key as f64 + self.intercept) as usize;
        // Binary search ±100 positions
        self.data[pos-100..pos+100].binary_search_by_key(&key, |k| k.0)
    }
}
```

## What NOT to Build

- ❌ Standalone database
- ❌ Embedded mode
- ❌ Server mode
- ❌ Complex neural networks
- ❌ Multi-dimensional indexes
- ❌ Distributed system

## Commands

```bash
# Create project
cargo new learned --lib
cargo add pgrx ndarray

# Test performance
cargo bench

# PostgreSQL extension
cargo pgrx init
cargo pgrx run
```

## If Asked About...

**Updates**: Delta buffer (like ALEX paper)
**Retraining**: Every 1M inserts initially
**Models**: Linear only, no neural nets yet
**Storage**: Memory-mapped sorted arrays
**Language**: Rust, not Mojo (we pivoted)

## The One Thing

**We must ship a PostgreSQL extension showing 10x faster lookups by Oct 7 or pivot.**

Everything else is noise.

---

*Load STATUS.md for current state, ARCHITECTURE.md for technical details.*