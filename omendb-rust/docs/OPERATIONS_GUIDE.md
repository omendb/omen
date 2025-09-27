# OmenDB Operations Guide

## 🚀 Production Operations Manual

This guide provides comprehensive operational procedures for running OmenDB in production environments.

## Table of Contents

1. [System Requirements](#system-requirements)
2. [Installation & Deployment](#installation--deployment)
3. [Configuration Management](#configuration-management)
4. [Monitoring & Alerting](#monitoring--alerting)
5. [Backup & Recovery](#backup--recovery)
6. [Performance Tuning](#performance-tuning)
7. [Troubleshooting](#troubleshooting)
8. [Security Operations](#security-operations)
9. [Disaster Recovery](#disaster-recovery)
10. [Maintenance Procedures](#maintenance-procedures)

---

## System Requirements

### Minimum Requirements
- **CPU**: 4 cores, 2.4 GHz
- **Memory**: 8 GB RAM
- **Storage**: 100 GB SSD
- **OS**: Linux (Ubuntu 20.04+, RHEL 8+)
- **Network**: 1 Gbps

### Recommended Production
- **CPU**: 16 cores, 3.0 GHz
- **Memory**: 32 GB RAM
- **Storage**: 500 GB NVMe SSD
- **OS**: Ubuntu 22.04 LTS
- **Network**: 10 Gbps

### Capacity Planning
```bash
# Estimate storage requirements
Data Size = (Expected Records × 24 bytes) × 1.3 (compression overhead)
WAL Size = Data Size × 0.1 (10% of data)
Backup Size = (Data Size + WAL Size) × 0.6 (compression ratio)
Total = Data Size + WAL Size + Backup Size × 3 (retention)
```

---

## Installation & Deployment

### Quick Start - Binary Installation

```bash
# Download latest release
wget https://github.com/omendb/releases/latest/omendb-linux-x86_64.tar.gz
tar -xzf omendb-linux-x86_64.tar.gz
sudo mv omendb /usr/local/bin/

# Create system user
sudo useradd -r -s /bin/false omendb
sudo mkdir -p /var/lib/omendb/{data,wal,backups}
sudo chown -R omendb:omendb /var/lib/omendb
```

### Docker Deployment

```bash
# Pull image
docker pull omendb/omendb:latest

# Run with persistence
docker run -d \
  --name omendb \
  -p 3000:3000 \
  -p 9090:9090 \
  -v omendb-data:/var/lib/omendb/data \
  -v omendb-wal:/var/lib/omendb/wal \
  -e OMENDB_AUTH_DISABLED=false \
  omendb/omendb:latest
```

### Kubernetes Deployment

```bash
# Deploy to production
cd k8s
./deploy.sh production

# Verify deployment
kubectl get pods -n omendb-prod
kubectl logs -n omendb-prod -l app=omendb
```

---

## Configuration Management

### Core Configuration (`omendb.toml`)

```toml
[database]
name = "omendb_production"
data_dir = "/var/lib/omendb/data"
max_memory_mb = 8192
expected_size = 100000000

[storage]
enable_compression = true
batch_size = 100000
sync_interval_ms = 5000

[index]
model_type = "recursive"
max_error = 50
retrain_threshold = 1000000

[wal]
enabled = true
sync_on_write = true
max_file_size_mb = 100
rotation_interval_hours = 24

[http]
host = "0.0.0.0"
port = 3000
request_timeout_ms = 30000
max_concurrent_requests = 1000

[security]
auth_enabled = true
session_timeout_seconds = 3600
tls_enabled = true

[monitoring]
prometheus_enabled = true
prometheus_port = 9090
health_check_interval_ms = 10000
log_level = "info"
log_format = "json"

[performance]
worker_threads = 8
io_threads = 4
enable_simd = true
prefetch_size = 2000

[limits]
max_connections = 1000
max_query_duration_ms = 30000
max_insert_batch_size = 50000
memory_limit_mb = 8192
```

### Environment Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `OMENDB_DATA_DIR` | Data directory path | `/var/lib/omendb/data` |
| `OMENDB_HTTP_PORT` | HTTP server port | `3000` |
| `OMENDB_AUTH_DISABLED` | Disable authentication | `false` |
| `OMENDB_LOG_LEVEL` | Logging level | `info` |
| `RUST_LOG` | Rust logging filter | `info` |

---

## Monitoring & Alerting

### Prometheus Metrics

Key metrics to monitor:

```promql
# Query Performance
omendb_query_duration_seconds_bucket
omendb_searches_total
omendb_search_failures_total

# Insert Performance
omendb_insert_duration_seconds_bucket
omendb_inserts_total
omendb_insert_failures_total

# System Health
omendb_memory_usage_bytes
omendb_active_connections
omendb_wal_size_bytes
omendb_index_size_bytes
```

### Critical Alerts

```yaml
# alerts.yml
groups:
- name: omendb
  rules:
  - alert: OmenDBDown
    expr: up{job="omendb"} == 0
    for: 30s
    labels:
      severity: critical
    annotations:
      summary: "OmenDB instance is down"

  - alert: HighQueryLatency
    expr: histogram_quantile(0.95, omendb_query_duration_seconds_bucket) > 1.0
    for: 2m
    labels:
      severity: warning
    annotations:
      summary: "95th percentile query latency > 1s"

  - alert: HighMemoryUsage
    expr: omendb_memory_usage_bytes / omendb_memory_limit_bytes > 0.9
    for: 5m
    labels:
      severity: critical
    annotations:
      summary: "Memory usage > 90%"

  - alert: WALSizeGrowth
    expr: increase(omendb_wal_size_bytes[1h]) > 1e9
    for: 15m
    labels:
      severity: warning
    annotations:
      summary: "WAL growing faster than 1GB/hour"
```

### Health Checks

```bash
#!/bin/bash
# health_check.sh

# Basic connectivity
curl -f http://localhost:3000/health || exit 1

# Detailed status
curl -f http://localhost:3000/ready || exit 1

# Metrics endpoint
curl -f http://localhost:9090/metrics | grep -q "omendb_" || exit 1

echo "✅ All health checks passed"
```

---

## Backup & Recovery

### Automated Backup Strategy

```bash
#!/bin/bash
# backup_schedule.sh

set -euo pipefail

BACKUP_DIR="/var/backups/omendb"
DATA_DIR="/var/lib/omendb"
RETENTION_DAYS=30

# Full backup on Sundays
if [ "$(date +%u)" -eq 7 ]; then
    /usr/local/bin/backup_tool \
        --data-dir "$DATA_DIR" \
        --backup-dir "$BACKUP_DIR" \
        full-backup --database production
else
    # Incremental backup on other days
    /usr/local/bin/backup_tool \
        --data-dir "$DATA_DIR" \
        --backup-dir "$BACKUP_DIR" \
        incremental-backup --database production
fi

# Cleanup old backups
/usr/local/bin/backup_tool \
    --data-dir "$DATA_DIR" \
    --backup-dir "$BACKUP_DIR" \
    cleanup --retention-days "$RETENTION_DAYS"

echo "✅ Backup completed successfully"
```

### Disaster Recovery Procedure

```bash
#!/bin/bash
# disaster_recovery.sh

# 1. Stop all OmenDB instances
kubectl scale deployment/prod-omendb --replicas=0 -n omendb-prod

# 2. Find latest backup
LATEST_BACKUP=$(backup_tool --data-dir /tmp --backup-dir "$BACKUP_DIR" \
    list --database production | head -n1 | awk '{print $1}')

# 3. Restore from backup
backup_tool --data-dir "$DATA_DIR" --backup-dir "$BACKUP_DIR" \
    restore --backup-id "$LATEST_BACKUP" --force

# 4. Restart OmenDB
kubectl scale deployment/prod-omendb --replicas=3 -n omendb-prod

# 5. Verify recovery
kubectl wait --for=condition=ready pod -l app=omendb -n omendb-prod --timeout=300s

echo "✅ Disaster recovery completed"
```

---

## Performance Tuning

### Index Optimization

```toml
# For high-throughput workloads
[index]
model_type = "recursive"
max_error = 100      # Higher error for better performance
retrain_threshold = 500000  # Less frequent retraining

# For high-accuracy workloads
[index]
model_type = "recursive"
max_error = 10       # Lower error for better accuracy
retrain_threshold = 100000   # More frequent retraining
```

### Memory Tuning

```bash
# Calculate optimal memory settings
TOTAL_RAM=$(free -m | awk '/^Mem:/{print $2}')
OMENDB_MEMORY=$((TOTAL_RAM * 75 / 100))  # 75% of total RAM

# Update configuration
sed -i "s/max_memory_mb = .*/max_memory_mb = $OMENDB_MEMORY/" omendb.toml
```

### Network Optimization

```bash
# Increase connection limits
echo 'net.core.somaxconn = 4096' >> /etc/sysctl.conf
echo 'net.ipv4.tcp_max_syn_backlog = 4096' >> /etc/sysctl.conf
sysctl -p
```

---

## Troubleshooting

### Common Issues

#### 1. High Memory Usage

**Symptoms:**
- Memory usage alerts
- OOMKilled pods in Kubernetes
- Slow query performance

**Diagnosis:**
```bash
# Check memory metrics
curl -s http://localhost:9090/metrics | grep omendb_memory

# Check for memory leaks
top -p $(pgrep omendb)
```

**Resolution:**
```bash
# Reduce memory limits temporarily
kubectl patch deployment/prod-omendb -n omendb-prod -p \
  '{"spec":{"template":{"spec":{"containers":[{"name":"omendb","resources":{"limits":{"memory":"6Gi"}}}]}}}}'

# Trigger index rebuild to free memory
curl -X POST http://localhost:3000/admin/rebuild-index
```

#### 2. WAL Corruption

**Symptoms:**
- WAL recovery failures on startup
- Data inconsistency errors
- Insert failures

**Diagnosis:**
```bash
# Check WAL integrity
backup_tool --data-dir /var/lib/omendb --backup-dir /tmp \
    verify --backup-id latest

# Examine WAL files
ls -la /var/lib/omendb/wal/
```

**Resolution:**
```bash
# Restore from latest backup
backup_tool --data-dir /var/lib/omendb --backup-dir /var/backups/omendb \
    restore --backup-id $(get_latest_backup_id) --force
```

#### 3. Performance Degradation

**Symptoms:**
- High query latency
- Timeout errors
- CPU saturation

**Diagnosis:**
```bash
# Check query patterns
curl -s http://localhost:9090/metrics | grep duration_seconds_bucket

# Analyze index performance
curl http://localhost:3000/admin/index-stats
```

**Resolution:**
```bash
# Retrain learned index
curl -X POST http://localhost:3000/admin/retrain-index

# Scale horizontally
kubectl scale deployment/prod-omendb --replicas=5 -n omendb-prod
```

### Log Analysis

```bash
# Real-time log monitoring
kubectl logs -f -n omendb-prod -l app=omendb

# Error pattern analysis
kubectl logs -n omendb-prod -l app=omendb --since=1h | \
    grep -E "(ERROR|WARN)" | sort | uniq -c | sort -nr

# Performance log analysis
kubectl logs -n omendb-prod -l app=omendb --since=1h | \
    grep "duration" | awk '{print $NF}' | sort -n
```

---

## Security Operations

### TLS Certificate Management

```bash
# Generate certificates
openssl req -new -x509 -keyout server.key -out server.pem -days 365 -nodes

# Update Kubernetes secret
kubectl create secret tls omendb-tls \
    --cert=server.pem --key=server.key -n omendb-prod
```

### Access Control

```bash
# Add new user
htpasswd -c /etc/omendb/users.htpasswd newuser

# Update user permissions
kubectl create configmap omendb-users \
    --from-file=/etc/omendb/users.htpasswd -n omendb-prod
```

### Security Auditing

```bash
# Audit authentication attempts
kubectl logs -n omendb-prod -l app=omendb | grep "auth"

# Check for unauthorized access
kubectl logs -n omendb-prod -l app=omendb | grep "401\|403"
```

---

## Disaster Recovery

### Recovery Time Objectives (RTO)

| Scenario | Target RTO | Target RPO |
|----------|------------|------------|
| Pod restart | < 2 minutes | 0 |
| Node failure | < 5 minutes | < 1 minute |
| Zone failure | < 15 minutes | < 5 minutes |
| Region failure | < 1 hour | < 15 minutes |

### Disaster Recovery Checklist

#### Data Center Failure
1. ☐ Activate backup data center
2. ☐ Deploy OmenDB in backup region
3. ☐ Restore from latest backup
4. ☐ Update DNS to point to new region
5. ☐ Verify data integrity
6. ☐ Resume normal operations

#### Corruption Recovery
1. ☐ Stop all write operations
2. ☐ Assess corruption scope
3. ☐ Identify latest clean backup
4. ☐ Restore from backup
5. ☐ Replay missing transactions
6. ☐ Validate data consistency

---

## Maintenance Procedures

### Routine Maintenance

#### Daily
- ☐ Check system health dashboards
- ☐ Review error logs
- ☐ Verify backup completion
- ☐ Monitor performance metrics

#### Weekly
- ☐ Index performance analysis
- ☐ Capacity planning review
- ☐ Security audit review
- ☐ Update documentation

#### Monthly
- ☐ Full backup verification
- ☐ Disaster recovery testing
- ☐ Performance benchmarking
- ☐ Security patches

### Upgrade Procedures

```bash
#!/bin/bash
# upgrade_omendb.sh

VERSION="$1"

# 1. Create backup
backup_tool --data-dir /var/lib/omendb --backup-dir /var/backups \
    full-backup --database production

# 2. Rolling update
kubectl set image deployment/prod-omendb -n omendb-prod \
    omendb=omendb:$VERSION

# 3. Wait for rollout
kubectl rollout status deployment/prod-omendb -n omendb-prod

# 4. Verify deployment
kubectl get pods -n omendb-prod -l app=omendb

# 5. Run health checks
./health_check.sh

echo "✅ Upgrade to $VERSION completed"
```

---

## Appendix

### Configuration Templates

- [Development Config](configs/development.toml)
- [Staging Config](configs/staging.toml)
- [Production Config](configs/production.toml)

### Runbook Templates

- [Incident Response](runbooks/incident_response.md)
- [Performance Issues](runbooks/performance.md)
- [Security Incidents](runbooks/security.md)

### Contact Information

| Role | Contact | Escalation |
|------|---------|------------|
| Primary On-Call | +1-555-0100 | Slack: #omendb-oncall |
| Database Team | db-team@company.com | Manager: +1-555-0101 |
| Security Team | security@company.com | CISO: +1-555-0102 |

---

*This operations guide is maintained by the OmenDB team. Last updated: September 2025*