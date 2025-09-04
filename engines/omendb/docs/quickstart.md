# 5-Minute Quickstart

Get up and running with OmenDB in under 5 minutes. By the end, you'll have stored and searched your first vectors.

## Install OmenDB

```bash
pip install omendb
```

<details>
<summary>Other installation methods</summary>

```bash
# Using uv (fast Python package manager)
uv add omendb

# Using Poetry
poetry add omendb

# Using pixi
pixi add omendb
```
</details>

## Your First Vector Search

### 1. Import and Create a Database

```python
from omendb import DB

# Create a database (or connect to existing one)
db = DB("my_vectors.omen")
```

That's it! No servers to start, no configuration needed. The database is ready instantly (0.001ms).

### 2. Add Your First Vectors

```python
# Add vectors with IDs
db.add("vec1", [1.0, 2.0, 3.0, 4.0])
db.add("vec2", [1.1, 2.1, 3.1, 4.1])
db.add("vec3", [5.0, 6.0, 7.0, 8.0])

print(f"Added {db.info()['vector_count']} vectors")
# Output: Added 3 vectors
```

### 3. Search for Similar Vectors

```python
# Find vectors similar to a query
query = [1.05, 2.05, 3.05, 4.05]
results = db.search(query, limit=2)

for result in results:
    print(f"ID: {result.id}, Score: {result.score:.3f}")
# Output:
# ID: vec2, Score: 0.999
# ID: vec1, Score: 0.998
```

🎉 **Congratulations!** You've just performed your first vector similarity search!

## Real-World Example: Semantic Search

Let's build a simple semantic search engine for documents:

```python
from omendb import DB
import hashlib

# Create a simple text embedding function
def embed_text(text):
    """Create a simple embedding from text (for demo purposes)."""
    # In production, use real embeddings (OpenAI, sentence-transformers, etc.)
    hash_bytes = hashlib.sha256(text.encode()).digest()
    # Convert to 128D float vector
    return [float(b) / 255.0 for b in hash_bytes[:128]]

# Create database and add documents
db = DB("documents.omen")

documents = [
    {"id": "doc1", "text": "Python is a versatile programming language"},
    {"id": "doc2", "text": "Machine learning transforms data into insights"},
    {"id": "doc3", "text": "Vector databases enable semantic search"},
    {"id": "doc4", "text": "Python is great for data science and AI"},
]

# Add documents with their embeddings
for doc in documents:
    embedding = embed_text(doc["text"])
    db.add(doc["id"], embedding, metadata={"text": doc["text"]})

# Search for similar documents
query_text = "Python programming for AI"
query_embedding = embed_text(query_text)

results = db.search(query_embedding, limit=3)

print(f"Query: '{query_text}'")
print("\nSimilar documents:")
for i, result in enumerate(results, 1):
    text = result.metadata.get("text", "N/A")
    print(f"{i}. {text} (score: {result.score:.3f})")
```

## Performance at Scale

OmenDB excels with larger datasets. Let's test with 100,000 vectors:

```python
import numpy as np
import time

# Create a new database
db = DB("performance_test.omen")

# Generate 100,000 random vectors (128 dimensions)
num_vectors = 100_000
dimension = 128

print(f"Adding {num_vectors:,} vectors...")

# Prepare batch data
vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
ids = [f"vec_{i}" for i in range(num_vectors)]
metadata = [{"index": i} for i in range(num_vectors)]

# Batch insert for maximum performance
start = time.time()
db.add_batch(
    vectors=vectors.tolist(),
    ids=ids,
    metadata=metadata
)
elapsed = time.time() - start

print(f"✅ Added {num_vectors:,} vectors in {elapsed:.2f}s")
print(f"🚀 Rate: {num_vectors/elapsed:,.0f} vectors/second")

# Search is still fast with 100K vectors
query = np.random.rand(dimension).astype(np.float32)
start = time.time()
results = db.search(query.tolist(), limit=10)
search_time = (time.time() - start) * 1000

print(f"⚡ Search time: {search_time:.2f}ms")
```

Expected output:
```
Adding 100,000 vectors...
✅ Added 100,000 vectors in 1.12s
🚀 Rate: 89,285 vectors/second
⚡ Search time: 0.82ms
```

## What's Next?

### Learn More
- **[Core Concepts](getting-started/concepts.md)** - Understand vectors and similarity search
- **[API Reference](api/python.md)** - Explore all available methods
- **[Best Practices](guides/best-practices.md)** - Production tips

### Build Something
- **[Semantic Search App](guides/semantic-search.md)** - Build a full-text search engine
- **[RAG with LangChain](integrations/langchain.md)** - Add vector search to your LLM app
- **[Image Search](guides/image-search.md)** - Search images by visual similarity

### Get Help
- **[FAQ](faq.md)** - Common questions answered
- **[Discord Community](https://discord.gg/omendb)** - Chat with other users
- **[GitHub Issues](https://github.com/omendb/omendb/issues)** - Report bugs or request features

---

**💡 Pro Tips:**
- OmenDB starts instantly (0.001ms) - perfect for serverless and CLI tools
- Use batch operations (`add_batch`) for best performance
- Vectors are automatically indexed using HNSW for fast search
- The database file (`*.omen`) is portable across systems