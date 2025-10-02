# LearnedDB - Python Bindings

This directory contains Python bindings (via PyO3) for the OmenDB learned index library.

## Purpose

Provides Python-native access to OmenDB's learned index implementation for:
- Prototyping and experimentation
- Python-based data science workflows
- Integration with Python tools (NumPy, Pandas, etc.)

## Building

```bash
# Build Python extension
cargo build --release --features python

# Or use maturin for development
pip install maturin
maturin develop --features python
```

## Usage

```python
import omendb_lib

# Create learned index
index = omendb_lib.LearnedIndex()

# Insert data
for i in range(1000):
    index.insert(i, f"value_{i}")

# Query
result = index.get(42)
print(result)  # "value_42"
```

## Current Status

⚠️ **Experimental** - Python bindings exist but may not be fully feature-complete with the main Rust library.

For production use, prefer the Rust API directly or use the PostgreSQL wire protocol.

## See Also

- `../python/` - Python examples and client scripts
- `../src/` - Core Rust implementation
