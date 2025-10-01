# OmenDB Kubernetes Deployment

Enterprise-grade Kubernetes deployment configuration for OmenDB vector database.

## 🏗️ Architecture

```
k8s/
├── base/                    # Base configurations
│   ├── deployment.yaml      # OmenDB deployment
│   ├── service.yaml         # ClusterIP and headless services
│   ├── configmap.yaml       # Configuration and settings
│   ├── secret.yaml          # Credentials and secrets
│   ├── pvc.yaml             # Persistent volume claims
│   ├── rbac.yaml            # Service account and permissions
│   └── kustomization.yaml   # Base kustomization
└── overlays/                # Environment-specific configs
    ├── development/         # Dev environment (low resources)
    ├── staging/             # Staging environment (2 replicas)
    └── production/          # Production environment (3 replicas)
```

## 🚀 Quick Start

### Prerequisites

- Kubernetes cluster (1.21+)
- kubectl configured
- kustomize (built into kubectl 1.14+)
- Docker registry access

### Deploy to Development

```bash
# Deploy
./deploy.sh development

# Access via port-forward
kubectl port-forward -n omendb-dev svc/dev-omendb 3000:3000

# Test
curl http://localhost:3000/health
```

### Deploy to Production

```bash
# Build and push image
docker build -t your-registry/omendb:v0.1.0 ..
docker push your-registry/omendb:v0.1.0

# Update image in production overlay
cd overlays/production
kustomize edit set image omendb=your-registry/omendb:v0.1.0

# Deploy
./deploy.sh production
```

## 🔧 Configuration

### Environment Configurations

| Environment | Replicas | CPU Limit | Memory Limit | Storage |
|-------------|----------|-----------|--------------|---------|
| Development | 1        | 1 core    | 1 GB         | 20 GB   |
| Staging     | 2        | 3 cores   | 3 GB         | 50 GB   |
| Production  | 3        | 4 cores   | 4 GB         | 100 GB  |

### Security Features

- **RBAC**: Minimal permissions with dedicated service account
- **Security Context**: Non-root user, read-only filesystem
- **Network Policies**: Pod-to-pod communication control
- **Secrets Management**: Encrypted credential storage
- **TLS**: Optional TLS encryption for client connections

### Monitoring & Observability

- **Prometheus Metrics**: Available at `:9090/metrics`
- **Health Checks**: Liveness, readiness, and startup probes
- **Structured Logging**: JSON format for log aggregation
- **Resource Monitoring**: CPU, memory, and disk usage

## 📊 Monitoring

### Prometheus Integration

```yaml
# Scrape configuration
- job_name: 'omendb'
  kubernetes_sd_configs:
  - role: pod
  relabel_configs:
  - source_labels: [__meta_kubernetes_pod_annotation_prometheus_io_scrape]
    action: keep
    regex: true
```

### Grafana Dashboard

Key metrics to monitor:
- `omendb_searches_total` - Search operations
- `omendb_inserts_total` - Insert operations
- `omendb_query_duration_seconds` - Query latency
- `omendb_memory_usage_bytes` - Memory consumption
- `omendb_active_connections` - Connection count

## 🔒 Security

### Authentication

Production environments use HTTP Basic Authentication:

```bash
# Default credentials (change in production!)
Username: admin
Password: admin123
```

### TLS Configuration

Enable TLS in production:

```yaml
# In configmap-patch.yaml
[security]
tls_enabled = true
```

### Secrets Management

Update secrets before production deployment:

```bash
# Encode new credentials
echo -n "new-admin-user" | base64
echo -n "secure-password" | base64
echo -n "jwt-secret-key" | base64

# Update k8s/base/secret.yaml
```

## 🧪 Testing

### Validation Commands

```bash
# Check deployment status
kubectl get pods -n omendb-prod -l app=omendb

# View logs
kubectl logs -n omendb-prod -l app=omendb

# Test connectivity
kubectl exec -n omendb-prod -it deployment/prod-omendb -- curl localhost:3000/health

# Performance test
kubectl run test-pod --rm -i --tty --image=curlimages/curl -- \
  curl -X POST http://prod-omendb:3000/insert \
  -H "Content-Type: application/json" \
  -d '{"id": "test", "vector": [0.1, 0.2, 0.3]}'
```

### Load Testing

```bash
# Port forward for external access
kubectl port-forward -n omendb-prod svc/prod-omendb 3000:3000 &

# Run load test (requires wrk)
wrk -t12 -c400 -d30s --latency http://localhost:3000/health
```

## 🔧 Operations

### Scaling

```bash
# Scale replicas
kubectl scale deployment/prod-omendb -n omendb-prod --replicas=5

# Auto-scaling (HPA)
kubectl autoscale deployment/prod-omendb -n omendb-prod --cpu-percent=70 --min=3 --max=10
```

### Updates

```bash
# Rolling update
kubectl set image deployment/prod-omendb -n omendb-prod omendb=omendb:v0.2.0

# Monitor rollout
kubectl rollout status deployment/prod-omendb -n omendb-prod

# Rollback if needed
kubectl rollout undo deployment/prod-omendb -n omendb-prod
```

### Backup

```bash
# Backup persistent data
kubectl exec -n omendb-prod -c omendb deployment/prod-omendb -- \
  tar czf - /var/lib/omendb/data > omendb-backup-$(date +%Y%m%d).tar.gz
```

### Troubleshooting

```bash
# Debug pod issues
kubectl describe pod -n omendb-prod -l app=omendb

# Check resource usage
kubectl top pods -n omendb-prod

# Access pod shell
kubectl exec -n omendb-prod -it deployment/prod-omendb -- /bin/sh

# View events
kubectl get events -n omendb-prod --sort-by=.metadata.creationTimestamp
```

## 🧹 Cleanup

### Remove Specific Environment

```bash
./cleanup.sh development  # Remove dev environment
./cleanup.sh staging      # Remove staging environment
./cleanup.sh production   # Remove production environment
```

### Remove Everything

```bash
./cleanup.sh all  # Remove all environments
```

## 📈 Performance Tuning

### Resource Optimization

For high-performance workloads:

```yaml
# Increase resources in deployment-patch.yaml
resources:
  requests:
    memory: "4Gi"
    cpu: "2000m"
  limits:
    memory: "8Gi"
    cpu: "4000m"
```

### Storage Optimization

```yaml
# Use faster storage class
storageClassName: premium-ssd  # or nvme-ssd
```

### CPU Affinity

```yaml
# Pin to specific nodes
nodeSelector:
  node-type: compute-optimized
```

## 📚 References

- [Kubernetes Documentation](https://kubernetes.io/docs/)
- [Kustomize Documentation](https://kustomize.io/)
- [Prometheus Monitoring](https://prometheus.io/docs/)
- [OmenDB Configuration Reference](../internal/ARCHITECTURE.md)