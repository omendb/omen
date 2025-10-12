# AI Agent Guide for OmenDB

**Created**: October 11, 2025
**Purpose**: Quick decision trees for AI assistants (inspired by nijaru/agent-contexts)
**Audience**: Claude Code, other AI coding assistants

---

## Core Principles

1. **Actionable over Informational** - Every section leads to a specific action
2. **Timeless over Trendy** - Focus on patterns that won't become outdated
3. **Universal over Personal** - Works for any AI assistant
4. **Concise over Comprehensive** - Optimized for context windows

---

## Quick Start Decision Tree

```
START HERE:
│
├─ New session?
│  └─ Read: CLAUDE.md (context)
│     └─ Read: internal/STATUS_REPORT_OCT_2025.md (status)
│        └─ Check: git log --oneline -10 (recent work)
│           └─ PROCEED TO TASK
│
├─ User asks "what's our status?"
│  └─ Read: internal/STATUS_REPORT_OCT_2025.md
│     └─ Answer with: Current phase, recent achievements, next steps
│
├─ User asks "why did we do X?"
│  └─ Search: internal/ for decision rationale
│     └─ Check: git log --all --grep="X" for commit context
│        └─ If not found: internal/archive/ for historical decisions
│
├─ User asks for benchmark/performance data
│  └─ Read: STATUS_UPDATE.md (quick numbers)
│     └─ Read: internal/research/100M_SCALE_RESULTS.md (detailed)
│        └─ Read: docs/PERFORMANCE.md (user-facing)
│
├─ User asks about architecture
│  └─ Read: ARCHITECTURE.md (high-level)
│     └─ Read: internal/technical/COMPREHENSIVE_ARCHITECTURAL_REVIEW_OCT_2025.md (detailed)
│        └─ Read: src/[component]/[file].rs for implementation
│
├─ Task requires coding
│  └─ Check: CONTRIBUTING.md (guidelines)
│     └─ Check: Any related design docs in internal/design/
│        └─ Write tests first, then implementation
│           └─ Run: cargo test && cargo build --release
│
└─ Major milestone completed
   └─ Update: Relevant STATUS or PHASE doc
      └─ Consider: Creating session summary if >2 hours work
         └─ Commit: Docs alongside code changes
```

---

## Common Tasks

### Task: Running Benchmarks

```
IF: Need to validate performance claims
THEN:
  1. cargo build --release
  2. ./target/release/benchmark_vs_sqlite [scale]
  3. Document results in internal/research/
  4. Update STATUS_REPORT if significant
```

### Task: Adding New Feature

```
IF: Implementing new feature
THEN:
  1. Check if design doc exists in internal/design/
  2. If not, consider creating one for complex features
  3. Write tests first: tests/[feature]_tests.rs
  4. Implement in src/
  5. Run: cargo test && cargo clippy
  6. Update relevant docs (ARCHITECTURE.md if structural change)
```

### Task: Fixing Performance Issue

```
IF: Performance regression or bottleneck
THEN:
  1. Run: cargo build --release --bin profile_[relevant]
  2. Identify bottleneck
  3. Check internal/research/ for similar past issues
  4. Implement fix
  5. Validate with benchmark
  6. Document in internal/research/OPTIMIZATION_*.md
```

### Task: Debugging Test Failure

```
IF: Test failure
THEN:
  1. cargo test [test_name] -- --nocapture (see output)
  2. Check git log for recent changes to that module
  3. Check if test needs updating due to intentional behavior change
  4. Fix code or update test
  5. Ensure full test suite passes: cargo test
```

### Task: Updating Documentation

```
IF: Code changes affect user-facing behavior
THEN:
  1. Update README.md if usage changed
  2. Update ARCHITECTURE.md if structure changed
  3. Update CLAUDE.md if major milestone reached
  4. Update internal/STATUS_REPORT_OCT_2025.md if milestone achieved
```

---

## Error Patterns

### Error: "Benchmark shows slower than expected"

```
DIAGNOSIS:
1. Check if test data matches production pattern (sequential vs random)
2. Check if cache warming needed
3. Check if correct release build: cargo build --release
4. Check if background processes interfering

FIX:
- Reference: internal/TESTING_SESSION_SUMMARY.md for profiler vs benchmark gap
- Use: profile_query_path for detailed analysis
- Compare: Sequential vs random data patterns
```

### Error: "Test passes locally but fails in CI"

```
DIAGNOSIS:
1. Check: .github/workflows/test.yml for CI environment differences
2. Check: Timing-dependent tests (may fail on slower CI)
3. Check: File path assumptions (absolute vs relative)

FIX:
- Make tests deterministic (no timing dependencies)
- Use relative paths or environment variables
- Add retries for flaky tests (with comment explaining why)
```

### Error: "Out of memory during benchmark"

