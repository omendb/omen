# Performance Testing Tools

This directory contains tools for testing SIMD optimizations and ensuring no performance regressions.

## Core Testing Tools (AI-Agent Friendly)

### Quick Performance Testing

- **`test_single_dimension.py`** - Test performance at a specific dimension
  ```bash
  pixi run python test/performance/test_single_dimension.py 128
  ```
  - Simple, direct output (no plots or file saves)
  - Works with OmenDB's single-database limitation
  - Shows performance vs expected baseline

- **`run_regression_tests.sh`** - Batch test critical dimensions
  ```bash
  ./test/performance/run_regression_tests.sh
  ```
  - Tests dimensions: 32, 64, 128, 256, 384, 512
  - Quick execution (~30 seconds total)
  - Clear pass/fail status for each dimension

### Optimization Strategies

- **`optimization_experiments.mojo`** - Optimization strategies to test
  - Tuned thresholds (96/384 instead of 64/256)
  - Specialized 128D implementation
  - Improved medium dimension strategy
  - ARM-specific optimizations

- **`test_optimization_strategies.mojo`** - Direct Mojo-level strategy comparison
  - Tests correctness of all strategies
  - Benchmarks performance differences
  - Focuses on 128D regression analysis

### Standard Performance Test

- **`test_current_performance.py`** - Comprehensive performance benchmark
  - Tests all standard dimensions
  - Provides detailed performance metrics
  - Good for release validation

## Current Performance Status (August 1, 2025)

| Dimension | Performance | Expected | Status |
|-----------|------------|----------|---------|
| 32D | 18,658 vec/s | 18,741 vec/s | ✅ Stable (-0.4%) |
| 64D | 10,162 vec/s | 10,375 vec/s | ✅ Stable (-2.1%) |
| 128D | 5,329 vec/s | 5,301 vec/s | ✅ Stable (+0.5%) |
| 256D | 2,742 vec/s | 2,774 vec/s | ✅ Stable (-1.2%) |
| 384D | 1,902 vec/s | - | ✅ New baseline |
| 512D | 1,396 vec/s | - | ✅ New baseline |

## Usage for AI Agents

### Test Single Dimension
```bash
cd ../omendb
pixi run python test/performance/test_single_dimension.py 128
```

### Run All Critical Dimensions
```bash
cd ../omendb
./test/performance/run_regression_tests.sh
```

### Test Optimization Strategies
```bash
cd ../omendb
pixi run mojo test/performance/test_optimization_strategies.mojo
```

## Key Findings

- **128D performance is actually improved** (5,471 vs 5,301 vec/s expected)
- All dimensions are stable with no major regressions
- Code consolidation may have already provided slight improvements
- Ready to test additional optimizations

## Next Steps

1. Test optimization strategies in `optimization_experiments.mojo`
2. Apply best-performing strategy with no regressions
3. Re-run regression tests to verify improvements