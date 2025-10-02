# OmenDB Observability Guide

**For Operators & SREs**

This guide covers monitoring, logging, and troubleshooting OmenDB in production.

---

## Quick Start

### Metrics Endpoint

```bash
curl http://localhost:9090/metrics
```

### Health Check

```bash
curl http://localhost:9090/health
```

Returns JSON with error rates and system status.

### Enable Logging

```bash
# Development (pretty formatted)
RUST_LOG=debug cargo run

# Production (JSON formatted)
RUST_LOG=info,omendb=debug ./omendb-server
```

---

## Metrics

### Core Operations

**Search Operations:**
- `omendb_searches_total` - Total point queries
- `omendb_searches_failed_total` - Failed searches
- `omendb_search_duration_seconds` - Search latency (histogram)

**Insert Operations:**
- `omendb_inserts_total` - Total inserts
- `omendb_inserts_failed_total` - Failed inserts
- `omendb_insert_duration_seconds` - Insert latency (histogram)

**Example Queries:**
```promql
# P95 search latency
histogram_quantile(0.95, omendb_search_duration_seconds)

# Insert throughput (ops/sec)
rate(omendb_inserts_total[5m])

# Error rate
rate(omendb_searches_failed_total[5m]) / rate(omendb_searches_total[5m])
```

---

### Learned Index Metrics

**Hit Rate:**
- `omendb_learned_index_hits_total` - Successful predictions
- `omendb_learned_index_misses_total` - Fallback to B-tree
- **Hit Rate**: `hits / (hits + misses)`

**Performance:**
- `omendb_learned_index_prediction_error_positions` - Prediction accuracy (histogram)
- `omendb_learned_index_window_hits_total` - Predictions within ±100 positions
- `omendb_learned_index_size_keys` - Number of keys indexed
- `omendb_learned_index_models_count` - Number of models trained

**Query Path Distribution:**
- `omendb_query_path_total{path="learned_index"}` - Queries using learned index
- `omendb_query_path_total{path="fallback_btree"}` - Direct B-tree lookups
- `omendb_query_path_total{path="full_scan"}` - Full table scans

**Example Queries:**
```promql
# Learned index hit rate (should be >90%)
omendb_learned_index_hits_total /
  (omendb_learned_index_hits_total + omendb_learned_index_misses_total)

# Average prediction error (should be <100 positions)
rate(omendb_learned_index_prediction_error_positions_sum[5m]) /
  rate(omendb_learned_index_prediction_error_positions_count[5m])

# Query path breakdown
sum by (path) (rate(omendb_query_path_total[5m]))
```

---

### System Metrics

- `omendb_active_connections` - Current active connections
- `omendb_database_size_bytes` - Database size on disk
- `omendb_index_size_keys` - Number of indexed keys
- `omendb_memory_usage_bytes` - Memory consumption

---

## Logging

### Log Levels

| Level | Use Case |
|-------|----------|
| `error` | Critical failures (always logged) |
| `warn` | Slow queries, degraded performance |
| `info` | Operations (batch inserts, index rebuilds) |
| `debug` | Individual queries, detailed timing |
| `trace` | Internal debugging (not recommended in production) |

### Structured Fields

All logs include structured fields for filtering:

**Storage Operations:**
```json
{
  "level": "debug",
  "target": "omendb::redb_storage",
  "span": {"name": "point_query", "key": 12345},
  "fields": {"duration_ms": 5},
  "message": "Point query completed"
}
```

**Learned Index Training:**
```json
{
  "level": "info",
  "target": "omendb::index",
  "span": {"name": "train", "keys": 100000},
  "fields": {
    "duration_ms": 250,
    "models": 16,
    "avg_max_error": 8
  },
  "message": "Learned index training completed"
}
```

### Common Log Patterns

**Slow Queries (>100ms):**
```bash
grep '"Slow.*query detected"' logs.json | jq '.fields'
```

**Index Rebuilds:**
```bash
grep '"Index rebuild"' logs.json | jq '{duration: .fields.duration_ms, keys: .fields.keys}'
```

**High Prediction Errors:**
```bash
grep '"High prediction error"' logs.json | jq '.fields'
```

---

## Alerting Rules

### Critical Alerts

**High Error Rate:**
```yaml
- alert: HighSearchErrorRate
  expr: |
    rate(omendb_searches_failed_total[5m]) /
    rate(omendb_searches_total[5m]) > 0.01
  for: 5m
  annotations:
    summary: "Search error rate above 1%"
```

**Slow Queries:**
```yaml
- alert: SlowP95Latency
  expr: histogram_quantile(0.95, omendb_search_duration_seconds) > 0.1
  for: 10m
  annotations:
    summary: "P95 search latency above 100ms"
```

### Warning Alerts

**Learned Index Degraded:**
```yaml
- alert: LearnedIndexLowHitRate
  expr: |
    omendb_learned_index_hits_total /
    (omendb_learned_index_hits_total + omendb_learned_index_misses_total) < 0.9
  for: 15m
  annotations:
    summary: "Learned index hit rate below 90%"
```

**Memory Growth:**
```yaml
- alert: MemoryGrowth
  expr: deriv(omendb_memory_usage_bytes[1h]) > 1000000
  for: 30m
  annotations:
    summary: "Memory usage growing >1MB/hour"
```

---

## Troubleshooting

### Slow Queries

1. **Check metrics:**
   ```promql
   histogram_quantile(0.95, omendb_search_duration_seconds)
   ```

2. **Check logs:**
   ```bash
   grep "Slow.*query" logs.json | jq '.fields.duration_ms' | sort -n | tail -10
   ```

