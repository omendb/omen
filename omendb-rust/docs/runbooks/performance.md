# Performance Troubleshooting Runbook

## 🔍 Performance Analysis & Optimization

**Purpose**: Systematic approach to diagnosing and resolving OmenDB performance issues.

## Performance Baseline

### Expected Performance Metrics

| Metric | Target | Alert Threshold |
|--------|--------|-----------------|
| Query Latency (p95) | < 100ms | > 500ms |
| Insert Latency (p95) | < 50ms | > 200ms |
| Throughput | > 10K QPS | < 5K QPS |
| Memory Usage | < 80% | > 90% |
| CPU Usage | < 70% | > 85% |
| Error Rate | < 0.1% | > 1% |

---

## Diagnosis Framework

### 1. Identify Performance Bottleneck

```bash
#!/bin/bash
# performance_check.sh

echo "🔍 OmenDB Performance Analysis"
echo "=============================="

# System resources
echo "📊 System Resources:"
kubectl top pods -n omendb-prod

# Query performance
echo "⚡ Query Performance:"
curl -s http://load-balancer:9090/metrics | grep omendb_query_duration_seconds | tail -5

# Insert performance
echo "📝 Insert Performance:"
curl -s http://load-balancer:9090/metrics | grep omendb_insert_duration_seconds | tail -5

# Error rates
echo "❌ Error Rates:"
curl -s http://load-balancer:9090/metrics | grep -E "(failures|errors)_total"

# Connection stats
echo "🔗 Connections:"
curl -s http://load-balancer:9090/metrics | grep omendb_active_connections

# Index efficiency
echo "📚 Index Performance:"
curl -s http://load-balancer:3000/admin/index-stats
```

### 2. Resource Utilization Analysis

#### CPU Analysis
```bash
# Check CPU utilization
kubectl exec -n omendb-prod deployment/prod-omendb -- top -bn1 | head -20

# CPU throttling detection
kubectl describe pods -n omendb-prod -l app=omendb | grep -A 5 -B 5 "cpu"

# Process-level CPU usage
kubectl exec -n omendb-prod deployment/prod-omendb -- \
    ps aux --sort=-%cpu | head -10
```

#### Memory Analysis
```bash
# Memory usage breakdown
kubectl exec -n omendb-prod deployment/prod-omendb -- free -h

# Memory pressure indicators
kubectl describe nodes | grep -A 10 "Allocated resources"

# OomenDB memory metrics
curl -s http://load-balancer:9090/metrics | grep omendb_memory
```

#### I/O Analysis
```bash
# Disk I/O statistics
kubectl exec -n omendb-prod deployment/prod-omendb -- iostat -x 1 3

# Network I/O
kubectl exec -n omendb-prod deployment/prod-omendb -- \
    netstat -i

# Storage performance
kubectl exec -n omendb-prod deployment/prod-omendb -- \
    dd if=/dev/zero of=/tmp/test bs=1M count=100 oflag=direct
```

---

## Common Performance Issues

### Issue 1: High Query Latency

**Symptoms:**
- p95 query latency > 500ms
- Timeouts in application logs
- User complaints about slow responses

**Diagnosis:**
```bash
# Check query patterns
kubectl logs -n omendb-prod -l app=omendb | grep "query" | \
    awk '{print $NF}' | sort -n | tail -20

# Identify slow queries
curl -s http://load-balancer:9090/metrics | \
    grep omendb_query_duration_seconds_bucket | \
    awk -F'le="' '{print $2}' | awk -F'"' '{print $1}' | sort -n

# Index performance
curl http://load-balancer:3000/admin/index-stats | jq '.efficiency'
```

**Solutions:**

1. **Optimize Learned Index**
   ```bash
   # Retrain with current data distribution
   curl -X POST http://load-balancer:3000/admin/retrain-index

   # Adjust index parameters
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"max_error":"50","retrain_threshold":"500000"}}'
   ```

2. **Scale Horizontally**
   ```bash
   # Add read replicas
   kubectl scale deployment/prod-omendb --replicas=5 -n omendb-prod

   # Implement read/write splitting
   # (Application-level change required)
   ```

3. **Cache Optimization**
   ```bash
   # Increase prefetch size
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"prefetch_size":"4000"}}'

   # Enable SIMD optimizations
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"enable_simd":"true"}}'
   ```

### Issue 2: High Insert Latency

**Symptoms:**
- Insert operations taking > 200ms
- WAL flush delays
- Memory pressure during bulk inserts

**Diagnosis:**
```bash
# WAL performance
ls -la /var/lib/omendb/wal/ | tail -10
du -sh /var/lib/omendb/wal/

# Batch processing efficiency
curl -s http://load-balancer:9090/metrics | grep batch_size

# Storage I/O wait
kubectl exec -n omendb-prod deployment/prod-omendb -- \
    iostat -x | grep -E "(Device|nvme|sda)"
```

