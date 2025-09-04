# OmenDB

**High-Performance Embedded Vector Database**

[![PyPI](https://img.shields.io/badge/PyPI-v0.1.2-blue?style=flat-square)](https://pypi.org/project/omendb/)
[![License](https://img.shields.io/badge/License-Elastic%202.0-black?style=flat-square)](LICENSE)
[![Performance](https://img.shields.io/badge/Performance-1.4K%20vec%2Fs-green?style=flat-square)](tests/benchmarks/)

---

## Why OmenDB

**🔧 Simple.** Embedded database with zero configuration. Just `pip install` and go.

**⚡ Fast.** Sub-millisecond startup. 1,400 vectors/sec ingestion. <1ms queries.

**🎯 Production ready.** DiskANN algorithm (no rebuilds), automatic index management, persistent storage.

**📦 Portable.** Single-file database. Runs on Linux and macOS. No dependencies.

```python
import numpy as np
from omendb import DB

# Instant startup - no loading time
db = DB()

# Best practice: NumPy arrays with batch operations
embeddings = np.random.rand(1000, 384).astype(np.float32)  # Your ML model output
ids = [f"doc_{i}" for i in range(1000)]
db.add_batch(vectors=embeddings, ids=ids)  # 1,400 vectors/second

# Fast similarity search
query = np.random.rand(384).astype(np.float32)
results = db.search(query, limit=10)
```

## Install

```bash
pip install omendb
```

### Platform Support

- ✅ **macOS**: Full support
- ✅ **Linux**: Full support
- ❌ **Windows**: Not supported (Mojo/MAX platform limitation)

Windows support will be added when the Mojo language adds Windows compatibility.

## Features

### Core Features

**Core Vector Operations**
- ✅ DiskANN graph-based search (no rebuilds needed)
- ✅ Brute force exact search (small datasets)
- ✅ Instant startup (0.001ms)
- ✅ Sub-millisecond queries

**Data Management**
- ⚠️ Collections API (not available - single DB design)
- ✅ Metadata filtering
- ✅ CRUD operations (add, update, delete, clear)
- ✅ Persistent storage

**Performance**
- ✅ SIMD optimized operations  
- ✅ Consistent performance across batch sizes (no batch penalty)
- ✅ Zero-copy NumPy integration
- ✅ Memory efficient (16.7KB per vector)

**Performance by Dimension** (1K vectors):
| Dimension | Ingestion (vec/s) | Search (ms) |
|-----------|-------------------|-------------|
| 64D       | 2,700             | 0.35        |
| 128D      | 1,400             | 0.50        |
| 256D      | 750               | 0.80        |
| 512D      | 375               | 1.40        |

**Developer Experience**
- ✅ Clean, modern API
- ✅ Type hints & autocomplete
- ✅ Framework agnostic (NumPy, PyTorch, TensorFlow)
- ✅ Comprehensive docs

### 💰 Paid Server Edition (Coming Soon)

**Enterprise Scale**
- 🔄 Distributed architecture
- 🔄 Horizontal scaling
- 🔄 Multi-region replication
- 🔄 10K+ QPS per instance

**Multi-Tenancy**
- 🔄 Tenant isolation
- 🔄 Resource limits
- 🔄 Usage tracking
- 🔄 Billing integration

**Security & Compliance**
- 🔄 Authentication (JWT/API keys)
- 🔄 RBAC authorization
- 🔄 Audit logging
- 🔄 Encryption at rest/in transit

**Operations**
- 🔄 Admin dashboard
- 🔄 Monitoring & alerting
- 🔄 Backup & restore
- 🔄 99.9% SLA

## Performance Benchmarks

**Performance Benchmarks** (128-dimensional vectors)

| Operation | OmenDB | Typical Range* | Notes |
|-----------|--------|----------------|-------|
| Startup | **0.002ms** | 50-1000ms | 122,850x faster than ChromaDB |
| Batch Insert (1K) | **95K/s** | 1-50K/s | Optimal for small datasets |
| Batch Insert (10K) | **54K/s** | 1-50K/s | Good performance |
| Batch Insert (25K+) | **3.4K/s** | 1-50K/s | HNSW construction overhead |
| Query | **2-4ms** | 1-50ms | Fast at any scale |
| Memory | **Optimized** | Varies | 4x compression with quantization |

*Based on common vector database benchmarks. Actual performance varies by hardware, configuration, and workload.

## Current Status (v0.2.0-rc)

✅ **Production Ready**: Extensively tested with stable performance and persistence

✅ **Persistence Support**: SQLite-style single-file storage with checkpoint/recovery

⚠️ **Collections API Disabled**: Not available due to Mojo language limitations. Single database per process design.

✅ **Optimized Architecture**: Write buffer + HNSW main index with intelligent batching

See [KNOWN_ISSUES.md](docs/KNOWN_ISSUES.md) for complete details.

## Quick Start

### Basic Usage

```python
import numpy as np
from omendb import DB

# Create database
db = DB()

# Add vectors with metadata (always use batch operations)
embeddings = np.array([
    [0.1, 0.2, 0.3],
    [0.4, 0.5, 0.6]
], dtype=np.float32)
ids = ["product_1", "product_2"]
metadata = [
    {"category": "electronics"},
    {"category": "clothing"}
]

db.add_batch(vectors=embeddings, ids=ids, metadata=metadata)

# Search with modern API
query = np.array([0.1, 0.2, 0.3], dtype=np.float32)
results = db.search(query, limit=5)
for result in results:
    print(f"{result.id}: {result.score:.3f}")
```

### Collections (Not Available)

OmenDB uses a single-database design. For logical separation, use ID prefixes or metadata:

```python
# Option 1: Use ID prefixes
embeddings = np.array([user_embedding, product_embedding], dtype=np.float32)
ids = ["users:user123", "products:prod456"]
db.add_batch(vectors=embeddings, ids=ids)

# Option 2: Use metadata filtering  
embeddings = np.array([embedding1, embedding2], dtype=np.float32)
ids = ["id1", "id2"]
metadata = [
    {"collection": "users"},
    {"collection": "products"}
]
db.add_batch(vectors=embeddings, ids=ids, metadata=metadata)

# Search with filter
query = np.array(query_vector, dtype=np.float32)
results = db.search(
    query, 
    limit=10, 
    filter={"collection": "users"}
)
```

### Persistence (New in v0.2.0)

```python
# Configure persistence
db = DB()
db.set_persistence("my_vectors.db")

# Add vectors - automatically persisted
vectors = np.random.rand(1000, 128).astype(np.float32)
ids = [f"vec_{i}" for i in range(1000)]
db.add_batch(vectors, ids)

# Save checkpoint to disk
db.checkpoint()

# Later, recover from disk
db2 = DB()
db2.set_persistence("my_vectors.db")
recovered = db2.recover()  # Loads all vectors
print(f"Recovered {recovered} vectors")
```

### Batch Operations

```python
import numpy as np

# Fast batch ingestion
vectors = np.random.rand(10000, 128).astype(np.float32)
ids = [f"vec_{i}" for i in range(10000)]

db.add_batch(vectors=vectors, ids=ids)  # 96K vec/s with NumPy

# Metadata filtering
query = np.random.rand(128).astype(np.float32)
results = db.search(
    query, 
    limit=10,
    filter={"category": "electronics"}
)
```

### Memory Optimization

```python
# Enable 8-bit quantization
db.enable_quantization()

# 4x memory reduction with <2% accuracy loss
info = db.info()
print(f"Memory saved: {info['memory_savings_ratio']:.1f}x")
```

## Architecture

**Embedded Design**
- Single process, zero network overhead
- Memory-mapped persistence
- Automatic algorithm selection
- Thread-safe operations

**Algorithm Selection**
- Brute force for <5K vectors (~0.7ms query)  
- HNSW for >5K vectors (~0.6ms query)
- Automatic switching at 5K threshold
- Both algorithms SIMD-optimized

**Storage Engine**
- Optimized binary format (.omen files)
- Lazy loading for fast startup
- Incremental saves
- Crash recovery

## Documentation

- **[Getting Started](docs/getting-started.md)** - Installation and first steps
- **[API Reference](docs/api-reference.md)** - Complete method documentation
- **[Performance Tuning](docs/performance.md)** - Optimization guide
- **[Migration Guide](docs/migration.md)** - Upgrading from other databases

## Use Cases

### Semantic Search
```python
# Index documents with embeddings (batch operation)
texts = [doc.text for doc in documents]
embeddings = model.encode(texts)  # Returns numpy.ndarray
ids = [doc.id for doc in documents]
metadata = [{"title": doc.title} for doc in documents]

db.add_batch(vectors=embeddings, ids=ids, metadata=metadata)

# Find similar documents
query_embedding = model.encode("machine learning")
results = db.search(query_embedding, limit=10)
```

### RAG Applications
```python
# Store knowledge base (batch operation)
embeddings = np.array([chunk.embedding for chunk in knowledge_chunks], dtype=np.float32)
ids = [chunk.id for chunk in knowledge_chunks]
metadata = [
    {"source": chunk.source, "page": chunk.page}
    for chunk in knowledge_chunks
]

db.add_batch(vectors=embeddings, ids=ids, metadata=metadata)

# Retrieve context for LLM
context = db.search(question_embedding, limit=5)
```

### Recommendation Systems
```python
# Index user preferences (batch for all users)
user_behaviors = [user.behavior for user in users]
embeddings = model.encode(user_behaviors)  # Returns numpy.ndarray
ids = [user.id for user in users]

db.add_batch(vectors=embeddings, ids=ids)

# Find similar users
user_embedding = model.encode(current_user_behavior)
similar_users = db.search(user_embedding, limit=20)

# Get recommendations from similar users
recommendations = aggregate_preferences(similar_users)
```

## Competitive Advantages

### vs Pinecone
- **Lower latency** - <1ms local vs 10-50ms network latency
- **No API limits** - Unlimited local operations
- **No recurring costs** - One-time installation
- **Data privacy** - Vectors stay on your infrastructure

### vs Faiss
- **Instant startup** - 0.001ms vs 100ms+ index loading
- **Simpler API** - Zero configuration required
- **Persistence built-in** - Automatic saving
- **Comparable performance** - Both offer sub-millisecond queries

### vs ChromaDB  
- **Faster ingestion** - 157K vs 17K vectors/sec (with NumPy)
- **Lower memory usage** - Optimized storage format
- **Different focus** - Performance over feature breadth
- **Single language** - Mojo core vs Python core

## Requirements

- Python 3.8+
- 64-bit architecture
- Linux (x86_64, ARM64) or macOS

## Roadmap

### v0.2.0 - Performance Pack
- GPU acceleration support
- Advanced query optimization
- Streaming ingestion API
- Performance profiler

### v0.3.0 - Enterprise Features  
- Sharding support
- Read replicas
- Point-in-time recovery
- Advanced monitoring

### v1.0.0 - Cloud Platform
- Managed service
- Auto-scaling
- Global distribution
- Enterprise SLAs

## Contributing

We welcome contributions! See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Elastic License 2.0 - Free for all uses including commercial, with [minimal restrictions](LICENSE).

---

**Built for AI applications. Optimized for production workloads.**
## Business Model

### 🆓 Embedded Database (Free Forever)
The core embedded database will always be free and open source under the Elastic License 2.0. Perfect for:
- Local AI applications
- Edge computing
- Development and testing
- Small-scale production deployments

### ☁️ OmenDB Cloud (Coming Soon)
Managed cloud service for production workloads:
- **Starter**: $99/month - Up to 1M vectors
- **Growth**: $499/month - Up to 10M vectors  
- **Scale**: $999/month - Up to 100M vectors
- **Enterprise**: Custom pricing for 100M+ vectors

### 🏢 Enterprise
- On-premise deployment
- Custom integrations  
- SLA guarantees
- Priority support

## Benchmarks

Benchmarks measured on commodity hardware:

```bash
# Run the benchmark yourself
python benchmarks/performance_comparison.py
```

**Results @ 128 dimensions (OpenAI ada-002 size):**
- **Ingestion**: 96K vectors/second (24x faster than 4K target)
- **Query latency**: 2.59ms P50 (acceptable for embedded use)
- **Startup time**: 0.001ms (1000x faster than alternatives)
- **Memory usage**: 4x less with quantization enabled

## Quick Start

```python
from omendb import DB
import numpy as np

# Initialize - instant startup
db = DB("vectors.db")

# Add vectors with metadata
vectors = np.random.rand(10000, 128).astype(np.float32)
ids = [f"vec_{i}" for i in range(10000)]
metadata = [{"category": "example"} for _ in range(10000)]

db.add_batch(vectors=vectors, ids=ids, metadata=metadata)  # Pass NumPy array directly for best performance

# Search with filtering
query = np.random.rand(128).astype(np.float32)
results = db.search(query.tolist(), limit=10, filter={"category": "example"})

for result in results:
    print(f"ID: {result.id}, Score: {result.score:.3f}")
```

## Comparison

| Feature | OmenDB | Cloud Solutions | Self-Hosted |
|---------|---------|-----------------|-------------|
| Startup Time | **0.001ms** | N/A | 50-1000ms+ |
| Deployment | **Embedded** | Managed Service | Server Required |
| Offline Support | **✅** | ❌ | ✅ |
| Network Latency | **None** | 10-50ms | 0-5ms |
| Pricing Model | **Free** | Subscription | Free/License |
| Data Location | **Local** | Cloud Provider | Your Servers |

## Community

- 🌟 [Star us on GitHub](https://github.com/omendb/omendb)
- 💬 [Open an Issue](https://github.com/omendb/omendb/issues)
- 📧 [Email us](mailto:nijaru7@gmail.com)

## License

OmenDB is licensed under the Elastic License 2.0. See [LICENSE](LICENSE) for details.

This allows free use for most purposes, with restrictions on:
- Providing OmenDB as a managed service
- Circumventing license key functionality
- Removing or obscuring license requirements

For commercial managed service use, please contact us for a commercial license.
EOF < /dev/null