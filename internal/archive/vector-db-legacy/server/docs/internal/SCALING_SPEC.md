# Scaling Specification - OmenDB Server

## Distributed Architecture

### Phase 1: Single Node (0-100M vectors)
- Vertical scaling with tiered storage
- Multiple DB instances via 0.0002ms startup
- Target: 10K QPS, <10ms P99

### Phase 2: Multi-Node (100M-10B vectors)
- Semantic sharding for 90% single-shard queries
- Raft consensus for metadata
- Target: 100K QPS, <50ms P99

### Phase 3: Planet Scale (10B+ vectors)
- ML-based query routing
- Multi-region deployment
- Target: 1M QPS, <100ms P99

## Sharding Strategy

### Semantic Sharding
```rust
pub struct SemanticSharding {
    centroids: Vec<Vector>,     // Cluster centers
    shard_map: HashMap<ClusterId, ShardId>,
}

impl SemanticSharding {
    fn route_query(&self, query: &Vector) -> ShardId {
        // Find nearest centroid
        let cluster = self.find_nearest_centroid(query);
        self.shard_map[&cluster]
    }
}
```

### Benefits
- 90% queries hit single shard
- Better cache locality
- Reduced network traffic

## Replication

### Read Replicas
- Async replication with <1s lag
- Automatic failover
- Load balancing across replicas

### Write Consistency
- Single master per shard
- Eventual consistency by default
- Strong consistency option for enterprise

## Deployment Architecture

### Kubernetes
```yaml
apiVersion: apps/v1
kind: StatefulSet
metadata:
  name: omendb-server
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: omendb
        image: omendb/server:latest
        resources:
          requests:
            memory: "32Gi"
            cpu: "8"
```

### Auto-scaling
- CPU-based scaling for compute nodes
- Storage-based scaling for data nodes
- Predictive scaling using query patterns

## Performance Targets

| Scale | Vectors | QPS | P99 Latency |
|-------|---------|-----|-------------|
| Small | 10M     | 1K  | 10ms        |
| Medium| 100M    | 10K | 20ms        |
| Large | 1B      | 100K| 50ms        |
| XL    | 10B     | 1M  | 100ms       |