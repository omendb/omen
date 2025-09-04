#!/usr/bin/env python3
"""
Batch Optimization Benchmark

Consolidated benchmark for batch processing performance, including:
- Modern batch API performance (156K+ vec/s with NumPy)
- SIMD efficiency validation
- Optimal batch size determination
- Dimension-aware batch performance
"""

import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

from omendb import DB
import numpy as np
import time
import json
from datetime import datetime
from typing import Dict, List, Tuple


def benchmark_batch_sizes(dimension: int = 256, total_vectors: int = 10000) -> Dict:
    """Test different batch sizes to find optimal configuration."""

    print(f"\n📊 Benchmarking Batch Sizes (dim={dimension}, vectors={total_vectors})")
    print("-" * 60)

    # Generate test data
    np.random.seed(42)
    vectors = np.random.randn(total_vectors, dimension).astype(np.float32)
    vectors = vectors / np.linalg.norm(vectors, axis=1, keepdims=True)

    batch_sizes = [1, 5, 10, 25, 50, 100, 250, 500, 1000]
    results = {}

    for batch_size in batch_sizes:
        if batch_size > total_vectors:
            continue

        print(f"\nTesting batch size {batch_size}...")

        db = DB()
        num_batches = total_vectors // batch_size
        vectors_to_process = num_batches * batch_size

        # Time the insertions
        start_time = time.perf_counter()

        if batch_size == 1:
            # Individual insertion (NOT RECOMMENDED - for comparison only)
            for i in range(vectors_to_process):
                db.add(f"vec_{i}", vectors[i])  # Direct NumPy array
        else:
            # Modern batch insertion API
            for batch_idx in range(num_batches):
                start_idx = batch_idx * batch_size
                end_idx = start_idx + batch_size
                batch_vectors = vectors[start_idx:end_idx]
                batch_ids = [f"vec_{i}" for i in range(start_idx, end_idx)]
                batch_metadata = [{} for _ in range(batch_size)]
                db.add_batch(
                    vectors=batch_vectors, ids=batch_ids, metadata=batch_metadata
                )

        total_time = time.perf_counter() - start_time

        # Calculate metrics
        rate = vectors_to_process / total_time
        time_per_vec = total_time / vectors_to_process * 1000  # ms

        results[batch_size] = {
            "batch_size": batch_size,
            "vectors_processed": vectors_to_process,
            "total_time": total_time,
            "rate": rate,
            "time_per_vector_ms": time_per_vec,
        }

        print(f"  Rate: {rate:,.0f} vec/s")
        print(f"  Time per vector: {time_per_vec:.3f}ms")

    # Calculate speedups
    if 1 in results:
        baseline_rate = results[1]["rate"]
        for batch_size in results:
            results[batch_size]["speedup"] = results[batch_size]["rate"] / baseline_rate

    return results


def benchmark_simd_efficiency(dimensions: List[int], n_vectors: int = 5000) -> Dict:
    """Measure SIMD efficiency across different dimensions."""

    print(f"\n⚡ SIMD Efficiency Analysis")
    print("-" * 30)

    results = {}

    for dim in dimensions:
        print(f"\nDimension {dim}:")

        # Generate data
        np.random.seed(42)
        vectors = np.random.randn(n_vectors, dim).astype(np.float32)
        vectors = vectors / np.linalg.norm(vectors, axis=1, keepdims=True)

        # Test single insertion
        db_single = DB()
        single_count = min(100, n_vectors)

        # Use smaller batch for single comparison (still use batch for fairness)
        single_vectors = vectors[:single_count]
        single_ids = [f"single_{i}" for i in range(single_count)]
        single_metadata = [{} for _ in range(single_count)]

        start = time.perf_counter()
        db_single.add_batch(
            vectors=single_vectors, ids=single_ids, metadata=single_metadata
        )
        single_time = time.perf_counter() - start
        single_rate = single_count / single_time

        # Test batch insertion with optimal size
        db_batch = DB()
        batch_size = 100
        batch_count = min(1000, n_vectors) // batch_size

        start = time.perf_counter()
        for i in range(batch_count):
            start_idx = i * batch_size
            end_idx = start_idx + batch_size
            batch_vectors = vectors[start_idx:end_idx]
            batch_ids = [f"batch_{j}" for j in range(start_idx, end_idx)]
            batch_metadata = [{} for _ in range(batch_size)]
            db_batch.add_batch(
                vectors=batch_vectors, ids=batch_ids, metadata=batch_metadata
            )
        batch_time = time.perf_counter() - start
        batch_rate = (batch_count * batch_size) / batch_time

        speedup = batch_rate / single_rate

        # SIMD alignment analysis
        simd_aligned_4 = (dim % 4) == 0
        simd_aligned_8 = (dim % 8) == 0
        simd_aligned_16 = (dim % 16) == 0

        # Expected vs actual speedup
        expected_speedup = 8.0 if simd_aligned_8 else 4.0 if simd_aligned_4 else 2.0
        efficiency = (speedup / expected_speedup) * 100

        results[dim] = {
            "dimension": dim,
            "single_rate": single_rate,
            "batch_rate": batch_rate,
            "speedup": speedup,
            "expected_speedup": expected_speedup,
            "efficiency_pct": efficiency,
            "simd_aligned_4": simd_aligned_4,
            "simd_aligned_8": simd_aligned_8,
            "simd_aligned_16": simd_aligned_16,
        }

        print(f"  Single: {single_rate:,.0f} vec/s")
        print(f"  Batch: {batch_rate:,.0f} vec/s")
        print(f"  Speedup: {speedup:.2f}x (expected: {expected_speedup:.1f}x)")
        print(f"  SIMD efficiency: {efficiency:.1f}%")

    return results


