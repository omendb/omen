#!/usr/bin/env python3
"""Test 768D performance specifically."""

import sys
import time
import numpy as np
import tempfile
import os

sys.path.insert(0, "/Users/nick/github/omendb/omendb/python")

from omendb import DB


def test_768d_performance():
    """Test performance with 768D vectors (standard embeddings)."""

    print("=" * 80)
    print("768D PERFORMANCE TEST (Standard Embeddings)")
    print("=" * 80)

    n_vectors = 10000
    dimension = 768

    # Generate test data
    np.random.seed(42)
    vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    vectors = vectors / np.linalg.norm(vectors, axis=1, keepdims=True)  # Normalize
    ids = [f"vec_{i:06d}" for i in range(n_vectors)]

    print(f"📊 Testing {n_vectors:,} vectors @ {dimension}D")
    print("-" * 60)

    results = []

    # Test different strategies
    strategies = [
        ("Default", {}),
        ("Force HNSW", {"force_algorithm": "hnsw"}),
        ("Force Brute", {"force_algorithm": "brute_force"}),
        ("High Threshold", {"migration_threshold": 1000000}),
    ]

    for name, config in strategies:
        with tempfile.NamedTemporaryFile(suffix=".omendb", delete=False) as tmp:
            db_path = tmp.name

        try:
            print(f"\n{name}:")
            db = DB(db_path=db_path, **config)

            # Time the insertion
            start = time.time()
            db.add_batch(vectors=vectors, ids=ids)
            elapsed = time.time() - start
            rate = n_vectors / elapsed if elapsed > 0 else 0

            # Get final algorithm
            stats = db.info()
            final_algo = stats.get("algorithm", "unknown")

            print(f"   Rate: {rate:>10,.0f} vec/s")
            print(f"   Time: {elapsed:>10.3f}s")
            print(f"   Algorithm: {final_algo}")

            results.append((name, rate, elapsed, final_algo))

            # Test query performance
            query = vectors[0]
            start = time.time()
            res = db.search(query, limit=10)
            query_time = (time.time() - start) * 1000
            print(f"   Query: {query_time:>9.2f}ms")

        except Exception as e:
            print(f"   ERROR: {e}")
            results.append((name, 0, 0, "error"))

        finally:
            if os.path.exists(db_path):
                os.unlink(db_path)

    # Find best strategy
    valid_results = [
        (name, rate, time, algo) for name, rate, time, algo in results if rate > 0
    ]
    if valid_results:
        winner = max(valid_results, key=lambda x: x[1])
        print(f"\n🏆 WINNER: {winner[0]} at {winner[1]:,.0f} vec/s")

    # Compare with competitors (from our benchmark)
    print("\n📊 vs Competitors (same 10K @ 768D):")
    print("   ChromaDB:  3,678 vec/s")
    print("   LanceDB:  38,683 vec/s")

    if valid_results:
        best_rate = winner[1]
        vs_chroma = best_rate / 3678 if 3678 > 0 else 0
        vs_lance = best_rate / 38683 if 38683 > 0 else 0
        print(f"\n📈 OmenDB Relative Performance:")
        print(f"   vs ChromaDB: {vs_chroma:.2f}x")
        print(f"   vs LanceDB:  {vs_lance:.2f}x")


def test_incremental_768d():
    """Test incremental addition of 768D vectors."""

    print("\n" + "=" * 80)
    print("INCREMENTAL 768D PERFORMANCE")
    print("=" * 80)

    n_vectors = 10000
    dimension = 768
    batch_size = 1000

    # Generate data
    np.random.seed(42)
    all_vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    all_ids = [f"vec_{i:06d}" for i in range(n_vectors)]

    print(f"📊 Adding {n_vectors:,} @ {dimension}D in batches of {batch_size}")
    print("-" * 60)

    # Test with optimal strategy (based on previous results)
    with tempfile.NamedTemporaryFile(suffix=".omendb", delete=False) as tmp:
        db_path = tmp.name

    try:
        # Try force HNSW to avoid migration
        db = DB(db_path=db_path, force_algorithm="hnsw")

        batch_times = []
        total_start = time.time()

        for i in range(0, n_vectors, batch_size):
            end = min(i + batch_size, n_vectors)
            batch_start = time.time()
            db.add_batch(vectors=all_vectors[i:end], ids=all_ids[i:end])
            batch_time = time.time() - batch_start
            batch_times.append(batch_time)

            batch_num = (i // batch_size) + 1
            rate = (end - i) / batch_time if batch_time > 0 else 0
            print(f"   Batch {batch_num:2d}: {batch_time:6.3f}s ({rate:>7.0f} vec/s)")

        total_time = time.time() - total_start
        total_rate = n_vectors / total_time if total_time > 0 else 0

        print(f"\n   Total: {total_time:.3f}s ({total_rate:,.0f} vec/s)")

        # Final stats
        stats = db.info()
        print(f"   Final algorithm: {stats.get('algorithm', 'unknown')}")

    finally:
        if os.path.exists(db_path):
            os.unlink(db_path)


if __name__ == "__main__":
    test_768d_performance()
    test_incremental_768d()

    print("\n" + "=" * 80)
    print("KEY INSIGHTS FOR 768D")
    print("=" * 80)
    print("""
1. High dimensions (768D) are inherently slower - this is expected
2. The best strategy depends on the specific use case
3. Forcing algorithm choice avoids migration overhead
4. Even with optimization, we're likely slower than columnar stores like LanceDB
5. Our main advantage remains instant startup time
""")
