# Single Database Design

OmenDB follows an embedded database architecture optimized for performance and simplicity.

## Design Philosophy

OmenDB uses a single database instance per process, similar to embedded databases like SQLite. This design enables:

- **Instant startup** (0.001ms) - No initialization overhead
- **Peak performance** - Direct memory access without handle lookups
- **Zero overhead** - No instance management or synchronization
- **Simplicity** - No complex lifecycle management

## How It Works

All `DB()` calls in a process share the same underlying database:

```python
# Both reference the same database
db1 = omendb.DB()
db2 = omendb.DB()

# This is by design for embedded use cases
db1.add("vec1", [1.0, 2.0, 3.0])
results = db2.search([1.0, 2.0, 3.0])  # Returns vec1
```

## Working with Multiple Dimensions

Since each database stores vectors of a fixed dimension, here are recommended patterns:

### 1. Process Isolation (Recommended)

Run each dimension in a separate process:

```python
# text_embeddings.py (768D)
import omendb
db = omendb.DB()
db.add("doc1", text_vector_768d)

# image_embeddings.py (512D)
import omendb
db = omendb.DB()
db.add("img1", image_vector_512d)
```

### 2. Subprocess Pattern

```python
import subprocess

# Process different dimensions separately
subprocess.run(["python", "process_text.py"])    # 768D vectors
subprocess.run(["python", "process_images.py"])  # 512D vectors
```

### 3. Microservice Architecture

Deploy separate services for different vector dimensions:

```python
# text_service.py
from fastapi import FastAPI
import omendb

app = FastAPI()
db = omendb.DB()  # 768D text embeddings

# image_service.py
from fastapi import FastAPI
import omendb

app = FastAPI()
db = omendb.DB()  # 512D image embeddings
```

## Benefits for Common Use Cases

This design is optimal for typical embedded vector database applications:

- **Single embedding model** - Most apps use one model (e.g., OpenAI ada-002)
- **Consistent dimensions** - Face recognition (128D), documents (768D), etc.
- **Maximum performance** - No overhead from multi-instance management
- **Simple deployment** - No configuration or setup required

## Comparison with Other Databases

| Database | Multiple Instances | Startup Time | Design |
|----------|-------------------|--------------|---------|
| OmenDB | Shared in process | 0.001ms | Embedded |
| SQLite | Separate files | ~1ms | Embedded |
| ChromaDB | Separate collections | 50ms | Client-server |
| Pinecone | Separate indexes | N/A | Cloud service |

## Future Roadmap

Future versions may support multiple collections within a process while maintaining the performance benefits of the current design. The API is designed to support this transparently when available.

## Best Practices

1. **Use one dimension per process** - Aligns with the embedded design
2. **Deploy services by dimension** - Natural microservice boundary
3. **Leverage process isolation** - Security and fault tolerance
4. **Keep it simple** - Let OmenDB handle the complexity

This design philosophy prioritizes the common case (single dimension per application) while providing clear patterns for advanced use cases.