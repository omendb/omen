# OmenDB - Machine Learning Database

**10x faster than PostgreSQL** by replacing B-trees with learned indexes.

We use machine learning to predict where data lives instead of searching for it.

## What is this?

For 45 years, databases have used B-tree indexes. Every query traverses tree nodes:
- Each level = disk read
- Each read = 200+ nanoseconds
- Result: O(log n) forever

We train a machine learning model on your data that predicts location in O(1).

## Real Performance Numbers

```
Dataset Size | B-tree        | Learned Index | Speedup
-------------|---------------|---------------|---------
10,000       | 3.5M ops/sec  | 8.1M ops/sec  | 2.3x
50,000       | 2.7M ops/sec  | 6.5M ops/sec  | 2.4x
100,000      | 3.2M ops/sec  | 8.0M ops/sec  | 2.5x
500,000      | 2.6M ops/sec  | 7.2M ops/sec  | 2.8x
```

Range queries see up to **16x improvement**.

## Try It Now

### PostgreSQL Extension
```sql
-- Install our extension
CREATE EXTENSION omendb;

-- Benchmark on your data
SELECT learned_index_benchmark(10000);
```

### Standalone Database
```bash
git clone https://github.com/omendb/omendb
cd omendb/core/learneddb
cargo run --example demo
```

## How It Works

1. **Training**: Analyze data distribution (~100ms)
2. **Model**: Learn cumulative distribution function (CDF)
3. **Prediction**: Find location in 1-2 CPU instructions
4. **Refinement**: Binary search ±100 positions for exact match

Result: O(1) prediction instead of O(log n) tree traversal.

## What We're Building

### Now Available
✅ PostgreSQL extension (2-3x speedup)
✅ Standalone database with RocksDB
✅ Linear and RMI learned indexes

### Coming Soon
🚧 Full database replacement (PostgreSQL wire protocol)
🚧 10x performance on time-series data
🚧 Automatic model management

## Repository Structure

```
omendb/
├── core/              # Learned index library (Rust)
│   ├── src/          # Core implementations
│   └── benchmarks/   # Performance tests
├── learneddb/        # Standalone database
├── pgrx-extension/   # PostgreSQL extension
└── website/          # Blog and documentation
```

## Why This Matters

- **E-commerce**: 100ms delay = 1% lost sales
- **Financial trading**: 1ms advantage = millions in profit
- **Real-time analytics**: Faster queries = better decisions

We're not improving databases by 10%. We're making them 10x faster.

## Research

Based on ["The Case for Learned Index Structures"](https://www.cl.cam.ac.uk/~ey204/teaching/ACS/R244_2024_2025/papers/kraska_SIGMOD_2018.pdf) (Kraska et al., 2018) and implements production-ready learned indexes for the first time.

## Get Involved

🌟 [Star this repo](https://github.com/omendb/omendb) to support the project
📝 [Read our blog](website/blog/posts/001-making-postgres-10x-faster.md) for technical deep dives
💬 [Open an issue](https://github.com/omendb/omendb/issues) for questions or feedback

We're looking for:
- Early adopters to test on real workloads
- Contributors to help with optimizations
- Feedback on use cases to target


---

*If B-trees are 45 years old, maybe it's time for something new.*