#!/usr/bin/env python3
"""Test optimized API performance vs original"""

import numpy as np
import time
import sys

sys.path.insert(0, "python")

# Import both APIs
from omendb.api import DB as OriginalDB
from omendb.api_optimized import OptimizedDB


def test_performance_comparison():
    """Compare original vs optimized API performance"""
    print("OPTIMIZED API PERFORMANCE TEST")
    print("=" * 60)

    n = 5000
    dim = 128
    vectors = np.random.randn(n, dim).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n)]

    # Test 1: Original API
    print("\n1. Original API Performance:")
    db_orig = OriginalDB()
    db_orig.clear()
    db_orig.configure(buffer_size=10000)

    start = time.perf_counter()
    for i in range(n):
        db_orig.add(ids[i], vectors[i])
    orig_time = time.perf_counter() - start

    orig_rate = n / orig_time
    print(f"   Add rate: {orig_rate:.0f} vec/s")
    print(f"   Time per op: {orig_time / n * 1000:.3f}ms")

    # Test 2: Optimized API
    print("\n2. Optimized API Performance:")
    db_opt = OptimizedDB()
    db_opt.clear()

    start = time.perf_counter()
    for i in range(n):
        db_opt.add(ids[i], vectors[i])
    opt_time = time.perf_counter() - start

    opt_rate = n / opt_time
    print(f"   Add rate: {opt_rate:.0f} vec/s")
    print(f"   Time per op: {opt_time / n * 1000:.3f}ms")

    # Test 3: Ultra-optimized batch
    print("\n3. Ultra-Optimized Batch:")
    db_opt.clear()

    start = time.perf_counter()
    db_opt.add_batch_optimized(vectors, ids)
    batch_time = time.perf_counter() - start

    batch_rate = n / batch_time
    print(f"   Add rate: {batch_rate:.0f} vec/s")
    print(f"   Time per op: {batch_time / n * 1000:.3f}ms")

    # Results
    print("\n" + "=" * 60)
    print("RESULTS:")
    print(f"Original API:     {orig_rate:6.0f} vec/s (baseline)")
    print(
        f"Optimized API:    {opt_rate:6.0f} vec/s ({opt_rate / orig_rate:.1f}x speedup)"
    )
    print(
        f"Batch Optimized:  {batch_rate:6.0f} vec/s ({batch_rate / orig_rate:.1f}x speedup)"
    )

    if opt_rate > orig_rate * 1.5:
        print("\n✅ Optimization successful! Significant speedup achieved.")
    else:
        print("\n⚠️ Limited improvement. FFI overhead still dominates.")


def test_validation_overhead():
    """Measure validation overhead specifically"""
    print("\n" + "=" * 60)
    print("VALIDATION OVERHEAD MEASUREMENT")
    print("=" * 60)

    n = 10000
    dim = 128

    # Test with numpy arrays (should skip validation in optimized)
    vectors_np = np.random.randn(n, dim).astype(np.float32)

    # Test with Python lists (requires validation)
    vectors_list = vectors_np.tolist()

    # Import validation functions
    from omendb.api import _validate_vector
    from omendb.api_optimized import _validate_vector_fast

    # Original validation - numpy
    start = time.perf_counter()
    for i in range(1000):
        vec_list = vectors_np[i].tolist()  # Original always converts
        _validate_vector(vec_list)
    orig_np_time = time.perf_counter() - start

    # Optimized validation - numpy
    start = time.perf_counter()
    for i in range(1000):
        _validate_vector_fast(vectors_np[i])  # Should be instant
    opt_np_time = time.perf_counter() - start

    # Original validation - lists
    start = time.perf_counter()
    for i in range(1000):
        _validate_vector(vectors_list[i])
    orig_list_time = time.perf_counter() - start

    # Optimized validation - lists
    start = time.perf_counter()
    for i in range(1000):
        _validate_vector_fast(vectors_list[i])
    opt_list_time = time.perf_counter() - start

    print("Validation time per vector (1000 iterations):")
    print(f"  NumPy arrays:")
    print(
        f"    Original:  {orig_np_time * 1000:.3f}ms total ({orig_np_time:.6f}ms per vector)"
    )
    print(
        f"    Optimized: {opt_np_time * 1000:.3f}ms total ({opt_np_time:.6f}ms per vector)"
    )
    print(f"    Speedup:   {orig_np_time / opt_np_time:.1f}x")

    print(f"\n  Python lists:")
    print(
        f"    Original:  {orig_list_time * 1000:.3f}ms total ({orig_list_time:.6f}ms per vector)"
    )
    print(
        f"    Optimized: {opt_list_time * 1000:.3f}ms total ({opt_list_time:.6f}ms per vector)"
    )
    print(f"    Speedup:   {orig_list_time / opt_list_time:.1f}x")


if __name__ == "__main__":
    test_performance_comparison()
    test_validation_overhead()
