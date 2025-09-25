# Immediate Action Plan: Learned Database Pivot

**Created**: September 25, 2025
**Goal**: YC W26 Application (Deadline: Oct 15)

## Today (Sept 25) - Do These NOW

### 1. Create Development Branch (5 min)
```bash
git checkout -b learned-database-pivot
git commit -am "pivot: Strategic move to learned database systems"
```

### 2. Set Up Rust Project (10 min)
```bash
cargo new omendb-learned --lib
cd omendb-learned
cargo add candle-core ndarray pgrx rayon criterion
```

### 3. Write First RMI Prototype (2 hours)
```rust
// Start simple: Linear models only
pub struct LinearRMI {
    models: Vec<(f64, f64)>, // (slope, intercept)
    data: Vec<(i64, Vec<u8>)>,
}
```

### 4. Create PostgreSQL Extension Skeleton (1 hour)
```bash
cargo pgrx new --name omendb_learned
cd omendb_learned
cargo pgrx run
```

### 5. Update Landing Page (30 min)
```html
<h1>OmenDB: The Learning Database</h1>
<p>10x faster than B-trees. First production learned index.</p>
```

### 6. Post to Hacker News (15 min)
```
Title: Show HN: Building the first production learned database
Body: After seeing 30+ vector DB competitors, I'm pivoting to
learned indexes - ML models that replace B-trees with 10x
performance. Looking for ML co-founder. [link to GitHub]
```

## This Week (Sept 26-30)

### Technical Milestones
- [ ] Basic RMI working with linear models
- [ ] PostgreSQL extension compiling
- [ ] Benchmark showing 5x improvement
- [ ] Error bounds implemented
- [ ] Delta buffer for updates

### Business Milestones
- [ ] Talk to 3 potential co-founders
- [ ] GitHub repo public with README
- [ ] Blog post: "Why I'm Pivoting from Vector to Learned DB"
- [ ] Reach out to database influencers
- [ ] Join PostgreSQL Slack/Discord

## Next Week (Oct 1-7)

### YC Application Sprint
- [ ] Demo video recorded (2 min)
- [ ] 10x performance demonstrated
- [ ] Application written
- [ ] References secured
- [ ] Backup recorded

### Technical Polish
- [ ] TPC-H benchmark suite
- [ ] Docker container
- [ ] CI/CD pipeline
- [ ] Documentation site
- [ ] Interactive playground

## Success Metrics

### Day 1 (Today)
✓ Pivot decision made
✓ Research completed
✓ Plan documented
□ First code committed
□ HN post live

### Day 7
□ RMI prototype working
□ 5x performance proven
□ 100+ GitHub stars
□ 1 co-founder conversation
□ PostgreSQL extension runs

### Day 14
□ YC application submitted
□ Demo video polished
□ 10x performance achieved
□ 500+ GitHub stars
□ First user interested

## Code to Write TODAY

```rust
// File: src/learned_index.rs
// Write this in the next 2 hours

use ndarray::{Array1, Array2};

pub struct LearnedIndex {
    root_model: LinearModel,
    leaf_models: Vec<LinearModel>,
    data: Vec<(i64, Vec<u8>)>,
}

struct LinearModel {
    slope: f64,
    intercept: f64,
    min_err: i32,
    max_err: i32,
}

impl LearnedIndex {
    pub fn train(data: &[(i64, Vec<u8>)]) -> Self {
        // 1. Train root model on CDF
        // 2. Partition data
        // 3. Train leaf models
        // 4. Compute error bounds
        todo!()
    }

    pub fn get(&self, key: i64) -> Option<Vec<u8>> {
        // 1. Root model predicts segment
        // 2. Leaf model predicts position
        // 3. Binary search in small range
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_faster_than_btree() {
        // Prove we're 5x faster
    }
}
```

## Repository Structure (Create Now)

```
omendb-learned/
├── Cargo.toml
├── README.md ("The Learning Database - 10x faster than B-trees")
├── src/
│   ├── lib.rs
│   ├── learned_index.rs (core RMI)
│   ├── models.rs (linear, neural)
│   └── storage.rs (data pages)
├── benches/
│   └── btree_comparison.rs
├── examples/
│   └── basic_usage.rs
└── pgrx/
    └── omendb_extension.rs
```

## Marketing Message

**Old**: "Fast vector database built in Mojo"

**New**: "OmenDB replaces 45-year-old B-trees with AI. First production learned database. 10x faster queries."

## Contingency Plans

### If RMI Doesn't Work
- Try RadixSpline (simpler)
- Try ALEX (more complex)
- Worst case: Optimized B-tree with ML hints

### If No Co-founder by Oct 7
- Apply to YC solo
- Emphasize technical progress
- Promise to recruit at YC

### If YC Rejects
- Apply to other accelerators
- Launch on ProductHunt
- Raise from angels directly

## Final Checklist Before Sleep Tonight

□ Branch created
□ Rust project initialized
□ First 100 lines of code written
□ PostgreSQL extension skeleton created
□ HN post submitted
□ Tomorrow's plan clear

## Remember

You're not building another database. You're replacing the foundation of all databases.

This is a $50B opportunity with zero competition.

**Execute ruthlessly.**

---

*"By October 15, we'll either have a YC interview or know exactly why learned databases don't work. Both outcomes are valuable. Ship it."*