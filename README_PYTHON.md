# OmenDB Python Package

**10x faster database with machine learning-powered indexing**

OmenDB uses learned indexes to predict data locations, achieving 2-10x performance improvements over traditional B-tree databases for sequential workloads.

## Installation

```bash
pip install omendb
```

## Quick Start

```python
import omendb

# Open database with learned index
db = omendb.open("./mydb.omen", index_type="linear")

# Insert data
db.put(1, b"hello")
db.put(2, b"world")

# Bulk insert for better performance
data = [(i, f"value_{i}") for i in range(10000)]
db.bulk_insert(data)

# Query data
value = db.get(1)  # b"hello"

# Range queries
results = db.range(0, 100)  # Returns list of (key, value) tuples

# Transactions
txn = db.begin_transaction()
db.txn_put(txn, 1000, b"transaction_value")
db.commit(txn)

# Benchmark performance
print(db.benchmark(10000))
```

## Index Types

- **`none`**: Standard B-tree index (baseline)
- **`linear`**: Linear learned index (2-5x faster)
- **`rmi`**: Recursive Model Index (3-10x faster)

Choose based on your data distribution:
- Use `linear` for uniformly distributed data
- Use `rmi` for complex distributions
- Use `none` as fallback for random access patterns

## Performance

On sequential workloads (timestamps, IDs, ordered data):

| Operation | B-tree | Linear Index | RMI Index |
|-----------|--------|--------------|-----------|
| Point Lookup | 1x | 2-3x faster | 3-5x faster |
| Range Query | 1x | 3-5x faster | 5-10x faster |
| Bulk Insert | 1x | 1.5x faster | 1.5x faster |

## NumPy Integration

```python
import numpy as np

# Generate keys with NumPy
keys = np.arange(0, 100000, dtype=np.int64)
values = [f"value_{i}".encode() for i in range(100000)]

# Efficient bulk insert from NumPy
db.bulk_insert_numpy(keys, values)
```

## Transaction Support

Full ACID compliance with MVCC (Multi-Version Concurrency Control):

```python
# Begin transaction with isolation level
txn = db.begin_transaction("read_committed")

try:
    db.txn_put(txn, 100, b"value1")
    db.txn_put(txn, 101, b"value2")

    # Read your writes
    value = db.txn_get(txn, 100)  # b"value1"

    db.commit(txn)
except Exception as e:
    db.rollback(txn)
    raise
```

Isolation levels:
- `read_uncommitted`
- `read_committed` (default)
- `repeatable_read`
- `serializable`

## Architecture

OmenDB uses a hybrid architecture:

1. **Hot Data (In-Memory)**: Recently accessed data with learned indexes for O(1) access
2. **Cold Data (RocksDB)**: Persistent storage with LSM-tree structure
3. **Learned Indexes**: ML models that predict data locations

This design achieves:
- Sub-microsecond latency for hot data
- Efficient memory usage
- Crash recovery and durability
- Horizontal scalability

## Use Cases

OmenDB excels at:
- **Time-series data**: Timestamps, logs, metrics
- **Sequential IDs**: User IDs, order numbers
- **Sorted datasets**: Leaderboards, rankings
- **IoT data**: Sensor readings with timestamps
- **Financial data**: Trade history, price feeds

## Benchmarking

Run the included benchmark:

```python
python -c "import omendb; db = omendb.open('./bench.db'); print(db.benchmark(100000))"
```

Or use the example script:

```bash
python example.py
```

## Requirements

- Python 3.8+
- NumPy 1.20+
- x86_64 or ARM64 processor
- Linux, macOS, or Windows

## Building from Source

```bash
# Clone repository
git clone https://github.com/omendb/core.git
cd core

# Install build dependencies
pip install maturin

# Build and install
maturin develop --release
```

## Contributing

We welcome contributions! See [CONTRIBUTING.md](https://github.com/omendb/core/blob/main/CONTRIBUTING.md)

## License

MIT License - see [LICENSE](https://github.com/omendb/core/blob/main/LICENSE)

## Support

- Documentation: https://docs.omendb.com
- Issues: https://github.com/omendb/core/issues
- Discord: https://discord.gg/omendb
- Email: support@omendb.com

## Citation

If you use OmenDB in your research, please cite:

```bibtex
@software{omendb2025,
  title = {OmenDB: Learned Database with ML-Powered Indexing},
  year = {2025},
  url = {https://github.com/omendb/core}
}
```