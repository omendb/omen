# Security Incident Response Runbook

## 🔒 Security Operations & Incident Response

**Purpose**: Systematic approach to detecting, responding to, and recovering from security incidents in OmenDB.

## Threat Model

### Attack Vectors

| Vector | Risk Level | Examples |
|--------|------------|----------|
| Unauthorized Access | High | Credential compromise, privilege escalation |
| Data Breach | Critical | Sensitive data exposure, exfiltration |
| Injection Attacks | Medium | SQL injection, command injection |
| DoS/DDoS | Medium | Resource exhaustion, service disruption |
| Insider Threats | High | Malicious employee, compromised accounts |

---

## Security Monitoring

### Real-time Security Monitoring

```bash
#!/bin/bash
# security_monitor.sh

echo "🔍 OmenDB Security Status Check"
echo "==============================="

# Authentication failures
echo "🔐 Authentication Failures (Last hour):"
kubectl logs -n omendb-prod -l app=omendb --since=1h | \
    grep -E "(401|403|Unauthorized|Forbidden)" | wc -l

# Suspicious query patterns
echo "⚠️  Suspicious Query Patterns:"
kubectl logs -n omendb-prod -l app=omendb --since=1h | \
    grep -E "(DROP|DELETE|ALTER|UNION|SELECT.*FROM|INSERT.*INTO)" | head -5

# Failed connection attempts
echo "🚫 Failed Connections:"
kubectl logs -n omendb-prod -l app=omendb --since=1h | \
    grep -E "(connection.*failed|timeout|refused)" | wc -l

# Unusual traffic patterns
echo "📊 Traffic Analysis:"
kubectl logs -n omendb-prod -l app=omendb --since=1h | \
    awk '{print $1}' | sort | uniq -c | sort -nr | head -10

# Certificate status
echo "📜 TLS Certificate Status:"
kubectl get secret omendb-tls -n omendb-prod -o yaml | \
    grep -A 1 "tls.crt" | tail -1 | base64 -d | \
    openssl x509 -noout -dates
```

### Security Metrics Dashboard

```promql
# Failed authentication rate
rate(omendb_auth_failures_total[5m])

# Suspicious query patterns
rate(omendb_suspicious_queries_total[5m])

# Connection anomalies
rate(omendb_connection_errors_total[5m])

# Data access patterns
rate(omendb_data_access_total[5m]) by (client_ip)
```

---

## Incident Classification

### Security Incident Levels

| Level | Description | Response Time | Examples |
|-------|-------------|---------------|----------|
| **SEC1** | Active breach | Immediate | Data exfiltration, admin compromise |
| **SEC2** | Potential compromise | 15 minutes | Suspicious access patterns |
| **SEC3** | Security violation | 1 hour | Failed login attempts, policy violations |

---

## SEC1: Active Security Breach

### Immediate Response (0-5 minutes)

1. **Incident Declaration**
   ```bash
   # Alert security team
   echo "SEC1 INCIDENT: Active breach detected in OmenDB" | \
       slack-cli -t $SECURITY_CHANNEL

   # Start incident timeline
   echo "$(date): SEC1 incident declared" >> /var/log/security/incident.log
   ```

2. **Isolate Affected Systems**
   ```bash
   # Emergency network isolation
   kubectl patch networkpolicy deny-all -n omendb-prod -p \
     '{"spec":{"podSelector":{"matchLabels":{"app":"omendb"}},"policyTypes":["Ingress","Egress"]}}'

   # Scale down to single instance
   kubectl scale deployment/prod-omendb --replicas=1 -n omendb-prod

   # Block external access
   kubectl patch service prod-omendb -n omendb-prod -p \
     '{"spec":{"type":"ClusterIP"}}'
   ```

3. **Preserve Evidence**
   ```bash
   # Capture current state
   kubectl get pods -n omendb-prod -o yaml > /tmp/breach-evidence-pods.yaml
   kubectl logs -n omendb-prod -l app=omendb --all-containers=true > /tmp/breach-evidence-logs.txt

   # Memory dump (if possible)
   kubectl exec -n omendb-prod deployment/prod-omendb -- \
     gcore $(pgrep omendb) > /tmp/breach-evidence-memory.dump

   # Network connections
   kubectl exec -n omendb-prod deployment/prod-omendb -- \
     netstat -tupln > /tmp/breach-evidence-network.txt
   ```

