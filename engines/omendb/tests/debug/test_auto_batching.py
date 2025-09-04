#!/usr/bin/env python3
"""Test auto-batching API performance"""

import numpy as np
import time
import sys

sys.path.insert(0, "python")

from omendb.api import DB as OriginalDB
from omendb.api_batched import AutoBatchDB


def test_auto_batching():
    """Test auto-batching performance"""
    print("AUTO-BATCHING API PERFORMANCE TEST")
    print("=" * 60)

    n = 5000
    dim = 128
    vectors = np.random.randn(n, dim).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n)]

    # Test different batch sizes
    batch_sizes = [10, 50, 100, 500, 1000]

    print("\nTesting different batch sizes:")
    print(f"{'Batch Size':<12} {'Add Rate':<12} {'Speedup':<10} {'Flushes'}")
    print("-" * 50)

    # Baseline: Original API
    db_orig = OriginalDB()
    db_orig.clear()
    db_orig.configure(buffer_size=10000)

    start = time.perf_counter()
    for i in range(n):
        db_orig.add(ids[i], vectors[i])
    orig_time = time.perf_counter() - start
    orig_rate = n / orig_time

    print(f"{'Original':<12} {orig_rate:<12.0f} {'(baseline)':<10} {'-'}")

    # Test different batch sizes
    best_batch = None
    best_rate = 0

    for batch_size in batch_sizes:
        db_batch = AutoBatchDB(
            batch_size=batch_size, batch_timeout=10.0
        )  # High timeout to test batch size effect
        db_batch.clear()

        start = time.perf_counter()
        for i in range(n):
            db_batch.add(ids[i], vectors[i])
        db_batch.flush()  # Ensure all vectors are added
        batch_time = time.perf_counter() - start

        batch_rate = n / batch_time
        speedup = batch_rate / orig_rate
        stats = db_batch.stats()

        print(
            f"{batch_size:<12} {batch_rate:<12.0f} {speedup:<10.1f}x {stats['total_flushes']}"
        )

        if batch_rate > best_rate:
            best_rate = batch_rate
            best_batch = batch_size

    print("-" * 50)
    print(f"\nBest batch size: {best_batch} ({best_rate:.0f} vec/s)")

    # Test timeout-based flushing
    print("\n" + "=" * 60)
    print("TESTING TIMEOUT-BASED FLUSHING")
    print("=" * 60)

    db_timeout = AutoBatchDB(batch_size=1000, batch_timeout=0.01)  # 10ms timeout
    db_timeout.clear()

    # Add vectors with delays to trigger timeout flushes
    start = time.perf_counter()
    for i in range(100):
        db_timeout.add(f"vec_{i}", vectors[i])
        if i % 10 == 0:
            time.sleep(0.02)  # 20ms delay to trigger timeout

    db_timeout.flush()
    timeout_time = time.perf_counter() - start

    stats = db_timeout.stats()
    print(f"Added 100 vectors with delays")
    print(f"Total flushes: {stats['total_flushes']}")
    print(f"Average batch size: {stats['average_batch_size']:.1f}")
    print(f"Time: {timeout_time:.2f}s")

    # Compare with search performance
    print("\n" + "=" * 60)
    print("SEARCH PERFORMANCE WITH AUTO-BATCHING")
    print("=" * 60)

    # Add vectors with auto-batching
    db_batch = AutoBatchDB(batch_size=100)
    db_batch.clear()

    for i in range(1000):
        db_batch.add(ids[i], vectors[i])

    # Search (should auto-flush)
    query = vectors[0]

    start = time.perf_counter()
    results = db_batch.search(query, limit=10)
    search_time = time.perf_counter() - start

    print(f"Search time (with auto-flush): {search_time * 1000:.2f}ms")
    print(f"Found {len(results)} results")

    if results and results[0]["id"] == "vec_0":
        print("✅ Correct result found")
    else:
        print("⚠️ Search accuracy issue")

    # Final stats
    final_stats = db_batch.stats()
    print(f"\nFinal statistics:")
    print(f"  Total adds: {final_stats['total_adds']}")
    print(f"  Total flushes: {final_stats['total_flushes']}")
    print(f"  Average batch size: {final_stats['average_batch_size']:.1f}")


if __name__ == "__main__":
    test_auto_batching()
