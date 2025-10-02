# Testing Requirements for Performance-Critical Features

**Date:** October 1, 2025
**Status:** Mandatory for all performance-critical code

## Motivation

On October 1, 2025, we discovered that our learned index feature was **never actually being used** despite all tests passing. The learned index was maintained (expensive) but bypassed in queries (no benefit). This went undetected for 1 day because tests only verified correctness, not implementation.

**Key insight:** "Tests passing" ≠ "Feature working"

## The 4-Level Testing Pyramid for Performance Features

### Level 1: Correctness Tests (Basic)
**What:** Verify feature returns correct results
**Example:** Point query returns the right value
**Coverage:** Basic functionality
**NOT sufficient for performance features!**

### Level 2: Implementation Verification Tests (REQUIRED)
**What:** Verify feature is actually being used, not bypassed
**Example:** Learned index.search() is called during point_query()
**Coverage:** Proves code path is exercised

**Required for ALL performance-critical features**

### Level 3: Baseline Comparison Tests (REQUIRED)
**What:** Measure performance with feature ON vs OFF
**Example:** Learned index ON provides 10x+ speedup vs OFF
**Coverage:** Proves feature provides benefit

**Required for ALL performance-critical features**

### Level 4: Performance Regression Tests (REQUIRED in CI)
**What:** Track performance over time, fail if it degrades
**Example:** Benchmark must complete in <100ms or CI fails
**Coverage:** Prevents regressions

**Required for ALL performance-critical features**

---

## Mandatory Requirements

For ANY feature claiming performance improvement, you MUST have:

### ✅ Requirement 1: Implementation Verification Test

**Purpose:** Prove the feature is actually being used

**Implementation approaches:**

#### Option A: Instrumentation/Metrics
```rust
// Add counters to track usage
pub struct LearnedIndexMetrics {
    pub search_calls: AtomicU64,
    pub predictions_used: AtomicU64,
    pub fallback_to_btree: AtomicU64,
}

#[test]
fn test_learned_index_is_called() {
    let storage = create_storage();
    let initial_calls = storage.metrics.search_calls.load(Ordering::Relaxed);

    storage.point_query(123).unwrap();

    let final_calls = storage.metrics.search_calls.load(Ordering::Relaxed);
    assert!(final_calls > initial_calls, "Learned index should be called");
}
```

#### Option B: Feature Flags for A/B Testing
```rust
#[cfg(feature = "learned_index")]
pub fn point_query(&self, key: i64) -> Result<Option<Vec<u8>>> {
    // Use learned index
}

#[cfg(not(feature = "learned_index"))]
pub fn point_query(&self, key: i64) -> Result<Option<Vec<u8>>> {
    // Direct lookup
}

#[test]
fn test_learned_index_provides_benefit() {
    #[cfg(feature = "learned_index")]
    let with_index_time = benchmark_point_query();

    #[cfg(not(feature = "learned_index"))]
    let without_index_time = benchmark_point_query();

    assert!(with_index_time < without_index_time / 5); // 5x speedup
}
```

#### Option C: Mock/Spy Pattern
```rust
#[test]
fn test_learned_index_called() {
    let mut mock_index = MockLearnedIndex::new();
    mock_index.expect_search().times(1).returning(|_| Some(100));

    let storage = Storage::with_index(mock_index);
    storage.point_query(123).unwrap();

    // Mock verifies search() was called
}
```

### ✅ Requirement 2: Baseline Comparison Test

**Purpose:** Prove feature provides measurable benefit

**Template:**
```rust
#[test]
fn test_FEATURE_baseline_comparison() {
    println!("\n=== Baseline Comparison: FEATURE ===");

    // Setup
    let dataset = create_test_dataset(10_000);

    // Test WITH feature (current implementation)
    let with_feature_time = measure_operation(&dataset);

    // Test WITHOUT feature (naive/baseline implementation)
    let without_feature_time = measure_baseline_operation(&dataset);

    let speedup = without_feature_time / with_feature_time;

    println!("With FEATURE: {:.3}ms", with_feature_time);
    println!("Without FEATURE (baseline): {:.3}ms", without_feature_time);
    println!("Speedup: {:.1}x", speedup);

    // Assert minimum expected improvement
    assert!(
        speedup >= 5.0,
        "FEATURE should provide >=5x speedup, got {:.1}x",
        speedup
    );
}
```

### ✅ Requirement 3: Performance Regression CI

**Purpose:** Prevent performance from degrading over time

**Implementation:**

1. **Add benchmark suite:**
```rust
// benches/critical_path_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_point_query(c: &mut Criterion) {
    let storage = setup_storage_10k();

    c.bench_function("point_query_10k", |b| {
        b.iter(|| {
            storage.point_query(black_box(5000)).unwrap()
        });
    });
}

criterion_group!(benches, bench_point_query);
criterion_main!(benches);
```

2. **Add CI job (`.github/workflows/performance.yml`):**
```yaml
name: Performance Regression Tests

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Run benchmarks
        run: cargo bench --bench critical_path_benchmarks

      - name: Compare against baseline
        run: |
          # Compare with main branch results
          # Fail if >10% slower
```

3. **Store benchmark results:**
```bash
# Store results in git
cargo bench -- --save-baseline main

# On PR, compare
cargo bench -- --baseline main
```