### Investigation (5-30 minutes)

#### Data Access Analysis
```bash
# Identify potentially compromised data
kubectl logs -n omendb-prod -l app=omendb | \
    grep -E "(query|select)" | \
    awk '{print $NF, $(NF-1), $(NF-2)}' | \
    sort | uniq -c | sort -nr > /tmp/data-access-analysis.txt

# Check for data exfiltration patterns
kubectl logs -n omendb-prod -l app=omendb | \
    grep -E "(large.*response|bulk.*export|download)" | \
    head -20
```

#### Attack Vector Analysis
```bash
# Check authentication bypass attempts
kubectl logs -n omendb-prod -l app=omendb | \
    grep -E "(auth.*bypass|token.*invalid|session.*hijack)"

# SQL injection patterns
kubectl logs -n omendb-prod -l app=omendb | \
    grep -E "('.*OR.*'|UNION.*SELECT|INSERT.*VALUES)"

# Command injection attempts
kubectl logs -n omendb-prod -l app=omendb | \
    grep -E "(;.*rm|&&.*wget|\|.*curl)"
```

#### Timeline Reconstruction
```bash
# Extract timeline of suspicious activity
kubectl logs -n omendb-prod -l app=omendb --timestamps | \
    grep -E "(ERROR|WARN|401|403)" | \
    sort > /tmp/security-timeline.txt

# Correlate with infrastructure logs
kubectl get events -n omendb-prod --sort-by=.metadata.creationTimestamp | \
    tail -50 >> /tmp/security-timeline.txt
```

### Containment & Eradication

1. **Change All Credentials**
   ```bash
   # Generate new passwords
   NEW_ADMIN_PASS=$(openssl rand -base64 32)
   NEW_JWT_SECRET=$(openssl rand -base64 64)

   # Update secrets
   kubectl create secret generic omendb-secret-new \
     --from-literal=admin_password="$NEW_ADMIN_PASS" \
     --from-literal=jwt_secret="$NEW_JWT_SECRET" \
     -n omendb-prod

   # Apply new secrets
   kubectl patch deployment/prod-omendb -n omendb-prod -p \
     '{"spec":{"template":{"spec":{"containers":[{"name":"omendb","env":[{"name":"OMENDB_ADMIN_PASSWORD","valueFrom":{"secretKeyRef":{"name":"omendb-secret-new","key":"admin_password"}}}]}]}}}}'
   ```

2. **System Hardening**
   ```bash
   # Enable additional security features
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"auth_enabled":"true","tls_enabled":"true","session_timeout_seconds":"900"}}'

   # Update security policies
   kubectl apply -f security-policies/strict-network-policy.yaml

   # Rotate TLS certificates
   kubectl delete secret omendb-tls -n omendb-prod
   kubectl create secret tls omendb-tls \
     --cert=new-server.pem --key=new-server.key -n omendb-prod
   ```

3. **Vulnerability Patching**
   ```bash
   # Update to latest secure version
   kubectl set image deployment/prod-omendb -n omendb-prod \
     omendb=omendb:latest-secure

   # Apply security patches
   kubectl rollout status deployment/prod-omendb -n omendb-prod
   ```

---

## SEC2: Potential Compromise

### Suspicious Activity Detection

#### Authentication Anomalies
```bash
# Multiple failed login attempts
kubectl logs -n omendb-prod -l app=omendb | \
    grep "401" | \
    awk '{print $1}' | sort | uniq -c | sort -nr | head -10

# Geographic anomalies (if geo-IP logging enabled)
kubectl logs -n omendb-prod -l app=omendb | \
    grep -E "country.*unusual|geo.*anomaly"

# Time-based anomalies
kubectl logs -n omendb-prod -l app=omendb | \
    awk '{print $2, $3}' | \
    grep -E "(02|03|04):" | wc -l  # Night time activity
```