```
DIAGNOSIS:
1. Check: Benchmark scale (100M rows = ~140MB minimum)
2. Check: Memory leaks (use valgrind or cargo-valgrind)
3. Check: Unbounded collections

FIX:
- Reference: internal/research/100M_SCALE_RESULTS.md for expected memory
- Use smaller scale for development: 1M or 10M
- Check: src/memory_pool.rs for memory management utilities
```

### Error: "Compilation takes forever"

```
DIAGNOSIS:
1. Check: cargo build --release vs cargo build (release is slower)
2. Check: Whether incremental compilation is enabled
3. Check: Dependency bloat

FIX:
- Development: Use cargo build (unoptimized, faster)
- Benchmarks: Use cargo build --release (optimized, slower)
- Cache: cargo-build-cache or sccache
```

---

## Code Patterns

### Pattern: Adding a New Benchmark

```rust
// Location: src/bin/benchmark_[name].rs
// Template: Copy from src/bin/benchmark_vs_sqlite.rs

use omendb::{Table, Value};
use std::time::Instant;

fn main() {
    // 1. Setup
    let table = Table::new(schema).unwrap();

    // 2. Warm up (important!)
    for _ in 0..1000 { /* warmup */ }

    // 3. Timed operation
    let start = Instant::now();
    // ... operation ...
    let elapsed = start.elapsed();

    // 4. Report results
    println!("Throughput: {:.2} ops/sec", count as f64 / elapsed.as_secs_f64());

    // 5. Compare baseline (if applicable)
    // Run SQLite equivalent and compare
}
```

### Pattern: Adding a New Table API Method

```rust
// Location: src/table.rs

impl Table {
    /// Brief description of what this does
    ///
    /// # Arguments
    /// * `arg` - What this argument means
    ///
    /// # Returns
    /// What this returns
    ///
    /// # Example
    /// ```
    /// let result = table.new_method(arg)?;
    /// ```
    pub fn new_method(&self, arg: Type) -> Result<ReturnType> {
        // 1. Validate inputs

        // 2. Acquire locks if needed

        // 3. Perform operation

        // 4. Return result

        Ok(result)
    }
}

// Don't forget: Add test in tests/[relevant]_tests.rs
```

### Pattern: Adding Multi-level ALEX Logic

```rust
// Location: src/alex/[component].rs
// Reference: src/alex/multi_level.rs for patterns

// Key principles:
// 1. Fixed 64 keys/leaf fanout (cache-line optimized)
// 2. Adaptive retraining (check needs_retrain())
// 3. Height 2-3 for 100M scale
// 4. Maintain hierarchical caching

impl MultiLevelAlex {
    // Pattern: Routing through levels
    fn route(&self, key: u64) -> Result<&Leaf> {
        let mut node = &self.root;

        // Traverse inner nodes (O(log n))
        while let NodeType::Inner(inner) = node {
            node = inner.route(key)?;
        }

        // Reached leaf
        match node {
            NodeType::Leaf(leaf) => Ok(leaf),
            _ => unreachable!(),
        }
    }
}
```

---

## Decision Trees by Task Type

### Competitive Benchmark Request

```
IF: User says "benchmark vs [competitor]"
THEN:
  1. Check if comparison already exists:
     - internal/research/ALEX_SQLITE_BENCHMARK_RESULTS.md (SQLite)
     - internal/research/YCSB_BENCHMARK_RESULTS.md (industry standard)

  2. If exists:
     → Report findings from existing doc

  3. If not exists:
     → Create new benchmark:
       a. Set up competitor (Docker if distributed DB)
       b. Create equivalent workload
       c. Run benchmarks (3+ runs, report median)
       d. Document in internal/research/[NAME]_BENCHMARK_RESULTS.md
       e. Update internal/STATUS_REPORT_OCT_2025.md
```

### Performance Regression

```
IF: Benchmark shows performance drop
THEN:
  1. Identify baseline: git log --grep="benchmark" (find last good run)

  2. Bisect: git bisect start HEAD [last_good_commit]

  3. Profile:
     - ./target/release/profile_query_path (identify bottleneck)
     - cargo flamegraph --bin [relevant] (visualize)

  4. Fix:
     - Reference: internal/TESTING_SESSION_SUMMARY.md for past regressions
     - Check: Similar issues in internal/archive/optimizations/

  5. Validate:
     - Run full benchmark suite
     - Ensure no other regressions
     - Document in internal/research/OPTIMIZATION_*.md
```

### Adding Enterprise Feature

```
IF: User requests enterprise feature (auth, backup, replication)
THEN:
  1. Check if already planned:
     - internal/STATUS_REPORT_OCT_2025.md "What's Missing" section
     - internal/business/YC_W25_ROADMAP.md

  2. If not planned:
     → Discuss trade-offs (complexity vs market need)
     → Get user confirmation on priority

  3. If approved:
     → Create design doc: internal/design/[FEATURE].md
     → Estimate: Timeline and complexity
     → Implement with tests
     → Update: internal/technical/PRODUCTION_READINESS_ASSESSMENT.md
