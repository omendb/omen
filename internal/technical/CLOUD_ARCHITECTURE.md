# OmenDB Cloud Architecture
*Last Updated: September 1, 2025*

## Strategic Decision: Embedded (Free) + Cloud (Paid)

No server licensing. Just two products:
- **Embedded**: Free, open source, like SQLite
- **Cloud**: Managed service, container-per-tenant

## Architecture: Container Per Tenant

### Critical Requirements
- **Performance Isolation**: One tenant can't affect another
- **Security Isolation**: Complete data separation
- **Resource Isolation**: Guaranteed CPU/memory
- **Failure Isolation**: Crashes contained

### Implementation
```yaml
# Each customer gets dedicated container
Customer → Container → Persistent Volume → Isolated OmenDB
```

## Technology Evolution

### Phase 1: Python/Litestar MVP (Sept-Oct 2025)
- Docker Compose orchestration
- 10-20 customers max
- 2 weeks to build
```python
# Control plane manages containers
# Each container runs embedded OmenDB
# FastAPI → Litestar (3x faster)
```

### Phase 2: Go Production (Nov-Dec 2025)
- Kubernetes orchestration  
- 1000+ customers
- Migrate when >10 paying customers
```go
// 10x better concurrency than Python
// Direct CGO bindings to OmenDB
```

### Phase 3: Pure Mojo (2026+)
- When Mojo has web frameworks
- Native performance throughout

## Pricing Model

| Tier | CPU | Memory | Storage | Price/mo | Margin |
|------|-----|--------|---------|----------|--------|
| Free | 0.1 | 256MB | 1GB | $0 | - |
| Starter | 0.5 | 1GB | 10GB | $25 | 76% |
| Growth | 1.0 | 2GB | 100GB | $99 | 90% |
| Scale | 2.0 | 4GB | 1TB | $499 | 96% |

## Infrastructure

### MVP: Docker Compose
```yaml
services:
  omendb-tenant-123:
    image: omendb/server
    mem_limit: 2g
    cpus: 1.0
    volumes:
      - /data/tenant-123:/var/lib/omendb
```

### Scale: Kubernetes
```yaml
apiVersion: v1
kind: Pod
spec:
  containers:
  - name: omendb
    resources:
      limits:
        memory: "2Gi"
        cpu: "1000m"
```

## Client Experience

```python
# Embedded (current)
from omendb import DB
db = DB("./vectors.db")

# Cloud (identical API!)
from omendb import Client  
client = Client(api_key="omen_xxx")
```

## Why This Architecture

1. **Simple**: No multi-tenancy complexity
2. **Proven**: Supabase, Neon use same model
3. **Profitable**: 75%+ margins
4. **Scalable**: Just add nodes

## Success Metrics

- 10 paying customers Month 1
- $1K MRR Month 3
- 100 customers Month 6
- Migrate to Go at 50 customers

## Decision: BUILD NOW

Start with Docker Compose MVP, ship in 2 weeks.