3. **Verify learned index:**
   ```promql
   omendb_learned_index_hits_total /
     (omendb_learned_index_hits_total + omendb_learned_index_misses_total)
   ```
   - If hit rate <90%, index may need retraining

### High Error Rates

1. **Identify error types:**
   ```bash
   grep "error" logs.json | jq -r '.message' | sort | uniq -c
   ```

2. **Check failed operations:**
   ```promql
   rate(omendb_searches_failed_total[5m])
   rate(omendb_inserts_failed_total[5m])
   ```

3. **Review error context:**
   ```bash
   grep '"level":"error"' logs.json | jq '.fields'
   ```

### Learned Index Issues

**Low Hit Rate (<90%):**
- Check if data distribution changed
- Verify prediction errors: `omendb_learned_index_prediction_error_positions`
- Consider manual retrain or wait for periodic retrain (every 10,000 keys)

**High Prediction Errors:**
```bash
# Check logs for high error warnings
grep "High prediction error" logs.json
```

**Causes:**
- Non-sequential key distribution
- Large gaps in key space
- Frequent deletes creating sparse index

**Solutions:**
- Increase error window size (default: ±100 positions)
- Retrain index after bulk operations
- Monitor `avg_max_error` metric

---

## Grafana Dashboards

### Overview Dashboard

**Panels:**
1. **Throughput**: `rate(omendb_searches_total[5m])`, `rate(omendb_inserts_total[5m])`
2. **Latency**: P50, P95, P99 of `omendb_search_duration_seconds`
3. **Error Rate**: Failed ops / Total ops
4. **Active Connections**: `omendb_active_connections`

### Learned Index Dashboard

**Panels:**
1. **Hit Rate**: Hits / (Hits + Misses)
2. **Prediction Accuracy**: `omendb_learned_index_prediction_error_positions`
3. **Query Path Distribution**: Pie chart of `omendb_query_path_total`
4. **Index Size**: `omendb_learned_index_size_keys`
5. **Model Count**: `omendb_learned_index_models_count`

### Example Panel JSON

```json
{
  "title": "Learned Index Hit Rate",
  "targets": [
    {
      "expr": "omendb_learned_index_hits_total / (omendb_learned_index_hits_total + omendb_learned_index_misses_total)",
      "legendFormat": "Hit Rate"
    }
  ],
  "yaxes": [{"min": 0, "max": 1, "format": "percentunit"}]
}
```

---

## Performance Baselines

### Expected Metrics (1M keys, sequential)

| Metric | Target | Warning | Critical |
|--------|--------|---------|----------|
| Search P95 | <10ms | >50ms | >100ms |
| Insert P95 | <20ms | >100ms | >200ms |
| Error Rate | <0.1% | >1% | >5% |
| Learned Index Hit Rate | >95% | <90% | <80% |
| Avg Prediction Error | <50 positions | >100 | >500 |

### Scaling Characteristics

| Dataset Size | Expected Models | Training Time | Hit Rate |
|--------------|----------------|---------------|----------|
| 10K keys | 4 | <10ms | >90% |
| 100K keys | 8 | <50ms | >92% |
| 1M keys | 16 | <250ms | >95% |
| 10M+ keys | 16 | <1s | >95% |

---

## Integration Examples

### Prometheus Configuration

```yaml
scrape_configs:
  - job_name: 'omendb'
    scrape_interval: 15s
    static_configs:
      - targets: ['localhost:9090']
```

### Loki Configuration

```yaml
clients:
  - url: http://loki:3100/loki/api/v1/push

pipeline_stages:
  - json:
      expressions:
        level: level
        message: message
        target: target
        duration_ms: fields.duration_ms
```

### Example Query

```bash
# Get metrics
curl -s http://localhost:9090/metrics | grep omendb_learned

# Get health
curl -s http://localhost:9090/health | jq .

# Stream logs
tail -f logs.json | jq 'select(.level == "warn")'
```

---

## Best Practices

### Monitoring

1. **Always monitor:**
   - Error rates (searches, inserts)
   - P95/P99 latency
   - Learned index hit rate

2. **Alert on:**
   - Error rate >1%
   - P95 latency >100ms
   - Hit rate <90%

3. **Review weekly:**
   - Slow query logs
   - Index rebuild frequency
   - Memory growth trends

### Logging

1. **Production log level**: `info` for general ops, `debug` for troubleshooting
2. **Retention**: 7+ days for metrics, 30+ days for critical logs
3. **Log rotation**: Use JSON format with external log aggregation (Loki, CloudWatch)

### Capacity Planning

1. **Monitor growth**:
   ```promql
   deriv(omendb_database_size_bytes[1d])
   ```

2. **Predict training cost**:
   - Training scales O(n log n) with dataset size
   - Expect 1 retrain per 10,000 inserts

3. **Memory estimate**:
   - Index: ~16 bytes per key
   - Storage: Variable (depends on value size)

---

## Support

**Metrics Not Updating?**
- Verify endpoint: `curl http://localhost:9090/metrics`
- Check Prometheus scrape config
- Ensure operations are happening (metrics only increment on use)

**Logs Not Appearing?**
- Check `RUST_LOG` environment variable
- Verify JSON format: `RUST_LOG=info,omendb=debug`
- Ensure logging initialized (calls `crate::logging::init_logging()`)

**High Learned Index Misses?**
- Normal for random access patterns
- Expected hit rate: 90-95% for sequential, 60-80% for random
- Check `omendb_learned_index_prediction_error_positions` histogram

---

**Last Updated**: January 2025
**OmenDB Version**: 0.1.0
