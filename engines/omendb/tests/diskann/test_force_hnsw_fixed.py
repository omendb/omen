#!/usr/bin/env python3
"""Test HNSW performance without migration overhead by forcing it from start."""

import sys
import time
import numpy as np

sys.path.insert(0, "/Users/nick/github/omendb/omendb/python")

from omendb import DB


def test_algorithm_performance():
    """Compare performance with different algorithm strategies."""

    print("=" * 80)
    print("ALGORITHM PERFORMANCE COMPARISON")
    print("=" * 80)

    # Test configurations
    configs = [
        (1000, 128, "Small dataset"),
        (5000, 128, "At threshold"),
        (10000, 128, "Above threshold"),
        (10000, 768, "High dimension"),
    ]

    for n_vectors, dimension, desc in configs:
        print(f"\n📊 {desc}: {n_vectors:,} vectors @ {dimension}D")
        print("-" * 60)

        # Generate test data
        np.random.seed(42)
        vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
        ids = [f"vec_{i:06d}" for i in range(n_vectors)]

        results = []

        # Test 1: Default (auto migration at 5K)
        print("\n1️⃣ DEFAULT (auto migration at 5K):")
        db1 = DB()
        start = time.time()
        db1.add_batch(vectors=vectors, ids=ids)
        elapsed = time.time() - start
        rate = n_vectors / elapsed if elapsed > 0 else 0
        print(f"   Rate: {rate:,.0f} vec/s")
        print(f"   Time: {elapsed:.3f}s")
        stats = db1.info()
        print(f"   Algorithm: {stats.get('algorithm', 'unknown')}")
        results.append(("Default", rate, elapsed))

        # Test 2: Force brute force
        print("\n2️⃣ FORCE BRUTE FORCE:")
        db2 = DB(force_algorithm="brute_force")
        start = time.time()
        db2.add_batch(vectors=vectors, ids=ids)
        elapsed = time.time() - start
        rate = n_vectors / elapsed if elapsed > 0 else 0
        print(f"   Rate: {rate:,.0f} vec/s")
        print(f"   Time: {elapsed:.3f}s")
        stats = db2.info()
        print(f"   Algorithm: {stats.get('algorithm', 'unknown')}")
        results.append(("Brute Force", rate, elapsed))

        # Test 3: Force HNSW from start (no migration)
        print("\n3️⃣ FORCE HNSW (no migration):")
        db3 = DB(force_algorithm="hnsw")
        start = time.time()
        db3.add_batch(vectors=vectors, ids=ids)
        elapsed = time.time() - start
        rate = n_vectors / elapsed if elapsed > 0 else 0
        print(f"   Rate: {rate:,.0f} vec/s")
        print(f"   Time: {elapsed:.3f}s")
        stats = db3.info()
        print(f"   Algorithm: {stats.get('algorithm', 'unknown')}")
        results.append(("HNSW", rate, elapsed))

        # Find winner
        winner = max(results, key=lambda x: x[1])
        print(f"\n🏆 WINNER: {winner[0]} ({winner[1]:,.0f} vec/s)")

        # Query performance comparison
        print("\n🔍 Query Performance:")
        query = vectors[0]

        # Default
        start = time.time()
        res1 = db1.search(query, limit=10)
        q1_time = (time.time() - start) * 1000

        # Brute force
        start = time.time()
        res2 = db2.search(query, limit=10)
        q2_time = (time.time() - start) * 1000

        # HNSW
        start = time.time()
        res3 = db3.search(query, limit=10)
        q3_time = (time.time() - start) * 1000

        print(f"   Default:     {q1_time:.2f}ms")
        print(f"   Brute Force: {q2_time:.2f}ms")
        print(f"   HNSW:        {q3_time:.2f}ms")

        # Find query winner
        query_times = [
            ("Default", q1_time),
            ("Brute Force", q2_time),
            ("HNSW", q3_time),
        ]
        query_winner = min(query_times, key=lambda x: x[1])
        print(f"   🏆 Fastest: {query_winner[0]} ({query_winner[1]:.2f}ms)")


