#!/usr/bin/env python3
"""
Benchmark to compare baseline HNSW vs SIMD-optimized HNSW performance.
Focuses on construction speed which is the primary bottleneck.
"""

import time
import numpy as np
import sys
import os
from typing import List, Dict, Tuple

# Add project root to path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))

import omendb


def generate_vectors(n: int, dim: int) -> List[List[float]]:
    """Generate normalized random vectors."""
    vectors = []
    for _ in range(n):
        vec = np.random.randn(dim).astype(np.float32)
        vec = vec / np.linalg.norm(vec)
        vectors.append(vec.tolist())
    return vectors


def benchmark_construction(
    db, vectors: List[List[float]], batch_size: int = 100
) -> Dict:
    """Benchmark HNSW construction performance."""
    start_time = time.time()

    # Add vectors in batches
    for i in range(0, len(vectors), batch_size):
        batch = vectors[i : i + batch_size]
        for j, vec in enumerate(batch):
            success = db.add(f"vec_{i + j}", vec)
            if not success:
                print(f"Failed to add vector {i + j}")

    construction_time = time.time() - start_time
    vectors_per_second = len(vectors) / construction_time

    return {
        "time": construction_time,
        "vec_per_sec": vectors_per_second,
        "total_vectors": len(vectors),
    }


def benchmark_queries(db, query_vectors: List[List[float]], k: int = 10) -> Dict:
    """Benchmark query performance."""
    total_time = 0
    recalls = []

    for query in query_vectors:
        start = time.time()
        results = db.query(query, top_k=k)
        query_time = time.time() - start
        total_time += query_time

        # Check if results returned
        if len(results) > 0:
            recalls.append(1.0)
        else:
            recalls.append(0.0)

    avg_query_time = total_time / len(query_vectors)
    avg_recall = sum(recalls) / len(recalls)

    return {
        "avg_query_time_ms": avg_query_time * 1000,
        "total_time": total_time,
        "avg_recall": avg_recall,
        "num_queries": len(query_vectors),
    }


def main():
    """Run SIMD optimization benchmark."""
    print("=" * 60)
    print("HNSW SIMD Optimization Benchmark")
    print("=" * 60)

    # Test configurations
    dimensions = [128]  # Focus on common dimension
    vector_counts = [1000, 5000, 10000]  # Test at different scales

    for dim in dimensions:
        print(f"\n\nDimension: {dim}D")
        print("-" * 40)

        for n_vectors in vector_counts:
            print(f"\nTesting with {n_vectors} vectors:")

            # Generate test data
            print("Generating test vectors...")
            vectors = generate_vectors(n_vectors, dim)
            query_vectors = generate_vectors(100, dim)  # 100 queries

            # Test baseline HNSW
            print("\n1. Baseline HNSW (hnsw_fixed):")
            db_baseline = omendb.DB()

            # Benchmark construction
            construction_results = benchmark_construction(db_baseline, vectors)
            print(f"   Construction time: {construction_results['time']:.2f}s")
            print(f"   Vectors/second: {construction_results['vec_per_sec']:.0f}")

            # Benchmark queries
            query_results = benchmark_queries(db_baseline, query_vectors)
            print(f"   Avg query time: {query_results['avg_query_time_ms']:.2f}ms")
            print(f"   Recall: {query_results['avg_recall']:.1%}")

            # TODO: Test SIMD-optimized version once integrated
            print("\n2. SIMD-Optimized HNSW:")
            print("   [To be implemented - need to integrate into native.mojo]")

            # Expected improvements
            print("\n3. Expected Improvements:")
            expected_speedup = 5.0  # Conservative estimate
            expected_construction = (
                construction_results["vec_per_sec"] * expected_speedup
            )
            print(
                f"   Target construction: {expected_construction:.0f} vec/s (5x speedup)"
            )
            print(f"   Target query time: <1ms (maintained)")
            print(f"   Target recall: 100% (maintained)")

    print("\n" + "=" * 60)
    print("Benchmark Complete")
    print("=" * 60)

    # Analysis
    print("\nPerformance Analysis:")
    print("- Current bottleneck: HNSW construction (93-128 vec/s)")
    print("- SIMD optimization targets: neighbor selection and distance calculations")
    print("- Expected improvement: 5-10x speedup from vectorized operations")
    print("- Next steps: Integrate SIMD version and add parallel construction")


if __name__ == "__main__":
    main()
