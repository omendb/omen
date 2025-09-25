# Python API Reference

Complete reference for the OmenDB Python API.

## DB Class

The main interface for interacting with OmenDB.

### Constructor

```python
DB(path: str = "") -> DB
```

Creates or connects to an OmenDB database.

**Parameters:**
- `path` (str, optional): Path to the database file. If empty, creates an in-memory database.

**Example:**
```python
from omendb import DB

# File-based database
db = DB("my_vectors.omen")

# In-memory database
db = DB()
```

### Methods

#### add

```python
add(id: str, vector: List[float], metadata: Optional[Dict[str, Any]] = None) -> bool
```

Add a single vector to the database.

**Parameters:**
- `id` (str): Unique identifier for the vector
- `vector` (List[float]): Vector data as a list of floats
- `metadata` (Dict[str, Any], optional): Associated metadata

**Returns:**
- bool: True if successful

**Example:**
```python
success = db.add("vec1", [1.0, 2.0, 3.0], {"category": "example"})
```

#### add_batch

```python
add_batch(
    vectors: List[List[float]], 
    ids: List[str], 
    metadata: Optional[List[Dict[str, Any]]] = None
) -> List[bool]
```

Add multiple vectors efficiently in a single operation.

**Parameters:**
- `vectors` (List[List[float]]): List of vectors
- `ids` (List[str]): List of unique identifiers
- `metadata` (List[Dict[str, Any]], optional): List of metadata dicts

**Returns:**
- List[bool]: List of success flags for each vector

**Example:**
```python
vectors = [[1.0, 2.0], [3.0, 4.0], [5.0, 6.0]]
ids = ["vec1", "vec2", "vec3"]
metadata = [{"type": "a"}, {"type": "b"}, {"type": "c"}]

results = db.add_batch(vectors=vectors, ids=ids, metadata=metadata)
```

#### search

```python
search(
    vector: List[float], 
    limit: int = 10,
    filter: Optional[Dict[str, Any]] = None
) -> List[SearchResult]
```

Search for similar vectors.

**Parameters:**
- `vector` (List[float]): Query vector
- `limit` (int): Maximum number of results to return (default: 10)
- `filter` (Dict[str, Any], optional): Metadata filter criteria

**Returns:**
- List[SearchResult]: List of search results

**Example:**
```python
# Basic search
results = db.search([1.0, 2.0, 3.0], limit=5)

# Search with metadata filtering
results = db.search(
    [1.0, 2.0, 3.0], 
    limit=5,
    filter={"category": "example"}
)

for result in results:
    print(f"ID: {result.id}, Score: {result.score}")
    print(f"Metadata: {result.metadata}")
```

#### delete

```python
delete(id: str) -> bool
```

Delete a vector by ID.

**Parameters:**
- `id` (str): ID of the vector to delete

**Returns:**
- bool: True if the vector was deleted, False if not found

**Example:**
```python
success = db.delete("vec1")
```

#### clear

```python
clear() -> None
```

Remove all vectors from the database.

**Example:**
```python
db.clear()
```

#### info

```python
info() -> Dict[str, Any]
```

Get database statistics and configuration.

**Returns:**
- Dict containing:
  - `vector_count` (int): Number of vectors stored
  - `dimension` (int): Vector dimension
  - `algorithm` (str): Current algorithm ("flat" or "hnsw")
  - `status` (str): Database status
  - Additional implementation details

**Example:**
```python
info = db.info()
print(f"Vectors: {info['vector_count']}")
print(f"Dimension: {info['dimension']}")
print(f"Algorithm: {info['algorithm']}")
```

## SearchResult Class

Represents a single search result.

### Attributes

- `id` (str): Vector identifier
- `score` (float): Similarity score (0.0 to 1.0, higher is more similar)
- `metadata` (Optional[Dict[str, Any]]): Associated metadata
- `distance` (float): Raw distance value
- `vector` (Optional[List[float]]): The vector data (if requested)

**Example:**
```python
results = db.search([1.0, 2.0, 3.0], limit=5)

for result in results:
    print(f"ID: {result.id}")
    print(f"Score: {result.score:.3f}")
    print(f"Distance: {result.distance:.3f}")
    if result.metadata:
        print(f"Metadata: {result.metadata}")
```

## Exceptions

### ValidationError

Raised when input validation fails.

```python
from omendb.exceptions import ValidationError

try:
    db.add("id1", [1.0, 2.0])  # 2D vector
    db.add("id2", [1.0, 2.0, 3.0])  # 3D vector - will fail
except ValidationError as e:
    print(f"Validation error: {e}")
```

### DatabaseError

Raised for database-level errors.

```python
from omendb.exceptions import DatabaseError

try:
    db = DB("/invalid/path/to/database.omen")
except DatabaseError as e:
    print(f"Database error: {e}")
```

### DimensionMismatchError

Raised when vector dimensions don't match.

```python
from omendb.exceptions import DimensionMismatchError

try:
    db.add("id1", [1.0, 2.0])  # 2D
    db.add("id2", [1.0, 2.0, 3.0])  # 3D - will raise error
except DimensionMismatchError as e:
    print(f"Dimension error: {e}")
```

## Configuration

### Environment Variables

- `OMENDB_DEBUG`: Enable debug logging (set to "1" or "true")
- `OMENDB_TEST_MODE`: Enable test mode with smaller datasets (set to "quick")

**Example:**
```bash
OMENDB_DEBUG=1 python my_script.py
```

### Algorithm Switching

OmenDB automatically switches between algorithms based on data size:
- **Flat/Brute Force**: Used for < 5,000 vectors (perfect accuracy)
- **HNSW**: Used for ≥ 5,000 vectors (approximate, fast at scale)

You can monitor algorithm status:
```python
info = db.info()
print(f"Current algorithm: {info['algorithm']}")
print(f"Migration status: {info['status']}")
```

## Performance Tips

### 1. Use Batch Operations
```python
# Slow: Individual adds
for i in range(10000):
    db.add(f"id_{i}", vector[i])

# Fast: Batch add
db.add_batch(vectors=vectors, ids=ids, metadata=metadata)
```

### 2. Use NumPy Arrays
```python
import numpy as np

# Generate vectors with NumPy
vectors = np.random.rand(10000, 128).astype(np.float32)

# Convert to list for add_batch
db.add_batch(
    vectors=vectors.tolist(),
    ids=[f"vec_{i}" for i in range(10000)],
    metadata=[{} for _ in range(10000)]
)
```

### 3. Reuse Database Connections
```python
# Good: Reuse connection
db = DB("vectors.omen")
for batch in batches:
    db.add_batch(...)

# Avoid: Creating new connections repeatedly
for batch in batches:
    db = DB("vectors.omen")  # Unnecessary overhead
    db.add_batch(...)
```

## Complete Example

```python
from omendb import DB
import numpy as np

# Initialize database
db = DB("example.omen")

# Add vectors
vectors = np.random.rand(1000, 128).astype(np.float32)
ids = [f"doc_{i}" for i in range(1000)]
metadata = [{"category": "example", "index": i} for i in range(1000)]

db.add_batch(vectors=vectors.tolist(), ids=ids, metadata=metadata)

# Search
query = np.random.rand(128).astype(np.float32)
results = db.search(
    query.tolist(), 
    limit=10,
    filter={"category": "example"}
)

# Process results
for result in results:
    print(f"{result.id}: {result.score:.3f}")

# Get statistics
info = db.info()
print(f"Total vectors: {info['vector_count']}")
```