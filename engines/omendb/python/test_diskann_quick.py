#!/usr/bin/env python3
"""Quick DiskANN performance test."""

import numpy as np
import time
import sys
import os

# Add the local development path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import omendb


def quick_test():
    """Quick test of DiskANN performance."""

    sizes = [100, 1000, 5000]

    for num_vectors in sizes:
        print(f"\n{'=' * 60}")
        print(f"Testing {num_vectors} vectors...")
        print(f"{'=' * 60}")

        # Create database with DiskANN
        db = omendb.DB(algorithm="diskann")

        # Generate test data
        dimension = 128
        vectors = np.random.randn(num_vectors, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(num_vectors)]

        # Measure batch add performance
        start_time = time.perf_counter()
        results = db.add_batch(vectors, ids=ids)
        add_time = time.perf_counter() - start_time

        vectors_per_sec = num_vectors / add_time if add_time > 0 else 0

        print(f"✅ Added {len(results)} vectors in {add_time:.2f}s")
        print(f"📊 Throughput: {vectors_per_sec:.0f} vec/s")

        # Quick search test
        query = vectors[0]
        start_time = time.perf_counter()
        search_results = db.search(query, limit=10)
        search_time = (time.perf_counter() - start_time) * 1000

        print(f"🔍 Search latency: {search_time:.2f}ms")

        db.clear()


if __name__ == "__main__":
    quick_test()
