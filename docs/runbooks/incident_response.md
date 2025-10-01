# Incident Response Runbook

## 🚨 Emergency Response Procedures

**Primary Goal**: Minimize downtime and data loss while restoring service quickly.

## Severity Levels

| Level | Description | Response Time | Examples |
|-------|-------------|---------------|----------|
| **SEV1** | Service Down | 15 minutes | Complete outage, data corruption |
| **SEV2** | Degraded Service | 1 hour | High latency, partial failures |
| **SEV3** | Minor Issues | 4 hours | Non-critical features affected |

---

## SEV1: Service Down

### Immediate Actions (0-5 minutes)

1. **Alert Acknowledgment**
   ```bash
   # Acknowledge alert in monitoring system
   # Update status page if applicable
   ```

2. **Initial Assessment**
   ```bash
   # Check service status
   kubectl get pods -n omendb-prod

   # Check recent deployments
   kubectl rollout history deployment/prod-omendb -n omendb-prod

   # Review error logs
   kubectl logs -n omendb-prod -l app=omendb --tail=100
   ```

3. **Communications**
   - Post in `#incidents` Slack channel
   - Update status page: "Investigating service disruption"
   - Notify stakeholders via predetermined escalation path

### Diagnostic Phase (5-15 minutes)

#### Check Infrastructure
```bash
# Node health
kubectl get nodes

# Resource usage
kubectl top pods -n omendb-prod

# Recent events
kubectl get events -n omendb-prod --sort-by=.metadata.creationTimestamp
```

#### Check Application Health
```bash
# Health endpoints
curl -f http://load-balancer/health || echo "Health check failed"

# Database connectivity
kubectl exec -n omendb-prod deployment/prod-omendb -- \
    curl -f localhost:3000/ready || echo "DB not ready"

# Metrics endpoint
curl -f http://load-balancer:9090/metrics | grep -q "omendb_" || echo "Metrics failed"
```

#### Common Failure Patterns

**Pattern 1: Pod CrashLoopBackOff**
```bash
# Check pod status
kubectl describe pod -n omendb-prod -l app=omendb

# Check resource limits
kubectl get pods -n omendb-prod -o yaml | grep -A 10 resources

# Review recent configuration changes
git log --oneline -10 k8s/overlays/production/
```

**Pattern 2: Out of Memory**
```bash
# Check memory usage
kubectl top pods -n omendb-prod

# Look for OOMKilled events
kubectl get events -n omendb-prod | grep OOMKilled

# Emergency memory increase
kubectl patch deployment/prod-omendb -n omendb-prod -p \
  '{"spec":{"template":{"spec":{"containers":[{"name":"omendb","resources":{"limits":{"memory":"16Gi"}}}]}}}}'
```

**Pattern 3: Disk Full**
```bash
# Check PVC usage
kubectl exec -n omendb-prod deployment/prod-omendb -- df -h

# Clean temporary files
kubectl exec -n omendb-prod deployment/prod-omendb -- \
    rm -rf /tmp/* /var/log/omendb/*.log.old

# Emergency PVC expansion
kubectl patch pvc omendb-data -n omendb-prod -p \
  '{"spec":{"resources":{"requests":{"storage":"200Gi"}}}}'
```

### Recovery Actions

#### Quick Fixes (Try in order)

1. **Restart Pods**
   ```bash
   kubectl rollout restart deployment/prod-omendb -n omendb-prod
   kubectl rollout status deployment/prod-omendb -n omendb-prod --timeout=300s
   ```

2. **Scale Up Replicas**
   ```bash
   kubectl scale deployment/prod-omendb --replicas=5 -n omendb-prod
   ```

3. **Rollback Recent Deployment**
   ```bash
   kubectl rollout undo deployment/prod-omendb -n omendb-prod
   kubectl rollout status deployment/prod-omendb -n omendb-prod
   ```

4. **Emergency Restore from Backup**
   ```bash
   # Find latest backup
   LATEST_BACKUP=$(backup_tool --data-dir /tmp --backup-dir /var/backups/omendb \
       list --database production | head -n1 | awk '{print $1}')

   # Scale down to 0
   kubectl scale deployment/prod-omendb --replicas=0 -n omendb-prod

   # Restore data
   kubectl exec -it -n omendb-prod job/restore-job -- \
       backup_tool restore --backup-id "$LATEST_BACKUP" --force

   # Scale back up
   kubectl scale deployment/prod-omendb --replicas=3 -n omendb-prod
   ```

### Post-Recovery (15+ minutes)