#### Data Access Patterns
```bash
# Unusual query patterns
kubectl logs -n omendb-prod -l app=omendb | \
    grep "query" | \
    awk '{print $NF}' | \
    awk -F'=' '{if($2-$1 > 1000000) print}' | head -10  # Large time ranges

# Bulk data access
kubectl logs -n omendb-prod -l app=omendb | \
    grep -E "(SELECT.*\*|query.*large|batch.*export)"

# Administrative function usage
kubectl logs -n omendb-prod -l app=omendb | \
    grep -E "(admin|config|backup|restore|delete)"
```

### Response Actions

1. **Enhanced Monitoring**
   ```bash
   # Enable debug logging temporarily
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"log_level":"debug"}}'

   # Add additional authentication logging
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"auth_debug":"true"}}'
   ```

2. **Access Controls**
   ```bash
   # Reduce session timeouts
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"session_timeout_seconds":"1800"}}'

   # Enable additional rate limiting
   kubectl patch configmap omendb-config -n omendb-prod -p \
     '{"data":{"rate_limit_enabled":"true","max_requests_per_minute":"100"}}'
   ```

---

## SEC3: Security Policy Violations

### Common Violations

- Unauthorized access attempts
- Weak password usage
- Unencrypted data transmission
- Policy configuration drift

### Standard Response

1. **Document Violation**
   ```bash
   # Log incident
   echo "$(date): SEC3 - Policy violation detected" >> /var/log/security/violations.log

   # Gather evidence
   kubectl logs -n omendb-prod -l app=omendb | \
       grep -E "(violation|policy|unauthorized)" > /tmp/violation-evidence.txt
   ```

2. **Remediate Issues**
   ```bash
   # Reset to secure configuration
   kubectl apply -f configs/security-baseline.yaml

   # Force password reset for affected users
   curl -X POST http://localhost:3000/admin/force-password-reset \
       -H "Authorization: Bearer $ADMIN_TOKEN"
   ```

---

## Security Hardening

### Authentication & Authorization

```bash
# Enable strong authentication
kubectl patch configmap omendb-config -n omendb-prod -p \
  '{"data":{"auth_enabled":"true","min_password_length":"12","require_mfa":"true"}}'

# Configure RBAC
kubectl apply -f - <<EOF
apiVersion: rbac.authorization.k8s.io/v1
kind: Role
metadata:
  namespace: omendb-prod
  name: omendb-readonly
rules:
- apiGroups: [""]
  resources: ["pods", "services"]
  verbs: ["get", "list"]
EOF
```

### Network Security

```bash
# Implement network policies
kubectl apply -f - <<EOF
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: omendb-netpol
  namespace: omendb-prod
spec:
  podSelector:
    matchLabels:
      app: omendb
  policyTypes:
  - Ingress
  - Egress
  ingress:
  - from:
    - namespaceSelector:
        matchLabels:
          name: frontend
    ports:
    - protocol: TCP
      port: 3000
  egress:
  - to: []
    ports:
    - protocol: TCP
      port: 53
EOF
```

### Data Encryption

```bash
# Enable TLS
kubectl patch configmap omendb-config -n omendb-prod -p \
  '{"data":{"tls_enabled":"true","tls_min_version":"1.2"}}'

# Implement data-at-rest encryption
kubectl patch pvc omendb-data -n omendb-prod -p \
  '{"spec":{"storageClassName":"encrypted-ssd"}}'
```

---

## Forensics & Evidence Collection

### Digital Forensics Checklist

1. **Memory Capture**
   ```bash
   # Create memory dump
   kubectl exec -n omendb-prod deployment/prod-omendb -- \
     gcore $(pgrep omendb)

   # Preserve volatile data
   kubectl exec -n omendb-prod deployment/prod-omendb -- \
     ps aux > /tmp/forensics-processes.txt
   ```

2. **Log Collection**
   ```bash
   # Comprehensive log export
   kubectl logs -n omendb-prod -l app=omendb --all-containers=true \
     --previous=false > /tmp/forensics-logs-current.txt

   kubectl logs -n omendb-prod -l app=omendb --all-containers=true \
     --previous=true > /tmp/forensics-logs-previous.txt

   # System events
   kubectl get events -n omendb-prod -o yaml > /tmp/forensics-events.yaml
   ```

