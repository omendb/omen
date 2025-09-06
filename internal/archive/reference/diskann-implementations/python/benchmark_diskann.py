#!/usr/bin/env python3
"""
Comprehensive DiskANN performance benchmark for OmenDB.
Tests various scales and measures actual performance metrics.
"""

import numpy as np
import time
import sys
import os
import json
from typing import List, Dict, Any

# Add the local development path
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def benchmark_batch_operations(
    sizes: List[int], dimension: int = 128
) -> Dict[str, Any]:
    """Benchmark batch add operations at various scales."""

    results = {}

    for num_vectors in sizes:
        print(f"\n{'=' * 60}")
        print(f"Benchmarking {num_vectors} vectors @ {dimension}D")
        print(f"{'=' * 60}")

        # Create fresh database with DiskANN
        db = omendb.DB(algorithm="diskann", buffer_size=min(num_vectors, 10000))

        # Generate test data
        print(f"Generating {num_vectors} random vectors...")
        vectors = np.random.randn(num_vectors, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(num_vectors)]

        # Measure batch add performance
        print(f"Adding batch...")
        start_time = time.perf_counter()
        added_ids = db.add_batch(vectors, ids=ids)
        add_time = time.perf_counter() - start_time

        vectors_per_sec = num_vectors / add_time if add_time > 0 else 0

        # Verify all vectors were added
        success_rate = len(added_ids) / num_vectors * 100

        # Measure search performance
        print(f"Testing search performance...")
        query = vectors[0]

        # Warm up
        _ = db.search(query, limit=10)

        # Measure search latency (average of 10 queries)
        search_times = []
        for i in range(10):
            query_idx = np.random.randint(0, min(100, num_vectors))
            query = vectors[query_idx]

            start_time = time.perf_counter()
            results_search = db.search(query, limit=10)
            search_time = time.perf_counter() - start_time
            search_times.append(search_time * 1000)  # Convert to ms

        avg_search_ms = np.mean(search_times)
        p95_search_ms = np.percentile(search_times, 95)

        # Get database info
        info = db.info()

        # Store results
        result = {
            "num_vectors": num_vectors,
            "dimension": dimension,
            "add_time_sec": round(add_time, 3),
            "vectors_per_sec": round(vectors_per_sec, 0),
            "success_rate": round(success_rate, 1),
            "avg_search_ms": round(avg_search_ms, 2),
            "p95_search_ms": round(p95_search_ms, 2),
            "buffer_size": info.get("buffer_size", 0),
            "main_index_size": info.get("main_index_size", 0),
            "algorithm": info.get("algorithm", "unknown"),
        }

        results[num_vectors] = result

        # Print summary
        print(f"\nResults for {num_vectors} vectors:")
        print(f"  Add time: {result['add_time_sec']}s")
        print(f"  Throughput: {result['vectors_per_sec']:.0f} vec/s")
        print(f"  Success rate: {result['success_rate']}%")
        print(f"  Avg search: {result['avg_search_ms']}ms")
        print(f"  P95 search: {result['p95_search_ms']}ms")

        # Clear database for next test
        db.clear()

    return results


def benchmark_incremental_vs_batch():
    """Compare incremental add vs batch add performance."""

    print(f"\n{'=' * 60}")
    print("Incremental vs Batch Performance Comparison")
    print(f"{'=' * 60}")

    num_vectors = 1000
    dimension = 128

    # Generate test data
    vectors = np.random.randn(num_vectors, dimension).astype(np.float32)
    ids = [f"vec_{i}" for i in range(num_vectors)]

    # Test incremental adds
    print("\n1. Testing incremental adds...")
    db1 = omendb.DB(algorithm="diskann")

    start_time = time.perf_counter()
    for i in range(num_vectors):
        db1.add(ids[i], vectors[i].tolist())
    incremental_time = time.perf_counter() - start_time
    incremental_rate = num_vectors / incremental_time

    # Test batch add
    print("\n2. Testing batch add...")
    db2 = omendb.DB(algorithm="diskann")

    start_time = time.perf_counter()
    db2.add_batch(vectors, ids=ids)
    batch_time = time.perf_counter() - start_time
    batch_rate = num_vectors / batch_time

    # Calculate speedup
    speedup = batch_rate / incremental_rate

    print(f"\nResults:")
    print(
        f"  Incremental: {incremental_rate:.0f} vec/s ({incremental_time:.2f}s total)"
    )
    print(f"  Batch: {batch_rate:.0f} vec/s ({batch_time:.2f}s total)")
    print(f"  Speedup: {speedup:.1f}x faster with batch")

    return {
        "incremental_rate": incremental_rate,
        "batch_rate": batch_rate,
        "speedup": speedup,
    }


