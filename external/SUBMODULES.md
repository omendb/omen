# External Submodules Setup

## Reference Implementations

### DiskANN (Microsoft Research)
```bash
git submodule add https://github.com/microsoft/DiskANN.git external/diskann
```
Primary reference for Vamana algorithm implementation.

### Modular/Mojo
```bash
git submodule add https://github.com/modularml/mojo.git external/modular
```
Mojo language reference and examples.

## Competitor Repositories (for benchmarking)

### Open Source Vector Databases

**Chroma** (Python, serverless architecture)
```bash
git submodule add https://github.com/chroma-core/chroma.git external/competitors/chroma
```

**Weaviate** (Go, HNSW-based)
```bash
git submodule add https://github.com/weaviate/weaviate.git external/competitors/weaviate
```

**Qdrant** (Rust, HNSW + filtering)
```bash
git submodule add https://github.com/qdrant/qdrant.git external/competitors/qdrant
```

**Milvus** (Go/C++, multiple indexes)
```bash
git submodule add https://github.com/milvus-io/milvus.git external/competitors/milvus
```

**LanceDB** (Rust, columnar format)
```bash
git submodule add https://github.com/lancedb/lancedb.git external/competitors/lancedb
```

### Embedded Databases (architecture reference)

**DuckDB** (C++, columnar analytical)
```bash
git submodule add https://github.com/duckdb/duckdb.git external/competitors/duckdb
```

## Benchmarking Frameworks

**ANN Benchmarks** (standard vector search benchmarks)
```bash
git submodule add https://github.com/erikbern/ann-benchmarks.git external/benchmarks/ann-benchmarks
```

**VectorDBBench** (comprehensive vector DB benchmarking)
```bash
git submodule add https://github.com/zilliztech/VectorDBBench.git external/benchmarks/vectordbbench
```

## Commands to Run

```bash
# Create directory structure
mkdir -p external/competitors
mkdir -p external/benchmarks

# Add core references
git submodule add https://github.com/microsoft/DiskANN.git external/diskann
git submodule add https://github.com/modularml/mojo.git external/modular

# Add competitors (choose relevant ones)
git submodule add https://github.com/chroma-core/chroma.git external/competitors/chroma
git submodule add https://github.com/lancedb/lancedb.git external/competitors/lancedb

# Add benchmark framework
git submodule add https://github.com/erikbern/ann-benchmarks.git external/benchmarks/ann-benchmarks

# Update all submodules
git submodule update --init --recursive
```

## Why These References Matter

### DiskANN
- Our core algorithm reference
- Contains C++ implementation we can validate against
- Has benchmark datasets and evaluation code

### Competitors
- **Chroma**: Similar Python interface, serverless architecture insights
- **LanceDB**: Rust embedded database, similar goals
- **Weaviate/Qdrant**: Production HNSW implementations to compare

### Benchmarking
- **ann-benchmarks**: Industry standard for vector search
- Includes SIFT, GIST, GloVe datasets
- Standard recall@k metrics

## Performance Comparison Strategy

1. **Use ANN-Benchmarks datasets**:
   - SIFT1M (128 dim, 1M vectors)
   - GIST1M (960 dim, 1M vectors)
   - GloVe (100 dim, 1.2M vectors)

2. **Standard Metrics**:
   - Recall@1, @10, @100
   - Queries per second
   - Index build time
   - Memory usage

3. **Comparison Script**:
```python
# tools/benchmark_comparison.py
import omendb
import chromadb
import lancedb

# Run same workload on each
# Measure: build time, query time, recall, memory
```

4. **Regular CI Benchmarks**:
```yaml
# .github/workflows/benchmark.yml
- Compare against previous OmenDB version
- Compare against fixed Chroma/Lance versions
- Track performance over time
```