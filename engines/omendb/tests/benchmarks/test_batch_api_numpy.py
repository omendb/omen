#!/usr/bin/env python3
"""
Test numpy zero-copy batch API performance improvements.
"""

import time
import numpy as np
import omendb


def test_batch_api_performance():
    """Compare batch API performance with numpy arrays vs Python lists."""

    # Test parameters
    num_vectors = 10000
    dimension = 128

    print("BATCH API PERFORMANCE TEST")
    print("=" * 70)
    print(f"Vectors: {num_vectors:,} | Dimension: {dimension}")
    print()

    # Generate test data
    print("Generating test data...")
    numpy_vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
    python_lists = numpy_vectors.tolist()
    ids = [f"vec_{i}" for i in range(num_vectors)]

    # Test 1: Python lists (baseline)
    print("\n1. Python Lists (element-by-element conversion)")
    print("-" * 40)
    db1 = omendb.DB()

    start = time.time()
    # Use new columnar API
    result_ids = db1.add_batch(vectors=python_lists, ids=ids)
    list_time = time.time() - start

    list_rate = num_vectors / list_time
    print(f"Time: {list_time:.3f}s")
    print(f"Rate: {list_rate:,.0f} vec/s")
    print(f"Added: {len(result_ids)} vectors")

    # Clear database
    db1.clear()

    # Test 2: Numpy arrays (zero-copy)
    print("\n2. NumPy Arrays (zero-copy path)")
    print("-" * 40)
    db2 = omendb.DB()

    start = time.time()
    # Pass numpy array directly - optimal for zero-copy
    result_ids = db2.add_batch(vectors=numpy_vectors, ids=ids)
    numpy_time = time.time() - start

    numpy_rate = num_vectors / numpy_time
    print(f"Time: {numpy_time:.3f}s")
    print(f"Rate: {numpy_rate:,.0f} vec/s")
    print(f"Added: {len(result_ids)} vectors")

    # Calculate speedup
    speedup = numpy_rate / list_rate
    print("\n" + "=" * 70)
    print(f"SPEEDUP: {speedup:.1f}x faster with NumPy zero-copy!")
    print("=" * 70)

    # Verify correctness - both should have same data
    print("\nVerifying correctness...")
    query_vec = numpy_vectors[0]

    db1.clear()
    db1.add_batch(vectors=python_lists[:100], ids=ids[:100])
    results1 = db1.search(query_vec, 5)

    db2.clear()
    db2.add_batch(vectors=numpy_vectors[:100], ids=ids[:100])
    results2 = db2.search(query_vec, 5)

    # Check if results match
    ids1 = [r.id for r in results1]
    ids2 = [r.id for r in results2]

    if ids1 == ids2:
        print("✅ Results match - zero-copy path is correct!")
    else:
        print("❌ Results differ - there may be an issue")
        print(f"List path: {ids1}")
        print(f"Numpy path: {ids2}")


if __name__ == "__main__":
    test_batch_api_performance()