### ✅ Requirement 4: Documentation

**Required sections in feature documentation:**

1. **Expected Performance:**
   - Quantitative targets (e.g., "10x speedup on 100K+ rows")
   - Dataset characteristics where it excels
   - Cases where it doesn't help

2. **How to Verify It's Working:**
   - Metrics to check
   - Log messages to look for
   - Performance characteristics to measure

3. **Test Coverage:**
   - Link to implementation verification test
   - Link to baseline comparison test
   - Link to benchmarks

**Example:**
```markdown
## Learned Index Performance

**Expected:** 10x+ speedup on point queries for datasets >100K rows

**How to verify it's working:**
1. Check metrics: `learned_index.search_calls > 0`
2. Run baseline comparison: `cargo test test_learned_index_baseline_comparison`
3. Point query should be <1ms, full scan should be ~100ms

**Test coverage:**
- Implementation verification: `tests/learned_index_verification_tests.rs::test_learned_index_sorted_keys_maintained`
- Baseline comparison: `tests/learned_index_verification_tests.rs::test_learned_index_provides_speedup_10k_rows`
- Benchmarks: `benches/learned_index_benchmarks.rs`
```

---

## Code Review Checklist

Before approving ANY performance-related PR:

- [ ] Does this feature have implementation verification tests?
- [ ] Does this feature have baseline comparison tests?
- [ ] Are there metrics/instrumentation to verify it's being used?
- [ ] Do benchmarks show the claimed improvement?
- [ ] Is there documentation on how to verify it's working?
- [ ] Are benchmarks added to CI?
- [ ] Do test helpers use the production code path?

---

## Test Helper Requirements

**Problem:** Test helpers can use slow paths, masking performance issues

**Requirements:**

1. **Use production code paths:**
   ```rust
   // ❌ BAD: Test helper uses slow path
   fn create_table(n: usize) {
       for i in 0..n {
           storage.insert(i, value); // One transaction per insert!
       }
   }

   // ✅ GOOD: Test helper uses fast path
   fn create_table(n: usize) {
       let entries: Vec<_> = (0..n).map(|i| (i, value)).collect();
       storage.insert_batch(entries); // Single transaction!
   }
   ```

2. **Document performance characteristics:**
   ```rust
   /// Creates a table with N rows using insert_batch for speed.
   /// Performance: ~30K rows/sec on typical hardware.
   /// If this is slow, check that insert_batch uses single transaction.
   fn create_table(n: usize) { ... }
   ```

3. **Add assertions about helper performance:**
   ```rust
   fn create_table(n: usize) {
       let start = Instant::now();
       // ... create table ...
       let rate = n as f64 / start.elapsed().as_secs_f64();

       assert!(
           rate > 10_000.0,
           "Test helper too slow: {:.0} rows/sec (expected >10K)",
           rate
       );
   }
   ```

---

## When These Requirements Apply

**ALWAYS required for:**
- Performance optimizations (caching, indexing, etc.)
- Algorithm changes claiming speedup
- Storage layer changes
- Query optimization features

**Sometimes required for:**
- New features with performance targets
- Refactorings touching critical paths

**Not required for:**
- Pure correctness features
- UI/UX changes
- Documentation updates

---

## Enforcement

1. **PR Template:** Includes performance checklist
2. **CI Pipeline:** Runs benchmarks, fails on regression
3. **Code Review:** Reviewers must verify all 4 requirements
4. **Post-merge:** Monitor metrics in production

---

## Lessons from Learned Index Issue

**What went wrong:**
1. Tests verified correctness but not implementation
2. No baseline comparison (never tested with learned index OFF)
3. Misinterpreted slow performance as "expected overhead"
4. Test helper used slow code path (individual inserts)
5. No instrumentation to see if learned index was called

**How we fixed it:**
1. Added implementation verification tests (prove it's called)
2. Added baseline comparison tests (measure speedup)
3. Fixed test helpers to use production code paths
4. Added instrumentation/metrics
5. Updated documentation with actual performance

**Time to fix:** 1 day
**Cost of not catching it:** Would have shipped broken feature to production

---

## Template: Performance Feature PR

When submitting a PR for a performance feature:

```markdown
## Performance Feature: [Name]

**Claimed benefit:** [e.g., "10x speedup on point queries"]

**Implementation verification:**
- [ ] Test file: [path]
- [ ] Test proves feature is actually used
- [ ] Test name: [test_name]

**Baseline comparison:**
- [ ] Test file: [path]
- [ ] Compares feature ON vs OFF
- [ ] Test name: [test_name]
- [ ] Results: [X]x speedup

**Benchmarks:**
- [ ] Benchmark file: [path]
- [ ] Added to CI pipeline
- [ ] Results: [numbers]

**Metrics/Instrumentation:**
- [ ] Counters added: [list]
- [ ] How to verify: [describe]

**Documentation:**
- [ ] Expected performance documented
- [ ] How to verify it's working
- [ ] Test coverage listed
```

---

## References

- **Case study:** CRITICAL_FINDINGS.md - Learned index was never used
- **Example tests:** tests/learned_index_verification_tests.rs
- **Example benchmarks:** benches/ (TODO: add learned_index_benchmarks.rs)

---

**Last Updated:** October 1, 2025
**Status:** Mandatory for all performance-critical code
**Owner:** Engineering team
