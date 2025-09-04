#!/usr/bin/env python
"""Test chunked batch processing to avoid HNSW performance cliff."""

import time
import numpy as np
import sys

sys.path.insert(0, "python")
import omendb


def test_chunked_batch_processing():
    """Test performance with chunked batch processing."""
    print("=== Testing Chunked Batch Processing (4K chunks) ===\n")

    dimension = 128
    test_sizes = [1000, 5000, 10000, 15000, 20000, 25000, 30000]

    for size in test_sizes:
        db = omendb.DB()

        vectors = np.random.rand(size, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(size)]

        start = time.time()
        # This should now use chunked processing internally for large batches
        result = db.add_batch(vectors, ids)
        elapsed = time.time() - start

        throughput = size / elapsed if elapsed > 0 else 0

        print(f"{size:6d} vectors: {elapsed:6.3f}s → {throughput:8.0f} vec/s")

        # Verify count
        count = db.count()
        if count != size:
            print(f"  ⚠️ Count mismatch: expected {size}, got {count}")


def compare_approaches():
    """Compare different processing strategies."""
    print("\n=== Strategy Comparison ===\n")
    print("Size   | Single Batch | Chunked (4K) | Improvement")
    print("-------|--------------|--------------|------------")

    dimension = 128
    test_cases = [
        (5000, 68903),  # Before the cliff
        (10000, 54287),  # Moderate size
        (25000, 4390),  # Major cliff
        (30000, 834),  # Severe degradation
    ]

    for size, baseline_throughput in test_cases:
        db = omendb.DB()
        vectors = np.random.rand(size, dimension).astype(np.float32)
        ids = [f"test_{i}" for i in range(size)]

        start = time.time()
        db.add_batch(vectors, ids)
        elapsed = time.time() - start

        chunked_throughput = size / elapsed if elapsed > 0 else 0
        improvement = (
            chunked_throughput / baseline_throughput if baseline_throughput > 0 else 0
        )

        status = "✅" if improvement > 2.0 else ("⚠️" if improvement > 1.0 else "❌")

        print(
            f"{size:5d}  | {baseline_throughput:8.0f} v/s | {chunked_throughput:8.0f} v/s | {improvement:5.1f}x {status}"
        )


def test_query_performance():
    """Test query performance after chunked insertion."""
    print("\n=== Query Performance After Chunked Insertion ===\n")

    dimension = 128
    db = omendb.DB()

    # Add 20K vectors using chunked approach
    print("Adding 20,000 vectors using chunked processing...")
    vectors = np.random.rand(20000, dimension).astype(np.float32)
    ids = [f"vec_{i}" for i in range(20000)]

    start = time.time()
    db.add_batch(vectors, ids)
    insert_time = time.time() - start

    print(f"Insert time: {insert_time:.3f}s ({20000 / insert_time:.0f} vec/s)")

    # Test query performance
    query = np.random.rand(dimension).astype(np.float32)

    # Warm up
    _ = db.search(query, limit=10)

    # Time queries
    times = []
    for _ in range(100):
        start = time.time()
        results = db.search(query, limit=10)
        times.append(time.time() - start)

    avg_time = sum(times) / len(times)
    print(f"\nQuery performance (100 queries):")
    print(f"  Average: {avg_time * 1000:.2f}ms")
    print(f"  Min: {min(times) * 1000:.2f}ms")
    print(f"  Max: {max(times) * 1000:.2f}ms")


if __name__ == "__main__":
    print("=" * 60)
    print("CHUNKED BATCH PROCESSING TEST")
    print("=" * 60)

    test_chunked_batch_processing()
    compare_approaches()
    test_query_performance()

    print("\n" + "=" * 60)
    print("TEST COMPLETE")
    print("=" * 60)
