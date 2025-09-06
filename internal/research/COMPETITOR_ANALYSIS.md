# Competitor Architecture Analysis

*Learning from production vector databases*

## Chroma Architecture Insights

### Serverless Design
**Source**: https://trychroma.com/engineering/serverless

**Key Patterns**:
1. **Metadata/Vector Separation**
   - SQLite for metadata (collections, documents)
   - Binary files for vectors (numpy arrays)
   - S3 backing for persistence

2. **Ephemeral Workers**
   - Workers have local SSD cache
   - Read-through from S3
   - No persistent state on workers

**What OmenDB Should Adopt**:
```mojo
struct OmenDB:
    var metadata: SQLiteDB      # Collection info, vector IDs
    var vectors: MmapStorage    # Actual vector data
    var index: DiskANNGraph     # Search structure
```

### WAL Implementation
**Source**: https://trychroma.com/engineering/wal3

**Evolution**:
- v1: SQLite WAL (too slow)
- v2: Custom WAL with batching
- v3: Async flush with double buffering

**Critical Pattern for 25K Fix**:
```python
# Chroma's approach (simplified)
class AsyncWAL:
    def write(self, data):
        self.buffer.append(data)
        if self.buffer.size > threshold:
            # DON'T BLOCK HERE
            spawn_async(self.flush_to_storage)
            self.swap_buffers()
```

## Architecture Comparison

| Feature | Chroma | Weaviate | Qdrant | LanceDB | OmenDB (Target) |
|---------|---------|----------|---------|----------|------------|
| **Language** | Python | Go | Rust | Rust | Mojo |
| **Index** | HNSW | HNSW | HNSW+Filters | IVF_PQ | DiskANN |
| **Storage** | SQLite+Files | Custom | RocksDB | Lance Format | Mmap |
| **Embedding** | 384 default | Any | Any | Any | 1536 focus |
| **Serverless** | Yes | No | No | Yes | Future |
| **WAL** | Custom v3 | Yes | Yes | Arrow-based | Needed |

## Performance Targets

Based on competitor benchmarks:

### Chroma (Python)
- **Build**: 10K vectors/sec
- **Query**: 1000 QPS (cached)
- **Memory**: 500 bytes/vector

### Weaviate (Go) 
- **Build**: 25K vectors/sec
- **Query**: 5000 QPS
- **Memory**: 400 bytes/vector

### Qdrant (Rust)
- **Build**: 30K vectors/sec  
- **Query**: 10000 QPS
- **Memory**: 350 bytes/vector

### LanceDB (Rust)
- **Build**: 50K vectors/sec
- **Query**: 2000 QPS
- **Memory**: 250 bytes/vector

### OmenDB Target (Mojo)
- **Build**: 40K vectors/sec (after 25K fix)
- **Query**: 8000 QPS
- **Memory**: 288 bytes/vector ✅ (already achieved)

## Key Learnings

### From Chroma
1. **Async WAL is critical** - Don't block writes
2. **Metadata separation** - SQLite for metadata works well
3. **Serverless needs caching** - Read-through pattern

### From Weaviate
1. **HNSW is proven** - But DiskANN scales better
2. **Go is fast enough** - But Mojo should be faster
3. **REST API standard** - Everyone uses it

### From Qdrant
1. **Rust is ideal** - Memory safety + performance
2. **Filtering matters** - Metadata filtering during search
3. **gRPC for performance** - Better than REST for high throughput

### From LanceDB
1. **Columnar is efficient** - For analytical queries
2. **Embedded first** - Like DuckDB approach
3. **Arrow integration** - Standard data format

## OmenDB Differentiation

### What We Do Better
1. **DiskANN scales beyond RAM** - HNSW doesn't
2. **Mojo performance** - Theoretical 2-5x over Rust
3. **PQ compression** - 288 bytes/vector is excellent
4. **Embedded focus** - Not trying to be cloud-first

### What We Need to Match
1. **Async operations** - Critical for scale
2. **WAL for durability** - Everyone has it
3. **Metadata filtering** - Table stakes
4. **Standard benchmarks** - ANN-benchmarks compatibility

## Benchmark Implementation Plan

```python
# benchmarks/compare_competitors.py
import time
import numpy as np

def benchmark_omendb():
    from omendb import DB
    db = DB()
    
    # Load standard dataset
    vectors = load_sift1m()
    
    # Measure build time
    start = time.time()
    db.add_batch(vectors)
    build_time = time.time() - start
    
    # Measure query time
    queries = load_sift1m_queries()
    start = time.time()
    for q in queries:
        db.search(q, k=10)
    query_time = time.time() - start
    
    return {
        'build_rate': len(vectors) / build_time,
        'qps': len(queries) / query_time,
        'memory': get_process_memory()
    }

# Similar for Chroma, Lance, etc.
```

## Action Items

1. **Implement async WAL** (learn from Chroma v3)
2. **Separate metadata storage** (SQLite like Chroma)
3. **Add standard benchmarks** (ANN-benchmarks)
4. **Compare regularly** (CI/CD pipeline)

## Commands to Run Comparisons

```bash
# Clone competitors
cd external/
./setup_competitors.sh

# Run benchmark suite
python benchmarks/compare_all.py --dataset sift1m

# Generate report
python benchmarks/generate_report.py --output results.md
```