```

---

## Documentation Update Triggers

```
UPDATE: CLAUDE.md
WHEN:
  - Major milestone achieved (e.g., 100M scale validated)
  - Architecture changes (e.g., multi-level ALEX added)
  - Performance characteristics change significantly
  - New competitive advantages validated

UPDATE: internal/STATUS_REPORT_OCT_2025.md
WHEN:
  - Monthly (comprehensive update)
  - Major milestone achieved
  - Competitive benchmark completed
  - Business metrics change (LOIs, customers, funding)

UPDATE: internal/README.md
WHEN:
  - New directory added to internal/
  - New important doc created
  - Phase completion
  - Archive reorganization

UPDATE: ARCHITECTURE.md
WHEN:
  - New major component added
  - Component responsibilities change
  - Data flow changes
  - API boundaries change

DO NOT UPDATE:
  - Internal docs for minor code changes
  - README.md for every small fix
  - Docs when only implementation details change (not interface)
```

---

## Anti-Patterns (What NOT to Do)

### ❌ Don't: Create duplicate status docs

```
# Anti-pattern
STATUS_NEW.md
STATUS_LATEST.md
CURRENT_STATUS.md
```

**Why bad**: Confusion about source of truth, outdated info, context waste

**Do instead**:
- Single STATUS_REPORT_OCT_2025.md (current)
- STATUS_REPORT_JAN_2025.md (marked as superseded)
- Archive old ones

### ❌ Don't: Write vague commit messages

```
# Anti-pattern
git commit -m "fix bug"
git commit -m "update stuff"
git commit -m "wip"
```

**Why bad**: Future debugging impossible, git log --grep useless

**Do instead**:
```bash
git commit -m "fix: Multi-level ALEX routing for keys at boundaries"
git commit -m "perf: Reduce memory allocations in hot path (15% speedup)"
git commit -m "test: Add 100M scale durability validation"
```

### ❌ Don't: Skip tests for "small" changes

```
# Anti-pattern
# Change code
cargo build --release
# Ship it
```

**Why bad**: Breaks CI, introduces regressions, wastes time debugging later

**Do instead**:
```bash
# Change code
cargo test                    # Unit tests
cargo test --release          # Integration tests
cargo clippy                  # Lints
./target/release/benchmark_vs_sqlite 10000000  # If performance-critical
```

### ❌ Don't: Compare different levels of stack

```
# Anti-pattern from past mistake
"In-memory ALEX vs full SQLite (with durability)" → Misleading 500x claim
```

**Why bad**: Unfair comparison, loses credibility, customer confusion

**Do instead**:
- Same features on both sides (both with/without durability)
- Document exactly what's tested
- Explicit caveats about differences
- Refer to: Global CLAUDE.md benchmarking guidelines

---

## Quick Reference

### Most Important Files (Start Here)

```
Priority 1 (Always read for new sessions):
1. CLAUDE.md                              - Project context
2. internal/STATUS_REPORT_OCT_2025.md     - Current status

Priority 2 (Read when relevant):
3. ARCHITECTURE.md                        - System structure
4. CONTRIBUTING.md                        - Code guidelines
5. internal/README.md                     - Docs index

Priority 3 (Read for specific tasks):
6. internal/research/                     - Benchmarks, validation
7. internal/technical/                    - Implementation details
8. internal/business/                     - Strategy, funding
```

### Command Quick Reference

```bash
# Development
cargo build                               # Fast, unoptimized
cargo test                                # All tests
cargo clippy                              # Lints

# Benchmarking
cargo build --release                     # Optimized (slow build)
./target/release/benchmark_vs_sqlite 10000000  # vs SQLite
./target/release/ycsb_benchmark           # Industry standard

# Profiling
./target/release/profile_query_path       # Detailed query profile
cargo flamegraph --bin [benchmark]        # Visual profile

# Servers
./target/release/postgres_server          # PostgreSQL (port 5433)
./target/release/rest_server              # REST API (port 8080)

# Testing at scale
./target/release/durability_validation_advanced  # Durability test
./target/release/stress_test_concurrent   # Concurrent stress
```

---

## Summary

This guide provides **decision trees and action patterns** for AI assistants working on OmenDB.

**Philosophy**:
- Quick decisions over deep reading
- Actionable steps over information dumps
- Consistent patterns over ad-hoc solutions

**Use this when**:
- Starting new session (Quick Start tree)
- Encountering errors (Error Patterns)
- Performing common tasks (Common Tasks)
- Making decisions (Decision Trees)

**Use STATUS_REPORT_OCT_2025.md when**:
- Need comprehensive project understanding
- Need business context
- Need competitive positioning
- Need full technical status

Both docs complement each other:
- This doc: Quick actions and decisions
- STATUS_REPORT: Comprehensive understanding

---

**Last Updated**: October 11, 2025
**Maintained By**: AI assistants (Claude Code)
**Inspired By**: [nijaru/agent-contexts](https://github.com/nijaru/agent-contexts)
**Status**: Living document - update as patterns emerge
