#!/usr/bin/env python3
"""
Compare performance between baseline HNSW and batch-optimized HNSW.
"""

import sys
import os
import time

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))

import omendb
import numpy as np


def test_construction_performance(algorithm_name, num_vectors=5000):
    """Test construction performance for a specific algorithm."""
    print(f"📊 Testing {algorithm_name} Construction Performance")
    print("=" * 50)

    db = omendb.DB()
    dimensions = 128

    vectors = []
    batch_size = 100

    construction_times = []
    hnsw_construction_times = []

    start_time = time.time()

    for batch_idx in range(num_vectors // batch_size):
        batch_vectors = []
        batch_ids = []

        for i in range(batch_size):
            vector_idx = batch_idx * batch_size + i
            vector = [float(np.random.randn()) for _ in range(dimensions)]
            vectors.append(vector)
            batch_vectors.append(vector)
            batch_ids.append(f"batch_{vector_idx}")

        batch_start = time.time()
        batch_data = list(zip(batch_ids, batch_vectors))
        db.add_batch(batch_data)
        batch_time = time.time() - batch_start

        total_vectors = (batch_idx + 1) * batch_size

        # Check if we switched to HNSW
        stats = db.stats()
        if stats.get("algorithm") == "hnsw" and len(hnsw_construction_times) == 0:
            print(f"   Switched to HNSW at {total_vectors} vectors")
            hnsw_start_time = time.time()

        if stats.get("algorithm") == "hnsw":
            hnsw_construction_times.append(batch_time)
        else:
            construction_times.append(batch_time)

        if (batch_idx + 1) % 10 == 0:
            elapsed = time.time() - start_time
            speed = total_vectors / elapsed if elapsed > 0 else 0
            print(
                f"   Added {total_vectors} vectors in {elapsed:.2f}s ({speed:.0f} vec/s)"
            )

    total_time = time.time() - start_time
    total_speed = num_vectors / total_time if total_time > 0 else 0

    # Calculate HNSW-specific performance
    hnsw_vectors = len(hnsw_construction_times) * batch_size
    hnsw_time = sum(hnsw_construction_times)
    hnsw_speed = hnsw_vectors / hnsw_time if hnsw_time > 0 else 0

    print(f"\n📈 {algorithm_name} Results:")
    print(f"   Total construction: {total_speed:.0f} vec/s")
    print(f"   HNSW construction: {hnsw_speed:.0f} vec/s ({hnsw_vectors} vectors)")
    print(f"   Algorithm: {db.stats().get('algorithm')}")

    return {
        "algorithm": algorithm_name,
        "total_speed": total_speed,
        "hnsw_speed": hnsw_speed,
        "hnsw_vectors": hnsw_vectors,
        "total_time": total_time,
    }


def test_query_performance(algorithm_name, num_queries=50):
    """Test query performance."""
    print(f"\n🔍 Testing {algorithm_name} Query Performance")
    print("=" * 40)

    db = omendb.DB()
    dimensions = 128

    # Add vectors to trigger HNSW
    vectors = []
    for i in range(5000):
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        vectors.append(vector)

    ids = [f"vec_{i}" for i in range(5000)]
    batch_data = list(zip(ids, vectors))
    db.add_batch(batch_data)

    # Test query performance
    query_times = []

    for i in range(num_queries):
        query = [float(np.random.randn()) for _ in range(dimensions)]

        start_time = time.time()
        results = db.search(query, limit=10)
        query_time = time.time() - start_time

        query_times.append(query_time)

    avg_query_time = sum(query_times) / len(query_times)
    min_query_time = min(query_times)
    max_query_time = max(query_times)

    print(f"   Average query time: {avg_query_time * 1000:.2f}ms")
    print(f"   Min query time: {min_query_time * 1000:.2f}ms")
    print(f"   Max query time: {max_query_time * 1000:.2f}ms")

    return {
        "algorithm": algorithm_name,
        "avg_query_time": avg_query_time,
        "min_query_time": min_query_time,
        "max_query_time": max_query_time,
    }


if __name__ == "__main__":
    print("🎯 Batch Optimization Performance Comparison")
    print("=" * 55)

    # Test current batch-optimized implementation
    construction_results = test_construction_performance("Batch Optimized HNSW")
    query_results = test_query_performance("Batch Optimized HNSW")

    # Performance comparison with Faiss
    faiss_target = 28000
    our_speed = construction_results["hnsw_speed"]
    speed_ratio = faiss_target / our_speed if our_speed > 0 else float("inf")

    print(f"\n⚡ Performance Summary:")
    print(f"   Our HNSW speed: {our_speed:.0f} vec/s")
    print(f"   Faiss target: {faiss_target} vec/s")
    print(f"   Speed gap: {speed_ratio:.1f}x slower")
    print(f"   Query performance: {query_results['avg_query_time'] * 1000:.2f}ms")

    # Theoretical analysis
    print(f"\n🧮 Batch Optimization Impact:")
    baseline_estimate = 736  # From previous test
    improvement = (
        (our_speed / baseline_estimate - 1) * 100 if baseline_estimate > 0 else 0
    )
    print(f"   Estimated improvement: +{improvement:.1f}% vs baseline")
    print(
        f"   Batch processing active: {'Yes' if our_speed > baseline_estimate else 'No'}"
    )

    if speed_ratio < 10:
        print("✅ Excellent progress - within 10x of Faiss!")
    elif speed_ratio < 30:
        print("🔄 Good progress - within 30x of Faiss")
    else:
        print("❌ Still significant gap, but batch optimization helps")
