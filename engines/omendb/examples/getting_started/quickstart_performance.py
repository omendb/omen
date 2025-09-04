#!/usr/bin/env python3
"""
OmenDB Performance Quickstart
============================

Demonstrates how to achieve 156,937 vectors/second insertion rate.
This example shows the zero-copy NumPy integration and compares
different insertion patterns for educational purposes.

Requirements:
- Python 3.9+
- NumPy
- OmenDB
"""

import os
import sys
import time
import numpy as np

# Add the python directory to path for development
current_dir = os.path.dirname(os.path.abspath(__file__))
python_dir = os.path.join(os.path.dirname(os.path.dirname(current_dir)), "python")
sys.path.insert(0, python_dir)

from omendb import DB


def benchmark_standard_api(n_vectors=10000, dimension=384):
    """Benchmark using individual adds (NOT RECOMMENDED - for comparison only)."""
    print("🐢 Individual Add API (NOT RECOMMENDED)")
    print("-" * 40)

    db = DB()

    # Generate data
    vectors = [
        [float(np.random.randn()) for _ in range(dimension)] for _ in range(n_vectors)
    ]
    ids = [f"doc_{i}" for i in range(n_vectors)]

    # Measure insertion - SLOW due to FFI overhead per call
    start = time.time()
    for i, vec in enumerate(vectors):
        db.add(ids[i], vec)  # ❌ SLOW: FFI overhead per vector
    elapsed = time.time() - start

    rate = n_vectors / elapsed
    print(f"Insertion rate: {rate:,.0f} vectors/second")
    print(f"Time for {n_vectors:,} vectors: {elapsed:.2f}s")
    print()

    return rate


def benchmark_batch_api(n_vectors=10000, dimension=384):
    """Benchmark using batch API with lists."""
    print("⚡ Batch API Performance (Python Lists)")
    print("-" * 40)

    db = DB()

    # Generate data
    vectors = [
        [float(np.random.randn()) for _ in range(dimension)] for _ in range(n_vectors)
    ]
    ids = [f"doc_{i}" for i in range(n_vectors)]
    metadata = [{} for _ in range(n_vectors)]

    # Measure insertion
    start = time.time()
    db.add_batch(vectors=vectors, ids=ids, metadata=metadata)
    elapsed = time.time() - start

    rate = n_vectors / elapsed
    print(f"Insertion rate: {rate:,.0f} vectors/second")
    print(f"Time for {n_vectors:,} vectors: {elapsed:.2f}s")
    print()

    return rate


def benchmark_numpy_api(n_vectors=100000, dimension=384):
    """Benchmark using zero-copy NumPy API."""
    print("🚀 Zero-Copy NumPy Performance (FASTEST)")
    print("-" * 40)

    db = DB()

    # Generate data - NumPy arrays with optimal layout
    vectors = np.random.rand(n_vectors, dimension).astype(np.float32, order="C")
    ids = [f"doc_{i}" for i in range(n_vectors)]
    metadata = [{} for _ in range(n_vectors)]

    # Measure insertion
    start = time.time()
    # Pass NumPy array directly for zero-copy optimization
    # Note: Converting to lists with .tolist() reduces performance
    results = db.add_batch(vectors=vectors, ids=ids, metadata=metadata)
    elapsed = time.time() - start

    rate = n_vectors / elapsed
    print(f"Insertion rate: {rate:,.0f} vectors/second")
    print(f"Time for {n_vectors:,} vectors: {elapsed:.2f}s")
    print()

    # Verify data
    stats = db.info()
    print(f"Vectors in database: {stats['vector_count']:,}")

    # Test query performance
    query = np.random.rand(dimension).astype(np.float32)
    start = time.time()
    results = db.search(query, limit=10)  # Pass NumPy array directly
    query_time = (time.time() - start) * 1000

    print(f"Query latency: {query_time:.2f}ms")
    print()

    return rate


def main():
    """Demonstrate OmenDB performance capabilities."""

    # Check if running in quick mode
    quick_mode = (
        os.environ.get("OMENDB_TEST_MODE") == "quick"
        or os.environ.get("CI") == "true"
        or "--quick" in sys.argv
    )

    print("🎯 OmenDB Performance Demonstration")
    if quick_mode:
        print("⚡ Running in QUICK MODE for CI/testing")
    print("=" * 50)
    print()

    print("This example shows how to achieve 156,937 vectors/second")
    print("using OmenDB's zero-copy NumPy integration.")
    print()

    # Run benchmarks with increasing sizes
    print("📊 Comparing Different APIs")
    print("=" * 50)
    print()

    # Adjust sizes based on mode
    small_size = 100 if quick_mode else 1000
    large_size = 1000 if quick_mode else 100000

    # Small dataset - show all methods
    print(f"Small Dataset ({small_size:,} vectors)")
    print("-" * 30)
    standard_rate = benchmark_standard_api(small_size)
    batch_rate = benchmark_batch_api(small_size)
    numpy_rate = benchmark_numpy_api(small_size)

    print(f"📈 Performance Summary ({small_size:,} vectors)")
    print(f"Standard API: {standard_rate:>10,.0f} vec/s")
    print(
        f"Batch API:    {batch_rate:>10,.0f} vec/s ({batch_rate / standard_rate:.1f}x faster)"
    )
    print(
        f"NumPy API:    {numpy_rate:>10,.0f} vec/s ({numpy_rate / standard_rate:.1f}x faster)"
    )
    print()

    # Large dataset - show true performance
    print("=" * 50)
    print(f"🚀 Maximum Performance Test ({large_size:,} vectors)")
    print("=" * 50)
    print()

    large_rate = benchmark_numpy_api(large_size)

    if large_rate > 150000:
        print("✅ SUCCESS: Achieved 150K+ vectors/second!")
        print(f"   Actual rate: {large_rate:,.0f} vec/s")
    else:
        print("⚠️  Performance below target. Check:")
        print("   1. NumPy arrays passed directly (not .tolist())")
        print("   2. Using float32 dtype")
        print("   3. Arrays are C-contiguous")

    print()
    print("💡 Key Insights:")
    print("   - Zero-copy NumPy: 156,937 vec/s (1.7x faster)")
    print("   - Batch with lists: 91,435 vec/s (baseline)")
    print("   - Individual adds: ~5K vec/s (FFI overhead)")
    print()
    print("📚 See docs/PERFORMANCE_GUIDE.md for optimization tips")


if __name__ == "__main__":
    main()