def test_incremental_addition():
    """Test incremental addition patterns."""

    print("\n" + "=" * 80)
    print("INCREMENTAL ADDITION PATTERNS")
    print("=" * 80)

    n_vectors = 10000
    dimension = 128
    batch_size = 1000

    # Generate all data
    np.random.seed(42)
    all_vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    all_ids = [f"vec_{i:06d}" for i in range(n_vectors)]

    print(f"\n📊 Adding {n_vectors:,} vectors in batches of {batch_size}")
    print("-" * 60)

    # Test 1: Default with incremental batches
    print("\n1️⃣ DEFAULT (will trigger migration):")
    db1 = DB()
    start = time.time()
    for i in range(0, n_vectors, batch_size):
        end = min(i + batch_size, n_vectors)
        db1.add_batch(vectors=all_vectors[i:end], ids=all_ids[i:end])
    elapsed = time.time() - start
    print(f"   Total time: {elapsed:.3f}s")
    print(f"   Rate: {n_vectors / elapsed:,.0f} vec/s")

    # Test 2: Force HNSW from start
    print("\n2️⃣ FORCE HNSW from start:")
    db2 = DB(force_algorithm="hnsw")
    start = time.time()
    for i in range(0, n_vectors, batch_size):
        end = min(i + batch_size, n_vectors)
        db2.add_batch(vectors=all_vectors[i:end], ids=all_ids[i:end])
    elapsed = time.time() - start
    print(f"   Total time: {elapsed:.3f}s")
    print(f"   Rate: {n_vectors / elapsed:,.0f} vec/s")

    # Test 3: High migration threshold (stay in brute force)
    print("\n3️⃣ HIGH THRESHOLD (stay in brute force):")
    db3 = DB(migration_threshold=1000000)
    start = time.time()
    for i in range(0, n_vectors, batch_size):
        end = min(i + batch_size, n_vectors)
        db3.add_batch(vectors=all_vectors[i:end], ids=all_ids[i:end])
    elapsed = time.time() - start
    print(f"   Total time: {elapsed:.3f}s")
    print(f"   Rate: {n_vectors / elapsed:,.0f} vec/s")

    # Compare final states
    print("\n📊 Final Algorithm States:")
    print(f"   Default: {db1.info().get('algorithm', 'unknown')}")
    print(f"   Force HNSW: {db2.info().get('algorithm', 'unknown')}")
    print(f"   High Threshold: {db3.info().get('algorithm', 'unknown')}")


def compare_with_competitors():
    """Compare our best performance with competitors."""

    print("\n" + "=" * 80)
    print("COMPETITOR COMPARISON (same data)")
    print("=" * 80)

    n_vectors = 10000
    dimension = 768  # Standard embedding dimension

    # Generate test data
    np.random.seed(42)
    vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    vectors = vectors / np.linalg.norm(vectors, axis=1, keepdims=True)
    ids = [f"vec_{i:06d}" for i in range(n_vectors)]

    print(f"\n📊 Testing {n_vectors:,} vectors @ {dimension}D (standard embeddings)")
    print("-" * 60)

    # Test OmenDB with different strategies
    print("\n🚀 OmenDB:")

    # Best strategy for OmenDB based on our tests
    strategies = [
        ("Default", DB()),
        ("Force HNSW", DB(force_algorithm="hnsw")),
        ("Force Brute", DB(force_algorithm="brute_force")),
        ("High Threshold", DB(migration_threshold=1000000)),
    ]

    best_rate = 0
    best_strategy = ""

    for name, db in strategies:
        start = time.time()
        db.add_batch(vectors=vectors, ids=ids)
        elapsed = time.time() - start
        rate = n_vectors / elapsed if elapsed > 0 else 0
        print(f"   {name:15}: {rate:>8,.0f} vec/s | {elapsed:.3f}s")

        if rate > best_rate:
            best_rate = rate
            best_strategy = name

    print(f"\n   🏆 Best: {best_strategy} at {best_rate:,.0f} vec/s")

    # Reference numbers from our benchmark
    print("\n📊 Competitor Performance (from benchmark):")
    print("   ChromaDB:        3,678 vec/s")
    print("   LanceDB:        38,683 vec/s")

    print("\n📈 Relative Performance:")
    if best_rate > 0:
        vs_chroma = best_rate / 3678
        vs_lance = best_rate / 38683
        print(f"   vs ChromaDB: {vs_chroma:.2f}x")
        print(f"   vs LanceDB:  {vs_lance:.2f}x")


if __name__ == "__main__":
    test_algorithm_performance()
    test_incremental_addition()
    compare_with_competitors()

    print("\n" + "=" * 80)
    print("RECOMMENDATIONS")
    print("=" * 80)
    print("""
1. For known large datasets: Use force_algorithm='hnsw'
2. For small datasets (<5K): Use force_algorithm='brute_force' or high threshold
3. For incremental growth: Set appropriate migration_threshold
4. The migration process itself is the bottleneck - avoid it when possible
5. HNSW without migration is much faster than with migration
""")
