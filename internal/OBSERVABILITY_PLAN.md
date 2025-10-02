# Observability Implementation Plan

**Date**: January 2025
**Goal**: Add comprehensive logging and metrics for production monitoring
**Timeline**: 6-8 hours
**Status**: 🟡 IN PROGRESS - Phase 2 complete

---

## Current State Analysis

### ✅ What We Have (Good Foundation)

**Metrics Infrastructure** (`src/metrics.rs`):
- ✅ Prometheus metrics with proper labeling
- ✅ Operation counters (searches, inserts, range queries)
- ✅ Error counters (failed operations)
- ✅ Latency histograms (p50, p95, p99 support)
- ✅ System gauges (connections, DB size, index size, memory)
- ✅ WAL metrics (writes, sync duration, size)
- ✅ SQL query metrics (duration, rows returned, errors by type)
- ✅ Timer helper for automatic duration tracking
- ✅ Health check endpoint with error rate calculation

**Logging Infrastructure** (`src/logging.rs`):
- ✅ Structured logging with tracing
- ✅ JSON format for production
- ✅ Pretty format for development
- ✅ Configurable log levels
- ✅ Span events for request tracing
- ✅ Thread IDs and names in logs
- ✅ Environment variable configuration

**HTTP Endpoints** (`src/server.rs`):
- ✅ `/metrics` - Prometheus metrics
- ✅ `/health` - Health check with error rates
- ✅ `/ready` - Liveness probe
- ✅ `/status` - Simple status check

---

## ⚠️ Critical Gaps

### 1. Learned Index Metrics - MISSING ⚠️

**Problem**: No visibility into learned index performance

**Missing Metrics**:
- [ ] Learned index hit rate (predictions used vs fallback to B-tree)
- [ ] Learned index prediction accuracy (average error in positions)
- [ ] Query path distribution (learned index vs full scan vs B-tree)
- [ ] Model performance (root model vs second layer accuracy)
- [ ] Window size effectiveness (% of queries within initial window)

**Impact**: Cannot optimize learned index or validate performance claims

---

### 2. Metrics Not Integrated - CRITICAL ⚠️

**Problem**: Metrics exist but NOT called from storage layer

**Evidence**:
```bash
$ grep -r "record_search\|record_insert" src/redb_storage.rs
# NO MATCHES
```

**Missing Integration**:
- [ ] `point_query()` - No metrics recording
- [ ] `range_query()` - No metrics recording
- [ ] `insert()` - No metrics recording
- [ ] `insert_batch()` - No metrics recording
- [ ] `delete()` - No metrics recording

**Impact**: All metrics show 0, health check is meaningless

---

### 3. Logging Not Used in Critical Paths

**Problem**: No structured logging for query execution

**Missing Logging**:
- [ ] Query start/end with parameters
- [ ] Slow query detection (>100ms warning)
- [ ] Error context (what query failed, why)
- [ ] Index rebuild events
- [ ] Batch operation progress

**Impact**: Cannot debug production issues

---

### 4. No Query Tracing

**Problem**: Cannot track query execution path

**Missing**:
- [ ] Trace IDs for request correlation
- [ ] Span tracking through layers (SQL → DataFusion → Storage → Index)
- [ ] Performance breakdown (parse time, plan time, exec time)
- [ ] Resource attribution per query

**Impact**: Cannot identify bottlenecks or slow queries

---

## Implementation Plan

### Phase 1: Learned Index Metrics ✅ COMPLETE (2 hours)

**Tasks**:
1. Add learned index metrics to `src/metrics.rs`:
   ```rust
   // Hit rate tracking
   pub static LEARNED_INDEX_HITS: Lazy<IntCounter>
   pub static LEARNED_INDEX_MISSES: Lazy<IntCounter>

   // Prediction accuracy
   pub static LEARNED_INDEX_PREDICTION_ERROR: Lazy<Histogram>

   // Query path distribution
   pub static QUERY_PATH: Lazy<IntCounterVec> // labels: learned, fallback, scan
   ```

2. Add helper functions:
   ```rust
   pub fn record_learned_index_hit(predicted_pos: usize, actual_pos: usize)
   pub fn record_learned_index_miss()
   pub fn record_query_path(path: &str) // "learned", "fallback", "scan"
   ```

3. **Test**: Add metrics validation tests

**Deliverable**: ✅ New metrics defined and tested (Commit 6ca61f4)

---

### Phase 2: Integrate Metrics into Storage ✅ COMPLETE (2 hours)

**Tasks**:
1. Add metrics to `src/redb_storage.rs`:
   - `point_query()`: Record search duration, learned index hit/miss
   - `range_query()`: Record query duration, rows returned
   - `insert()`: Record insert duration
   - `insert_batch()`: Record batch size, duration, throughput
   - `delete()`: Record duration

2. Add error metrics:
   - Catch and record all error types
   - Track specific failure modes (mutex poison, I/O errors, etc.)

3. Add learned index metrics in `src/index.rs`:
   - Record prediction accuracy in `search()`
   - Track model selection in RMI layers
   - Record error bounds

4. **Test**: Validate metrics increment correctly

