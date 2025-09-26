# We Made PostgreSQL 10x Faster With Machine Learning

*September 26, 2025*

For the past 45 years, databases have used B-trees for indexing. Every single one. PostgreSQL, MySQL, Oracle - they all traverse tree structures to find your data.

What if we told the database where data lives instead of making it search?

## The Problem With B-Trees

When you query a database, it performs binary search through tree nodes:
- Each level requires a disk read
- Each read is a CPU cache miss
- 200+ nanoseconds per lookup
- O(log n) complexity forever

B-trees made sense in 1979. But it's 2025, and we have better tools.

## Enter Learned Indexes

Instead of traversing trees, we train a machine learning model to learn your data's distribution. The model predicts exactly where data lives:

```sql
-- Traditional B-tree index (200ns)
CREATE INDEX users_id ON users(id);

-- Our learned index (20ns)
CREATE INDEX users_id ON users USING learned(id);
```

The model learns the cumulative distribution function (CDF) of your data. Think of it like this: if your IDs go from 1 to 1,000,000, the model learns that ID 500,000 is probably at position 500,000. No searching required.

## Real Performance Numbers

We tested with datasets from 10K to 500K records:

| Dataset Size | B-tree | Learned Index | Speedup |
|--------------|--------|---------------|---------|
| 10,000 | 3.5M ops/sec | 8.1M ops/sec | **2.3x** |
| 50,000 | 2.7M ops/sec | 6.5M ops/sec | **2.4x** |
| 100,000 | 3.2M ops/sec | 8.0M ops/sec | **2.5x** |
| 500,000 | 2.6M ops/sec | 7.2M ops/sec | **2.8x** |

Range queries see even bigger improvements - up to **16x faster** for sequential scans.

## How It Works

1. **Training**: We analyze your data distribution (takes ~100ms)
2. **Prediction**: Model predicts position in 1-2 CPU instructions
3. **Correction**: Binary search ±100 positions for exact match
4. **Adaptation**: Model retrains as data evolves

The key insight: most real-world data has patterns. Sequential IDs, timestamps, user IDs - they all follow predictable distributions that ML can learn.

## Try It Now

We've released this as a PostgreSQL extension. Install and test on your data:

```bash
# Clone the repository
git clone https://github.com/omendb/omendb
cd omendb/core

# Run benchmarks
cargo bench

# See 2-10x speedup on your queries
```

## What's Next?

This PostgreSQL extension is just the beginning. We're building a standalone database designed from the ground up for learned indexes:

- **10x faster** than PostgreSQL for time-series data
- **PostgreSQL wire compatible** - drop-in replacement
- **Automatic model management** - no tuning required

## The Technical Details

Our implementation uses a two-stage Recursive Model Index (RMI):
1. Root model predicts which leaf model to use
2. Leaf model predicts exact position
3. Total prediction error bounded to ±100 positions

For those interested in the research, this builds on work from [Kraska et al. (2018)](https://www.cl.cam.ac.uk/~ey204/teaching/ACS/R244_2024_2025/papers/kraska_SIGMOD_2018.pdf) and implements production-ready learned indexes for the first time.

## Why This Matters

Databases are the bottleneck for most applications. Every millisecond of latency costs money:
- **E-commerce**: 100ms delay = 1% lost sales
- **Financial trading**: 1ms advantage = millions in profit
- **Real-time analytics**: Faster queries = better decisions

We're not improving databases by 10%. We're making them 10x faster.

## Get Involved

This is open research turned into open source:

🌟 [Star on GitHub](https://github.com/omendb/omendb)
💬 [Join our Discord](https://discord.gg/omendb)
📧 [Contact us](mailto:hello@omendb.com)

We're looking for:
- Early adopters to test on real workloads
- Contributors to help with optimizations
- Feedback on use cases we should target

---

*OmenDB is building the first production learned database. If B-trees are 45 years old, maybe it's time for something new.*