1. **Verification**
   ```bash
   # Health checks
   ./health_check.sh

   # Basic functionality test
   curl -X POST http://load-balancer:3000/insert \
       -H "Content-Type: application/json" \
       -d '{"timestamp": 1000000, "value": 42.0, "series_id": 1}'

   # Query test
   curl "http://load-balancer:3000/query?start=999999&end=1000001"
   ```

2. **Communications**
   - Update status page: "Service restored"
   - Post resolution in `#incidents`
   - Schedule post-mortem meeting

---

## SEV2: Degraded Performance

### Assessment Checklist

- [ ] Query latency > 1 second for 95th percentile
- [ ] Error rate > 1%
- [ ] Memory usage > 90%
- [ ] CPU usage > 80%

### Immediate Actions

1. **Gather Performance Data**
   ```bash
   # Check metrics
   curl -s http://load-balancer:9090/metrics | grep -E "(duration|error|memory|cpu)"

   # Query statistics
   kubectl logs -n omendb-prod -l app=omendb | grep "duration" | tail -100
   ```

2. **Scale Horizontally**
   ```bash
   kubectl scale deployment/prod-omendb --replicas=5 -n omendb-prod
   ```

3. **Check Resource Bottlenecks**
   ```bash
   # CPU throttling
   kubectl top pods -n omendb-prod

   # Memory pressure
   kubectl describe nodes | grep -A 10 "Allocated resources"

   # Disk I/O
   kubectl exec -n omendb-prod deployment/prod-omendb -- iostat 1 3
   ```

### Performance Optimization

```bash
# Trigger index rebuild
curl -X POST http://load-balancer:3000/admin/rebuild-index

# Adjust batch sizes
kubectl patch configmap omendb-config -n omendb-prod -p \
  '{"data":{"batch_size":"25000"}}'

# Restart to apply changes
kubectl rollout restart deployment/prod-omendb -n omendb-prod
```

---

## SEV3: Minor Issues

### Non-Critical Failures

- Authentication service timeouts
- Metrics collection gaps
- Non-essential feature failures

### Standard Response

1. **Document Issue**
   - Create ticket in issue tracker
   - Add to technical debt backlog
   - Schedule fix for next maintenance window

2. **Monitor for Escalation**
   - Set up temporary alerts
   - Check if issue affects more users
   - Prepare escalation plan

---

## Post-Incident Procedures

### Immediate (Within 24 hours)

1. **Root Cause Analysis**
   - Timeline of events
   - Contributing factors
   - Detection gaps

2. **Data Integrity Check**
   ```bash
   # Verify backup integrity
   backup_tool verify --backup-id latest

   # Check data consistency
   curl http://load-balancer:3000/admin/integrity-check
   ```

### Follow-up (Within 1 week)

1. **Post-Mortem Meeting**
   - What went well?
   - What could be improved?
   - Action items with owners

2. **Update Procedures**
   - Update runbooks
   - Improve monitoring
   - Add preventive measures

### Action Items Template

```markdown
## Post-Incident Action Items

### Prevention
- [ ] Add monitoring for X
- [ ] Implement circuit breaker for Y
- [ ] Update capacity planning

### Detection
- [ ] Create alert for early warning
- [ ] Improve log aggregation
- [ ] Add health check for Z

### Response
- [ ] Update runbook with new procedure
- [ ] Train team on new tools
- [ ] Test disaster recovery plan
```

---

## Emergency Contacts

| Escalation Level | Contact | Phone | Slack |
|------------------|---------|-------|-------|
| L1 - On-Call Engineer | Current rotation | +1-555-0100 | @oncall |
| L2 - Team Lead | Alice Johnson | +1-555-0101 | @alice.johnson |
| L3 - Engineering Manager | Bob Smith | +1-555-0102 | @bob.smith |
| L4 - VP Engineering | Carol Davis | +1-555-0103 | @carol.davis |

### External Vendors

| Service | Contact | Account ID |
|---------|---------|------------|
| Cloud Provider | +1-800-SUPPORT | ACC-12345 |
| Monitoring | support@datadog.com | org-67890 |
| Security | security@company.com | INT-11111 |

---

## Quick Reference Commands

```bash
# Service status
kubectl get pods -n omendb-prod -l app=omendb

# Recent logs
kubectl logs -n omendb-prod -l app=omendb --tail=50 --timestamps

# Restart service
kubectl rollout restart deployment/prod-omendb -n omendb-prod

# Scale service
kubectl scale deployment/prod-omendb --replicas=N -n omendb-prod

# Emergency rollback
kubectl rollout undo deployment/prod-omendb -n omendb-prod

# Health check
curl -f http://load-balancer:3000/health

# Manual backup
backup_tool full-backup --database production

# Restore from backup
backup_tool restore --backup-id BACKUP_ID --force
```