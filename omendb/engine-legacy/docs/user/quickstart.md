# OmenDB Quickstart Guide

**Get started with OmenDB in 5 minutes**

## 🚀 Installation

### Python Package (Recommended)
```bash
pip install omendb
```

### From Source
```bash
git clone https://github.com/omendb/omendb
cd omendb
pixi shell
```

## 💡 Basic Usage

### Simple Vector Database
```python
from omendb import DB

# Create database (auto-saves to file)
db = DB("my_vectors.omen")

# Add vectors (plain Python lists)
db.add("doc1", [0.1, 0.2, 0.3, 0.4])
db.add("doc2", [0.5, 0.6, 0.7, 0.8])

# Search for similar vectors
results = db.search([0.1, 0.2, 0.3, 0.4], limit=5)
for result in results:
    print(f"ID: {result.id}, Score: {result.score:.3f}")
```

### Context Manager (Recommended)
```python
from omendb import DB

# Automatic resource management
with DB("my_vectors.omen") as db:
    # Add vectors
    db.add("doc1", [0.1, 0.2, 0.3])
    db.add("doc2", [0.4, 0.5, 0.6])
    
    # Search
    results = db.search([0.1, 0.2, 0.3], limit=2)
    print(f"Found {len(results)} results")
    
    # Database auto-saved on exit
```

## 🔧 Working with Vectors

### Vector Format
```python
# Vectors are plain Python lists - no wrapper classes needed
vector = [1.0, 2.0, 3.0, 4.0]  # 4-dimensional vector
db.add("my_id", vector)

# Works with any dimension (auto-detected)
high_dim_vector = [0.1] * 128  # 128-dimensional
db.add("high_dim", high_dim_vector)
```

### Integration with ML Libraries
```python
import numpy as np
from omendb import DB

# Works seamlessly with NumPy
embedding = np.array([0.1, 0.2, 0.3, 0.4])
db.add("numpy_vector", embedding.tolist())

# From pandas DataFrame
import pandas as pd
df = pd.DataFrame({'vectors': [[1,2,3], [4,5,6]]})
for i, vector in enumerate(df['vectors']):
    db.add(f"row_{i}", vector)
```

## 📊 Database Operations

### Check Database Info
```python
# Get database information
info = db.info()
print(f"Vectors: {db.count()}")  # Convenience method!
print(f"Algorithm: {info.get('algorithm', 'brute_force')}")
print(f"Dimension: {info.get('dimension', 0)}")
```

### Similarity Testing
```python
# Get specific vector by ID
vector_data = db.get("doc1")
if vector_data:
    vector, metadata = vector_data
    print(f"Vector: {vector}")
```

## 🎯 Complete Example

```python
from omendb import DB
import random

# Sample data (simulating embeddings)
documents = [
    ("doc1", "Machine learning transforms software"),
    ("doc2", "Vector databases enable semantic search"), 
    ("doc3", "Python is great for data science"),
    ("doc4", "Embedded databases simplify deployment")
]

# Create database and add documents
with DB("example.omen") as db:
    # Generate sample embeddings (normally from embedding model)
    for doc_id, text in documents:
        # Simulate embedding generation
        embedding = [random.uniform(-1, 1) for _ in range(64)]
        db.add(doc_id, embedding)
    
    # Search for similar documents
    query_embedding = [random.uniform(-1, 1) for _ in range(64)]
    results = db.search(query_embedding, limit=3)
    
    print("Search Results:")
    for result in results:
        print(f"  {result.id}: {result.score:.3f}")
    
    # Database automatically saved
```

## 🚀 Next Steps

### Advanced Features
- **Multiple Distance Metrics**: L2, cosine, inner product, L2 squared
- **Mixed Precision**: Automatic memory optimization with float16/int8
- **Metadata Filtering**: Complex queries with where clauses
- **Zero-overhead Metrics**: Export to Prometheus, StatsD, JSON

### Learn More
- **API Reference**: See `docs/user/api-reference.md` for complete API
- **Examples**: Check `examples/` directory for real applications
- **Tutorial**: Build a semantic search engine with OmenDB

### Get Help
- **Website**: Visit [omendb.io](https://omendb.io) for more information
- **GitHub Issues**: Report bugs at [github.com/omendb/omendb/issues](https://github.com/omendb/omendb/issues)
- **Documentation**: Complete guides at [omendb.io/docs](https://omendb.io/docs)
- **Community**: Join discussions at [github.com/omendb/omendb/discussions](https://github.com/omendb/omendb/discussions)

## 📝 API Summary

### Core Operations
```python
from omendb import DB

db = DB("file.omen")          # Create/open database
db.add(id, vector)            # Add vector (List[float])
results = db.search(vector)   # Search similar vectors
db.info()                     # Database information
db.close()                    # Close database
```

### Best Practices
- ✅ Use context managers: `with DB() as db:`
- ✅ Consistent vector dimensions within database
- ✅ Unique IDs for each vector
- ✅ Check `info()` for database information

OmenDB makes vector search simple and fast! 🚀