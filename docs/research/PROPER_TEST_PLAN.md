# Proper Test Plan: Validating Learned Indexes

## Why Our Previous Tests Failed

Based on research into successful implementations, we were testing under completely wrong conditions:

| Condition | Papers (Success) | Our Tests (Failure) |
|-----------|------------------|---------------------|
| **Scale** | 10M-800M keys | 10K-1M keys |
| **Values** | 1KB+ per record | Tiny strings |
| **Workload** | Zipfian (hot keys) | Uniform random |
| **Storage** | Disk-based | Pure memory |
| **Baseline** | RocksDB/LSM | Binary search |
| **Architecture** | Hybrid/hints | Pure learned |

## Test Plan: Replicate LearnedKV Results

### Phase 1: Large-Scale Disk-Based Test

```python
#!/usr/bin/env python3
"""
Test learned indexes under proper conditions:
- 50M keys (matches papers)
- 1KB values (matches LearnedKV)
- Zipfian workload (80% queries hit 20% of keys)
- Compare to RocksDB (not binary search)
- Two-tier architecture (hot + cold)
"""

import numpy as np
import rocksdb
import time
from typing import List, Tuple
import random

class ProperLearnedTest:
    def __init__(self):
        # Test parameters that match successful papers
        self.num_keys = 50_000_000       # 50M keys
        self.value_size = 1024           # 1KB values
        self.hot_ratio = 0.2             # 20% hot keys
        self.query_hot_ratio = 0.8       # 80% queries to hot keys
        self.num_queries = 1_000_000     # 1M queries

    def generate_zipfian_data(self):
        """Generate Zipfian distributed data (hot keys)"""
        # Hot keys (accessed frequently)
        hot_count = int(self.num_keys * self.hot_ratio)
        hot_keys = list(range(hot_count))

        # Cold keys (accessed rarely)
        cold_keys = list(range(hot_count, self.num_keys))

        # Generate large values (1KB each)
        def make_value(key_id):
            base = f"value_{key_id}_"
            padding = "x" * (self.value_size - len(base.encode()))
            return (base + padding).encode()

        # Create dataset
        all_keys = hot_keys + cold_keys
        data = [(k, make_value(k)) for k in all_keys]

        # Generate Zipfian queries (80% to 20% of keys)
        hot_query_count = int(self.num_queries * self.query_hot_ratio)
        cold_query_count = self.num_queries - hot_query_count

        queries = (
            random.choices(hot_keys, k=hot_query_count) +
            random.choices(cold_keys, k=cold_query_count)
        )
        random.shuffle(queries)

        return data, queries, hot_keys
```

### Phase 2: Two-Tier Architecture (LearnedKV Style)

```python
class TwoTierLearnedDB:
    """
    Replicate LearnedKV architecture:
    - Hot tier: In-memory with learned index
    - Cold tier: RocksDB for bulk data
    - Non-blocking conversion during queries
    """

    def __init__(self, hot_capacity=10_000_000):
        # Hot tier (learned index)
        self.hot_data = {}
        self.hot_capacity = hot_capacity
        self.hot_model = FastLinearModel()

        # Cold tier (RocksDB)
        opts = rocksdb.Options()
        opts.create_if_missing = True
        opts.max_background_compactions = 4
        self.cold_db = rocksdb.DB("/tmp/learned_cold.db", opts)

        # Statistics
        self.hot_hits = 0
        self.cold_hits = 0

    def load_data(self, data, hot_keys):
        """Load data with hot/cold separation"""
        hot_set = set(hot_keys)

        # Load hot data into learned index
        hot_data = [(k, v) for k, v in data if k in hot_set]
        self.train_hot_index(hot_data)

        # Load cold data into RocksDB
        batch = rocksdb.WriteBatch()
        for k, v in data:
            if k not in hot_set:
                batch.put(str(k).encode(), v)
        self.cold_db.write(batch)

    def query(self, key):
        """Two-tier lookup with statistics"""
        # Try hot tier first
        if self.hot_model.predict_exists(key):
            result = self.hot_data.get(key)
            if result:
                self.hot_hits += 1
                return result

        # Fall back to cold tier
        result = self.cold_db.get(str(key).encode())
        if result:
            self.cold_hits += 1
        return result
```

### Phase 3: Proper Comparison Against RocksDB

