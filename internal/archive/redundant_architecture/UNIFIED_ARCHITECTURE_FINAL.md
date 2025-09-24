# 🏗️ OmenDB Unified Architecture (Final)

**Version**: 1.0 Final
**Status**: Ready for implementation
**Target**: Beat Qdrant (15-25K vec/s) with better DX

## Executive Summary

Combining:
- ChatGPT's **enterprise robustness** (durability, operations)
- Our **performance innovations** (streaming, smart routing)
- Industry **best practices** (Raft consensus, Arrow format)
- Startup **agility** (embedded-first, developer love)

**Result**: 100K+ vec/s writes, <3ms searches, 95% recall, scales from SQLite to cloud

## Core Architecture

### The Three Pillars

```
1. STREAMING LAYER (Writes)
   ├── Append-only log (100K+ vec/s)
   ├── Zero-copy from NumPy
   ├── WAL for durability
   └── Never blocks on indexing

2. STORAGE LAYER (Data)
   ├── Arrow-compatible segments
   ├── Smart payload routing
   ├── Compaction & maintenance
   └── Tiered (memory/disk/cloud)

3. COMPUTE LAYER (Search)
   ├── Adaptive algorithms
   ├── GPU acceleration
   ├── Distributed execution
   └── Multi-stage ranking
```

## Detailed Component Design

### 1. Write Path (Streaming Ingestion)

```mojo
struct StreamingBuffer:
    var wal: WriteAheadLog          # Durability
    var buffer: RingBuffer           # In-memory
    var indexed_cursor: Atomic[Int]  # Progress tracking

    fn append(vector, metadata) -> Int:
        # Step 1: WAL for durability
        wal.append(OpType.INSERT, vector, metadata)

        # Step 2: Memory buffer (zero-copy)
        id = buffer.append_atomic(vector)

        # Step 3: Return immediately (don't wait for index)
        return id  # 100K+ vec/s
```

**Key Innovations**:
- Never blocks on indexing
- Atomic operations for thread safety
- Memory-mapped for zero-copy

### 2. Storage Format (Arrow Segments)

```
segment_001/
├── manifest.json         # Metadata
├── vectors.arrow        # Dense vectors (Arrow format)
├── sparse.arrow         # Sparse vectors (CSR format)
├── metadata.parquet     # Structured data
├── index.hnsw           # Graph index
├── index.ivf            # Inverted file
└── stats/
    ├── bloom.bin        # Bloom filters
    ├── minmax.json      # Range stats
    └── quantiles.json   # Distribution
```

**Benefits**:
- Arrow = zero-copy interop with Pandas/Polars
- Parquet = columnar compression for metadata
- Stats = efficient pruning

### 3. Index Selection (Adaptive)

```mojo
fn select_index(segment: Segment) -> Index:
    # Auto-select based on characteristics

    if segment.count < 10_000:
        return FlatIndex()  # Exact, SIMD optimized

    elif segment.count < 1_000_000:
        if segment.dimension <= 256:
            return HNSWIndex(M=32, ef=200)  # Balanced
        else:
            return IVFPQIndex(nlist=1024, m=64)  # Memory efficient

    else:  # >1M vectors
        if segment.updates_frequent:
            return SegmentedHNSW(segments=8)  # Update friendly
        else:
            return DiskANNIndex()  # Disk-based for scale
```

### 4. Query Processing Pipeline

```mojo
fn search(query: Query) -> Results:
    # Stage 1: Planning
    plan = QueryPlanner()
        .parse_filters(query.metadata_filter)
        .select_segments(query.tenant_id)
        .choose_algorithm(query.recall_target)

    # Stage 2: Pre-filtering (Pushed down)
    candidates = PreFilter()
        .apply_ranges(plan.range_filters)
        .apply_terms(plan.term_filters)
        .build_bitset()

    # Stage 3: Vector search (Parallel)
    results = parallel_map(plan.segments, |segment| {
        if plan.use_approximate:
            segment.index.search(query.vector, k * oversample)
        else:
            segment.flat_search(query.vector, k, candidates)
    })

    # Stage 4: Hybrid fusion (If needed)
    if query.sparse_query:
        sparse_results = BM25.search(query.text)
        results = HybridFusion.combine(results, sparse_results, alpha=0.7)

    # Stage 5: Re-ranking (GPU if available)
    if GPU.available():
        results = GPU.rerank_exact(query.vector, results, top_k * 10)

    return results.top_k()
```

### 5. Background Services

```mojo
struct BackgroundWorkers:
    var indexer: IndexBuilder      # Builds indices
    var compactor: Compactor       # Merges segments
    var optimizer: QueryOptimizer  # Tunes parameters
    var replicator: Replicator     # Syncs replicas

    fn start_all():
        # All run in separate threads/processes
        spawn(indexer.run_forever())
        spawn(compactor.run_forever())
        spawn(optimizer.run_forever())
        spawn(replicator.run_forever())
```

### 6. Distributed Architecture (Raft-based)

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Node 1    │     │   Node 2    │     │   Node 3    │
│  (Leader)   │────▶│  (Follower) │────▶│  (Follower) │
└─────────────┘     └─────────────┘     └─────────────┘
       │                   │                    │
       ▼                   ▼                    ▼
  ┌─────────┐        ┌─────────┐         ┌─────────┐
  │Shard 1,3│        │Shard 2,1│         │Shard 3,2│
  └─────────┘        └─────────┘         └─────────┘
