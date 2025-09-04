#!/usr/bin/env python3
"""Test FFI overhead to understand the bottleneck."""

import time
import numpy as np
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

import omendb
import omendb.native as native


def test_ffi_conversion():
    """Test different data formats through FFI."""

    # Test data
    n_vectors = 100
    dim = 128

    # Python list of lists
    py_vectors = [[float(i * j) for j in range(dim)] for i in range(n_vectors)]
    py_ids = [f"vec_{i}" for i in range(n_vectors)]
    py_metadata = [{}] * n_vectors

    # NumPy array
    np_vectors = np.array(py_vectors, dtype=np.float32)

    print("Testing FFI conversion overhead:")
    print("-" * 50)

    # Test 1: Python lists
    start = time.time()
    for _ in range(10):
        try:
            native.add_vector_batch(py_ids, py_vectors, py_metadata)
        except:
            pass
    py_time = (time.time() - start) / 10
    print(f"Python lists: {py_time * 1000:.1f}ms for {n_vectors} vectors")

    # Test 2: NumPy array
    start = time.time()
    for _ in range(10):
        try:
            native.add_vector_batch(py_ids, np_vectors, py_metadata)
        except:
            pass
    np_time = (time.time() - start) / 10
    print(f"NumPy array:  {np_time * 1000:.1f}ms for {n_vectors} vectors")

    # Test 3: Individual adds
    start = time.time()
    for i in range(n_vectors):
        try:
            native.add_vector(py_ids[i], py_vectors[i], {})
        except:
            pass
    ind_time = time.time() - start
    print(f"Individual:   {ind_time * 1000:.1f}ms for {n_vectors} vectors")

    print(f"\nSpeedup from batching: {ind_time / py_time:.1f}x")
    print(f"NumPy vs Python lists: {py_time / np_time:.1f}x faster")


def test_data_sizes():
    """Test how data size affects FFI overhead."""
    print("\nFFI overhead by data size:")
    print("-" * 50)

    for n_vectors in [10, 50, 100, 500, 1000]:
        vectors = [[float(i) for i in range(128)] for _ in range(n_vectors)]
        ids = [f"v_{i}" for i in range(n_vectors)]
        metadata = [{}] * n_vectors

        # Time just the FFI call
        start = time.time()
        try:
            native.add_vector_batch(ids, vectors, metadata)
        except:
            pass
        elapsed = time.time() - start

        per_vector = elapsed * 1000 / n_vectors
        print(
            f"{n_vectors:4d} vectors: {elapsed * 1000:6.1f}ms total, {per_vector:.2f}ms per vector"
        )


def test_dimension_impact():
    """Test how dimension affects FFI overhead."""
    print("\nFFI overhead by dimension:")
    print("-" * 50)

    n_vectors = 100
    for dim in [3, 16, 64, 128, 256, 512, 1024]:
        vectors = [[float(i) for i in range(dim)] for _ in range(n_vectors)]
        ids = [f"v_{i}" for i in range(n_vectors)]
        metadata = [{}] * n_vectors

        # Time just the FFI call
        start = time.time()
        try:
            native.add_vector_batch(ids, vectors, metadata)
        except:
            pass
        elapsed = time.time() - start

        print(f"{dim:4d}D: {elapsed * 1000:6.1f}ms for {n_vectors} vectors")


if __name__ == "__main__":
    # Initialize database once
    db = omendb.DB()
    db.clear()

    test_ffi_conversion()
    test_data_sizes()
    test_dimension_impact()
