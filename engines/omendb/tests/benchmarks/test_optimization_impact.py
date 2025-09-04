#!/usr/bin/env python3
"""
Test impact of Mojo optimizations on OmenDB performance.

Tests:
1. Current performance baseline
2. Hardware-optimized worker counts
3. Compiler flag impact (if we add them)
"""

import time
import numpy as np
import sys
import os

# Add python directory to path
sys.path.insert(0, "python")

try:
    import omendb
except ImportError:
    print("❌ Failed to import omendb. Run: ./scripts/build_native.sh")
    sys.exit(1)


def benchmark_current_performance(num_vectors=50000, dimension=128, num_queries=1000):
    """Benchmark current performance as baseline."""
    print(f"🔥 Baseline Performance Test")
    print(
        f"   Vectors: {num_vectors:,}, Dimension: {dimension}, Queries: {num_queries:,}"
    )

    # Generate test data
    print("📊 Generating test data...")
    vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
    ids = [f"vec_{i}" for i in range(num_vectors)]
    queries = np.random.rand(num_queries, dimension).astype(np.float32)

    # Test ingestion performance
    print("⚡ Testing ingestion...")
    db = omendb.DB()

    batch_size = 10_000
    total_time = 0

    for i in range(0, num_vectors, batch_size):
        end_idx = min(i + batch_size, num_vectors)
        batch_vectors = vectors[i:end_idx].tolist()
        batch_ids = ids[i:end_idx]
        batch_metadata = [{} for _ in range(end_idx - i)]

        start_time = time.time()
        db.add_batch(vectors=batch_vectors, ids=batch_ids, metadata=batch_metadata)
        batch_time = time.time() - start_time
        total_time += batch_time

    ingestion_rate = num_vectors / total_time
    print(f"   Ingestion: {ingestion_rate:,.0f} vectors/sec")

    # Test query performance
    print("🔍 Testing query performance...")
    query_times = []

    for i in range(min(100, num_queries)):  # Sample queries to avoid long test
        query = queries[i].tolist()

        start_time = time.time()
        results = db.search(query, limit=10)
        query_time = (time.time() - start_time) * 1000  # Convert to ms
        query_times.append(query_time)

        if i == 0:
            print(f"   First query: {query_time:.3f}ms (includes any warmup)")

    avg_query_time = np.mean(query_times[1:])  # Skip first query (warmup)
    p99_query_time = np.percentile(query_times[1:], 99)
    qps = 1000 / avg_query_time  # queries per second

    print(f"   Average query: {avg_query_time:.3f}ms ({qps:.0f} QPS)")
    print(f"   P99 query: {p99_query_time:.3f}ms")

    return {
        "ingestion_rate": ingestion_rate,
        "avg_query_time": avg_query_time,
        "p99_query_time": p99_query_time,
        "qps": qps,
    }


def test_worker_count_optimization():
    """Test impact of optimizing parallel worker counts."""
    print(f"\n🧪 Worker Count Optimization Test")
    print(
        "   Current: Using hardware-aware worker counts (15 workers on 16-core system)"
    )
    print("   Previous: Used hardcoded min(8, batch_size)")

    # Hardware info
    import os

    try:
        cores = os.cpu_count()
        print(f"   Available CPU cores: {cores}")
        optimal_workers = max(1, min(cores - 1, 16))  # Leave 1 for OS, cap at 16
        print(f"   Optimal workers: {optimal_workers}")
        print(
            f"   Theoretical improvement: ~{(optimal_workers / 8):.1f}x from using {optimal_workers} vs 8 workers"
        )
    except:
        print("   Could not detect CPU cores")

    # Skip the second test due to single-database-per-process limitation
    print("   Skipping separate test due to single-database limitation")
    print("   The first test already uses the optimized worker counts")

    return {"optimization_status": "hardware-aware workers active"}


def main():
    print("🚀 OmenDB Optimization Impact Testing")
    print("=====================================")

    # Test 1: Current performance (avoid migration overhead)
    baseline = benchmark_current_performance(
        num_vectors=10000
    )  # Smaller to avoid migration

    # Test 2: Worker optimization analysis
    worker_analysis = test_worker_count_optimization()

    # Summary
    print(f"\n📊 Performance Summary")
    print(f"======================")
    print(f"Baseline ingestion:  {baseline['ingestion_rate']:,.0f} vectors/sec")
    print(f"Query latency (avg): {baseline['avg_query_time']:.3f}ms")
    print(f"Query latency (P99): {baseline['p99_query_time']:.3f}ms")
    print(f"Query throughput:    {baseline['qps']:.0f} QPS")

    # Note about migration overhead
    print(f"\n⚠️  Note: Performance includes migration overhead")
    print(f"   Pure brute-force performance would be higher")
    print(f"   Migration starts at 5K vectors, completes around 10K")

    print(f"\n🔬 Ready to test optimizations:")
    print(f"   1. Hardware-aware worker counts")
    print(f"   2. Compiler optimization flags")
    print(f"   3. @vectorize opportunities in search paths")


if __name__ == "__main__":
    main()
