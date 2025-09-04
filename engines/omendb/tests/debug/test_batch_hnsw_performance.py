#!/usr/bin/env python
"""Test HNSW batch insertion performance improvement."""

import time
import numpy as np
import sys

sys.path.insert(0, "python")
import omendb


def test_batch_performance():
    """Test performance with batch HNSW insertion."""
    print("\n=== Testing HNSW Batch Insertion Performance ===\n")

    # Test different batch sizes
    test_sizes = [1000, 5000, 10000, 25000, 50000]
    dimension = 128

    for size in test_sizes:
        print(f"\nTest with {size} vectors @{dimension}D:")
        print("-" * 40)

        # Create database with buffer size to trigger HNSW flush
        db = omendb.DB(buffer_size=1000)  # Small buffer to force HNSW usage

        # Generate test data
        vectors = np.random.rand(size, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(size)]

        # Time the batch operation
        start = time.time()
        db.add_batch(vectors, ids)  # Correct order: vectors first, then ids
        elapsed = time.time() - start

        # Calculate throughput
        throughput = size / elapsed
        print(f"Time: {elapsed:.2f}s")
        print(f"Throughput: {throughput:,.0f} vec/s")

        # Get stats to see buffer vs main index distribution
        print(f"Total vectors in DB: {db.count()}")

        # Test query performance
        query = np.random.rand(dimension).astype(np.float32)
        start = time.time()
        results = db.search(query, limit=10)
        query_time = (time.time() - start) * 1000
        print(f"Query latency: {query_time:.2f}ms")

        # Cleanup
        db.clear()


if __name__ == "__main__":
    test_batch_performance()
