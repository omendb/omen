# OmenDB Examples

Learn by example with our comprehensive collection of code samples.

## 🚀 Quick Start

**New to OmenDB?** Start with these examples in order:

1. **[Getting Started](getting_started/quickstart.py)** - Your first vector search (5 min)
2. **[Working with Data](basics/working_with_data.py)** - CRUD operations and batch processing
3. **[Performance Demo](performance/performance_showcase.py)** - See 157K vectors/second in action

## 📚 Examples by Use Case

### 🔍 Search Applications
- **[Semantic Search](use_cases/semantic_search.py)** - Build text search with embeddings
- **[Image Search](use_cases/image_search.py)** - Find similar images
- **[Recommendation System](use_cases/recommendation_system.py)** - User/item recommendations
- **[Anomaly Detection](use_cases/anomaly_detection.py)** - Find outliers in data

### 🤖 AI/LLM Applications  
- **[RAG Quickstart](use_cases/rag_quickstart.py)** - Simple RAG in under 100 lines
- **[Production RAG](use_cases/rag_production.py)** - Production-ready RAG with best practices
- **[LangChain Integration](integrations/langchain_example.py)** - Use as LangChain VectorStore
- **[OpenAI Embeddings](integrations/openai_embeddings_example.py)** - Store OpenAI embeddings

### ⚡ Performance & Scale
- **[Performance Showcase](performance/performance_showcase.py)** - 157K vectors/sec demo
- **[Instant Startup](performance/instant_startup_demo.py)** - 0.001ms initialization
- **[Batch Operations](performance/batch_operations.py)** - Optimize for throughput
- **[Memory Optimization](performance/memory_optimization.py)** - Use quantization

### 🔧 Framework Integrations
- **[PyTorch](integrations/pytorch_integration.py)** - Store PyTorch tensors
- **[TensorFlow](integrations/tensorflow_integration.py)** - TensorFlow integration
- **[NumPy](performance/performance_numpy.py)** - Zero-copy NumPy arrays
- **[LlamaIndex](integrations/llamaindex_example.py)** - LlamaIndex vector store

### 🏗️ Production Patterns
- **[Production Deployment](production/deployment_patterns.py)** - Best practices
- **[Monitoring](production/observability_example.py)** - Logging and metrics
- **[Error Handling](production/error_handling.py)** - Robust error management
- **[Data Persistence](production/backup_restore.py)** - Backup strategies

## 🎯 Choose Your Path

### "I want to build a search engine"
→ Start with [Semantic Search](use_cases/semantic_search.py)

### "I need RAG for my chatbot"  
→ Try [RAG Quickstart](use_cases/rag_quickstart.py) then [Production RAG](use_cases/rag_production.py)

### "I'm evaluating OmenDB's performance"
→ Run [Performance Showcase](performance/performance_showcase.py) and [Benchmarks](benchmarks/performance_comparison.py)

### "I use LangChain/LlamaIndex"
→ See [LangChain Integration](integrations/langchain_example.py) or [LlamaIndex Integration](integrations/llamaindex_example.py)

## 💡 Tips for Running Examples

### Installation
```bash
# Install OmenDB first
pip install omendb

# Some examples have optional dependencies
pip install numpy sentence-transformers openai langchain
```

### Running Examples
```bash
# Run any example directly
python examples/getting_started/quickstart.py

# Examples handle missing dependencies gracefully
python examples/integrations/openai_embeddings_example.py
# Output: ⚠️ OpenAI not available, using mock embeddings
```

### Quick Mode for CI/Testing
```bash
# Run examples in quick mode (smaller datasets)
OMENDB_TEST_MODE=quick python examples/performance/performance_showcase.py
```

## 📊 Example Categories Explained

### Getting Started
Basic examples for new users. No external dependencies required.

### Use Cases
Complete applications showing real-world usage patterns.

### Performance
Demonstrations of OmenDB's speed and efficiency.

### Integrations
How to use OmenDB with popular ML/AI frameworks.

### Production
Patterns for deploying OmenDB in production environments.

## 🤝 Contributing Examples

We welcome example contributions! Good examples should:
- ✅ Solve a real problem
- ✅ Be well-commented
- ✅ Handle dependencies gracefully
- ✅ Include expected output
- ✅ Run in under 30 seconds

See [CONTRIBUTING.md](../CONTRIBUTING.md) for guidelines.

## ❓ FAQ

**Q: Do I need to install all dependencies?**  
A: No, examples work with fallbacks when dependencies are missing.

**Q: Why do some examples use mock embeddings?**  
A: To let you try examples without API keys or large models.

**Q: Can I use these examples in my project?**  
A: Yes! All examples are MIT licensed. Copy and adapt freely.

**Q: How do I report an issue with an example?**  
A: Open an issue on [GitHub](https://github.com/omendb/omendb/issues).

---

**Ready to start?** Try the [5-minute quickstart](getting_started/quickstart.py) →