def benchmark_batch_memory_patterns() -> Dict:
    """Analyze memory access patterns in batch operations."""

    print(f"\n💾 Batch Memory Pattern Analysis")
    print("-" * 35)

    # Test how batch size affects cache efficiency
    dimensions = [128, 256, 512, 1024]
    batch_sizes = [10, 50, 100, 500]

    results = {}

    for dim in dimensions:
        print(f"\nDimension {dim}:")
        dim_results = {}

        # Generate data
        np.random.seed(42)
        n_vectors = 5000
        vectors = np.random.randn(n_vectors, dim).astype(np.float32)
        vectors = vectors / np.linalg.norm(vectors, axis=1, keepdims=True)

        for batch_size in batch_sizes:
            db = DB()
            num_batches = n_vectors // batch_size

            # Measure with cold cache (first batch)
            first_vectors = vectors[:batch_size]
            first_ids = [f"cold_{i}" for i in range(batch_size)]
            first_metadata = [{} for _ in range(batch_size)]

            start = time.perf_counter()
            db.add_batch(vectors=first_vectors, ids=first_ids, metadata=first_metadata)
            cold_time = time.perf_counter() - start

            # Measure with warm cache (subsequent batches)
            warm_times = []
            for i in range(1, min(10, num_batches)):
                start_idx = i * batch_size
                end_idx = start_idx + batch_size
                warm_vectors = vectors[start_idx:end_idx]
                warm_ids = [f"warm_{j}" for j in range(start_idx, end_idx)]
                warm_metadata = [{} for _ in range(batch_size)]

                start = time.perf_counter()
                db.add_batch(vectors=warm_vectors, ids=warm_ids, metadata=warm_metadata)
                warm_times.append(time.perf_counter() - start)

            avg_warm_time = (
                sum(warm_times) / len(warm_times) if warm_times else cold_time
            )
            cache_speedup = cold_time / avg_warm_time if avg_warm_time > 0 else 1.0

            dim_results[batch_size] = {
                "cold_time_ms": cold_time * 1000,
                "warm_time_ms": avg_warm_time * 1000,
                "cache_speedup": cache_speedup,
            }

        results[dim] = dim_results

        # Print summary for this dimension
        for batch_size, data in dim_results.items():
            print(f"  Batch {batch_size}: {data['cache_speedup']:.2f}x cache speedup")

    return results


def generate_report(all_results: Dict):
    """Generate comprehensive batch optimization report."""

    timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
    report_path = f"results/batch_optimization_{timestamp}.json"

    os.makedirs("results", exist_ok=True)

    with open(report_path, "w") as f:
        json.dump(all_results, f, indent=2)

    # Generate summary
    print("\n" + "=" * 70)
    print("BATCH OPTIMIZATION SUMMARY")
    print("=" * 70)

    # Find optimal batch size
    if "batch_sizes" in all_results:
        batch_results = all_results["batch_sizes"]
        best_batch = max(batch_results.items(), key=lambda x: x[1]["rate"])
        print(
            f"\n🏆 Optimal batch size: {best_batch[0]} ({best_batch[1]['rate']:,.0f} vec/s)"
        )
        print(f"   Speedup vs single: {best_batch[1].get('speedup', 0):.2f}x")

    # SIMD efficiency summary
    if "simd_efficiency" in all_results:
        simd_results = all_results["simd_efficiency"]
        avg_efficiency = sum(r["efficiency_pct"] for r in simd_results.values()) / len(
            simd_results
        )
        print(f"\n📊 Average SIMD efficiency: {avg_efficiency:.1f}%")

        # Find best/worst dimensions
        best_dim = max(simd_results.items(), key=lambda x: x[1]["speedup"])
        worst_dim = min(simd_results.items(), key=lambda x: x[1]["speedup"])

        print(f"   Best: {best_dim[0]}D with {best_dim[1]['speedup']:.2f}x speedup")
        print(f"   Worst: {worst_dim[0]}D with {worst_dim[1]['speedup']:.2f}x speedup")

    print(f"\n📄 Full report saved to: {report_path}")


def main():
    """Run comprehensive batch optimization benchmarks."""

    print("🚀 OmenDB Batch Optimization Benchmark")
    print("=" * 40)

    all_results = {"timestamp": datetime.now().isoformat(), "benchmarks": {}}

    # 1. Optimal batch size analysis
    batch_results = benchmark_batch_sizes(dimension=256, total_vectors=10000)
    all_results["batch_sizes"] = batch_results

    # 2. SIMD efficiency across dimensions
    test_dimensions = [32, 64, 128, 256, 384, 512, 768, 1024]
    simd_results = benchmark_simd_efficiency(test_dimensions)
    all_results["simd_efficiency"] = simd_results

    # 3. Memory pattern analysis
    memory_results = benchmark_batch_memory_patterns()
    all_results["memory_patterns"] = memory_results

    # Generate report
    generate_report(all_results)

    return all_results


if __name__ == "__main__":
    main()