3. **Configuration Snapshot**
   ```bash
   # Capture current configuration
   kubectl get all -n omendb-prod -o yaml > /tmp/forensics-config.yaml

   # Network configuration
   kubectl get networkpolicies -n omendb-prod -o yaml >> /tmp/forensics-config.yaml

   # RBAC settings
   kubectl get rolebindings -n omendb-prod -o yaml >> /tmp/forensics-config.yaml
   ```

---

## Post-Incident Activities

### Immediate (Within 4 hours)

1. **Security Assessment**
   ```bash
   # Vulnerability scan
   kubectl run security-scan --rm -it --image=aquasec/trivy -- \
     trivy image omendb:latest

   # Configuration audit
   kubectl run config-audit --rm -it --image=aquasec/kube-bench -- \
     kube-bench --targets node,policies,managedservices
   ```

2. **Access Review**
   ```bash
   # Review all user accounts
   curl http://localhost:3000/admin/users | jq '.users[]'

   # Check recent access logs
   kubectl logs -n omendb-prod -l app=omendb | \
     grep "login\|auth" | tail -100
   ```

### Follow-up (Within 24 hours)

1. **Security Improvements**
   - Update security policies
   - Implement additional monitoring
   - Enhanced logging configuration
   - Security training for team

2. **Documentation**
   - Incident report
   - Lessons learned
   - Process improvements
   - Updated runbooks

---

## Security Monitoring Automation

### Automated Alerting

```yaml
# security-alerts.yml
groups:
- name: security
  rules:
  - alert: UnauthorizedAccess
    expr: rate(omendb_auth_failures_total[5m]) > 10
    for: 1m
    labels:
      severity: critical
    annotations:
      summary: "High rate of authentication failures"

  - alert: SuspiciousQuery
    expr: rate(omendb_suspicious_queries_total[5m]) > 1
    for: 30s
    labels:
      severity: warning
    annotations:
      summary: "Suspicious query patterns detected"

  - alert: DataExfiltration
    expr: rate(omendb_large_queries_total[5m]) > 5
    for: 2m
    labels:
      severity: critical
    annotations:
      summary: "Potential data exfiltration activity"
```

### Security Automation Scripts

```bash
#!/bin/bash
# security_automation.sh

# Auto-block suspicious IPs
SUSPICIOUS_IPS=$(kubectl logs -n omendb-prod -l app=omendb | \
    grep "401" | awk '{print $1}' | sort | uniq -c | \
    awk '$1 > 100 {print $2}')

for ip in $SUSPICIOUS_IPS; do
    echo "Blocking suspicious IP: $ip"
    # Add to network policy or firewall rules
done

# Auto-disable compromised accounts
COMPROMISED_USERS=$(kubectl logs -n omendb-prod -l app=omendb | \
    grep "suspicious.*activity" | awk '{print $3}' | sort | uniq)

for user in $COMPROMISED_USERS; do
    echo "Disabling compromised user: $user"
    curl -X POST http://localhost:3000/admin/disable-user \
        -H "Content-Type: application/json" \
        -d "{\"username\": \"$user\"}"
done
```

---

## Emergency Contacts

| Escalation Level | Role | Contact | Phone |
|------------------|------|---------|-------|
| L1 | Security Engineer | security-oncall@company.com | +1-555-9900 |
| L2 | Security Manager | security-manager@company.com | +1-555-9901 |
| L3 | CISO | ciso@company.com | +1-555-9902 |
| External | Law Enforcement | 911 | 911 |
| External | Legal Counsel | legal@company.com | +1-555-9903 |

---

## Quick Reference

### Security Commands
```bash
# Block all external access
kubectl patch service prod-omendb -n omendb-prod -p '{"spec":{"type":"ClusterIP"}}'

# Enable emergency mode
kubectl patch configmap omendb-config -n omendb-prod -p '{"data":{"emergency_mode":"true"}}'

# Rotate all secrets
./rotate-secrets.sh

# Capture forensic evidence
./collect-evidence.sh

# Emergency shutdown
kubectl scale deployment/prod-omendb --replicas=0 -n omendb-prod
```