```python
def benchmark_rocksdb_baseline(data, queries):
    """Proper baseline: RocksDB (what papers compare against)"""

    # Setup RocksDB with optimization
    opts = rocksdb.Options()
    opts.create_if_missing = True
    opts.max_background_compactions = 4
    opts.write_buffer_size = 64 * 1024 * 1024  # 64MB
    opts.target_file_size_base = 64 * 1024 * 1024

    db = rocksdb.DB("/tmp/rocksdb_baseline.db", opts)

    # Load data
    start = time.perf_counter()
    batch = rocksdb.WriteBatch()
    for i, (k, v) in enumerate(data):
        batch.put(str(k).encode(), v)
        if i % 100000 == 0:  # Batch writes
            db.write(batch)
            batch = rocksdb.WriteBatch()
    if batch:
        db.write(batch)
    load_time = time.perf_counter() - start

    # Force compaction
    db.compact_range()

    # Query
    start = time.perf_counter()
    found = 0
    for key in queries:
        if db.get(str(key).encode()):
            found += 1
    query_time = time.perf_counter() - start

    return {
        'name': 'RocksDB Baseline',
        'load_time': load_time,
        'query_time': query_time,
        'qps': len(queries) / query_time,
        'found': found
    }
```

## Test Script Implementation

```bash
# Create the proper test
cat > test_proper_learned.py << 'EOF'
#!/usr/bin/env python3

# [Include all the above code]

def main():
    print("=== PROPER LEARNED INDEX TEST ===")
    print("Matching LearnedKV conditions:")
    print("- 50M keys, 1KB values")
    print("- Zipfian workload (80% -> 20% hot)")
    print("- Two-tier architecture")
    print("- Compare vs RocksDB\n")

    test = ProperLearnedTest()

    print("1. Generating large-scale Zipfian data...")
    data, queries, hot_keys = test.generate_zipfian_data()
    print(f"   Created {len(data):,} records, {len(queries):,} queries")

    print("2. Testing RocksDB baseline...")
    rocksdb_result = benchmark_rocksdb_baseline(data, queries)

    print("3. Testing two-tier learned index...")
    learned_result = benchmark_two_tier_learned(data, queries, hot_keys)

    print("\n=== RESULTS ===")
    print(f"RocksDB:     {rocksdb_result['qps']:8.0f} QPS")
    print(f"Learned:     {learned_result['qps']:8.0f} QPS")
    print(f"Speedup:     {learned_result['qps']/rocksdb_result['qps']:8.2f}x")
    print(f"Hot hits:    {learned_result.get('hot_ratio', 0)*100:6.1f}%")

    if learned_result['qps'] > rocksdb_result['qps'] * 1.3:
        print("\n✅ LEARNED INDEXES WORK!")
        print("   Proceed with specialized database")
    else:
        print("\n❌ LEARNED INDEXES STILL DON'T WIN")
        print("   Pivot to unified OLTP/OLAP opportunity")

if __name__ == "__main__":
    main()
EOF
```

## Expected Results

Based on the research, if we test under proper conditions:

### Success Case (Papers are right)
- **2-4x speedup** on hot data queries
- **Hot hit ratio >80%** due to Zipfian distribution
- **I/O dominates** so model overhead becomes negligible
- **Large values** make disk seeks expensive

### Failure Case (We're still right)
- **Still slower** even under ideal conditions
- **Model overhead** still exceeds benefits
- **RocksDB** is too optimized to beat

## Decision Tree

```
IF learned_indexes_show_2x_speedup:
    → Build specialized time-series/log database
    → Target hot/cold workloads specifically
    → Use learned indexes as core differentiator

ELIF learned_indexes_show_modest_gains:
    → Use as optimization in larger system
    → Focus on unified OLTP/OLAP opportunity
    → Learned indexes for hot/cold placement

ELSE:
    → Abandon learned indexes completely
    → Pivot to unified OLTP/OLAP database
    → Focus on $22.8B ETL elimination market
```

## Timeline

- **Week 1:** Implement proper test with 50M keys
- **Week 2:** Run comprehensive benchmarks vs RocksDB
- **Week 3:** Analyze results and make pivot decision
- **Week 4:** Begin implementation of chosen direction

## Resources Needed

1. **Hardware:** Large memory (32GB+) for 50M key dataset
2. **Storage:** Fast SSD for RocksDB performance
3. **Dependencies:** `rocksdb`, `numpy`, `zipfian` distribution
4. **Time:** ~40 hours for implementation and testing

This test will definitively answer whether learned indexes work under the conditions where papers claim success.

---
*September 26, 2025*
*Proper test plan based on successful paper analysis*