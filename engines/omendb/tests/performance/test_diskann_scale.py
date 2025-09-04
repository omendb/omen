#!/usr/bin/env python3
"""
DiskANN Performance Testing at Scale

Tests:
1. Insert performance (1K to 1M vectors)
2. Query latency and recall
3. Memory usage
4. Comparison with HNSW
5. Degradation over time (or lack thereof)
"""

import time
import numpy as np
import psutil
import sys
import os

sys.path.append(os.path.join(os.path.dirname(__file__), "..", "..", "python"))

import omendb


def measure_memory():
    """Get current process memory usage in MB."""
    process = psutil.Process()
    return process.memory_info().rss / 1024 / 1024


def generate_dataset(n, dim=128, seed=42):
    """Generate reproducible random dataset."""
    np.random.seed(seed)
    vectors = np.random.randn(n, dim).astype(np.float32)
    # Normalize for cosine similarity
    norms = np.linalg.norm(vectors, axis=1, keepdims=True)
    vectors = vectors / norms
    return vectors


def test_diskann_performance(n_vectors, dim=128):
    """Test DiskANN performance at given scale."""
    print(f"\n{'=' * 60}")
    print(f"Testing DiskANN with {n_vectors:,} vectors ({dim}D)")
    print("=" * 60)

    # Generate data
    print("Generating dataset...")
    vectors = generate_dataset(n_vectors, dim)
    queries = vectors[:100]  # Use first 100 as queries

    # Test DiskANN
    print("\n🚀 Testing DiskANN (no rebuilds)")
    print("-" * 40)

    db = omendb.DB(algorithm="diskann", buffer_size=1000)

    # Measure insert performance
    start_mem = measure_memory()
    start_time = time.perf_counter()

    # Insert in batches to simulate real usage
    batch_size = 1000
    for i in range(0, n_vectors, batch_size):
        batch = vectors[i : i + batch_size]
        ids = [f"vec_{j}" for j in range(i, min(i + batch_size, n_vectors))]

        if len(batch) == len(ids):
            db.add_batch(batch, ids)

        if (i + batch_size) % 10000 == 0:
            elapsed = time.perf_counter() - start_time
            rate = (i + batch_size) / elapsed
            print(f"  Inserted {i + batch_size:,} vectors @ {rate:.0f} vec/s")

    insert_time = time.perf_counter() - start_time
    end_mem = measure_memory()
    memory_used = end_mem - start_mem

    print(f"\n📊 Insert Performance:")
    print(f"  Time: {insert_time:.2f}s")
    print(f"  Rate: {n_vectors / insert_time:.0f} vec/s")
    print(f"  Memory: {memory_used:.1f} MB")
    print(f"  Memory/vector: {memory_used * 1024 / n_vectors:.1f} KB")

    # Test query performance
    print(f"\n🔍 Query Performance (100 queries):")

    latencies = []
    for q in queries:
        start = time.perf_counter()
        results = db.search(q, limit=10)
        latencies.append((time.perf_counter() - start) * 1000)

    latencies = np.array(latencies)
    print(f"  Mean: {np.mean(latencies):.2f}ms")
    print(f"  P50: {np.percentile(latencies, 50):.2f}ms")
    print(f"  P95: {np.percentile(latencies, 95):.2f}ms")
    print(f"  P99: {np.percentile(latencies, 99):.2f}ms")

    # Test recall (ground truth is brute force)
    print(f"\n📈 Recall@10 Test:")

    # Create ground truth with brute force
    db_exact = omendb.DB(algorithm="flat")
    for i in range(min(10000, n_vectors)):
        db_exact.add(f"vec_{i}", vectors[i])

    # Compare results
    recalls = []
    for q in queries[:10]:  # Test 10 queries
        exact_results = db_exact.search(q, limit=10)
        approx_results = db.search(q, limit=10)

        exact_ids = set(r[0] for r in exact_results)
        approx_ids = set(r[0] for r in approx_results)

        recall = len(exact_ids & approx_ids) / len(exact_ids) if exact_ids else 0
        recalls.append(recall)

    mean_recall = np.mean(recalls)
    print(f"  Recall@10: {mean_recall:.1%}")

    return {
        "n_vectors": n_vectors,
        "insert_time": insert_time,
        "insert_rate": n_vectors / insert_time,
        "memory_mb": memory_used,
        "query_p50": np.percentile(latencies, 50),
        "query_p99": np.percentile(latencies, 99),
        "recall": mean_recall,
    }


