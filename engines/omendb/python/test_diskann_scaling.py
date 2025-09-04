#!/usr/bin/env python3
"""DiskANN scaling benchmark - test at various scales."""

import numpy as np
import time
import sys
import os

# Add the local development path
sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))

import omendb


def benchmark_scale():
    """Benchmark DiskANN at various scales."""

    # Test various scales
    sizes = [100, 500, 1000, 2500, 5000, 10000, 15000, 25000]
    dimension = 128

    print("=" * 60)
    print("DiskANN Scaling Benchmark")
    print("=" * 60)
    print(f"{'Size':<10} {'Add (s)':<10} {'Vec/s':<12} {'Search (ms)':<12}")
    print("-" * 50)

    results = []

    for num_vectors in sizes:
        # Create database with DiskANN
        db = omendb.DB(algorithm="diskann", buffer_size=min(10000, num_vectors))

        # Generate test data
        vectors = np.random.randn(num_vectors, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(num_vectors)]

        # Measure batch add performance
        start_time = time.perf_counter()
        added = db.add_batch(vectors, ids=ids)
        add_time = time.perf_counter() - start_time

        vectors_per_sec = num_vectors / add_time if add_time > 0 else 0

        # Measure search performance (average of 5 queries)
        search_times = []
        for _ in range(5):
            query_idx = np.random.randint(0, min(100, num_vectors))
            query = vectors[query_idx]

            start_time = time.perf_counter()
            search_results = db.search(query, limit=10)
            search_time = (time.perf_counter() - start_time) * 1000
            search_times.append(search_time)

        avg_search = np.mean(search_times)

        print(
            f"{num_vectors:<10} {add_time:<10.2f} {vectors_per_sec:<12.0f} {avg_search:<12.2f}"
        )

        results.append(
            {
                "size": num_vectors,
                "add_time": add_time,
                "vec_per_sec": vectors_per_sec,
                "search_ms": avg_search,
            }
        )

        db.clear()

    # Compare with competitors
    print("\n" + "=" * 60)
    print("Comparison with Known Competitor Performance")
    print("=" * 60)

    # Find 10K result
    for r in results:
        if r["size"] == 10000:
            our_rate = r["vec_per_sec"]
            print(f"OmenDB (DiskANN) @ 10K: {our_rate:.0f} vec/s")
            print(f"ChromaDB @ 10K: 4,772 vec/s")
            print(f"Speedup: {our_rate / 4772:.1f}x faster than ChromaDB")
            break

    return results


if __name__ == "__main__":
    benchmark_scale()
