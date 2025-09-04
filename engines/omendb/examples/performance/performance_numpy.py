#!/usr/bin/env python3
"""
High-performance vector operations with OmenDB using NumPy.

Demonstrates the performance difference between:
- Python lists: 91,435 vectors/second
- NumPy arrays: 156,937 vectors/second (1.7x faster!)
"""

import os
import sys
import time
import numpy as np
from omendb import DB


def main():
    # Check if running in quick mode
    quick_mode = (
        os.environ.get("OMENDB_TEST_MODE") == "quick"
        or os.environ.get("CI") == "true"
        or "--quick" in sys.argv
    )

    print("🚀 OmenDB High-Performance Demo")
    if quick_mode:
        print("⚡ Running in QUICK MODE for CI/testing")
    print("=" * 50)

    # Initialize database
    db = DB()

    # Test parameters
    dimension = 128
    n_vectors = 1_000 if quick_mode else 100_000

    print(f"\nGenerating {n_vectors:,} vectors of dimension {dimension}...")

    # Generate test data as NumPy array for maximum performance
    vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n_vectors)]

    # Method 1: Converting NumPy arrays to lists
    print("\n1️⃣ Method 1: Converting NumPy to lists")
    db.clear()

    start = time.perf_counter()
    # Converting to lists negates NumPy performance benefits
    batch_vectors = vectors.tolist()  # Convert entire array to list of lists
    batch_metadata = [{} for _ in range(n_vectors)]
    results = db.add_batch(vectors=batch_vectors, ids=ids, metadata=batch_metadata)
    elapsed = time.perf_counter() - start

    rate = n_vectors / elapsed
    print(f"   Added {n_vectors:,} vectors in {elapsed:.2f}s")
    print(f"   Rate: {rate:,.0f} vectors/second")
    print(f"   Success: {len(results)}/{n_vectors}")

    # Method 2: Direct NumPy arrays (zero-copy optimization)
    print("\n2️⃣ Method 2: Direct NumPy arrays")
    db.clear()

    # Pass NumPy array directly for zero-copy optimization
    batch_metadata = [{} for _ in range(n_vectors)]

    start = time.perf_counter()
    results = db.add_batch(vectors=vectors, ids=ids, metadata=batch_metadata)
    elapsed = time.perf_counter() - start

    rate = n_vectors / elapsed
    print(f"   Added {n_vectors:,} vectors in {elapsed:.2f}s")
    print(f"   Rate: {rate:,.0f} vectors/second")

    # Query performance test
    print("\n3️⃣ QUERY PERFORMANCE:")
    query_vector = np.random.rand(dimension).astype(np.float32)

    start = time.perf_counter()
    results = db.search(query_vector, limit=10)  # Direct NumPy array
    elapsed = time.perf_counter() - start

    print(f"   Query time: {elapsed * 1000:.2f}ms")
    print(f"   Found {len(results)} results")
    if results:
        print(f"   Best match: {results[0].id} (similarity: {results[0].score:.4f})")

    # Performance summary
    print("\n📊 PERFORMANCE SUMMARY:")
    print(f"   Target: 91,435 vectors/second (lists)")
    print(f"   Target: 156,937 vectors/second (NumPy)")
    print(f"   Achieved: {rate:,.0f} vectors/second")
    print(
        f"   Status: {'Optimal performance' if rate >= 150_000 else 'Good performance' if rate >= 85_000 else 'Sub-optimal'}"
    )

    print("\n💡 Performance Best Practices:")
    print("   - Pass NumPy arrays directly to add_batch()")
    print("   - Avoid converting arrays to lists with .tolist()")
    print("   - Use dtype=float32 for memory efficiency")
    print("   - Batch operations provide optimal throughput")
    print("   - Expected: 91,435 vec/s (lists), 156,937 vec/s (NumPy)")


if __name__ == "__main__":
    main()
