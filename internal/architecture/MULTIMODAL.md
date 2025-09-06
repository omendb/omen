# Multimodal Database Architecture

## Overview
OmenDB is a multimodal database supporting vectors, text, and structured metadata in a unified system.

## Core Components

### 1. HNSW+ Vector Index
```mojo
struct HNSWIndex:
    var layers: List[Graph]         # Hierarchical layers
    var M: Int = 16                 # Connections per node
    var ef_construction: Int = 200  # Build quality
    var entry_point: Int            # Top layer entry
```

### 2. BM25 Text Index
```mojo
struct BM25Index:
    var inverted_index: Dict[String, List[DocId]]
    var doc_frequencies: Dict[String, Int]
    var doc_lengths: List[Int]
```

### 3. Metadata Store
```mojo
struct MetadataStore:
    var columns: ColumnStore        # Columnar for analytics
    var indexes: Dict[String, BTree] # Fast filtering
```

## Query Language

### SQL with Vector Extensions
```sql
-- Hybrid search combining all modalities
SELECT id, title, similarity
FROM products
WHERE 
    vector <-> @query_embedding < 0.8  -- Vector similarity
    AND MATCH(description, 'smartphone') -- Text search
    AND price < 1000                    -- Metadata filter
ORDER BY similarity DESC
LIMIT 10;
```

## Storage Architecture

### Tiered Storage System
```
Hot Tier (NVMe SSD):
- Last 7 days of data
- <1ms access latency
- Uncompressed vectors

Warm Tier (SSD):
- 7-30 days old
- <10ms access latency
- Lightly compressed

Cold Tier (S3):
- >30 days old
- <100ms access latency
- Heavily compressed
```

### File Layout
```
/data/
├── vectors/
│   ├── hot/     # Memory-mapped files
│   ├── warm/    # Compressed blocks
│   └── cold/    # S3 references
├── text/
│   ├── index/   # Inverted index
│   └── docs/    # Document store
└── metadata/
    ├── columns/ # Columnar data
    └── indexes/ # B-tree indexes
```

## Query Planning

### Selectivity-Based Optimization
```mojo
fn plan_query(query: HybridQuery) -> ExecutionPlan:
    # Estimate selectivity of each component
    var filter_sel = estimate_filter_selectivity(query.filters)
    var text_sel = estimate_text_selectivity(query.text)
    var vector_sel = estimate_vector_candidates(query.vector)
    
    # Choose optimal execution order
    if filter_sel < 0.01:
        return FilterFirst_ThenVector_ThenText()
    elif text_sel < 0.05:
        return TextFirst_ThenFilter_ThenVector()
    else:
        return VectorFirst_ThenFilter_ThenText()
```

### Cost Model
- Metadata filter: O(log n) with B-tree
- Text search: O(k) with inverted index
- Vector search: O(log n) with HNSW
- Combined: Minimize intermediate results

## API Design

### Python Client
```python
from omendb import Client

# Connect
db = Client("localhost:8080")

# Insert multimodal data
db.insert({
    "id": "product_123",
    "vector": embedding,
    "text": "iPhone 15 Pro description",
    "metadata": {
        "price": 999,
        "category": "electronics"
    }
})

# Hybrid search
results = db.search(
    vector=query_embedding,
    text="smartphone camera",
    filters={"price": {"$lt": 1000}},
    limit=10
)
```

### REST API
```http
POST /search
{
  "vector": [0.1, 0.2, ...],
  "text": "smartphone camera",
  "filters": {
    "price": {"$lt": 1000}
  },
  "limit": 10
}
```

## Performance Targets

| Operation | Target | Current |
|-----------|--------|---------|
| Vector insert | 100K/sec | TBD |
| Text indexing | 50K docs/sec | TBD |
| Hybrid query | <10ms | TBD |
| Memory/vector | 2-4 bytes | TBD |

## Scalability

### Single Node
- Up to 10M vectors
- 100GB metadata
- 1TB text corpus
- 10K QPS

### Distributed (Future)
- Sharding by hash(id)
- Replication factor 3
- Eventual consistency
- Cross-shard queries

## Cloud Deployment

### Infrastructure
```yaml
API Layer:
  - Load Balancer (ALB)
  - API Gateway nodes (t3.xlarge)
  
Compute Layer:
  - Vector nodes (g4dn.xlarge for GPU)
  - Text nodes (m5.xlarge)
  - Metadata nodes (r5.large)
  
Storage Layer:
  - Hot: NVMe instance storage
  - Warm: EBS gp3 volumes
  - Cold: S3 with lifecycle policies
```

### Cost Model (1M vectors, 100 QPS)
- Infrastructure: ~$500/month
- Can charge: $5,000/month
- Margin: 10x

## Development Phases

### Phase 1: Core (Month 1)
- HNSW+ vector index
- Basic metadata filtering
- Python bindings

### Phase 2: Multimodal (Month 2)
- BM25 text search
- Query planner
- SQL parser

### Phase 3: Production (Month 3)
- Tiered storage
- Monitoring
- Cloud deployment

### Phase 4: Scale (Month 4)
- GPU acceleration
- Distributed sharding
- Enterprise features

---
*Architecture optimized for multimodal AI applications*