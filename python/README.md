# Python Examples and Client Scripts

This directory contains Python examples and client code for connecting to OmenDB.

## Contents

- `example.py` - Example usage of OmenDB Python client
- `omendb/` - Python client library for OmenDB

## Usage

### Installing

```bash
pip install -e .  # Install from pyproject.toml at root
```

### Running Examples

```bash
# Start OmenDB server first
cargo run --release

# Then run Python client
python example.py
```

## Client Library

The `omendb/` package provides Python client access to OmenDB via:
- PostgreSQL wire protocol (using `psycopg2` or `asyncpg`)
- REST API (using `requests` or `httpx`)

## See Also

- `../learneddb/` - Python bindings (PyO3) for direct library access
- `../examples/` - More extensive examples in various languages