**Solutions:**

1. **Optimize WAL Settings**
   ```bash
   # Increase batch size
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"batch_size":"100000"}}'

   # Adjust sync interval
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"sync_interval_ms":"10000"}}'
   ```

2. **Storage Optimization**
   ```bash
   # Upgrade to faster storage class
   kubectl patch pvc omendb-data -n omendb-prod -p \
     '{"spec":{"storageClassName":"premium-ssd"}}'

   # Enable compression for better I/O
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"enable_compression":"true"}}'
   ```

### Issue 3: Memory Pressure

**Symptoms:**
- Memory usage > 90%
- OOMKilled pods
- Garbage collection pauses

**Diagnosis:**
```bash
# Memory breakdown
kubectl exec -n omendb-prod deployment/prod-omendb -- \
    cat /proc/meminfo | grep -E "(MemTotal|MemAvailable|MemFree)"

# OmenDB memory usage
curl -s http://load-balancer:9090/metrics | \
    grep omendb_memory_usage_bytes

# Check for memory leaks
kubectl exec -n omendb-prod deployment/prod-omendb -- \
    ps -p $(pgrep omendb) -o pid,vsz,rss,pcpu,pmem,comm
```

**Solutions:**

1. **Immediate Relief**
   ```bash
   # Increase memory limits
   kubectl patch deployment/prod-omendb -n omendb-prod -p \
     '{"spec":{"template":{"spec":{"containers":[{"name":"omendb","resources":{"limits":{"memory":"16Gi"}}}]}}}}'

   # Trigger garbage collection
   curl -X POST http://load-balancer:3000/admin/gc
   ```

2. **Long-term Optimization**
   ```bash
   # Reduce memory footprint
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"max_memory_mb":"12288"}}'

   # Optimize batch processing
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"batch_size":"50000"}}'
   ```

### Issue 4: Connection Pool Exhaustion

**Symptoms:**
- Connection refused errors
- High connection count
- Client timeouts

**Diagnosis:**
```bash
# Current connections
curl -s http://load-balancer:9090/metrics | grep omendb_active_connections

# Connection pool settings
kubectl get configmap omendb-config -n omendb-prod -o yaml | grep max_connections

# Network connections
kubectl exec -n omendb-prod deployment/prod-omendb -- \
    netstat -an | grep :3000 | wc -l
```

**Solutions:**

1. **Increase Connection Limits**
   ```bash
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"max_connections":"2000"}}'
   ```

2. **Connection Pooling**
   ```bash
   # Implement connection pooling in application
   # Configure connection timeouts
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"session_timeout_seconds":"1800"}}'
   ```

---

## Performance Optimization Strategies

### 1. Index Tuning

```bash
# Analyze query patterns
QUERY_LOG="/tmp/query_analysis.log"
kubectl logs -n omendb-prod -l app=omendb | grep "query" > $QUERY_LOG

# Most frequent query ranges
awk '{print $5, $6}' $QUERY_LOG | sort | uniq -c | sort -nr | head -10

# Optimal index parameters based on data
RECORD_COUNT=$(curl -s http://load-balancer:3000/admin/stats | jq '.record_count')
OPTIMAL_ERROR=$((RECORD_COUNT / 10000))

kubectl patch configmap omendb-config -n omendb-prod -p \
  "{\"data\":{\"max_error\":\"$OPTIMAL_ERROR\"}}"
```

### 2. Resource Right-Sizing

```bash
# Calculate optimal resource allocation
CURRENT_MEMORY=$(kubectl top pods -n omendb-prod -l app=omendb --no-headers | \
    awk '{sum+=$3} END {print sum "Mi"}')

CURRENT_CPU=$(kubectl top pods -n omendb-prod -l app=omendb --no-headers | \
    awk '{sum+=$2} END {print sum "m"}')

echo "Current usage: CPU=$CURRENT_CPU, Memory=$CURRENT_MEMORY"

# Recommended sizing (add 30% headroom)
RECOMMENDED_MEMORY=$((${CURRENT_MEMORY%Mi} * 130 / 100))
RECOMMENDED_CPU=$((${CURRENT_CPU%m} * 130 / 100))

echo "Recommended: CPU=${RECOMMENDED_CPU}m, Memory=${RECOMMENDED_MEMORY}Mi"
```

### 3. Storage Optimization

```bash
# Analyze storage usage patterns
kubectl exec -n omendb-prod deployment/prod-omendb -- \
    find /var/lib/omendb -name "*.parquet" -exec ls -lh {} \; | \
    awk '{print $5}' | sort | uniq -c

# Compression efficiency
kubectl exec -n omendb-prod deployment/prod-omendb -- \
    du -sh /var/lib/omendb/data/

# WAL size trends
kubectl exec -n omendb-prod deployment/prod-omendb -- \
    ls -lht /var/lib/omendb/wal/ | head -20
```

