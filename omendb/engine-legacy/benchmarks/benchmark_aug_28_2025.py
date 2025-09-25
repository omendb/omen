#!/usr/bin/env python3
"""Comprehensive benchmark suite - August 28, 2025

Records actual performance metrics after optimizations.
"""

import sys

sys.path.insert(0, "python")

import numpy as np
import time
import psutil
import os
import json
from datetime import datetime


def benchmark_individual_adds():
    """Benchmark individual add operations."""
    import omendb

    dimensions = [64, 128, 256, 512, 768]
    n_vectors = 1000

    results = {}

    for dim in dimensions:
        db = omendb.DB()
        db.clear()

        vectors = np.random.randn(n_vectors, dim).astype(np.float32)

        start = time.perf_counter()
        for i in range(n_vectors):
            db.add(f"id_{i}", vectors[i])
        elapsed = time.perf_counter() - start

        vec_per_sec = n_vectors / elapsed
        results[f"dim_{dim}"] = {
            "vec_per_sec": vec_per_sec,
            "ms_per_vec": (elapsed / n_vectors) * 1000,
        }

    return results


def benchmark_batch_operations():
    """Benchmark batch add operations."""
    import omendb

    dimensions = [128, 256, 512]
    batch_sizes = [100, 500, 1000, 2000, 5000, 10000]
    n_vectors = 10000

    results = {}

    for dim in dimensions:
        dim_results = {}
        vectors = np.random.randn(n_vectors, dim).astype(np.float32)
        ids = [f"id_{i}" for i in range(n_vectors)]

        for batch_size in batch_sizes:
            db = omendb.DB(buffer_size=batch_size)
            db.clear()

            start = time.perf_counter()
            for i in range(0, n_vectors, batch_size):
                end_idx = min(i + batch_size, n_vectors)
                db.add_batch(vectors[i:end_idx], ids[i:end_idx])
            elapsed = time.perf_counter() - start

            vec_per_sec = n_vectors / elapsed
            dim_results[f"batch_{batch_size}"] = vec_per_sec

        results[f"dim_{dim}"] = dim_results

    return results


def benchmark_search_performance():
    """Benchmark search performance at different scales."""
    import omendb

    dataset_sizes = [1000, 5000, 10000, 20000, 50000]
    dimension = 128
    n_queries = 100

    results = {}

    for n_vectors in dataset_sizes:
        db = omendb.DB(buffer_size=10000)
        db.clear()

        # Build index
        vectors = np.random.randn(n_vectors, dimension).astype(np.float32)
        ids = [f"id_{i}" for i in range(n_vectors)]

        build_start = time.perf_counter()
        db.add_batch(vectors, ids)
        build_time = time.perf_counter() - build_start

        # Search performance
        queries = np.random.randn(n_queries, dimension).astype(np.float32)

        search_times = []
        for query in queries:
            start = time.perf_counter()
            results_list = db.search(query, 10)
            search_times.append(time.perf_counter() - start)

        results[f"size_{n_vectors}"] = {
            "build_time": build_time,
            "build_vec_per_sec": n_vectors / build_time,
            "avg_search_ms": np.mean(search_times) * 1000,
            "p50_search_ms": np.percentile(search_times, 50) * 1000,
            "p95_search_ms": np.percentile(search_times, 95) * 1000,
            "p99_search_ms": np.percentile(search_times, 99) * 1000,
        }

    return results


def benchmark_memory_efficiency():
    """Benchmark memory usage patterns."""
    import omendb

    dataset_sizes = [1000, 5000, 10000, 20000]
    dimension = 128

    process = psutil.Process(os.getpid())
    results = {}

    for n_vectors in dataset_sizes:
        db = omendb.DB(buffer_size=10000)
        db.clear()

        # Measure baseline
        mem_before = process.memory_info().rss / 1024 / 1024

        # Add vectors
        vectors = np.random.randn(n_vectors, dimension).astype(np.float32)
        ids = [f"id_{i}" for i in range(n_vectors)]
        db.add_batch(vectors, ids)

        # Measure after
        mem_after = process.memory_info().rss / 1024 / 1024
        mem_used = mem_after - mem_before

        # Theoretical memory
        theoretical = (n_vectors * dimension * 4) / (1024 * 1024)
        overhead = (mem_used / theoretical - 1) * 100 if theoretical > 0 else 0

        results[f"size_{n_vectors}"] = {
            "memory_used_mb": mem_used,
            "theoretical_mb": theoretical,
            "overhead_percent": overhead,
            "bytes_per_vector": (mem_used * 1024 * 1024) / n_vectors,
        }

    return results