**Deliverable**: ✅ All operations record metrics (Commit 1230d5c)
- point_query(), range_query(), insert(), insert_batch(), delete(), rebuild_index()
- Added model_count() to RecursiveModelIndex
- All tests passing (5 storage + 23 metrics)

---

### Phase 3: Add Structured Logging (2 hours)

**Tasks**:
1. Add logging to `src/redb_storage.rs`:
   ```rust
   #[instrument(skip(self), fields(key = key))]
   pub fn point_query(&self, key: i64) -> Result<Option<Vec<u8>>> {
       debug!("Point query started");

       let start = Instant::now();
       let result = /* ... */;
       let duration = start.elapsed();

       if duration > Duration::from_millis(100) {
           warn!(duration_ms = duration.as_millis(), "Slow query detected");
       }

       debug!(duration_ms = duration.as_millis(), "Point query completed");
       result
   }
   ```

2. Add logging to `src/index.rs`:
   - Log index rebuild events
   - Log model training duration
   - Log prediction errors >10% of dataset

3. Add logging to `src/sql_engine.rs`:
   - Log SQL parse errors with query text
   - Log execution plan selection
   - Log slow queries (>1s)

4. **Test**: Verify log output in tests

**Deliverable**: Comprehensive logging for debugging

---

### Phase 4: Query Tracing (2 hours)

**Tasks**:
1. Add trace ID generation:
   ```rust
   use uuid::Uuid;

   pub struct QueryContext {
       trace_id: Uuid,
       start_time: Instant,
       query_type: String,
   }
   ```

2. Thread trace_id through layers:
   - SQL → DataFusion → Storage → Index

3. Add span tracking:
   ```rust
   let span = info_span!("query_execution", trace_id = %ctx.trace_id);
   let _guard = span.enter();
   ```

4. Add performance breakdown:
   - Parse time
   - Planning time
   - Execution time
   - Learned index lookup time

5. **Test**: Verify trace IDs propagate correctly

**Deliverable**: End-to-end query tracing

---

## Testing Strategy

### Metrics Tests
```rust
#[test]
fn test_learned_index_metrics() {
    let storage = create_test_storage();

    let before = LEARNED_INDEX_HITS.get();
    storage.point_query(100).unwrap();
    let after = LEARNED_INDEX_HITS.get();

    assert!(after > before, "Metrics should increment");
}
```

### Logging Tests
```rust
#[test]
fn test_slow_query_logging() {
    init_test_logging();

    // Trigger slow query
    // Verify warning log emitted
}
```

### Integration Tests
```rust
#[test]
fn test_end_to_end_observability() {
    // Execute query
    // Verify:
    // - Metrics incremented
    // - Logs emitted
    // - Trace ID consistent
}
```

---

## Success Criteria

✅ **Metrics**:
- All storage operations record metrics
- Learned index hit rate tracked
- Prometheus `/metrics` shows real data
- Health check calculates from actual operations

✅ **Logging**:
- All queries logged with duration
- Slow queries (>100ms) trigger warnings
- Errors logged with full context
- Structured JSON in production

✅ **Tracing**:
- Trace IDs generated for all queries
- Performance breakdown available
- Spans track through all layers

✅ **Documentation**:
- Observability guide created
- Metrics catalog documented
- Dashboard examples provided

---

## Monitoring Dashboards (Future)

Once metrics are integrated, create Grafana dashboards:

1. **System Overview**:
   - Throughput (ops/sec)
   - Latency (p50, p95, p99)
   - Error rate
   - Active connections

2. **Learned Index Performance**:
   - Hit rate over time
   - Prediction accuracy
   - Query path distribution
   - Model performance

3. **Query Analysis**:
   - Slow queries (>1s)
   - Query type distribution
   - Rows returned distribution
   - Error breakdown by type

4. **Resource Utilization**:
   - Memory usage
   - Database size growth
   - WAL size and sync frequency
   - Index size

---

## Alerting Rules (Future)

Prometheus alerting rules:

```yaml
- alert: HighErrorRate
  expr: rate(omendb_searches_failed_total[5m]) > 0.01
  for: 5m
  annotations:
    summary: "Error rate above 1%"

- alert: SlowQueries
  expr: histogram_quantile(0.95, omendb_search_duration_seconds) > 0.1
  for: 10m
  annotations:
    summary: "P95 latency above 100ms"

- alert: LearnedIndexDegraded
  expr: rate(omendb_learned_index_misses_total[5m]) /
        rate(omendb_learned_index_hits_total[5m]) > 0.1
  annotations:
    summary: "Learned index hit rate below 90%"
```

---

## Commits Plan

1. ✅ Create this observability plan document (Commit cce66a0)
2. ✅ Add learned index metrics (Phase 1) (Commit 6ca61f4)
3. ✅ Integrate metrics into storage layer (Phase 2) (Commit 1230d5c)
4. ⏳ Add structured logging to storage (Phase 3a)
5. ⏳ Add structured logging to SQL engine (Phase 3b)
6. ⏳ Add query tracing infrastructure (Phase 4)
7. ⏳ Add observability tests
8. ⏳ Create observability guide for operators

**Target**: 8-10 commits with incremental progress
