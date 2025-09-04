# OmenDB Python Bindings

High-performance Python interface to the OmenDB embedded vector database.

## Features

- **Embedded Database**: SQLite-style single-file vector storage
- **High Performance**: SIMD-optimized operations via Mojo backend
- **Dual-Mode Architecture**: Supports both embedded and server deployments
- **Binary Quantization**: 32x memory compression
- **Cross-Platform**: Works on Linux, macOS, and Windows
- **Python-Friendly**: Familiar dict/list interfaces with type hints

## Quick Start

```python
import omendb

# Create embedded database
with omendb.EmbeddedDB("vectors.omen") as db:
    # Set vector dimension
    db.set_dimension(384)
    
    # Insert vectors with metadata
    db.insert("doc1", [0.1, 0.2, ...], {"title": "Document 1"})
    db.insert("doc2", [0.3, 0.4, ...], {"title": "Document 2"})
    
    # Search for similar vectors
    results = db.search([0.15, 0.25, ...], limit=5)
    print(f"Found similar documents: {results}")
```

## Installation

### Option 1: Pre-built Wheels (Recommended)

```bash
pip install omendb
```

### Option 2: Build from Source

```bash
# Clone repository
git clone https://github.com/user/omendb.git
cd omendb

# Build Python bindings
cd python
python build_bindings.py

# Install in development mode
pip install -e .
```

### Requirements

- Python 3.8+
- Mojo runtime (for building from source)

## API Reference

### EmbeddedDB

Main database class providing vector storage and search.

```python
class EmbeddedDB:
    def __init__(self, path: str, read_only: bool = False, log_level: int = 2)
    def set_dimension(self, dimension: int) -> None
    def insert(self, id: str, vector: List[float], metadata: Optional[Dict[str, str]] = None) -> bool
    def search(self, query: List[float], limit: int = 10) -> List[str]
    def delete(self, id: str) -> bool
    def flush(self) -> None
    def is_healthy(self) -> bool
    def get_stats(self) -> str
    def close(self) -> None
```

### Vector

High-performance vector wrapper.

```python
class Vector:
    def __init__(self, data: List[float])
    @property
    def dimension(self) -> int
    def to_list(self) -> List[float]
    def __len__(self) -> int
    def __getitem__(self, index: int) -> float
```

### Metadata

Key-value metadata storage.

```python
class Metadata:
    def __init__(self, data: Optional[Dict[str, str]] = None)
    def set(self, key: str, value: str) -> None
    def get(self, key: str, default: Optional[str] = None) -> Optional[str]
    def contains(self, key: str) -> bool
    def __setitem__(self, key: str, value: str) -> None
    def __getitem__(self, key: str) -> str
```

## Examples

### Basic Usage

```python
import omendb

# Create database
db = omendb.EmbeddedDB("my_vectors.omen")

# Configure for 128-dimensional vectors
db.set_dimension(128)

# Insert document embeddings
documents = [
    ("intro.txt", [0.1] * 128, {"title": "Introduction", "author": "Alice"}),
    ("methods.txt", [0.2] * 128, {"title": "Methods", "author": "Bob"}),
    ("results.txt", [0.15] * 128, {"title": "Results", "author": "Alice"}),
]

for doc_id, embedding, metadata in documents:
    db.insert(doc_id, embedding, metadata)

# Search for similar documents
query_embedding = [0.12] * 128
similar_docs = db.search(query_embedding, limit=3)
print(f"Similar documents: {similar_docs}")

# Close database
db.close()
```

### Context Manager

```python
import omendb

with omendb.EmbeddedDB("vectors.omen") as db:
    db.set_dimension(256)
    
    # Database automatically closed when exiting context
    db.insert("vec1", [1.0] * 256)
    results = db.search([1.1] * 256)
```

### Error Handling

```python
import omendb
from omendb import DatabaseError, MemoryError

try:
    db = omendb.EmbeddedDB("vectors.omen")
    db.set_dimension(384)
    db.insert("doc1", [0.1] * 384)
    
except DatabaseError as e:
    print(f"Database error: {e}")
except MemoryError as e:
    print(f"Memory error: {e}")
finally:
    if 'db' in locals():
        db.close()
```

### Batch Operations

```python
import omendb

with omendb.EmbeddedDB("batch.omen") as db:
    db.set_dimension(128)
    
    # Insert multiple vectors
    vectors = [(f"vec_{i}", [i * 0.1] * 128) for i in range(1000)]
    
    for vec_id, data in vectors:
        db.insert(vec_id, data)
    
    # Flush to ensure persistence
    db.flush()
    
    print(f"Database stats:\\n{db.get_stats()}")
```

## Performance

The Python bindings provide high-performance access to the Mojo-optimized core:

- **Insertion**: >10K vectors/second (depending on dimension)
- **Search**: >1K queries/second with sub-millisecond latency
- **Memory**: <50MB idle footprint for embedded mode
- **Compression**: 32x reduction with binary quantization

## Architecture

The Python bindings use two approaches:

### Modern Bindings (Default)
- Direct Mojo-Python integration via `python.bindings`
- Automatic memory management
- Optimal performance with minimal overhead

### C FFI Fallback
- Compatible with all Mojo versions
- Manual handle management
- Slightly higher overhead but universal compatibility

The bindings automatically select the best available approach.

## Building from Source

### Prerequisites

1. **Mojo SDK**: Install from [Modular](https://www.modular.com/mojo)
2. **Python 3.8+**: With development headers
3. **Build Tools**: Platform-specific compilers

### Build Steps

```bash
# Navigate to Python bindings
cd python

# Build the native library
python build_bindings.py

# Install in development mode
pip install -e .

# Run tests
python -m pytest tests/
```

### Build Options

```bash
# Build with debug symbols
OMENDB_DEBUG=1 python build_bindings.py

# Force C FFI approach
OMENDB_USE_FFI=1 python build_bindings.py

# Specify Mojo path
MOJO_PATH=/path/to/mojo python build_bindings.py
```

## Troubleshooting

### Common Issues

**ImportError: No module named 'omendb'**
- Solution: Install with `pip install -e .` or build bindings first

**Library not found errors**
- Solution: Ensure Mojo runtime is installed and accessible
- Check `LD_LIBRARY_PATH` (Linux) or `DYLD_LIBRARY_PATH` (macOS)

**Permission errors**
- Solution: Use virtual environment or install with `--user` flag

**Performance issues**
- Check if native bindings are loaded (vs. stub mode)
- Verify SIMD optimizations are enabled in Mojo build

### Debug Information

```python
import omendb

# Check binding mode
db = omendb.EmbeddedDB("test.omen")
print(repr(db))  # Shows binding mode (native/stub)

# Verify database health
print(f"Database healthy: {db.is_healthy()}")
print(f"Stats:\\n{db.get_stats()}")
```

## Contributing

1. Fork the repository
2. Create feature branch: `git checkout -b feature-name`
3. Make changes and add tests
4. Ensure all tests pass: `python -m pytest`
5. Submit pull request

### Development Setup

```bash
# Clone with submodules
git clone --recursive https://github.com/user/omendb.git

# Set up development environment
cd omendb/python
pip install -e ".[dev]"

# Run tests
python -m pytest tests/ -v

# Format code
black omendb/
isort omendb/
```

## License

Apache License 2.0 - see [LICENSE](../LICENSE) for details.

## Support

- Documentation: [docs.omendb.dev](https://docs.omendb.dev)
- Issues: [GitHub Issues](https://github.com/user/omendb/issues)
- Discussions: [GitHub Discussions](https://github.com/user/omendb/discussions)