def benchmark_quantization():
    """Benchmark quantization performance."""
    from omendb.quantization import QuantizedDB

    n_vectors = 5000
    dimension = 256

    quantization_types = ["none", "int8", "binary", "product"]
    results = {}

    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)
    ids = [f"id_{i}" for i in range(n_vectors)]

    for q_type in quantization_types:
        db = QuantizedDB(quantization=q_type, buffer_size=10000)

        # Build performance
        start = time.perf_counter()
        db.add_batch(vectors, ids)
        build_time = time.perf_counter() - start

        # Search performance
        search_times = []
        for _ in range(100):
            query = np.random.randn(dimension).astype(np.float32)
            start = time.perf_counter()
            results_list = db.search(query, 10)
            search_times.append(time.perf_counter() - start)

        # Memory stats
        stats = db.get_memory_usage()

        results[q_type] = {
            "build_vec_per_sec": n_vectors / build_time,
            "avg_search_ms": np.mean(search_times) * 1000,
            "compression_ratio": stats.get("compression_ratio", 1.0),
            "vector_count": stats.get("vector_count", n_vectors),
        }

        db.clear()

    return results


def profile_ffi_overhead():
    """Profile Python-Mojo FFI overhead."""
    import omendb
    import cProfile
    import pstats
    from io import StringIO

    db = omendb.DB()
    db.clear()

    dimension = 128
    n_operations = 100

    # Profile individual adds
    profiler = cProfile.Profile()
    vectors = np.random.randn(n_operations, dimension).astype(np.float32)

    profiler.enable()
    for i in range(n_operations):
        db.add(f"id_{i}", vectors[i])
    profiler.disable()

    # Get stats
    stream = StringIO()
    stats = pstats.Stats(profiler, stream=stream)
    stats.sort_stats("cumulative")
    stats.print_stats(10)

    # Extract key metrics
    total_time = stats.total_tt
    ffi_time_estimate = total_time * 0.79  # Based on previous analysis

    return {
        "total_time_ms": total_time * 1000,
        "estimated_ffi_overhead_ms": ffi_time_estimate * 1000,
        "overhead_per_call_ms": (ffi_time_estimate / n_operations) * 1000,
    }


def save_results(results):
    """Save benchmark results to file."""
    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    filename = f"benchmarks/results_{timestamp}.json"

    os.makedirs("benchmarks", exist_ok=True)

    with open(filename, "w") as f:
        json.dump(results, f, indent=2)

    return filename


def main():
    """Run comprehensive benchmark suite."""
    print("=" * 70)
    print("COMPREHENSIVE BENCHMARK SUITE")
    print(f"Date: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}")
    print("=" * 70)
    print()

    all_results = {}

    # Run benchmarks
    print("Running individual add benchmark...")
    all_results["individual_adds"] = benchmark_individual_adds()

    print("Running batch operations benchmark...")
    all_results["batch_operations"] = benchmark_batch_operations()

    print("Running search performance benchmark...")
    all_results["search_performance"] = benchmark_search_performance()

    print("Running memory efficiency benchmark...")
    all_results["memory_efficiency"] = benchmark_memory_efficiency()

    print("Running quantization benchmark...")
    all_results["quantization"] = benchmark_quantization()

    print("Profiling FFI overhead...")
    all_results["ffi_overhead"] = profile_ffi_overhead()

    # Save results
    filename = save_results(all_results)
    print(f"\nResults saved to: {filename}")

    # Print summary
    print("\n" + "=" * 70)
    print("BENCHMARK SUMMARY")
    print("=" * 70)

    # Individual adds
    ind_128 = all_results["individual_adds"]["dim_128"]
    print(f"\nIndividual Adds (128d): {ind_128['vec_per_sec']:.0f} vec/s")
    print(f"  FFI overhead: {ind_128['ms_per_vec']:.2f} ms/vec")

    # Batch operations
    batch_128 = all_results["batch_operations"]["dim_128"]
    best_batch = max(batch_128.items(), key=lambda x: x[1])
    print(f"\nBest Batch Performance (128d): {best_batch[1]:.0f} vec/s")
    print(f"  Optimal batch size: {best_batch[0]}")

    # Search performance
    search_10k = all_results["search_performance"]["size_10000"]
    print(f"\nSearch Performance (10K vectors):")
    print(f"  Average: {search_10k['avg_search_ms']:.2f} ms")
    print(f"  P95: {search_10k['p95_search_ms']:.2f} ms")
    print(f"  P99: {search_10k['p99_search_ms']:.2f} ms")

    # Memory efficiency
    mem_10k = all_results["memory_efficiency"]["size_10000"]
    print(f"\nMemory Efficiency (10K vectors):")
    print(f"  Used: {mem_10k['memory_used_mb']:.1f} MB")
    print(f"  Overhead: {mem_10k['overhead_percent']:.0f}%")

    # Quantization
    print(f"\nQuantization Performance:")
    for q_type, stats in all_results["quantization"].items():
        print(
            f"  {q_type}: {stats['build_vec_per_sec']:.0f} vec/s, "
            f"{stats['compression_ratio']:.1f}x compression"
        )

    # FFI overhead
    ffi = all_results["ffi_overhead"]
    print(f"\nFFI Overhead:")
    print(f"  Per call: {ffi['overhead_per_call_ms']:.3f} ms")

    return all_results


if __name__ == "__main__":
    results = main()
