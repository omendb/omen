#!/usr/bin/env python3
"""Test NumPy optimization is working correctly"""

import time
import numpy as np
import sys

sys.path.insert(0, "python")
import omendb


def test_numpy_optimization():
    """Test NumPy zero-copy optimization"""
    print("🧪 Testing NumPy Zero-Copy Optimization")
    print("=" * 60)

    batch_size = 1000
    dimension = 128
    ids = [f"vec_{i}" for i in range(batch_size)]
    metadata = [{} for _ in range(batch_size)]

    # Test 1: Python lists (baseline)
    print("\n📊 Test 1: Python Lists (baseline)")
    vectors_list = [[float(j) for j in range(dimension)] for i in range(batch_size)]

    db1 = omendb.DB()
    start = time.time()
    db1.add_batch(vectors=vectors_list, ids=ids, metadata=metadata)
    list_time = time.time() - start
    list_rate = batch_size / list_time
    print(f"   Rate: {list_rate:,.0f} vec/s")

    # Test 2: NumPy with .tolist() (WRONG - converts to lists)
    print("\n📊 Test 2: NumPy with .tolist() (converts to lists)")
    vectors_numpy = np.random.rand(batch_size, dimension).astype(np.float32)

    db2 = omendb.DB()
    start = time.time()
    db2.add_batch(
        vectors=vectors_numpy.tolist(), ids=ids, metadata=metadata
    )  # ❌ tolist()
    tolist_time = time.time() - start
    tolist_rate = batch_size / tolist_time
    print(f"   Rate: {tolist_rate:,.0f} vec/s")

    # Test 3: NumPy direct (CORRECT - zero-copy)
    print("\n📊 Test 3: NumPy Direct (zero-copy)")
    vectors_numpy2 = np.random.rand(batch_size, dimension).astype(np.float32)

    db3 = omendb.DB()
    start = time.time()
    db3.add_batch(vectors=vectors_numpy2, ids=ids, metadata=metadata)  # ✅ Direct array
    numpy_time = time.time() - start
    numpy_rate = batch_size / numpy_time
    print(f"   Rate: {numpy_rate:,.0f} vec/s")

    # Test 4: NumPy C-order vs F-order
    print("\n📊 Test 4: NumPy Memory Layout")

    # C-order (row-major, default)
    vectors_c = np.random.rand(batch_size, dimension).astype(np.float32, order="C")
    db4 = omendb.DB()
    start = time.time()
    db4.add_batch(vectors=vectors_c, ids=ids, metadata=metadata)
    c_time = time.time() - start
    c_rate = batch_size / c_time
    print(f"   C-order: {c_rate:,.0f} vec/s")

    # F-order (column-major)
    vectors_f = np.random.rand(batch_size, dimension).astype(np.float32, order="F")
    db5 = omendb.DB()
    start = time.time()
    db5.add_batch(vectors=vectors_f, ids=ids, metadata=metadata)
    f_time = time.time() - start
    f_rate = batch_size / f_time
    print(f"   F-order: {f_rate:,.0f} vec/s")

    # Summary
    print("\n📈 Summary:")
    print(f"   Python lists:      {list_rate:8,.0f} vec/s (baseline)")
    print(
        f"   NumPy .tolist():   {tolist_rate:8,.0f} vec/s ({tolist_rate / list_rate:.1f}x)"
    )
    print(
        f"   NumPy direct:      {numpy_rate:8,.0f} vec/s ({numpy_rate / list_rate:.1f}x)"
    )
    print(f"   NumPy C-order:     {c_rate:8,.0f} vec/s ({c_rate / list_rate:.1f}x)")
    print(f"   NumPy F-order:     {f_rate:8,.0f} vec/s ({f_rate / list_rate:.1f}x)")

    print("\n🎯 Conclusion:")
    if numpy_rate > list_rate * 1.5:
        print("   ✅ NumPy optimization is WORKING!")
        print(f"   ✅ {numpy_rate / list_rate:.1f}x speedup achieved")
    else:
        print("   ❌ NumPy optimization is NOT working")
        print("   ❌ Expected 2-3x speedup, got {numpy_rate/list_rate:.1f}x")


if __name__ == "__main__":
    test_numpy_optimization()