```

**Consensus**:
- Raft for metadata (collections, schemas)
- Leaderless for data (eventual consistency)
- Quorum reads/writes configurable

## Implementation Phases

### Phase 1: MVP Core (Weeks 1-4)
**Goal**: Embedded database with streaming writes

```python
# This should work by Week 4
import omendb
db = omendb.open("./vectors.db")
db.add_batch(vectors)  # 100K+ vec/s
results = db.search(query, k=10)  # <5ms
```

**Components**:
- [x] StreamingBuffer with WAL
- [x] FlatIndex for <10K vectors
- [x] Basic HNSW for >10K
- [x] Python bindings with zero-copy

### Phase 2: Production Features (Weeks 5-8)
**Goal**: Production-ready with monitoring

**Components**:
- [ ] Segment compaction
- [ ] Metadata filtering
- [ ] Hybrid search (dense + sparse)
- [ ] Monitoring & metrics
- [ ] Backup/restore

### Phase 3: Scale & Distribution (Weeks 9-12)
**Goal**: Distributed mode with cloud API

**Components**:
- [ ] Raft consensus
- [ ] Sharding & replication
- [ ] Cloud API (HTTP/gRPC)
- [ ] GPU acceleration
- [ ] Auto-tuning

### Phase 4: Enterprise (Months 4-6)
**Goal**: Enterprise features

**Components**:
- [ ] Multi-tenancy
- [ ] RBAC & audit logs
- [ ] Encryption at rest
- [ ] Disaster recovery
- [ ] SOC2 compliance

## Performance Targets

### Insertion Performance
```
Dataset Size    Current     Target      Method
< 10K          45K vec/s   100K vec/s  Streaming buffer
10K - 100K     5K vec/s    50K vec/s   Background HNSW
100K - 1M      3K vec/s    30K vec/s   Segmented + parallel
> 1M           1K vec/s    20K vec/s   Distributed shards
```

### Search Performance
```
Dataset Size    Latency     Recall      Method
< 10K          < 1ms       100%        Flat SIMD
10K - 100K     < 3ms       95%         HNSW
100K - 1M      < 5ms       95%         Segmented HNSW
> 1M           < 10ms      90%         IVF-PQ or DiskANN
```

### Memory Usage
```
Component           Per 1M vectors (128d)
Raw vectors         512 MB (float32)
HNSW index          200 MB
Metadata            100 MB
Working memory      200 MB
Total              ~1 GB
```

## Configuration Profiles

### Developer Mode (Default)
```yaml
mode: embedded
persistence: wal
index: auto
memory_limit: 4GB
background_threads: 2
```

### Production Mode
```yaml
mode: server
persistence: wal + snapshots
index: hnsw
memory_limit: 32GB
background_threads: 8
monitoring: prometheus
```

### Cloud Mode
```yaml
mode: distributed
persistence: s3
index: adaptive
memory_limit: unlimited
background_threads: auto
replication: 3
```

## API Design

### Python (Primary)
```python
import omendb
from omendb import FilterOp as F

# Embedded mode (default)
db = omendb.open("./my.db")

# Add vectors with metadata
ids = db.add_batch(
    vectors=numpy_array,  # (N, D) float32
    metadata={"category": categories, "timestamp": times}
)

# Search with filters
results = db.search(
    vector=query_vector,
    k=10,
    filters=F.AND(
        F.eq("category", "product"),
        F.gte("timestamp", 1234567890)
    ),
    include=["distance", "metadata"]
)

# Hybrid search
results = db.hybrid_search(
    vector=query_vector,
    text="neural networks",
    k=10,
    alpha=0.7  # 0.7 vector, 0.3 text
)
```

### REST API
```http
POST /collections/products/search
{
  "vector": [0.1, 0.2, ...],
  "k": 10,
  "filters": {
    "category": {"$eq": "electronics"},
    "price": {"$lte": 1000}
  },
  "include": ["id", "distance", "metadata"]
}
```

## Monitoring & Operations

### Key Metrics
```
# Performance
omendb_insertion_rate_per_sec
omendb_search_latency_p95
omendb_recall_at_10

# Health
omendb_memory_usage_bytes
omendb_disk_usage_bytes
omendb_index_build_queue_size

# Business
omendb_total_vectors
omendb_active_collections
omendb_daily_searches
```

### Auto-Tuning
```mojo
struct AutoTuner:
    fn tune(workload: Workload) -> Config:
        if workload.is_write_heavy():
            return Config(
                buffer_size=100_000,
                index_delay=60,  # seconds
                compaction_interval=3600
            )
        elif workload.is_read_heavy():
            return Config(
                buffer_size=10_000,
                index_delay=5,
                cache_size="80%"
            )
```

## Security & Compliance

### Security Features
- TLS 1.3 for transport
- AES-256 for encryption at rest
- JWT/OAuth2 authentication
- API key management
- Audit logging

### Compliance
- GDPR: Right to deletion
- SOC2: Access controls
- HIPAA: Encryption (future)

## Why This Architecture Wins

### 1. **Developer Experience**
- Starts as embedded (no setup)
- Scales to cloud (same API)
- Smart defaults (auto-tuning)

### 2. **Performance**
- Streaming writes (never blocks)
- Adaptive algorithms (always optimal)
- GPU acceleration (when available)

### 3. **Production Ready**
- Durability (WAL + snapshots)
- Operations (monitoring, backup)
- Scale (distributed, sharding)

### 4. **Future Proof**
- Modular (swap algorithms)
- Extensible (plugin system)
- Cloud-native (K8s ready)

## Next Steps

1. **Implement StreamingBuffer** (Week 1)
2. **Add Background Indexing** (Week 2)
3. **Python SDK with Zero-Copy** (Week 3)
4. **Launch on Hacker News** (Week 4)

**Success Metric**: 1000 GitHub stars in first month

---

*This architecture combines the best of all worlds: ChatGPT's enterprise features, our performance innovations, and industry best practices.*