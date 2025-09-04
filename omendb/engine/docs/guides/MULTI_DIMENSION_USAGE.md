# Multi-Dimension Use Cases Guide

OmenDB's single-dimension-per-process design covers 99% of embedded use cases. Here's how to handle the rare multi-dimension scenarios.

## Common Patterns

### 1. Process Isolation (Recommended)

```python
import subprocess
import json

def query_dimension(dimension, vector):
    """Query vectors of specific dimension in isolated process"""
    cmd = [
        "python", "-c",
        f"""
import json
from omendb import DB
db = DB('vectors_{dimension}d.omen')
results = db.query({vector}, top_k=10)
print(json.dumps([{{'id': r.id, 'score': r.similarity}} for r in results]))
"""
    ]
    result = subprocess.run(cmd, capture_output=True, text=True)
    return json.loads(result.stdout)

# Use different dimensions
text_results = query_dimension(768, text_vector)
image_results = query_dimension(512, image_vector)
```

### 2. Using MultiDimensionDB Helper

```python
from omendb.multi import MultiDimensionDB

# Handles process isolation automatically
mdb = MultiDimensionDB()

# Add vectors of different dimensions
mdb.add("text_1", text_vector_768d, dimension=768)
mdb.add("image_1", image_vector_512d, dimension=512)

# Query each dimension
text_results = mdb.query(query_768d, dimension=768, top_k=10)
image_results = mdb.query(query_512d, dimension=512, top_k=10)
```

### 3. Microservice Pattern

```python
# text_service.py (768D embeddings)
from flask import Flask, jsonify, request
from omendb import DB

app = Flask(__name__)
db = DB("text_embeddings.omen")

@app.route("/add", methods=["POST"])
def add_text():
    data = request.json
    db.add(data["id"], data["vector"])
    return jsonify({"status": "ok"})

# image_service.py (512D embeddings)
# Similar structure for image embeddings
```

## Real-World Examples

### Multimodal Search System

```python
# search_service.py
class MultimodalSearch:
    def __init__(self):
        self.text_port = 5001
        self.image_port = 5002
        
    def search(self, text_query=None, image_query=None):
        results = []
        
        if text_query:
            # Query text service
            resp = requests.post(f"http://localhost:{self.text_port}/query", 
                               json={"vector": text_query})
            results.extend(resp.json()["results"])
            
        if image_query:
            # Query image service  
            resp = requests.post(f"http://localhost:{self.image_port}/query",
                               json={"vector": image_query})
            results.extend(resp.json()["results"])
            
        return self.merge_results(results)
```

### Model Migration

```python
# Gradual migration from old to new embeddings
def migrate_embeddings():
    # Old model process
    old_db = subprocess.Popen(["python", "old_model_service.py"])
    
    # New model process
    new_db = subprocess.Popen(["python", "new_model_service.py"])
    
    # Gradually migrate traffic
    for doc in documents:
        if should_use_new_model(doc):
            add_to_new_model(doc)
        else:
            add_to_old_model(doc)
```

## Performance Considerations

| Approach | Overhead | Use Case |
|----------|----------|----------|
| Process Isolation | ~10ms IPC | Best for different models |
| Microservices | ~1-5ms HTTP | Best for scaling |
| MultiDimensionDB | ~10ms subprocess | Best for simplicity |

## Future: Collections API (v0.2.0)

```python
# Coming soon - idiomatic collections API
db = DB()
db.collection("text", dimension=768).add("doc1", text_vector)
db.collection("images", dimension=512).add("img1", image_vector)
```

Until then, process isolation provides clean separation with minimal overhead.