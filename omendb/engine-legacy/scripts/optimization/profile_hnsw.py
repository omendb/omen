#!/usr/bin/env python
"""Profile HNSW performance to identify bottlenecks."""

import time
import numpy as np
import sys
import cProfile
import pstats
from io import StringIO

sys.path.insert(0, "python")
import omendb


def profile_hnsw_flush():
    """Profile what happens during HNSW flush."""
    print("=== Profiling HNSW Flush Operation ===\n")

    # Create database with small buffer to force flush
    db = omendb.DB()
    db.configure(buffer_size=1000)  # Small buffer

    dimension = 128

    # Add vectors that will trigger flush
    print("Adding 2000 vectors (will trigger flush at 1000)...")
    vectors = np.random.rand(2000, dimension).astype(np.float32)
    ids = [f"vec_{i}" for i in range(2000)]

    start = time.time()
    db.add_batch(vectors, ids)
    elapsed = time.time() - start

    throughput = 2000 / elapsed if elapsed > 0 else 0
    print(f"Time: {elapsed:.3f}s, Throughput: {throughput:.0f} vec/s\n")

    # Check where vectors are
    print(f"Total vectors: {db.count()}")


def profile_incremental_hnsw():
    """Profile incremental HNSW building."""
    print("\n=== Profiling Incremental HNSW Building ===\n")

    sizes = [100, 500, 1000, 2000, 5000, 10000]
    dimension = 128

    for size in sizes:
        db = omendb.DB()
        db.configure(buffer_size=50)  # Very small buffer to force HNSW

        # Force immediate HNSW by adding initial batch
        init_vectors = np.random.rand(100, dimension).astype(np.float32)
        init_ids = [f"init_{i}" for i in range(100)]
        db.add_batch(init_vectors, init_ids)

        # Now add more vectors to measure HNSW insertion
        vectors = np.random.rand(size, dimension).astype(np.float32)
        ids = [f"test_{i}" for i in range(size)]

        start = time.time()
        db.add_batch(vectors, ids)
        elapsed = time.time() - start

        throughput = size / elapsed if elapsed > 0 else 0
        print(
            f"HNSW insert {size:5d} vectors: {elapsed:6.3f}s → {throughput:8.0f} vec/s"
        )


def profile_buffer_vs_hnsw():
    """Compare buffer performance vs HNSW performance."""
    print("\n=== Buffer vs HNSW Performance ===\n")

    dimension = 128
    test_size = 5000

    # Test 1: All in buffer (no HNSW)
    print("Test 1: Pure buffer (no HNSW flush)")
    db1 = omendb.DB()
    db1.configure(buffer_size=10000)  # Large buffer, no flush

    vectors = np.random.rand(test_size, dimension).astype(np.float32)
    ids = [f"buf_{i}" for i in range(test_size)]

    start = time.time()
    db1.add_batch(vectors, ids)
    buffer_time = time.time() - start
    buffer_throughput = test_size / buffer_time
    print(f"  Time: {buffer_time:.3f}s, Throughput: {buffer_throughput:.0f} vec/s")

    # Test 2: Force HNSW
    print("\nTest 2: Force HNSW flush")
    db2 = omendb.DB()
    db2.configure(buffer_size=1000)  # Small buffer, will flush

    start = time.time()
    db2.add_batch(vectors, ids)
    hnsw_time = time.time() - start
    hnsw_throughput = test_size / hnsw_time
    print(f"  Time: {hnsw_time:.3f}s, Throughput: {hnsw_throughput:.0f} vec/s")

    print(
        f"\nSlowdown factor: {buffer_time / hnsw_time:.1%} (buffer is {hnsw_time / buffer_time:.1f}x faster)"
    )


def profile_with_cprofile():
    """Use cProfile to get detailed performance data."""
    print("\n=== Detailed Profiling with cProfile ===\n")

    db = omendb.DB()
    db.configure(buffer_size=500)

    vectors = np.random.rand(1000, 128).astype(np.float32)
    ids = [f"prof_{i}" for i in range(1000)]

    # Profile the batch operation
    profiler = cProfile.Profile()
    profiler.enable()

    db.add_batch(vectors, ids)

    profiler.disable()

    # Print stats
    s = StringIO()
    ps = pstats.Stats(profiler, stream=s).sort_stats("cumulative")
    ps.print_stats(20)  # Top 20 functions

    print("Top time-consuming operations:")
    print(s.getvalue())


if __name__ == "__main__":
    print("=" * 60)
    print("HNSW PERFORMANCE PROFILING")
    print("=" * 60)

    profile_hnsw_flush()
    profile_incremental_hnsw()
    profile_buffer_vs_hnsw()
    # profile_with_cprofile()  # Commented out as it's very verbose

    print("\n" + "=" * 60)
    print("PROFILING COMPLETE")
    print("=" * 60)
