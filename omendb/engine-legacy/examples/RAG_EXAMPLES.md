# RAG Pipeline Examples with OmenDB

This directory contains comprehensive examples of Retrieval-Augmented Generation (RAG) pipelines built with OmenDB.

## 📚 Available Examples

### 1. Simple RAG Pipeline (`simple_rag_pipeline.py`)
A working demonstration with minimal dependencies, perfect for understanding RAG fundamentals.

**Features:**
- ✅ Document ingestion and chunking
- ✅ Simple TF-IDF embeddings (no external dependencies)
- ✅ Vector storage with metadata
- ✅ Similarity-based retrieval
- ✅ Basic answer generation
- ✅ Performance metrics

**Usage:**
```bash
PYTHONPATH=python pixi run python examples/simple_rag_pipeline.py
```

### 2. Production RAG Pipeline (`production_rag_pipeline.py`)
A comprehensive, production-ready implementation with advanced features.

**Features:**
- 🚀 Multiple embedding providers (OpenAI, Sentence Transformers, Cohere)
- 📄 Sophisticated document processing and chunking
- 🔍 Advanced query processing with reranking
- 📊 Performance monitoring and metrics
- ⚙️ Configurable settings
- 🛡️ Error handling and fallbacks
- 📋 Async processing support
- 💾 State persistence

**Dependencies (optional):**
```bash
pip install sentence-transformers  # For local embeddings
pip install openai                 # For OpenAI embeddings
pip install cohere                 # For Cohere embeddings
pip install tiktoken               # For better tokenization
```

**Usage:**
```bash
PYTHONPATH=python pixi run python examples/production_rag_pipeline.py
```

## 🏗️ RAG Architecture Overview

### Core Components

1. **Document Processor**
   - Splits documents into manageable chunks
   - Handles overlap for context preservation
   - Extracts and preserves metadata

2. **Embedding Provider**
   - Converts text to numerical vectors
   - Supports multiple embedding models
   - Caching for performance optimization

3. **Vector Database (OmenDB)**
   - Stores document embeddings
   - Provides fast similarity search
   - Manages metadata associations

4. **Query Pipeline**
   - Processes user queries
   - Retrieves relevant document chunks
   - Generates contextual responses

### Data Flow

```
Document → Chunking → Embedding → Storage (OmenDB)
                                      ↓
Query → Embedding → Similarity Search → Context → LLM → Response
```

## 🚀 Getting Started

### Quick Start (Simple Pipeline)

```python
from simple_rag_pipeline import SimpleRAGPipeline, Document

# Initialize pipeline
pipeline = SimpleRAGPipeline("my_rag.omen")

# Add documents
doc = Document(
    id="example_doc",
    content="Your document content here...",
    metadata={"title": "Example", "category": "demo"}
)
pipeline.ingest_document(doc)

# Query
response = pipeline.query("What is this document about?")
print(response["answer"])
```

### Advanced Usage (Production Pipeline)

```python
from production_rag_pipeline import ProductionRAGPipeline, RAGConfig, Document

# Configure pipeline
config = RAGConfig(
    db_path="production_rag.omen",
    embedding_provider="sentence_transformers",
    chunk_size=512,
    retrieval_k=10
)

# Initialize
pipeline = ProductionRAGPipeline(config)

# Batch ingest documents
documents = [...]  # Your documents
results = pipeline.ingest_documents_batch(documents)

# Advanced querying
response = pipeline.query("Your question here", k=5)
print(f"Answer: {response.answer}")
print(f"Sources: {len(response.sources)}")
print(f"Processing time: {response.processing_time:.2f}s")
```

## 📊 Performance Optimization

### Embedding Strategy
- **Local Models**: Fast, private, no API costs
- **API Models**: Higher quality, require internet and API keys
- **Hybrid**: Use local for indexing, API for queries

### Chunking Best Practices
- **Size**: 256-512 tokens for most use cases
- **Overlap**: 10-20% for context preservation
- **Strategy**: Sentence-aware splitting preferred

