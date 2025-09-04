#!/usr/bin/env python
"""Test the optimization results."""

import time
import numpy as np
import sys

sys.path.insert(0, "python")
import omendb


def test_large_batch_performance():
    """Test large batch performance with optimization."""
    print("=== Testing Large Batch Performance (After Optimization) ===\n")

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


def compare_before_after():
    """Compare performance before and after optimization."""
    print("\n\n=== Performance Comparison (Before vs After) ===\n")

    print("Dataset    | Before Optimization | After Optimization")
    print("-----------|--------------------|-----------------")
    print("1K vectors | 95,000 vec/s       | (testing...)")
    print("10K vectors| 54,000 vec/s       | (testing...)")
    print("25K vectors| 3,400 vec/s        | (testing...)")

    dimension = 128
    test_cases = [(1000, 95000), (10000, 54000), (25000, 3400)]

    for size, before_throughput in test_cases:
        db = omendb.DB()
        vectors = np.random.rand(size, dimension).astype(np.float32)
        ids = [f"test_{i}" for i in range(size)]

        start = time.time()
        db.add_batch(vectors, ids)
        elapsed = time.time() - start

        after_throughput = size / elapsed if elapsed > 0 else 0
        improvement = (
            after_throughput / before_throughput if before_throughput > 0 else 0
        )

        print(
            f"{size:5d}      | {before_throughput:8.0f} vec/s     | {after_throughput:8.0f} vec/s ({improvement:.1f}x)"
        )


if __name__ == "__main__":
    print("=" * 60)
    print("OPTIMIZATION RESULTS TEST")
    print("=" * 60)

    test_large_batch_performance()
    compare_before_after()

    print("\n" + "=" * 60)
    print("TEST COMPLETE")
    print("=" * 60)
