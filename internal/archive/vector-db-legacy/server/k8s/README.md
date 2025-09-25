# OmenDB Server Kubernetes Deployment

Complete Kubernetes deployment configuration for OmenDB Server with production-ready features.

## Quick Start

```bash
# Deploy to staging
./scripts/deploy.sh

# Deploy to production  
./scripts/deploy.sh -e production -t v0.1.0

# Dry run to preview changes
./scripts/deploy.sh --dry-run
```

## Architecture Overview

```
┌─────────────────────────────────────────┐
│  AWS Load Balancer / Ingress           │
├─────────────────────────────────────────┤
│  OmenDB Server Pods (3-100 replicas)   │
│  ├── Rust HTTP/gRPC Server             │
│  ├── Mojo Vector Engine (FFI)          │
│  └── Local SSD Storage (Warm Tier)     │
├─────────────────────────────────────────┤
│  Persistent Storage                     │
│  ├── Hot Tier: Memory (1% of vectors)  │
│  ├── Warm Tier: SSD (9% of vectors)    │
│  └── Cold Tier: S3 (90% of vectors)    │
└─────────────────────────────────────────┘
```

## Components

### Core Resources
- **Deployment**: Main application with 3-100 pod replicas
- **Service**: ClusterIP service for internal communication
- **Ingress**: External HTTP/HTTPS and gRPC access
- **ConfigMap**: Application configuration
- **Secret**: JWT keys and credentials
- **PVC**: Persistent storage for data

### Scaling & Availability
- **HPA**: Horizontal Pod Autoscaler (CPU, memory, custom metrics)
- **VPA**: Vertical Pod Autoscaler for resource optimization
- **PDB**: Pod Disruption Budget (min 2 pods available)

### Security
- **ServiceAccount**: Minimal RBAC permissions
- **NetworkPolicy**: Restrict ingress/egress traffic
- **SecurityContext**: Non-root, read-only filesystem

### Monitoring
- **ServiceMonitor**: Prometheus metrics scraping
- **PrometheusRule**: Alerting rules for SLOs
- **Grafana Dashboard**: Pre-built visualization

## Configuration

### Environment Variables
```bash
OMENDB_HTTP_PORT=8080           # HTTP API port
OMENDB_GRPC_PORT=9090          # gRPC API port  
OMENDB_WORKER_THREADS=8        # Rust async workers
OMENDB_DIMENSION=128           # Vector dimension
OMENDB_JWT_SECRET=secret       # JWT signing key
OMENDB_DATA_DIR=/app/data      # Data directory
```

### Storage Classes
- `gp3`: Standard EBS volumes
- `gp3-optimized`: High IOPS for warm tier (10K IOPS, 1000 MB/s)

### Resource Limits

| Environment | CPU Request | Memory Request | CPU Limit | Memory Limit |
|-------------|-------------|----------------|-----------|--------------|
| Staging     | 500m        | 2Gi           | 4         | 8Gi          |
| Production  | 1000m       | 4Gi           | 8         | 16Gi         |

## Deployment Process

### Prerequisites
```bash
# Install tools
kubectl version --client
kustomize version

# Configure cluster access
kubectl config current-context
kubectl cluster-info

# Verify storage classes
kubectl get storageclass
```

### Step-by-Step Deployment

1. **Build and Push Image**
   ```bash
   docker build -t omendb/server:v0.1.0 .
   docker push omendb/server:v0.1.0
   ```

2. **Deploy Infrastructure**
   ```bash
   # Create namespace and base resources
   kubectl apply -f k8s/namespace.yaml
   kubectl apply -f k8s/rbac.yaml
   kubectl apply -f k8s/pvc.yaml
   ```

3. **Configure Secrets**
   ```bash
   # Generate JWT secret
   JWT_SECRET=$(openssl rand -base64 32)
   kubectl create secret generic omendb-secrets \
     --from-literal=jwt-secret="$JWT_SECRET" \
     -n omendb-system
   ```

4. **Deploy Application**
   ```bash
   # Use deployment script (recommended)
   ./scripts/deploy.sh -e production -t v0.1.0
   
   # Or manual kustomize
   kubectl apply -k k8s/
   ```

5. **Verify Deployment**
   ```bash
   kubectl get pods -n omendb-system
   kubectl logs -f deployment/omendb-server -n omendb-system
   curl https://api.omendb.com/health
   ```

