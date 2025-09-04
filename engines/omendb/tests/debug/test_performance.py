#!/usr/bin/env python3
"""Comprehensive performance test for OmenDB with DiskANN."""

import time
import numpy as np
import sys
import os

# Add the local development path
parent_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
python_dir = os.path.join(parent_dir, "python")
sys.path.insert(0, python_dir)

import omendb


def generate_random_vectors(num_vectors, dimension):
    """Generate random unit vectors."""
    vectors = np.random.randn(num_vectors, dimension).astype(np.float32)
    # Normalize to unit vectors
    norms = np.linalg.norm(vectors, axis=1, keepdims=True)
    vectors = vectors / norms
    return vectors


def benchmark_batch_insert(num_vectors, dimension=128, batch_size=1000):
    """Benchmark batch insertion performance."""
    print(f"\n📊 Benchmarking {num_vectors} vectors (dim={dimension})")
    print("=" * 60)

    # Generate test data
    print(f"Generating {num_vectors} random vectors...")
    vectors = generate_random_vectors(num_vectors, dimension)
    ids = [f"vec_{i}" for i in range(num_vectors)]

    # Create fresh database
    db = omendb.DB(algorithm="diskann")
    db.clear()

    # Batch insertion
    print(f"Inserting in batches of {batch_size}...")
    start_time = time.time()

    for i in range(0, num_vectors, batch_size):
        batch_end = min(i + batch_size, num_vectors)
        batch_vectors = vectors[i:batch_end]
        batch_ids = ids[i:batch_end]

        results = db.add_batch(batch_vectors, batch_ids)
        if len(results) != len(batch_ids):
            print(f"⚠️ Only added {len(results)}/{len(batch_ids)} vectors in batch")

    insert_time = time.time() - start_time
    insert_rate = num_vectors / insert_time

    print(f"✅ Insertion: {insert_rate:,.0f} vec/s ({insert_time:.2f}s total)")

    # Query performance
    print(f"\nTesting query performance...")
    query_times = []
    for _ in range(100):
        query_idx = np.random.randint(0, num_vectors)
        query_vector = vectors[query_idx]

        start_time = time.time()
        results = db.search(query_vector, limit=10)
        query_time = time.time() - start_time
        query_times.append(query_time)

    avg_query_time = np.mean(query_times) * 1000  # Convert to ms
    p99_query_time = np.percentile(query_times, 99) * 1000

    print(f"✅ Query: avg={avg_query_time:.2f}ms, p99={p99_query_time:.2f}ms")

    # Verify search accuracy
    query_vector = vectors[0]
    results = db.search(query_vector, limit=5)
    if results and results[0].id == ids[0]:
        print(f"✅ Search accuracy: correct (best match score={results[0].score:.4f})")
    else:
        print(f"❌ Search accuracy: incorrect")

    # Memory stats
    stats = db.info()
    print(f"\n📈 Database stats:")
    print(f"  Vectors: {stats.get('vector_count', 0)}")
    print(f"  Algorithm: {stats.get('algorithm', 'unknown')}")

    return insert_rate, avg_query_time


def benchmark_individual_insert(num_vectors=1000, dimension=128):
    """Benchmark individual insertion performance."""
    print(f"\n📊 Benchmarking individual inserts ({num_vectors} vectors)")
    print("=" * 60)

    vectors = generate_random_vectors(num_vectors, dimension)

    db = omendb.DB(algorithm="diskann")
    db.clear()

    start_time = time.time()
    for i in range(num_vectors):
        db.add(f"vec_{i}", vectors[i])

    insert_time = time.time() - start_time
    insert_rate = num_vectors / insert_time

    print(f"✅ Individual insert: {insert_rate:,.0f} ops/s")
    return insert_rate


def main():
    """Run comprehensive performance benchmarks."""
    print("🚀 OmenDB Performance Benchmark Suite")
    print("=" * 60)

    # Test at different scales
    test_sizes = [1000, 5000, 10000, 15000, 25000]

    results = {}
    for size in test_sizes:
        insert_rate, query_time = benchmark_batch_insert(size)
        results[size] = {"insert_rate": insert_rate, "query_time": query_time}

    # Test individual operations
    individual_rate = benchmark_individual_insert(1000)

    # Summary
    print("\n" + "=" * 60)
    print("📊 PERFORMANCE SUMMARY")
    print("=" * 60)

    print("\nBatch Insertion Performance:")
    for size, metrics in results.items():
        print(f"  {size:6d} vectors: {metrics['insert_rate']:8,.0f} vec/s")

    print("\nQuery Latency (avg):")
    for size, metrics in results.items():
        print(f"  {size:6d} vectors: {metrics['query_time']:6.2f}ms")

    print(f"\nIndividual Operations: {individual_rate:,.0f} ops/s")

    # Performance targets from docs
    print("\n" + "=" * 60)
    print("📋 PERFORMANCE VS TARGETS")
    print("=" * 60)

    target_10k = 48000  # From CURRENT_SPRINT.md
    actual_10k = results.get(10000, {}).get("insert_rate", 0)

    if actual_10k > 0:
        ratio = actual_10k / target_10k
        if ratio >= 0.9:
            print(
                f"✅ 10K performance: {actual_10k:,.0f} vec/s ({ratio * 100:.0f}% of target)"
            )
        else:
            print(
                f"❌ 10K performance: {actual_10k:,.0f} vec/s ({ratio * 100:.0f}% of target)"
            )

    # ChromaDB comparison (they get ~4.7K vec/s)
    chromadb_rate = 4700
    if actual_10k > chromadb_rate:
        speedup = actual_10k / chromadb_rate
        print(f"✅ vs ChromaDB: {speedup:.1f}x faster")
    else:
        print(f"❌ vs ChromaDB: slower than expected")


if __name__ == "__main__":
    main()