def test_degradation():
    """Test if DiskANN degrades over time with updates."""
    print("\n" + "=" * 60)
    print("🔄 Testing Degradation with Updates")
    print("=" * 60)

    db = omendb.DB(algorithm="diskann")

    # Initial batch
    print("\nInitial load: 10,000 vectors")
    vectors = generate_dataset(10000)
    for i in range(10000):
        db.add(f"vec_{i}", vectors[i])

    # Measure initial performance
    queries = vectors[:100]
    start = time.perf_counter()
    for q in queries:
        _ = db.search(q, limit=10)
    initial_time = time.perf_counter() - start
    print(f"Initial query time: {initial_time * 10:.2f}ms per query")

    # Add more vectors incrementally
    for round in range(5):
        print(f"\nRound {round + 1}: Adding 10,000 more vectors")
        new_vectors = generate_dataset(10000, seed=100 + round)

        for i in range(10000):
            db.add(f"new_{round}_{i}", new_vectors[i])

        # Measure performance after updates
        start = time.perf_counter()
        for q in queries:
            _ = db.search(q, limit=10)
        current_time = time.perf_counter() - start

        degradation = (current_time - initial_time) / initial_time * 100
        print(f"Query time: {current_time * 10:.2f}ms per query")
        print(f"Degradation: {degradation:+.1f}%")

        if abs(degradation) < 10:
            print("✅ No significant degradation!")
        else:
            print(f"⚠️ Performance changed by {degradation:.1f}%")


def compare_with_hnsw():
    """Compare DiskANN with HNSW."""
    print("\n" + "=" * 60)
    print("⚔️ DiskANN vs HNSW Comparison")
    print("=" * 60)

    test_sizes = [1000, 5000, 10000, 50000]

    for n in test_sizes:
        print(f"\n📊 Testing {n:,} vectors")
        print("-" * 40)

        vectors = generate_dataset(n)
        queries = vectors[:100]

        # Test HNSW
        print("HNSW:")
        db_hnsw = omendb.DB(algorithm="hnsw", buffer_size=1000)

        start = time.perf_counter()
        for i in range(n):
            db_hnsw.add(f"vec_{i}", vectors[i])
        hnsw_time = time.perf_counter() - start

        hnsw_rate = n / hnsw_time
        print(f"  Insert: {hnsw_rate:.0f} vec/s")

        # Test DiskANN
        print("DiskANN:")
        db_diskann = omendb.DB(algorithm="diskann", buffer_size=1000)

        start = time.perf_counter()
        for i in range(n):
            db_diskann.add(f"vec_{i}", vectors[i])
        diskann_time = time.perf_counter() - start

        diskann_rate = n / diskann_time
        print(f"  Insert: {diskann_rate:.0f} vec/s")

        # Analysis
        speedup = diskann_rate / hnsw_rate
        if speedup > 1:
            print(f"\n✅ DiskANN is {speedup:.1f}x faster!")
        else:
            print(f"\n⚠️ HNSW is {1 / speedup:.1f}x faster")


def main():
    print("\n" + "=" * 70)
    print("🚀 DiskANN PERFORMANCE TEST SUITE")
    print("=" * 70)

    # Test at different scales
    scales = [1000, 5000, 10000, 50000, 100000]
    results = []

    for n in scales:
        result = test_diskann_performance(n)
        results.append(result)

    # Summary table
    print("\n" + "=" * 70)
    print("📊 PERFORMANCE SUMMARY")
    print("=" * 70)
    print(
        f"{'Vectors':<12} {'Insert Rate':<15} {'Memory':<12} {'Query P50':<12} {'Query P99':<12} {'Recall'}"
    )
    print("-" * 70)

    for r in results:
        print(
            f"{r['n_vectors']:<12,} {r['insert_rate']:<15.0f} {r['memory_mb']:<12.1f} {r['query_p50']:<12.2f} {r['query_p99']:<12.2f} {r['recall']:.1%}"
        )

    # Test degradation
    test_degradation()

    # Compare with HNSW
    compare_with_hnsw()

    print("\n" + "=" * 70)
    print("✅ TESTING COMPLETE")
    print("=" * 70)


if __name__ == "__main__":
    main()
