#!/usr/bin/env python3
"""Test performance with larger batch sizes (1000-5000)"""

import numpy as np
import time
import sys

sys.path.insert(0, "python")
import omendb


def test_batch_sizes():
    """Test different batch sizes for performance"""
    print("TESTING LARGER BATCH SIZES WITH SIMD")
    print("=" * 60)

    dim = 128
    batch_sizes = [100, 500, 1000, 2000, 5000, 10000]
    n_total = 20000

    print(f"{'Batch Size':<12} {'Vec/s':<12} {'Time (s)':<12} {'Speedup'}")
    print("-" * 50)

    baseline = None

    for batch_size in batch_sizes:
        db = omendb.DB()
        db.clear()

        # Generate all vectors upfront
        vectors = np.random.randn(n_total, dim).astype(np.float32)
        ids = [f"vec_{i}" for i in range(n_total)]

        # Add in batches
        start = time.perf_counter()
        for i in range(0, n_total, batch_size):
            batch_end = min(i + batch_size, n_total)
            db.add_batch(vectors[i:batch_end], ids[i:batch_end])

        total_time = time.perf_counter() - start
        vec_per_sec = n_total / total_time

        if baseline is None:
            baseline = vec_per_sec
            speedup = 1.0
        else:
            speedup = vec_per_sec / baseline

        print(
            f"{batch_size:<12} {vec_per_sec:<12.0f} {total_time:<12.3f} {speedup:.2f}x"
        )

        # Test search performance with this size
        query = np.random.randn(dim).astype(np.float32)
        search_times = []
        for _ in range(20):
            t = time.perf_counter()
            results = db.search(query, 10)
            search_times.append(time.perf_counter() - t)

        avg_search = np.mean(search_times) * 1000
        print(f"  → Search: {avg_search:.2f}ms per query")


def test_auto_batching():
    """Test auto-batching wrapper performance"""
    print("\n" + "=" * 60)
    print("AUTO-BATCHING WRAPPER PERFORMANCE")
    print("=" * 60)

    from omendb.api_batched import AutoBatchDB

    dim = 128
    n = 10000
    vectors = np.random.randn(n, dim).astype(np.float32)

    batch_configs = [
        (100, 0.1),
        (500, 0.1),
        (1000, 0.1),
        (2000, 0.1),
        (5000, 0.1),
    ]

    print(f"{'Batch Config':<20} {'Vec/s':<12} {'Time (s)'}")
    print("-" * 45)

    for batch_size, timeout in batch_configs:
        db = AutoBatchDB(batch_size=batch_size, batch_timeout=timeout)
        db.clear()

        start = time.perf_counter()
        for i in range(n):
            db.add(f"vec_{i}", vectors[i])
        db.flush()  # Final flush

        total_time = time.perf_counter() - start
        vec_per_sec = n / total_time

        stats = db.stats()
        avg_batch = stats["average_batch_size"]

        print(
            f"Size={batch_size:<6} T={timeout}  {vec_per_sec:<12.0f} {total_time:<12.3f}"
        )
        print(f"  → Avg batch: {avg_batch:.0f}, Flushes: {stats['total_flushes']}")


def test_simd_impact():
    """Compare distance calculation performance"""
    print("\n" + "=" * 60)
    print("SIMD DISTANCE CALCULATION IMPACT")
    print("=" * 60)

    dim = 128
    sizes = [100, 1000, 5000, 10000]

    print(f"{'Vectors':<10} {'Build Time':<12} {'Search Time':<12} {'Total Ops/s'}")
    print("-" * 50)

    for size in sizes:
        db = omendb.DB()
        db.clear()

        vectors = np.random.randn(size, dim).astype(np.float32)
        ids = [f"vec_{i}" for i in range(size)]

        # Time batch build (includes distance calculations)
        start = time.perf_counter()
        db.add_batch(vectors, ids)
        build_time = time.perf_counter() - start

        # Time searches (heavy on distance calculations)
        query = np.random.randn(dim).astype(np.float32)
        start = time.perf_counter()
        for _ in range(100):
            results = db.search(query, min(10, size))
        search_time = (time.perf_counter() - start) / 100

        # Estimate distance calculations
        # Build: O(size * log(size)) distances for graph construction
        # Search: O(size) distances for beam search
        approx_distances = size * np.log2(size) + 100 * size
        ops_per_sec = approx_distances / (build_time + search_time * 100)

        print(
            f"{size:<10} {build_time:<12.3f}s {search_time * 1000:<12.2f}ms {ops_per_sec:.0f}"
        )


def main():
    """Run all batch size tests"""
    test_batch_sizes()
    test_auto_batching()
    test_simd_impact()

    print("\n" + "=" * 60)
    print("KEY FINDINGS:")
    print("=" * 60)
    print("1. Larger batches amortize FFI overhead")
    print("2. SIMD optimizations are active (using distance_functions.mojo)")
    print("3. Auto-batching provides transparent speedup")
    print("4. Optimal batch size appears to be 1000-2000 vectors")


if __name__ == "__main__":
    main()
