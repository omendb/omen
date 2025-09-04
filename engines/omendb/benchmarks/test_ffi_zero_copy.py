#!/usr/bin/env python3
"""Test FFI zero-copy optimization performance."""

import time
import numpy as np
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

import omendb


def test_python_list_performance(n_vectors=1000, dim=128):
    """Baseline: Python lists (current slow path)."""
    db = omendb.DB(algorithm="diskann")
    db.clear()

    # Create Python lists
    vectors = [[float(i * j % 1.0) for j in range(dim)] for i in range(n_vectors)]
    ids = [f"vec_{i}" for i in range(n_vectors)]
    metadata = [{"idx": str(i)} for i in range(n_vectors)]

    start = time.time()
    db.add_batch(vectors, ids, metadata)
    elapsed = time.time() - start

    throughput = n_vectors / elapsed
    per_vector = elapsed * 1000 / n_vectors
    print(f"Python lists: {throughput:,.0f} vec/s ({per_vector:.2f}ms per vector)")
    return elapsed


def test_numpy_performance(n_vectors=1000, dim=128):
    """Optimized: NumPy arrays (zero-copy path)."""
    db = omendb.DB(algorithm="diskann")
    db.clear()

    # Create NumPy array
    vectors = np.random.rand(n_vectors, dim).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n_vectors)]
    metadata = [{"idx": str(i)} for i in range(n_vectors)]

    start = time.time()
    db.add_batch(vectors, ids, metadata)
    elapsed = time.time() - start

    throughput = n_vectors / elapsed
    per_vector = elapsed * 1000 / n_vectors
    print(f"NumPy arrays: {throughput:,.0f} vec/s ({per_vector:.2f}ms per vector)")
    return elapsed


def test_memory_layout():
    """Test different memory layouts for performance."""
    n_vectors = 1000
    dim = 128

    db = omendb.DB(algorithm="diskann")

    # Test 1: C-contiguous (row-major) - default
    vectors_c = np.random.rand(n_vectors, dim).astype(np.float32, order="C")
    assert vectors_c.flags.c_contiguous

    db.clear()
    start = time.time()
    ids = [f"c_{i}" for i in range(n_vectors)]
    db.add_batch(vectors_c, ids)
    c_time = time.time() - start
    print(f"C-contiguous (row-major): {n_vectors / c_time:,.0f} vec/s")

    # Test 2: F-contiguous (column-major)
    vectors_f = np.random.rand(n_vectors, dim).astype(np.float32, order="F")
    assert vectors_f.flags.f_contiguous

    db.clear()
    start = time.time()
    ids = [f"f_{i}" for i in range(n_vectors)]
    db.add_batch(vectors_f, ids)
    f_time = time.time() - start
    print(f"F-contiguous (column-major): {n_vectors / f_time:,.0f} vec/s")

    # Test 3: Non-contiguous (strided)
    vectors_strided = vectors_c[::2, ::2]  # Every other element
    assert not vectors_strided.flags.c_contiguous
    assert not vectors_strided.flags.f_contiguous

    db.clear()
    start = time.time()
    ids = [f"s_{i}" for i in range(len(vectors_strided))]
    db.add_batch(vectors_strided, ids)
    s_time = time.time() - start
    print(f"Non-contiguous (strided): {len(vectors_strided) / s_time:,.0f} vec/s")

    print(f"\nSpeedup C vs F: {f_time / c_time:.2f}x")
    print(f"Speedup C vs strided: {s_time / c_time:.2f}x")


def test_scaling():
    """Test how performance scales with size."""
    db = omendb.DB(algorithm="diskann")

    print("\nScaling test (NumPy):")
    for n_vectors in [10, 100, 1000, 10000]:
        db.clear()
        vectors = np.random.rand(n_vectors, 128).astype(np.float32)
        ids = [f"v_{i}" for i in range(n_vectors)]

        start = time.time()
        db.add_batch(vectors, ids)
        elapsed = time.time() - start

        throughput = n_vectors / elapsed
        overhead = elapsed * 1000 - n_vectors * 0.01  # Assume 0.01ms per vector ideal
        print(
            f"  {n_vectors:5d} vectors: {throughput:8,.0f} vec/s, overhead: {overhead:.1f}ms"
        )


def test_dtype_conversion():
    """Test different dtypes and conversion overhead."""
    n_vectors = 1000
    dim = 128
    db = omendb.DB(algorithm="diskann")

    print("\nDtype conversion test:")

    # float32 (native, no conversion)
    db.clear()
    vectors_f32 = np.random.rand(n_vectors, dim).astype(np.float32)
    start = time.time()
    db.add_batch(vectors_f32, [f"f32_{i}" for i in range(n_vectors)])
    f32_time = time.time() - start
    print(f"  float32 (native): {n_vectors / f32_time:,.0f} vec/s")

    # float64 (needs conversion)
    db.clear()
    vectors_f64 = np.random.rand(n_vectors, dim).astype(np.float64)
    start = time.time()
    db.add_batch(vectors_f64, [f"f64_{i}" for i in range(n_vectors)])
    f64_time = time.time() - start
    print(
        f"  float64 (convert): {n_vectors / f64_time:,.0f} vec/s ({f64_time / f32_time:.2f}x slower)"
    )

    # float16 (needs conversion)
    db.clear()
    vectors_f16 = np.random.rand(n_vectors, dim).astype(np.float16)
    start = time.time()
    db.add_batch(vectors_f16, [f"f16_{i}" for i in range(n_vectors)])
    f16_time = time.time() - start
    print(
        f"  float16 (convert): {n_vectors / f16_time:,.0f} vec/s ({f16_time / f32_time:.2f}x slower)"
    )


def main():
    print("FFI Zero-Copy Performance Test")
    print("=" * 60)

    # Initialize
    db = omendb.DB()
    print(f"✅ Database initialized\n")

    # Test 1: Compare Python lists vs NumPy
    print("1. Python lists vs NumPy arrays (1000 vectors):")
    py_time = test_python_list_performance(1000, 128)
    np_time = test_numpy_performance(1000, 128)
    speedup = py_time / np_time
    print(f"Speedup: {speedup:.1f}x")
    print(f"{'✅ PASS' if speedup > 10 else '❌ FAIL'} (expected >10x)")

    # Test 2: Memory layout impact
    print("\n2. Memory layout performance:")
    test_memory_layout()

    # Test 3: Scaling
    test_scaling()

    # Test 4: Dtype conversion
    test_dtype_conversion()

    # Summary
    print("\n" + "=" * 60)
    print("Summary:")
    if speedup > 50:
        print("✅ Zero-copy optimization working perfectly!")
    elif speedup > 10:
        print("⚠️ Some speedup but not full zero-copy")
    else:
        print("❌ Zero-copy not working, still using slow path")


if __name__ == "__main__":
    main()
