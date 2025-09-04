#!/usr/bin/env python3
"""Test performance without HNSW migration."""

import sys
import time
import numpy as np

sys.path.insert(0, "/Users/nick/github/omendb/omendb/python")

from omendb import DB


def test_performance():
    """Test performance at different scales."""

    # Test 1: Below migration threshold (pure brute force)
    print("📊 Test 1: 4000 vectors @128D (no migration)")
    db1 = DB()
    n_vectors = 4000
    dimension = 128

    vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    ids = [f"test_{i}" for i in range(n_vectors)]

    start = time.time()
    db1.add_batch(vectors=vectors, ids=ids)
    elapsed = time.time() - start

    rate = n_vectors / elapsed if elapsed > 0 else float("inf")
    print(f"  Rate: {rate:,.0f} vec/s")
    print(f"  Time: {elapsed:.3f}s")

    # Test 2: Small batch at 128D
    print("\n📊 Test 2: 1000 vectors @128D")
    db2 = DB()
    n_vectors = 1000

    vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    ids = [f"test_{i}" for i in range(n_vectors)]

    start = time.time()
    db2.add_batch(vectors=vectors, ids=ids)
    elapsed = time.time() - start

    rate = n_vectors / elapsed if elapsed > 0 else float("inf")
    print(f"  Rate: {rate:,.0f} vec/s")
    print(f"  Time: {elapsed:.3f}s")

    # Test 3: Test single-vector adds like the benchmark
    print("\n📊 Test 3: Single adds @768D (like benchmark)")
    db3 = DB()
    n_vectors = 1000
    dimension = 768

    vectors = np.random.rand(n_vectors, dimension).astype(np.float32)

    start = time.time()
    for i in range(n_vectors):
        db3.add(f"test_{i}", vectors[i].tolist())
    elapsed = time.time() - start

    rate = n_vectors / elapsed if elapsed > 0 else float("inf")
    print(f"  Rate: {rate:,.0f} vec/s")
    print(f"  Time: {elapsed:.3f}s")

    # Test 4: Batch at 768D
    print("\n📊 Test 4: Batch adds @768D")
    db4 = DB()

    vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    ids = [f"test_{i}" for i in range(n_vectors)]

    start = time.time()
    db4.add_batch(vectors=vectors, ids=ids)
    elapsed = time.time() - start

    rate = n_vectors / elapsed if elapsed > 0 else float("inf")
    print(f"  Rate: {rate:,.0f} vec/s")
    print(f"  Time: {elapsed:.3f}s")


if __name__ == "__main__":
    test_performance()
