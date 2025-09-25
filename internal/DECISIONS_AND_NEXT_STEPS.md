# Critical Decisions & Next Steps

**Date**: September 25, 2025
**YC Deadline**: November 10, 2025 (45 days - you were right!)

## Decisions Made ✅

### 1. Architecture: Yes to Both Embedded & Server
Like DuckDB, we'll support multiple deployment modes:
- **PostgreSQL Extension** (first, fastest adoption)
- **Embedded Library** (compete with SQLite/DuckDB)
- **Server Mode** (future, after traction)

### 2. Documentation Structure (Optimized for AI)
Consolidated from 15+ files to 5 core files:
- `ARCHITECTURE.md` - Technical truth
- `BUSINESS.md` - Strategy & pitch
- `ROADMAP.md` - Timeline & tasks
- `STATUS.md` - Daily progress
- `CONTEXT.md` - AI agent instructions

### 3. External Cleanup Complete
- Removed: diskann, modular, competitors, benchmarks
- Kept: agent-contexts (essential)
- Todo: Add learned-systems/RMI, SOSD when starting

### 4. Business Structure
Created `internal/BUSINESS.md` with:
- Market analysis (zero competitors!)
- YC pitch strategy
- Revenue model
- Exit scenarios

## What Still Needs Research 🔍

### Before Starting Implementation

**1. Update Handling Strategy**
```bash
# Critical unknown - how to handle inserts efficiently?
- Read Google ALEX paper (updatable learned index)
- Study LSM tree approaches
- Consider hybrid model (learned + small B-tree)
```

**2. Storage Format Decision**
```bash
# Apache Arrow vs Custom?
- Arrow: Great interop, existing tooling
- Custom: Optimized for learned access patterns
- Decision: Start with Arrow, optimize later
```

**3. Retraining Triggers**
```bash
# When to retrain models?
- Every N inserts?
- When error bounds exceeded?
- Time-based (hourly/daily)?
- Research: Read "Adaptive Learned Indexes" paper
```

## Immediate Next Steps (Do Now)

### 1. Create Rust Project (30 min)
```bash
cd /Users/nick/github/omendb/core
cargo new omendb-learned --lib
cd omendb-learned

# Add dependencies
cargo add ndarray candle-core pgrx criterion rayon

# Create structure
mkdir -p src/{models,storage,index,pgext}
touch src/models/linear.rs
touch src/index/rmi.rs
touch src/storage/page.rs
```

### 2. Read Critical Papers (2 hours)
Priority order:
1. "The Case for Learned Index Structures" - Core RMI algorithm
2. "ALEX: An Updatable Learned Index" - Update handling
3. "RadixSpline" - Simpler alternative if RMI complex

### 3. Write First Prototype (2 hours)
```rust
// Start dead simple - linear model only
pub struct SimpleLearnedIndex {
    model: (f64, f64), // slope, intercept
    data: Vec<(i64, Vec<u8>)>,
    error_bound: i32,
}

impl SimpleLearnedIndex {
    pub fn train(data: Vec<(i64, Vec<u8>)>) -> Self {
        // Linear regression on CDF
    }

    pub fn get(&self, key: i64) -> Option<Vec<u8>> {
        let predicted = (self.model.0 * key as f64 + self.model.1) as usize;
        // Binary search around predicted position
    }
}
```

### 4. Post to Hacker News (15 min)
```
Title: Show HN: Building first production learned database (10x faster than B-trees)

After discovering 30+ vector DB competitors but ZERO learned database
implementations, I'm pivoting. Learned indexes use ML to predict where
data lives instead of tree traversal - proven 10x faster in research
but never productized.

Looking for ML co-founder. Starting with PostgreSQL extension.

GitHub: [link]
Discord: [link]
```

## Critical Path to YC (45 Days)

### Week 1 (Sept 25 - Oct 1)
**Must achieve**: 5x performance proof
- Linear RMI working
- Basic benchmarks
- PostgreSQL skeleton

### Week 2 (Oct 2-8)
**Must achieve**: Demo-ready
- 10x performance
- PostgreSQL CREATE INDEX working
- Video script ready

### Week 3-4 (Oct 9-22)
**Must achieve**: YC application
- Demo video recorded
- Application written
- GitHub public with docs

### Week 5-6 (Oct 23-Nov 5)
**Must achieve**: Traction
- 100+ GitHub stars
- 3+ users interested
- Submit early (Nov 1)

## Architecture We Need (Not Decided Yet)

### Query Processing
```
Do we need full SQL or just key-value initially?
- Start with key-value (simpler)
- Add SQL in Phase 2
- Focus on index performance first
```

### Concurrency Model
```
MVCC or simpler locking?
- Start with read-heavy optimization
- Add MVCC later if needed
- Don't over-engineer early
```

### Distribution Strategy
```
Single-node or plan for sharding?
- Single-node first (6 months)
- Design with sharding in mind
- Don't build until needed
```

## Why This Will Work

1. **Timing Perfect**: Research done, no competition
2. **Clear Path**: PostgreSQL → Embedded → Cloud
3. **Technical Feasibility**: RMI proven in papers
4. **Market Need**: Everyone wants faster databases
5. **Your Skills**: Rust + systems perfect fit

## Final Checklist Before Starting

- [x] Documentation consolidated
- [x] Strategy clear
- [x] Timeline realistic (45 days)
- [ ] Rust project created
- [ ] First 100 lines written
- [ ] Papers downloaded
- [ ] HN post drafted

## The One Thing to Remember

**We're not building a better B-tree. We're replacing B-trees entirely.**

This is a 40-year paradigm shift, not an incremental improvement.

---

*"Stop planning. Start building. 45 days to change databases forever."*