# OmenDB API Reference

**Complete Python API documentation for v0.0.1**

## Core Classes

### DB (Main Database Interface)

```python
from omendb import DB

# Create database with instant startup
db = DB()

# Or with persistence
db = DB("vectors.omen")
```

#### Constructor Parameters
- **`path`** (str, optional): Path to the database file. If None, runs in-memory only.

#### Core Methods

##### `add(id: str, vector: List[float], metadata: Optional[Dict[str, Any]] = None) -> bool`
Add a single vector to the database.

```python
success = db.add("doc1", [1.0, 2.0, 3.0], {"category": "tech"})
```

- **`id`**: Unique identifier for the vector
- **`vector`**: List of float values representing the vector
- **`metadata`**: Optional dictionary of metadata
- **Returns**: True if successful, False otherwise
- **Raises**: `ValidationError` if vector dimension doesn't match database dimension

**Note**: The first vector added sets the database dimension. All subsequent vectors must have the same dimension.

##### `add_batch(vectors: Union[List[List[float]], np.ndarray], ids: Optional[List[str]] = None, metadata: Optional[List[Dict[str, Any]]] = None) -> List[str]`
Add multiple vectors efficiently using columnar format for optimal performance.

```python
# With numpy arrays (fastest - zero-copy optimization)
import numpy as np
vectors = np.random.rand(1000, 128).astype(np.float32)
ids = [f"doc_{i}" for i in range(1000)]
metadata = [{"category": "tech"} for _ in range(1000)]

result_ids = db.add_batch(vectors=vectors, ids=ids, metadata=metadata)

# With lists
vectors = [[1.0, 2.0, 3.0], [4.0, 5.0, 6.0], [7.0, 8.0, 9.0]]
result_ids = db.add_batch(vectors=vectors)  # Auto-generates IDs
```

- **`vectors`**: 2D array-like of shape (n_vectors, dimension)
- **`ids`**: Optional list of unique IDs. Auto-generated if None
- **`metadata`**: Optional list of metadata dicts
- **Returns**: List of IDs for all successfully added vectors
- **Performance**: 99K+ vectors/second with numpy arrays

##### `search(vector: List[float], limit: int = 10, filter: Optional[Dict[str, Any]] = None) -> List[SearchResult]` 🆕
Search for similar vectors with optional metadata filtering.

```python
results = db.search([1.0, 2.0, 3.0], limit=5, filter={"category": "tech"})
for result in results:
    print(f"ID: {result.id}, Score: {result.score:.3f}")
```

- **`vector`**: Query vector (must match database dimension)
- **`limit`**: Number of results to return (default: 10)
- **`filter`**: Optional metadata filter
- **Returns**: List of SearchResult objects sorted by score (highest first)
- **Raises**: `ValidationError` if query vector dimension doesn't match
- **Performance**: Sub-millisecond queries with production-ready latency


##### `count() -> int` 🆕
Get the total number of vectors in the database.

```python
total = db.count()
print(f"Database contains {total} vectors")
```

##### `size() -> int` 🆕
Alias for count() for compatibility.

##### `clear() -> bool` 🆕
Remove all vectors from the database and reset to initial state.

```python
# Clear all data
db.clear()

# Can now add vectors with different dimension
db.add("new_vec", [1.0, 2.0, 3.0, 4.0])  # 4D instead of previous 3D
```

- **Returns**: True if successful
- **Note**: Resets dimension state, allowing new dimension on next add

##### `delete(id: str) -> bool`
Remove a vector from the database.

```python
success = db.delete("doc1")
```

##### `update(id: str, vector: List[float], metadata: Optional[Dict[str, Any]] = None) -> bool`
Update an existing vector.

```python
success = db.update("doc1", [1.1, 2.1, 3.1], {"category": "updated"})
```

##### `get(id: str) -> Optional[VectorData]`
Retrieve a vector and its metadata by ID.

```python
data = db.get("doc1")
if data:
    print(f"Vector: {data.vector}, Metadata: {data.metadata}")
```

##### `info() -> Dict[str, Any]`
Get database information and statistics.

```python
info = db.info()
print(f"Vectors: {info['vector_count']}")
print(f"Dimension: {info['dimension']}")
print(f"Algorithm: {info['algorithm']}")
print(f"Memory: {info['memory_usage_mb']:.2f} MB")
```

##### `save(path: str) -> bool`
Save database to file.

```python
success = db.save("my_vectors.omen")
```

##### `load(path: str) -> bool`
Load database from file.

```python
success = db.load("my_vectors.omen")
```

##### `enable_quantization() -> bool`
Enable 8-bit scalar quantization for 4x memory savings.

```python
db.enable_quantization()
# Now uses ~1 byte per dimension instead of 4
```

#### Collections Methods ⚠️ **Not Available**

**Note**: Collections API is not available due to Mojo language limitations (requires module-level variables).

