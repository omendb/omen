#!/usr/bin/env python3
"""Test numpy zero-copy performance improvements"""

import omendb
import numpy as np
import time


def test_numpy_performance():
    """Test performance with numpy arrays vs lists"""
    print("NUMPY ZERO-COPY PERFORMANCE TEST")
    print("=" * 60)

    n = 5000
    dim = 128

    # Generate numpy arrays
    vectors_np = np.random.randn(n, dim).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n)]

    # Test 1: Individual adds with numpy arrays
    db = omendb.DB()
    db.clear()
    db.configure(buffer_size=10000)

    start = time.perf_counter()
    for i in range(n):
        db.add(ids[i], vectors_np[i])  # Pass numpy array directly
    numpy_time = time.perf_counter() - start

    numpy_rate = n / numpy_time
    print(f"Numpy arrays:    {numpy_rate:.0f} vec/s")

    # Test 2: Individual adds with Python lists
    db.clear()
    vectors_list = vectors_np.tolist()  # Convert to Python lists

    start = time.perf_counter()
    for i in range(n):
        db.add(ids[i], vectors_list[i])  # Pass Python list
    list_time = time.perf_counter() - start

    list_rate = n / list_time
    print(f"Python lists:    {list_rate:.0f} vec/s")

    speedup = numpy_rate / list_rate
    print(f"\nSpeedup: {speedup:.2f}x")

    # Test 3: Batch with numpy (should be fastest)
    db.clear()

    start = time.perf_counter()
    db.add_batch(vectors_np, ids)
    batch_time = time.perf_counter() - start

    batch_rate = n / batch_time
    print(f"\nBatch numpy:     {batch_rate:.0f} vec/s")

    # Test 4: Check memory usage pattern
    print("\n" + "=" * 60)
    print("MEMORY COPY ANALYSIS")
    print("=" * 60)

    # Small test to see if we're copying
    small_vec = np.array([1.0, 2.0, 3.0, 4.0], dtype=np.float32)
    db.clear()

    # Modify after adding - if zero-copy, DB would see change
    db.add("test", small_vec)
    small_vec[0] = 999.0  # Modify original

    # Search for the vector
    results = db.search([1.0, 2.0, 3.0, 4.0], limit=1)
    if results and results[0].id == "test":
        print("✅ Data was copied (correct behavior)")
    else:
        print("⚠️ Data might be aliased (potential issue)")


if __name__ == "__main__":
    test_numpy_performance()
