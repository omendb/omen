# Performance Regression Tracking

## Overview
Created September 1, 2025 to prevent performance regressions from going unnoticed.

## Current Status

### ⚠️ Discovered Regression (Sep 1)
- **10K vectors**: 4.3K vec/s (was 97K vec/s) - **95% regression**
- **50K vectors**: 3.5K vec/s (was 99K vec/s) - **96% regression**
- **Search**: Still fast (0.7-1.0ms) ✅

### Likely Cause
The DiskANN capacity fix (4K → 100K) may have introduced overhead at medium scales.

## Tracking System

### 1. Manual Tracking
```bash
# Run regression tests
cd /Users/nick/github/omendb/omendb
PYTHONPATH=python:$PYTHONPATH python benchmarks/regression_tracker.py

# View history
cat benchmarks/regression_history.json | jq '.benchmarks[-1]'
```

### 2. Automated Tracking
```bash
# Run every 6 hours
./benchmarks/track_metrics.sh

# Add to crontab
0 */6 * * * cd /path/to/omendb && ./benchmarks/track_metrics.sh >> benchmarks/metrics.log 2>&1
```

### 3. CI/CD Integration
- GitHub Actions workflow: `.github/workflows/regression_tests.yml`
- Runs on every push to main
- Comments on PRs with performance impact

## Baseline Metrics (Target)

| Scale | Throughput | Search P50 |
|-------|------------|------------|
| 1K | 60K+ vec/s | <1ms |
| 10K | 90K+ vec/s | <2ms |
| 50K | 95K+ vec/s | <2ms |
| 100K | 90K+ vec/s | <3ms |

## Regression Thresholds

- **Critical**: >50% performance drop
- **Warning**: >20% performance drop
- **Search**: >50% latency increase

## Files

- `benchmarks/regression_tracker.py` - Main tracking system
- `benchmarks/regression_history.json` - Historical data
- `benchmarks/track_metrics.sh` - Shell wrapper
- `.github/workflows/regression_tests.yml` - CI integration

## Next Steps

1. **Investigate current regression** at 10K-50K scale
2. **Set up monitoring** dashboard
3. **Add more metrics**: memory, disk I/O, CPU usage
4. **Create alerts** for critical regressions

---
*Performance matters. Track it.*