### Database Optimization
- **Batch Operations**: Use `add_batch()` for multiple documents
- **Memory Management**: Monitor vector count and dimensions
- **Persistence**: Regular saves for data durability

## 🔧 Configuration Guide

### Embedding Providers

#### Sentence Transformers (Recommended for Local)
```python
config = RAGConfig(
    embedding_provider="sentence_transformers",
    embedding_model="sentence-transformers/all-MiniLM-L6-v2"  # Fast, good quality
)
```

#### OpenAI (Best Quality)
```python
config = RAGConfig(
    embedding_provider="openai",
    embedding_model="text-embedding-ada-002"  # Requires OPENAI_API_KEY
)
```

### Query Optimization
```python
config = RAGConfig(
    retrieval_k=20,           # Initial retrieval count
    rerank_k=5,               # Final results after reranking
    similarity_threshold=0.7,  # Filter low-similarity results
    max_context_tokens=4000   # LLM context limit
)
```

## 🛠️ Integration Examples

### With LangChain
```python
from langchain.embeddings import SentenceTransformerEmbeddings
from omendb import DB

# Use LangChain embeddings with OmenDB
embeddings = SentenceTransformerEmbeddings()
db = DB("langchain_rag.omen")

# Integration in your LangChain pipeline...
```

### With PyTorch/TensorFlow
See `examples/pytorch_integration.py` and `examples/tensorflow_integration.py` for tensor-based workflows.

## 📈 Monitoring & Metrics

### Key Metrics to Track
- **Ingestion Rate**: Documents/chunks per second
- **Query Latency**: End-to-end response time
- **Retrieval Accuracy**: Relevance of retrieved chunks
- **Cache Hit Rate**: Embedding cache effectiveness

### Example Monitoring
```python
metrics = pipeline.get_metrics()
print(f"Documents indexed: {metrics['documents_indexed']}")
print(f"Average query time: {metrics['avg_query_time']:.2f}s")
print(f"Cache hit rate: {metrics['cache_hits'] / metrics['queries_processed']:.2%}")
```

## 🐛 Troubleshooting

### Common Issues

1. **Dimension Mismatch**
   - Ensure embedding dimensions match database initialization
   - Check model documentation for expected dimensions

2. **Poor Retrieval Quality**
   - Try different embedding models
   - Adjust chunk size and overlap
   - Implement reranking strategies

3. **Slow Performance**
   - Use batch operations for ingestion
   - Enable embedding caching
   - Consider dimensionality reduction

4. **Memory Issues**
   - Monitor vector count and dimensions
   - Implement periodic cleanup
   - Use memory-efficient embedding models

### Debug Mode
```python
import logging
logging.basicConfig(level=logging.DEBUG)

# Now you'll see detailed pipeline logs
```

## 🔮 Future Enhancements

- **Multi-modal RAG**: Support for images, audio, video
- **Graph RAG**: Incorporating knowledge graphs
- **Adaptive Chunking**: Dynamic chunk sizing based on content
- **Advanced Reranking**: Cross-encoder and learned reranking
- **Distributed RAG**: Multi-node deployment strategies

## 📖 Additional Resources

- [OmenDB Documentation](../README.md)
- [RAG Best Practices](https://arxiv.org/abs/2005.11401)
- [Embedding Model Comparison](https://huggingface.co/spaces/mteb/leaderboard)
- [Vector Database Benchmarks](https://benchmark.vectorview.ai/)

## 🤝 Contributing

We welcome contributions to improve these RAG examples:

1. **New Embedding Providers**: Add support for more models
2. **Advanced Chunking**: Implement semantic chunking strategies  
3. **Reranking Models**: Integrate cross-encoders and learned reranking
4. **Domain-Specific Examples**: Add specialized use cases
5. **Performance Optimizations**: Improve speed and accuracy

## 📄 License

These examples are provided under the same license as OmenDB. See the main project LICENSE file for details.