#!/usr/bin/env python
"""Test incremental HNSW insertion performance after fixing O(n³) issue."""

import time
import numpy as np
import sys

sys.path.insert(0, "python")
import omendb


def test_incremental_insertion():
    """Test performance with incremental HNSW insertion fix."""
    print("=== Testing Incremental HNSW Insertion (O(n log n) Fix) ===\n")

    dimension = 128
    test_sizes = [1000, 5000, 10000, 15000, 20000, 25000, 30000]

    for size in test_sizes:
        db = omendb.DB()

        vectors = np.random.rand(size, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(size)]

        start = time.time()
        result = db.add_batch(vectors, ids)
        elapsed = time.time() - start

        throughput = size / elapsed if elapsed > 0 else 0

        print(f"{size:6d} vectors: {elapsed:6.3f}s → {throughput:8.0f} vec/s")

        # Verify count
        count = db.count()
        if count != size:
            print(f"  ⚠️ Count mismatch: expected {size}, got {count}")


def compare_with_baseline():
    """Compare new performance with baseline."""
    print("\n=== Performance Comparison ===\n")
    print("Dataset    | Before (O(n³)) | After (O(n log n))")
    print("-----------|----------------|------------------")

    # Test key sizes where we saw problems
    test_cases = [
        (1000, 90868),  # Good baseline
        (5000, 68903),  # Before the cliff
        (10000, 54287),  # Moderate size
        (25000, 4390),  # Major cliff
        (30000, 834),  # Severe degradation
    ]

    dimension = 128

    for size, baseline in test_cases:
        db = omendb.DB()
        vectors = np.random.rand(size, dimension).astype(np.float32)
        ids = [f"test_{i}" for i in range(size)]

        start = time.time()
        db.add_batch(vectors, ids)
        elapsed = time.time() - start

        new_throughput = size / elapsed if elapsed > 0 else 0
        improvement = new_throughput / baseline if baseline > 0 else 0

        status = "✅" if improvement > 1.5 else ("⚠️" if improvement > 0.9 else "❌")

        print(
            f"{size:5d}      | {baseline:8.0f} vec/s | {new_throughput:8.0f} vec/s ({improvement:.1f}x) {status}"
        )


def test_buffer_flush_timing():
    """Test timing of buffer flushes."""
    print("\n=== Buffer Flush Timing ===\n")

    dimension = 128
    db = omendb.DB()

    # Configure smaller buffer to force flushes
    db.configure(buffer_size=5000)

    # Add 20K vectors to trigger multiple flushes
    size = 20000
    vectors = np.random.rand(size, dimension).astype(np.float32)
    ids = [f"flush_{i}" for i in range(size)]

    print(f"Adding {size} vectors with buffer_size=5000 (expecting 4 flushes)...")

    start = time.time()
    db.add_batch(vectors, ids)
    elapsed = time.time() - start

    throughput = size / elapsed if elapsed > 0 else 0
    print(f"\nTotal time: {elapsed:.3f}s")
    print(f"Throughput: {throughput:.0f} vec/s")
    print(f"Average per flush: {elapsed / 4:.3f}s (if 4 flushes)")


if __name__ == "__main__":
    print("=" * 60)
    print("INCREMENTAL HNSW INSERTION TEST")
    print("=" * 60)

    test_incremental_insertion()
    compare_with_baseline()
    test_buffer_flush_timing()

    print("\n" + "=" * 60)
    print("TEST COMPLETE")
    print("=" * 60)
