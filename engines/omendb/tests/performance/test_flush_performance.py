#!/usr/bin/env python
"""Test flush performance to understand the bottleneck."""

import time
import numpy as np
import sys

sys.path.insert(0, "python")
import omendb


def test_flush_sizes():
    """Test different flush sizes to find the performance cliff."""
    print("=== Testing Flush Performance at Different Sizes ===\n")

    dimension = 128
    flush_sizes = [100, 500, 1000, 2500, 5000, 10000, 15000, 20000, 25000]

    for size in flush_sizes:
        # Create fresh DB for each test
        db = omendb.DB()

        # Set buffer size to exactly the test size to control when flush happens
        db.configure(buffer_size=size)

        # Create vectors that will fill buffer and trigger flush
        vectors = np.random.rand(size + 1, dimension).astype(
            np.float32
        )  # +1 to trigger flush
        ids = [f"vec_{i}" for i in range(size + 1)]

        print(f"Testing flush of {size} vectors:")

        # Time the operation
        start = time.time()
        db.add_batch(vectors, ids)
        elapsed = time.time() - start

        throughput = (size + 1) / elapsed if elapsed > 0 else 0

        # Extract timing
        print(f"  Total time: {elapsed:.3f}s")
        print(f"  Throughput: {throughput:.0f} vec/s")
        print(f"  Per-vector: {elapsed * 1000 / (size + 1):.3f}ms")
        print()


def test_incremental_vs_batch():
    """Compare incremental HNSW building vs batch."""
    print("\n=== Incremental vs Batch HNSW Building ===\n")

    dimension = 128
    test_size = 5000

    # Test 1: Build HNSW incrementally (many small flushes)
    print("Test 1: Incremental building (buffer_size=100)")
    db1 = omendb.DB()
    db1.configure(buffer_size=100)  # Small buffer, many flushes

    vectors = np.random.rand(test_size, dimension).astype(np.float32)
    ids = [f"inc_{i}" for i in range(test_size)]

    start = time.time()
    # Add in small batches to simulate incremental
    for i in range(0, test_size, 100):
        batch = vectors[i : i + 100]
        batch_ids = ids[i : i + 100]
        db1.add_batch(batch, batch_ids)
    incremental_time = time.time() - start

    print(f"  Time: {incremental_time:.3f}s")
    print(f"  Throughput: {test_size / incremental_time:.0f} vec/s")

    # Test 2: Build HNSW in one large batch
    print("\nTest 2: Single large batch (buffer_size=10000)")
    db2 = omendb.DB()
    db2.configure(buffer_size=10000)  # Large buffer

    start = time.time()
    db2.add_batch(vectors, ids)
    batch_time = time.time() - start

    print(f"  Time: {batch_time:.3f}s")
    print(f"  Throughput: {test_size / batch_time:.0f} vec/s")

    print(f"\nIncremental is {incremental_time / batch_time:.1f}x slower than batch")


def test_no_flush_performance():
    """Test performance when everything stays in buffer."""
    print("\n=== Performance Without Flush (Buffer Only) ===\n")

    dimension = 128
    sizes = [1000, 5000, 10000, 20000]

    for size in sizes:
        db = omendb.DB()
        # Set buffer larger than test size to avoid flush
        db.configure(buffer_size=size * 2)

        vectors = np.random.rand(size, dimension).astype(np.float32)
        ids = [f"nof_{i}" for i in range(size)]

        start = time.time()
        db.add_batch(vectors, ids)
        elapsed = time.time() - start

        throughput = size / elapsed if elapsed > 0 else 0
        print(f"{size:6d} vectors (no flush): {elapsed:.3f}s → {throughput:8.0f} vec/s")


if __name__ == "__main__":
    print("=" * 60)
    print("FLUSH PERFORMANCE ANALYSIS")
    print("=" * 60)

    test_flush_sizes()
    test_incremental_vs_batch()
    test_no_flush_performance()

    print("\n" + "=" * 60)
    print("ANALYSIS COMPLETE")
    print("=" * 60)
