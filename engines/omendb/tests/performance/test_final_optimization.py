#!/usr/bin/env python
"""Test final optimization with progressive HNSW building."""

import time
import numpy as np
import sys

sys.path.insert(0, "python")
import omendb


def test_final_performance():
    """Test performance with final optimizations."""
    print("=== Testing Final Optimized Performance ===\n")
    print("Using progressive HNSW building with limited neighbor search\n")

    dimension = 128
    test_sizes = [1000, 5000, 10000, 15000, 20000, 25000, 30000]

    results = []
    for size in test_sizes:
        db = omendb.DB()

        # Don't use chunking in Python - let native handle it
        db.configure(buffer_size=5000)  # Set reasonable buffer

        vectors = np.random.rand(size, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(size)]

        start = time.time()
        # Remove chunking from Python - process as single batch
        returned_ids = db.add_batch(vectors, ids)
        elapsed = time.time() - start

        throughput = size / elapsed if elapsed > 0 else 0

        print(f"{size:6d} vectors: {elapsed:6.3f}s → {throughput:8.0f} vec/s")
        results.append((size, throughput))

        # Verify count
        count = db.count()
        if count != size:
            print(f"  ⚠️ Count mismatch: expected {size}, got {count}")

    return results


def compare_all_approaches():
    """Compare all approaches we've tried."""
    print("\n=== Performance Comparison of All Approaches ===\n")
    print("Dataset | Original | Incremental | Chunked | Final")
    print("--------|----------|-------------|---------|-------")

    # Baseline numbers from testing
    baselines = {
        1000: (90868, 97012, 90652),
        5000: (68903, 70007, 2258),
        10000: (54287, 53939, 2507),
        25000: (4390, 394, 478),
        30000: (834, None, None),  # Timed out
    }

    # Test final approach
    dimension = 128
    for size in [1000, 5000, 10000, 25000]:
        baseline = baselines.get(size, (0, 0, 0))
        original = baseline[0]
        incremental = baseline[1] if baseline[1] else "N/A"
        chunked = baseline[2] if baseline[2] else "N/A"

        # Test final
        db = omendb.DB()
        db.configure(buffer_size=5000)

        vectors = np.random.rand(size, dimension).astype(np.float32)
        ids = [f"test_{i}" for i in range(size)]

        start = time.time()
        db.add_batch(vectors, ids)
        elapsed = time.time() - start

        final = size / elapsed if elapsed > 0 else 0

        print(
            f"{size:5d}   | {original:8.0f} | {incremental:>8} | {chunked:>7} | {final:7.0f}"
        )


def test_scalability():
    """Test how performance scales with dataset size."""
    print("\n=== Scalability Analysis ===\n")

    results = test_final_performance()

    print("\nScaling efficiency (throughput ratio vs 1K baseline):")
    baseline = results[0][1]  # 1K throughput

    for size, throughput in results:
        efficiency = throughput / baseline
        print(f"  {size:6d} vectors: {efficiency:.2%} of baseline")


if __name__ == "__main__":
    print("=" * 60)
    print("FINAL OPTIMIZATION TEST")
    print("=" * 60)
    print()

    test_scalability()
    print()
    compare_all_approaches()

    print("\n" + "=" * 60)
    print("OPTIMIZATION COMPLETE")
    print("=" * 60)
