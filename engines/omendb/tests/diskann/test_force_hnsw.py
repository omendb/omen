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
        (1000, 128),
        (5000, 128),
        (10000, 128),
        (10000, 768),
    ]

    for n_vectors, dimension in configs:
        print(f"\n📊 Testing {n_vectors:,} vectors @ {dimension}D")
        print("-" * 60)

        # Generate test data
        np.random.seed(42)
        vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
        ids = [f"vec_{i:06d}" for i in range(n_vectors)]

        # Test 1: Default (auto migration)
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

        # Test 2: Force brute force
        print("\n2️⃣ FORCE BRUTE FORCE:")
        db2 = DB()
        start = time.time()
        db2.add_batch(
            vectors=vectors, ids=ids, config={"force_algorithm": "brute_force"}
        )
        elapsed = time.time() - start
        rate = n_vectors / elapsed if elapsed > 0 else 0
        print(f"   Rate: {rate:,.0f} vec/s")
        print(f"   Time: {elapsed:.3f}s")
        stats = db2.info()
        print(f"   Algorithm: {stats.get('algorithm', 'unknown')}")

        # Test 3: Force HNSW from start (no migration)
        print("\n3️⃣ FORCE HNSW (no migration):")
        db3 = DB()
        start = time.time()
        db3.add_batch(vectors=vectors, ids=ids, config={"force_algorithm": "hnsw"})
        elapsed = time.time() - start
        rate = n_vectors / elapsed if elapsed > 0 else 0
        print(f"   Rate: {rate:,.0f} vec/s")
        print(f"   Time: {elapsed:.3f}s")
        stats = db3.info()
        print(f"   Algorithm: {stats.get('algorithm', 'unknown')}")

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

        print(f"   Default: {q1_time:.2f}ms")
        print(f"   Brute:   {q2_time:.2f}ms")
        print(f"   HNSW:    {q3_time:.2f}ms")


def test_smart_algorithm_selection():
    """Test if we can be smarter about algorithm selection."""

    print("\n" + "=" * 80)
    print("SMART ALGORITHM SELECTION TEST")
    print("=" * 80)

    # Scenario: We know we'll have 10K vectors
    n_vectors = 10000
    dimension = 128

    print(f"\n📊 Adding {n_vectors:,} vectors in batches")

    # Generate all data
    np.random.seed(42)
    all_vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    all_ids = [f"vec_{i:06d}" for i in range(n_vectors)]

    # Test 1: Add all at once with force_algorithm
    print("\n1️⃣ Single batch with force_algorithm='hnsw':")
    db1 = DB()
    start = time.time()
    db1.add_batch(vectors=all_vectors, ids=all_ids, config={"force_algorithm": "hnsw"})
    elapsed = time.time() - start
    print(f"   Rate: {n_vectors / elapsed:,.0f} vec/s")
    print(f"   Time: {elapsed:.3f}s")

    # Test 2: Add in chunks but force HNSW from start
    print("\n2️⃣ Multiple batches with force_algorithm='hnsw':")
    db2 = DB()
    batch_size = 1000
    start = time.time()
    for i in range(0, n_vectors, batch_size):
        end = min(i + batch_size, n_vectors)
        db2.add_batch(
            vectors=all_vectors[i:end],
            ids=all_ids[i:end],
            config={"force_algorithm": "hnsw"},
        )
    elapsed = time.time() - start
    print(f"   Rate: {n_vectors / elapsed:,.0f} vec/s")
    print(f"   Time: {elapsed:.3f}s")

    # Test 3: Let it auto-migrate (current behavior)
    print("\n3️⃣ Multiple batches with auto-migration:")
    db3 = DB()
    start = time.time()
    for i in range(0, n_vectors, batch_size):
        end = min(i + batch_size, n_vectors)
        db3.add_batch(vectors=all_vectors[i:end], ids=all_ids[i:end])
    elapsed = time.time() - start
    print(f"   Rate: {n_vectors / elapsed:,.0f} vec/s")
    print(f"   Time: {elapsed:.3f}s")


if __name__ == "__main__":
    test_algorithm_performance()
    test_smart_algorithm_selection()

    print("\n" + "=" * 80)
    print("KEY INSIGHTS")
    print("=" * 80)
    print("""
1. If you know final size, force HNSW from start
2. Migration during insertion is the bottleneck
3. Brute force is actually faster for <5K vectors
4. HNSW without migration should be much faster
""")
