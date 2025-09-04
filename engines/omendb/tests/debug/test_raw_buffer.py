#!/usr/bin/env python3
"""Test raw buffer performance to identify bottlenecks"""

import omendb
import numpy as np
import time
import cProfile
import pstats
from io import StringIO


def profile_adds():
    """Profile individual adds to find bottlenecks"""
    db = omendb.DB()
    db.clear()
    db.configure(buffer_size=10000)

    n = 1000
    dim = 128
    vectors = np.random.randn(n, dim).astype(np.float32)

    # Profile the adds
    profiler = cProfile.Profile()
    profiler.enable()

    for i in range(n):
        db.add(f"vec_{i}", vectors[i])

    profiler.disable()

    # Print statistics
    s = StringIO()
    ps = pstats.Stats(profiler, stream=s).sort_stats("cumulative")
    ps.print_stats(20)

    print("TOP 20 TIME-CONSUMING FUNCTIONS:")
    print("=" * 60)
    print(s.getvalue())


def test_minimal_overhead():
    """Test with minimal vectors to measure overhead"""
    print("\nMINIMAL OVERHEAD TEST")
    print("=" * 60)

    db = omendb.DB()
    db.clear()
    db.configure(buffer_size=10000)

    # Test with tiny vectors
    for dim in [1, 4, 16, 64, 128, 256]:
        vectors = np.random.randn(1000, dim).astype(np.float32)

        db.clear()
        start = time.perf_counter()
        for i in range(1000):
            db.add(f"v{i}", vectors[i])
        elapsed = time.perf_counter() - start

        rate = 1000 / elapsed
        per_op = elapsed / 1000 * 1000  # milliseconds
        print(f"Dim {dim:3d}: {rate:6.0f} vec/s ({per_op:.3f}ms per op)")


if __name__ == "__main__":
    test_minimal_overhead()
    print()
    profile_adds()
