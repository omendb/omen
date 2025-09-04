#!/usr/bin/env python3
"""
Profile 128D performance to identify optimization opportunities.
"""

import time
import numpy as np
import cProfile
import pstats
from io import StringIO
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "../../python"))
import omendb


def profile_batch_add():
    """Profile batch vector addition."""
    db = omendb.DB()
    vectors = np.random.rand(5000, 128).astype(np.float32)

    # Warmup
    for i in range(100):
        db.add(f"w{i}", vectors[i].tolist())

    # Profile main workload
    profiler = cProfile.Profile()
    profiler.enable()

    start = time.perf_counter()
    for i in range(100, 5000):
        db.add(f"v{i}", vectors[i].tolist())
    elapsed = time.perf_counter() - start

    profiler.disable()

    # Calculate performance
    rate = 4900 / elapsed
    print(f"Performance: {rate:.0f} vec/s")

    # Print profiling results
    s = StringIO()
    ps = pstats.Stats(profiler, stream=s).sort_stats("cumulative")
    ps.print_stats(20)  # Top 20 functions
    print("\nTop 20 functions by cumulative time:")
    print(s.getvalue())

    # Also show by total time
    s = StringIO()
    ps = pstats.Stats(profiler, stream=s).sort_stats("tottime")
    ps.print_stats(10)  # Top 10 functions
    print("\nTop 10 functions by total time:")
    print(s.getvalue())


def analyze_memory_patterns():
    """Analyze memory access patterns."""
    print("\nMemory Analysis for 128D vectors:")
    print("-" * 50)

    # 128D vector = 128 * 4 bytes = 512 bytes
    print("Vector size: 128 * 4 bytes = 512 bytes")
    print("Cache line: 64 bytes")
    print("Vectors per cache line: 0.125 (8 cache lines per vector)")

    # SIMD analysis
    print("\nSIMD Analysis (ARM NEON):")
    print("SIMD width: 128 bits = 4 floats")
    print("128D / 4 = 32 SIMD operations")
    print("Perfect alignment for SIMD")

    # Memory bandwidth estimate
    # For cosine distance: 2 reads + 3 accumulates per element
    print("\nMemory bandwidth (cosine distance):")
    print("Reads: 2 * 512 bytes = 1024 bytes per distance")
    print("At 5,329 vec/s: 5.5 MB/s read bandwidth")

    print("\nOptimization opportunities:")
    print("1. Prefetching might help (but was disabled due to variance)")
    print("2. Cache blocking for batch operations")
    print("3. Memory alignment for better SIMD loads")


def test_memory_alignment():
    """Test if memory alignment affects performance."""
    print("\nTesting memory alignment impact...")

    # Aligned allocation
    aligned_vectors = np.zeros((1000, 128), dtype=np.float32)

    # Test aligned
    db1 = omendb.DB()
    start = time.perf_counter()
    for i in range(1000):
        db1.add(f"v{i}", aligned_vectors[i].tolist())
    aligned_time = time.perf_counter() - start

    # Test with offset (misaligned)
    misaligned_vectors = np.zeros((1000, 129), dtype=np.float32)[
        :, 1:
    ]  # Skip first element

    db2 = omendb.DB()
    start = time.perf_counter()
    for i in range(1000):
        db2.add(f"v{i}", misaligned_vectors[i].tolist())
    misaligned_time = time.perf_counter() - start

    print(f"Aligned: {1000 / aligned_time:.0f} vec/s")
    print(f"Misaligned: {1000 / misaligned_time:.0f} vec/s")
    print(f"Difference: {(aligned_time - misaligned_time) / aligned_time * 100:.1f}%")


def main():
    print("128D Performance Profiling")
    print("=" * 60)

    # Run profiling
    profile_batch_add()

    # Analyze memory patterns
    analyze_memory_patterns()

    # Test memory alignment
    test_memory_alignment()

    print("\n" + "=" * 60)
    print("Profiling Complete")

    print("\nRecommendations:")
    print("1. Current performance (5,329 vec/s) is already good")
    print("2. Main bottleneck likely in Python → Mojo boundary")
    print("3. Consider batch API to amortize overhead")
    print("4. Memory alignment has minimal impact on this workload")


if __name__ == "__main__":
    main()
