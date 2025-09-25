# Getting Started with OmenDB

Quick examples to get you started with OmenDB embedded vector database.

## Prerequisites

```bash
# Install Pixi package manager
curl -fsSL https://pixi.sh/install.sh | bash

# Clone OmenDB
git clone --recurse-submodules https://github.com/omendb/omendb.git
cd omenDB

# Setup environment
pixi install
pixi shell
```

## Examples Overview

### 🚀 Quick Start Examples
- **[quickstart.py](quickstart.py)** - 5-minute introduction to OmenDB
- **[basic_operations.py](basic_operations.py)** - Core operations: insert, search, metadata
- **[file_storage.py](file_storage.py)** - Persistent storage with database.omen files

### 📊 Production Examples  
- **[production_scale.py](production_scale.py)** - 10K+ vector production patterns
- **[performance_testing.py](performance_testing.py)** - Benchmarking and monitoring
- **[error_handling.py](error_handling.py)** - Robust error handling patterns

### 🔧 MLOps Integration
- **[mlops_pipeline.py](mlops_pipeline.py)** - ML model deployment integration
- **[batch_processing.py](batch_processing.py)** - Efficient batch operations
- **[versioning_example.py](versioning_example.py)** - Model versioning and lifecycle

### 🎯 Use Case Examples
- **[document_search.py](document_search.py)** - Document similarity search
- **[recommendation_engine.py](recommendation_engine.py)** - Product recommendations  
- **[semantic_search.py](semantic_search.py)** - Semantic text search
- **[image_search.py](image_search.py)** - Image similarity search

## Running Examples

```bash
# Start with the quickstart
python examples/getting_started/quickstart.py

# Try production scale example
python examples/getting_started/production_scale.py

# Run performance testing
python examples/getting_started/performance_testing.py
```

## Next Steps

1. **Try the [quickstart.py](quickstart.py)** - Get OmenDB running in 5 minutes
2. **Explore [production examples](#-production-examples)** - Learn production patterns
3. **Check [MLOps integration](#-mlops-integration)** - Integrate with your ML pipeline
4. **Browse [use case examples](#-use-case-examples)** - Find patterns for your domain

## Need Help?

- 📚 **Documentation**: [omendb.io/docs](https://omendb.io/docs)
- 🐛 **Issues**: [GitHub Issues](https://github.com/omendb/omendb/issues)
- 💬 **Community**: [Discussions](https://github.com/omendb/omendb/discussions)
- 🌐 **Website**: [omendb.io](https://omendb.io)
- 📧 **Support**: [support@omendb.io](mailto:support@omendb.io)