**Alternative**: Use ID prefixes for logical separation:
```python
# Instead of collections, use prefixed IDs
db.add("images_img1", image_vector, {"type": "image"})
db.add("text_doc1", text_vector, {"type": "text"})

# Search with metadata filtering
image_results = db.search(query, limit=10, filter={"type": "image"})
```

### Collection Class ⚠️ **Not Available**

**Status**: Collections are not available due to Mojo language limitations. Use the main DB with ID prefixes and metadata filtering as an alternative.

**Future**: Will be available when Mojo adds support for module-level variables.

### SearchResult Class

Represents a search result with similarity score.

```python
@dataclass
class SearchResult:
    id: str                                    # Vector ID
    score: float                              # Similarity score (0-1, higher=better)
    vector: Optional[List[float]] = None      # Vector data if requested
    metadata: Optional[Dict[str, Any]] = None # Metadata if exists
```

### VectorData Class

Represents stored vector data.

```python
@dataclass
class VectorData:
    id: str                          # Vector ID
    vector: List[float]              # Vector data
    metadata: Dict[str, Any]         # Metadata (empty dict if none)
```

## Exceptions

### ValidationError
Raised for invalid inputs (dimension mismatch, bad IDs, etc).

```python
from omendb.exceptions import ValidationError

try:
    db.add("vec1", [1.0, 2.0])  # 2D
    db.add("vec2", [1.0, 2.0, 3.0])  # 3D - error!
except ValidationError as e:
    print(f"Error: {e}")
```

### DatabaseError
Raised for database operation failures.

```python
from omendb.exceptions import DatabaseError

try:
    db.load("corrupted.omen")
except DatabaseError as e:
    print(f"Error: {e}")
```

## Migration Guide

### From Other Databases

#### From Pinecone
```python
# Pinecone
index.upsert(vectors=[("id1", [1.0, 2.0, 3.0], {"meta": "data"})])
results = index.query(vector=[1.0, 2.0, 3.0], top_k=10)

# OmenDB
db.add("id1", [1.0, 2.0, 3.0], {"meta": "data"})
results = db.search([1.0, 2.0, 3.0], limit=10)
```

#### From ChromaDB
```python
# ChromaDB
collection = client.create_collection("name")
collection.add(ids=["id1"], embeddings=[[1.0, 2.0, 3.0]])

# OmenDB (using prefixes as alternative)
db.add("name_id1", [1.0, 2.0, 3.0], {"collection": "name"})
```

#### From Faiss
```python
# Faiss (complex setup)
import faiss
index = faiss.IndexFlatL2(128)
index.add(vectors)

# OmenDB (simple, instant)
db = DB()
db.add_batch(vectors=vectors)
```

## Performance Tips

1. **Use numpy arrays** for batch operations (99K+ vec/s vs slower with lists)
2. **Enable quantization** for 4x memory savings with <2% accuracy loss
3. **Use ID prefixes** to separate different vector types (Collections not available)
4. **Batch operations** when adding many vectors
5. **Pre-allocate IDs** instead of auto-generating for better performance

## Examples

### Basic Usage
```python
from omendb import DB

# Create database
db = DB()

# Add vectors
db.add("doc1", [0.1, 0.2, 0.3], {"type": "document"})
db.add("doc2", [0.4, 0.5, 0.6], {"type": "image"})

# Search
results = db.search([0.15, 0.25, 0.35], limit=1)
print(f"Most similar: {results[0].id} (score: {results[0].score:.3f})")
```

### Logical Separation Example
```python
# Use ID prefixes for different embedding types (Collections not available)
db.add("images_img_001", clip_embedding, {"type": "image", "dim": 512})
db.add("text_doc_001", bert_embedding, {"type": "text", "dim": 768})

# Note: All vectors must be same dimension (single DB design)
# Use separate DB instances for different dimensions if needed

# Search within types using metadata filtering
similar_images = db.search(query_image, limit=10, filter={"type": "image"})
similar_docs = db.search(query_text, limit=5, filter={"type": "text"})
```

### High-Performance Batch Loading
```python
import numpy as np
from omendb import DB

# Generate test data
n_vectors = 1_000_000
dimension = 128
vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
ids = [f"vec_{i}" for i in range(n_vectors)]

# Load with batching for optimal performance
db = DB()
batch_size = 10_000

for i in range(0, n_vectors, batch_size):
    batch_vectors = vectors[i:i+batch_size]
    batch_ids = ids[i:i+batch_size]
    db.add_batch(vectors=batch_vectors, ids=batch_ids)
    
print(f"Loaded {db.count()} vectors")
```

### Metadata Filtering
```python
# Add vectors with rich metadata
db.add("product_1", embedding, {
    "name": "Laptop",
    "category": "electronics",
    "price": 999.99,
    "brand": "TechCorp"
})

# Filter by metadata
electronics = db.search(
    query_embedding,
    limit=10,
    filter={"category": "electronics"}
)

# Multiple filters (exact match)
affordable_tech = db.search(
    query_embedding,
    limit=10,
    filter={
        "category": "electronics",
        "brand": "TechCorp"
    }
)
```