---

## Performance Testing

### Load Testing Setup

```bash
#!/bin/bash
# load_test.sh

# Generate synthetic load
for i in {1..1000}; do
    curl -X POST http://load-balancer:3000/insert \
        -H "Content-Type: application/json" \
        -d "{\"timestamp\": $((1000000 + i)), \"value\": $RANDOM, \"series_id\": 1}" &

    if [ $((i % 100)) -eq 0 ]; then
        wait
        echo "Completed $i inserts"
    fi
done

# Query load
for i in {1..100}; do
    START=$((1000000 + RANDOM % 900))
    END=$((START + 100))
    curl "http://load-balancer:3000/query?start=$START&end=$END" > /dev/null &
done

wait
echo "Load test completed"
```

### Performance Benchmarking

```bash
#!/bin/bash
# benchmark.sh

echo "🏃 Performance Benchmark"
echo "======================="

# Insert performance
echo "Testing insert performance..."
time_insert=$(curl -w "%{time_total}" -o /dev/null -s \
    -X POST http://load-balancer:3000/insert \
    -H "Content-Type: application/json" \
    -d '{"timestamp": 2000000, "value": 42.0, "series_id": 1}')

echo "Insert latency: ${time_insert}s"

# Query performance
echo "Testing query performance..."
time_query=$(curl -w "%{time_total}" -o /dev/null -s \
    "http://load-balancer:3000/query?start=1999999&end=2000001")

echo "Query latency: ${time_query}s"

# Throughput test
echo "Testing throughput..."
start_time=$(date +%s)
for i in {1..1000}; do
    curl -s -X POST http://load-balancer:3000/insert \
        -H "Content-Type: application/json" \
        -d "{\"timestamp\": $((3000000 + i)), \"value\": $i, \"series_id\": 1}" > /dev/null
done
end_time=$(date +%s)

duration=$((end_time - start_time))
throughput=$((1000 / duration))

echo "Throughput: ${throughput} inserts/second"
```

---

## Monitoring & Alerting Setup

### Key Performance Metrics

```yaml
# prometheus-rules.yml
groups:
- name: omendb-performance
  rules:
  # Query performance
  - alert: HighQueryLatency
    expr: histogram_quantile(0.95, omendb_query_duration_seconds_bucket) > 0.5
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "Query latency is high ({{ $value }}s)"

  # Insert performance
  - alert: HighInsertLatency
    expr: histogram_quantile(0.95, omendb_insert_duration_seconds_bucket) > 0.2
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "Insert latency is high ({{ $value }}s)"

  # Throughput
  - alert: LowThroughput
    expr: rate(omendb_inserts_total[5m]) < 5000
    for: 5m
    labels:
      severity: warning
    annotations:
      summary: "Insert throughput is low ({{ $value }} ops/sec)"

  # Error rate
  - alert: HighErrorRate
    expr: rate(omendb_insert_failures_total[5m]) / rate(omendb_inserts_total[5m]) > 0.01
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "Error rate is high ({{ $value | humanizePercentage }})"
```

### Performance Dashboard

```json
{
  "dashboard": {
    "title": "OmenDB Performance",
    "panels": [
      {
        "title": "Query Latency",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, omendb_query_duration_seconds_bucket)"
          }
        ]
      },
      {
        "title": "Insert Throughput",
        "targets": [
          {
            "expr": "rate(omendb_inserts_total[5m])"
          }
        ]
      },
      {
        "title": "Resource Utilization",
        "targets": [
          {
            "expr": "omendb_memory_usage_bytes / omendb_memory_limit_bytes"
          }
        ]
      }
    ]
  }
}
```

---

## Quick Reference

### Performance Commands

```bash
# Real-time performance monitoring
watch -n 1 'kubectl top pods -n omendb-prod'

# Query latency check
curl -s http://load-balancer:9090/metrics | grep query_duration | tail -1

# Memory usage
curl -s http://load-balancer:9090/metrics | grep memory_usage_bytes

# Index statistics
curl http://load-balancer:3000/admin/index-stats

# Trigger index rebuild
curl -X POST http://load-balancer:3000/admin/retrain-index

# Scale replicas
kubectl scale deployment/prod-omendb --replicas=N -n omendb-prod

# Update configuration
kubectl patch configmap omendb-config -n omendb-prod -p '{"data":{"KEY":"VALUE"}}'
```

### Performance Tuning Checklist

- [ ] Monitor p95 latencies
- [ ] Check resource utilization
- [ ] Analyze query patterns
- [ ] Optimize index parameters
- [ ] Review storage performance
- [ ] Validate connection pooling
- [ ] Test under load
- [ ] Document changes