## Scaling Configuration

### Horizontal Pod Autoscaler
- **Min Replicas**: 3 (staging), 5 (production)
- **Max Replicas**: 50 (staging), 100 (production)
- **CPU Target**: 70% (staging), 60% (production)
- **Memory Target**: 80% (staging), 70% (production)
- **Custom Metrics**: Requests/sec, P99 latency

### Storage Scaling
- **Hot Tier**: Auto-scales with memory allocation
- **Warm Tier**: EBS volumes with auto-expansion
- **Cold Tier**: S3 with lifecycle policies

## Monitoring & Alerting

### Key Metrics
- `omendb_requests_total`: Total HTTP requests
- `omendb_query_latency_seconds`: Query latency histogram
- `omendb_vectors_total`: Total vectors stored
- `omendb_memory_usage_bytes`: Memory usage
- `omendb_errors_total`: Error count by type

### Critical Alerts
- **OmenDBServerDown**: Pod unavailable >1min
- **OmenDBHighLatency**: P99 >50ms for >2min
- **OmenDBHighErrorRate**: >1% error rate
- **OmenDBEnginePoolExhausted**: No available engines

### Grafana Dashboards
Access via: http://grafana.internal/d/omendb-server

## Troubleshooting

### Common Issues

**Pod Stuck in Pending**
```bash
kubectl describe pod <pod-name> -n omendb-system
# Check: resource requests, storage class, node selectors
```

**High Memory Usage**
```bash
kubectl top pods -n omendb-system
# Check: vector count, tiered storage balance
```

**Connection Refused**
```bash
kubectl port-forward svc/omendb-server 8080:80 -n omendb-system
curl localhost:8080/health
# Check: service ports, ingress configuration
```

### Debug Commands
```bash
# Pod logs
kubectl logs -f deployment/omendb-server -n omendb-system

# Pod shell access
kubectl exec -it <pod-name> -n omendb-system -- /bin/bash

# Service endpoints
kubectl get endpoints omendb-server -n omendb-system

# Resource usage
kubectl top pods -n omendb-system
kubectl describe hpa omendb-server -n omendb-system
```

## Security Considerations

### Network Security
- All pods run as non-root user (UID 1000)
- Read-only root filesystem
- NetworkPolicy restricts traffic
- TLS encryption for all external traffic

### Data Security
- Secrets stored in Kubernetes secrets
- JWT tokens with 24h expiration
- API keys with tenant isolation
- Persistent volumes encrypted at rest

### Access Control
- RBAC with minimal permissions
- ServiceAccount per component
- Ingress with rate limiting
- Internal metrics endpoint restriction

## Performance Tuning

### Pod Resources
```yaml
resources:
  requests:
    cpu: 1000m      # Guaranteed CPU
    memory: 4Gi     # Guaranteed memory
  limits:
    cpu: 8          # Maximum CPU
    memory: 16Gi    # Maximum memory
```

### Storage Performance
- Warm tier: 10K IOPS, 1000 MB/s throughput
- Hot tier: Memory-resident for <1ms access
- Cold tier: S3 with intelligent tiering

### Network Optimization
- Pod anti-affinity for availability
- Node affinity for performance
- Keep-alive connections
- Connection pooling

## Backup & Recovery

### Data Backup
```bash
# Snapshot PVCs
kubectl patch pvc omendb-data -p '{"metadata":{"annotations":{"volume.beta.kubernetes.io/storage-provisioner":"snapshot"}}}'

# S3 cold storage is automatically replicated
# Warm tier backed up to S3 nightly
```

### Disaster Recovery
1. Restore PVC from snapshot
2. Deploy with same configuration
3. Cold tier data automatically available
4. Warm tier rebuilt from S3 backup

## Cost Optimization

### Resource Efficiency
- VPA for optimal pod sizing
- HPA for demand-based scaling
- Spot instances for non-critical workloads
- S3 Intelligent Tiering for cold storage

### Monitoring Costs
```bash
# Check resource usage
kubectl cost --namespace omendb-system

# Optimize based on utilization
kubectl top pods --sort-by=cpu
kubectl top pods --sort-by=memory
```

## Next Steps

1. **Set up CI/CD pipeline** for automated deployments
2. **Configure monitoring alerts** in production
3. **Implement backup procedures** for disaster recovery
4. **Load test** with production traffic patterns
5. **Set up multi-region** deployment for global scale