# OmenDB API Philosophy

**Make the fast path the default path**

## 🎯 Core Principles

### 1. **Performance by Default**
- Auto-detect NumPy arrays and use zero-copy
- Auto-batch single operations when beneficial
- Auto-convert lists to optimal format internally
- Pre-allocate memory based on usage patterns

### 2. **Universal Input Support**
```python
# All of these should "just work" and be fast:
db.add("id1", [1.0, 2.0, 3.0])                    # Python list
db.add("id2", np.array([1.0, 2.0, 3.0]))         # NumPy array
db.add("id3", torch.tensor([1.0, 2.0, 3.0]))     # PyTorch tensor
db.add("id4", tf.constant([1.0, 2.0, 3.0]))      # TensorFlow tensor
db.add("id5", pd.Series([1.0, 2.0, 3.0]))        # Pandas Series
```

### 3. **Smart Batching**
```python
# Auto-batch for performance
for id, vec in data:
    db.add(id, vec)  # Internally batches every 1000 vectors

# Or explicit batching
with db.batch():  # Context manager
    for id, vec in data:
        db.add(id, vec)  # Queued until context exits
```

### 4. **Common Workload Optimizations**

#### RAG/Semantic Search
```python
# Single API that handles everything
results = db.search(
    query="What is machine learning?",  # Auto-embeds if string
    embedding_model="openai",            # Or "sentence-transformers", etc.
    top_k=10
)
```

#### Recommendation Systems
```python
# Native support for user-item matrices
db.add_matrix(user_item_matrix, 
              row_ids=user_ids, 
              col_metadata=item_features)

# Fast similarity search
similar_users = db.find_similar_rows("user123", n=10)
similar_items = db.find_similar_cols("item456", n=20)
```

#### Time-Series Embeddings
```python
# Sliding window support
db.add_timeseries(
    embeddings,
    timestamps=timestamps,
    window_size=100,
    stride=50
)

# Temporal queries
results = db.query_timerange(
    vector=query,
    start_time="2024-01-01",
    end_time="2024-01-31"
)
```

## 🚀 Implementation Strategy

### Phase 1: Smart Defaults
```python
class DB:
    def __init__(self, path=None, **kwargs):
        # Auto-detect optimal settings
        self.batch_size = self._detect_optimal_batch_size()
        self.use_mmap = self._should_use_mmap()
        self.preallocate = self._estimate_size()
```

### Phase 2: Universal Input Handling
```python
def add(self, id: str, vector: VectorLike, metadata=None):
    """Accept any vector-like input and optimize automatically."""
    
    # Convert to optimal format
    if isinstance(vector, np.ndarray):
        # Already optimal, use zero-copy
        return self._add_numpy(id, vector, metadata)
    
    elif torch and isinstance(vector, torch.Tensor):
        # Convert torch to numpy (zero-copy if possible)
        return self._add_numpy(id, vector.numpy(), metadata)
    
    elif tf and isinstance(vector, tf.Tensor):
        # TensorFlow to numpy
        return self._add_numpy(id, vector.numpy(), metadata)
    
    elif pd and isinstance(vector, pd.Series):
        # Pandas to numpy
        return self._add_numpy(id, vector.values, metadata)
    
    else:
        # Python list - batch if beneficial
        if self._should_batch:
            self._batch_buffer.append((id, vector, metadata))
            if len(self._batch_buffer) >= self.batch_size:
                self._flush_batch()
        else:
            # Direct add for small datasets
            return self._add_list(id, vector, metadata)
```

### Phase 3: Workload-Specific APIs
```python
# RAG Workload
class RAGDB(DB):
    def __init__(self, embedding_model="sentence-transformers/all-MiniLM-L6-v2"):
        super().__init__()
        self.embedder = AutoModel(embedding_model)
    
    def add_documents(self, documents, batch_size=100):
        """Embed and index documents efficiently."""
        embeddings = self.embedder.encode(documents, batch_size=batch_size)
        ids = [f"doc_{i}" for i in range(len(documents))]
        self.add_numpy(ids, embeddings, 
                      metadata=[{"text": doc} for doc in documents])

# Recommendation Workload  
class RecDB(DB):
    def add_interactions(self, user_item_df):
        """Optimized for recommendation systems."""
        # Convert sparse interactions to dense embeddings
        embeddings = self._factorize(user_item_df)
        self.add_dataframe(embeddings)
```

## 📊 Benchmarks Across Frameworks

| Input Type | Current | Target | Method |
|------------|---------|--------|--------|
| NumPy Array | 246K vec/s | 250K vec/s | Zero-copy |
| PyTorch Tensor | N/A | 200K vec/s | .numpy() zero-copy |
| TF Tensor | N/A | 200K vec/s | .numpy() |
| Pandas DataFrame | N/A | 180K vec/s | .values + batch |
| Python Lists | 50K vec/s | 100K vec/s | Smart batching |

## 🎨 Final API Examples

### Simple & Fast
```python
# Just works, automatically fast
db = DB()
db.add("id1", [1, 2, 3])  # Auto-optimized

# Batch operations are automatic
for i, vec in enumerate(large_dataset):
    db.add(f"id_{i}", vec)  # Internally batched
```

### Framework Integration
```python
# PyTorch workflow
model = torch.load("model.pt")
embeddings = model.encode(data)
db.add_batch(ids, embeddings)  # Zero-copy from PyTorch

# Pandas workflow  
df = pd.read_csv("embeddings.csv")
db.add_dataframe(df, id_col="id", vector_col="embedding")

# HuggingFace workflow
from transformers import AutoModel
model = AutoModel.from_pretrained("sentence-transformers/all-MiniLM-L6-v2")
embeddings = model.encode(sentences)
db.add_batch(sentences, embeddings)  # IDs auto-generated
```

### Production Ready
```python
# Monitor performance
with db.profile() as prof:
    db.add_batch(vectors)
print(f"Achieved {prof.vectors_per_second} vec/s")

# Auto-scaling
db = DB(autoscale=True)  # Automatically adjusts algorithm thresholds
```

## 🔑 Key Insight

**Users shouldn't need to think about performance** - the API should guide them to the fast path naturally. By accepting common formats and auto-optimizing, we make high performance the default experience.