def benchmark_scaling():
    """Test how performance scales with dataset size."""

    print(f"\n{'=' * 60}")
    print("Performance Scaling Analysis")
    print(f"{'=' * 60}")

    # Test at different scales
    sizes = [100, 500, 1000, 2500, 5000, 10000, 25000]
    dimension = 128

    results = benchmark_batch_operations(sizes, dimension)

    # Analyze scaling
    print(f"\n{'=' * 60}")
    print("Scaling Summary")
    print(f"{'=' * 60}")
    print(f"{'Size':<10} {'Add (vec/s)':<15} {'Search (ms)':<12} {'Success %':<10}")
    print("-" * 50)

    for size in sizes:
        if size in results:
            r = results[size]
            print(
                f"{size:<10} {r['vectors_per_sec']:<15.0f} {r['avg_search_ms']:<12.2f} {r['success_rate']:<10.1f}"
            )

    # Calculate scaling efficiency
    if 1000 in results and 10000 in results:
        small_rate = results[1000]["vectors_per_sec"]
        large_rate = results[10000]["vectors_per_sec"]
        scaling_efficiency = (large_rate / small_rate) * (1000 / 10000) * 100
        print(f"\nScaling efficiency (1K→10K): {scaling_efficiency:.1f}%")

    return results


def compare_with_competitors():
    """Compare OmenDB DiskANN performance with known competitor benchmarks."""

    print(f"\n{'=' * 60}")
    print("Competitor Comparison (from published benchmarks)")
    print(f"{'=' * 60}")

    # Run our benchmark
    num_vectors = 10000
    dimension = 128

    db = omendb.DB(algorithm="diskann", buffer_size=5000)
    vectors = np.random.randn(num_vectors, dimension).astype(np.float32)
    ids = [f"vec_{i}" for i in range(num_vectors)]

    start_time = time.perf_counter()
    added_ids = db.add_batch(vectors, ids=ids)
    add_time = time.perf_counter() - start_time
    our_rate = num_vectors / add_time

    # Known competitor rates (from various benchmarks)
    competitors = {
        "ChromaDB": 4772,  # From our testing
        "Faiss (CPU)": 50000,  # Approximate for HNSW
        "Qdrant": 15000,  # Approximate
        "Weaviate": 8000,  # Approximate
        "Pinecone": "N/A (cloud only)",
        "OmenDB (DiskANN)": our_rate,
    }

    print(f"\nInsert Performance @ {dimension}D, {num_vectors} vectors:")
    print("-" * 50)
    for name, rate in competitors.items():
        if isinstance(rate, (int, float)):
            print(f"{name:<20}: {rate:>10.0f} vec/s")
        else:
            print(f"{name:<20}: {rate:>10}")

    # Calculate relative performance
    if "ChromaDB" in competitors and isinstance(competitors["ChromaDB"], (int, float)):
        vs_chroma = our_rate / competitors["ChromaDB"]
        print(f"\nOmenDB is {vs_chroma:.1f}x faster than ChromaDB")


def main():
    """Run all benchmarks."""

    print("=" * 60)
    print("OmenDB DiskANN Performance Benchmark Suite")
    print("=" * 60)

    # Check if we're using the right version
    print(f"\nUsing OmenDB from: {omendb.__file__}")

    # Run benchmarks
    try:
        # 1. Incremental vs Batch
        inc_vs_batch = benchmark_incremental_vs_batch()

        # 2. Scaling analysis
        scaling_results = benchmark_scaling()

        # 3. Competitor comparison
        compare_with_competitors()

        # Save results
        all_results = {
            "incremental_vs_batch": inc_vs_batch,
            "scaling": scaling_results,
            "timestamp": time.strftime("%Y-%m-%d %H:%M:%S"),
        }

        with open("diskann_benchmark_results.json", "w") as f:
            json.dump(all_results, f, indent=2)

        print(f"\n{'=' * 60}")
        print("Benchmark Complete!")
        print(f"Results saved to diskann_benchmark_results.json")
        print(f"{'=' * 60}")

    except Exception as e:
        print(f"\n❌ Benchmark failed: {e}")
        import traceback

        traceback.print_exc()
        return 1

    return 0


if __name__ == "__main__":
    sys.exit(main())
