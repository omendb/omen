# OmenDB API Guide

**Version**: v0.3.0 (December 2024)  
**Performance**: 24,029 vec/s with auto-batching

## Quick Start

OmenDB offers three Python APIs optimized for different use cases:

### 1. Auto-batching API (Recommended) ✨

**Best for**: Most applications - provides 7.6x speedup with zero code changes

```python
from omendb.api_batched import AutoBatchDB
import numpy as np

# Create database with automatic batching
db = AutoBatchDB(batch_size=500, batch_timeout=0.1)

# Add vectors - automatically batched internally!
vectors = np.random.randn(1000, 128).astype(np.float32)
for i, vec in enumerate(vectors):
    db.add(f"doc_{i}", vec, {"type": "embedding"})

# Search triggers auto-flush
results = db.search(vectors[0], limit=10)

# Check batching statistics
stats = db.stats()
print(f"Average batch size: {stats['average_batch_size']:.1f}")
print(f"Total flushes: {stats['total_flushes']}")
```

**Performance**: 24,029 vec/s (7.6x faster than standard API)

### 2. Standard API

**Best for**: Simple applications, small datasets, compatibility

```python
from omendb import DB
import numpy as np

# Create standard database
db = DB()
db.clear()

# Add vectors one by one
vector = np.random.randn(128).astype(np.float32)
db.add("doc1", vector, {"type": "text"})

# Batch add for better performance
vectors = np.random.randn(1000, 128).astype(np.float32)
ids = db.add_batch(vectors)  # Auto-generates IDs

# Search
results = db.search(vector, limit=10)
for r in results:
    print(f"{r.id}: {r.score:.3f}")
```

**Performance**: 3,100 vec/s (individual), 26,451 vec/s (batch)

### 3. Optimized API

**Best for**: NumPy-heavy workloads with trusted inputs

```python
from omendb.api_optimized import OptimizedDB
import numpy as np

# Create optimized database
db = OptimizedDB()
db.clear()

# Skips validation for numpy arrays
vectors = np.random.randn(1000, 128).astype(np.float32)
for i, vec in enumerate(vectors):
    db.add(f"doc_{i}", vec)  # Faster numpy handling

# Ultra-fast batch operation
db.add_batch_optimized(vectors)  # Direct numpy passing
```

**Performance**: 3,381 vec/s (marginal improvement due to FFI bottleneck)

## Performance Comparison

| API | Individual Add | Batch Add | Use Case |
|-----|---------------|-----------|----------|
| Standard | 3,100 vec/s | 26,451 vec/s | General use |
| **Auto-batching** | **24,029 vec/s** | 26,451 vec/s | **Production** ✨ |
| Optimized | 3,381 vec/s | 26,451 vec/s | NumPy-heavy |

## API Configuration

### Auto-batching Parameters

```python
db = AutoBatchDB(
    batch_size=500,      # Vectors per batch (optimal: 100-1000)
    batch_timeout=0.1    # Max seconds before flush (optimal: 0.01-1.0)
)
```

**Tuning tips:**
- **High throughput**: Large batch_size (500-1000), longer timeout
- **Low latency**: Small batch_size (50-100), short timeout
- **Mixed workload**: Medium batch_size (200), 0.1s timeout

### Buffer Configuration

```python
db = DB()
db.configure(
    buffer_size=10000,   # Internal buffer size
    algorithm='diskann'  # Always DiskANN (deprecated parameter)
)
```

## Common Patterns

### 1. Bulk Import with Progress

```python
from omendb.api_batched import AutoBatchDB
import numpy as np
from tqdm import tqdm

db = AutoBatchDB(batch_size=1000)

# Import with progress bar
vectors = np.random.randn(100000, 128).astype(np.float32)
for i in tqdm(range(len(vectors))):
    db.add(f"vec_{i}", vectors[i])

# Final flush
db.flush()
print(f"Imported {db.size()} vectors")
```

### 2. Semantic Search

```python
def semantic_search(query_text, top_k=10):
    # Convert text to embedding (using your model)
    query_vector = embed_text(query_text)
    
    # Search with auto-batching DB
    results = db.search(query_vector, limit=top_k)
    
    return [(r['id'], r['score']) for r in results]
```

### 3. Context Manager Pattern

```python
with AutoBatchDB(batch_size=500) as db:
    # Add vectors
    for vec in vectors:
        db.add(generate_id(), vec)
    # Auto-flushes on exit
```

### 4. Monitoring Performance

```python
db = AutoBatchDB(batch_size=500)

# ... add vectors ...

# Check performance
stats = db.stats()
print(f"Total adds: {stats['total_adds']}")
print(f"Total flushes: {stats['total_flushes']}")
print(f"Average batch: {stats['average_batch_size']:.1f}")
print(f"Efficiency: {stats['average_batch_size'] / stats['batch_size_config'] * 100:.1f}%")
```

## Best Practices

### DO ✅

1. **Use Auto-batching API** for production workloads
2. **Keep vectors in NumPy arrays** for best performance
3. **Configure batch_size** based on your workload
4. **Call flush()** before critical searches
5. **Use context managers** for automatic cleanup

### DON'T ❌

1. **Don't use individual adds** in tight loops without batching
2. **Don't convert NumPy to lists** unnecessarily
3. **Don't set batch_size too small** (<50) or too large (>5000)
4. **Don't forget to flush** before program exit
5. **Don't mix dimensions** - all vectors must be same size

## Migration Guide

### From Standard to Auto-batching

```python
# Before (Standard API)
from omendb import DB
db = DB()
for i, vec in enumerate(vectors):
    db.add(f"doc_{i}", vec)  # 3,100 vec/s

# After (Auto-batching API)
from omendb.api_batched import AutoBatchDB
db = AutoBatchDB(batch_size=500)
for i, vec in enumerate(vectors):
    db.add(f"doc_{i}", vec)  # 24,029 vec/s - 7.6x faster!
```

**No other code changes needed!**

## Performance Tips

1. **Optimal batch size**: 500 vectors (based on testing)
2. **Use NumPy arrays**: Avoid Python lists when possible
3. **Pre-allocate arrays**: Better memory efficiency
4. **Batch similar operations**: Group adds, searches, deletes
5. **Monitor statistics**: Use `db.stats()` to tune parameters

## Troubleshooting

### Slow Performance

```python
# Check if batching is working
stats = db.stats()
if stats['average_batch_size'] < 50:
    print("Batches too small - increase batch_size or timeout")
```

### Memory Issues

```python
# Flush more frequently
db = AutoBatchDB(batch_size=100, batch_timeout=0.01)
```

### Search Not Finding Recent Adds

```python
# Manually flush before searching
db.flush()
results = db.search(query)
```

## Architecture

OmenDB uses a three-tier architecture:

1. **Python API Layer** - Handles batching and validation
2. **Buffer Layer** - SimpleBuffer for O(1) insertion
3. **Index Layer** - DiskANN for scalable search

The auto-batching API reduces FFI overhead by 7.6x while maintaining the same simple interface.

---

For more details, see [ARCHITECTURE.md](ARCHITECTURE.md)