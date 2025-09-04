#!/usr/bin/env python3
"""
Compare baseline HNSW vs optimized SIMD version to identify regression.
"""

import sys
import os
import time
import numpy as np

# Add the parent directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))

import omendb


def benchmark_hnsw_construction():
    """Test construction performance to identify regression source."""
    print("🔬 HNSW Performance Regression Analysis")
    print("=" * 50)

    # Test parameters
    dimensions = 128
    test_sizes = [1000, 2000, 3000, 4000, 5000, 6000]  # Cross the 5K threshold

    for size in test_sizes:
        print(f"\n📊 Testing {size} vectors:")

        # Generate test data
        vectors = []
        for i in range(size):
            vector = [float(np.random.randn()) for _ in range(dimensions)]
            vectors.append(vector)

        ids = [f"vec_{i}" for i in range(size)]

        # Test construction
        db = omendb.DB()

        # Time the construction
        start_time = time.time()

        # Add in batches to see progression
        batch_size = 1000
        for batch_start in range(0, size, batch_size):
            batch_end = min(batch_start + batch_size, size)
            batch_vectors = vectors[batch_start:batch_end]
            batch_ids = ids[batch_start:batch_end]

            batch_start_time = time.time()
            db.add_batch(list(zip(batch_ids, batch_vectors)))
            batch_time = time.time() - batch_start_time

            current_count = db.count()
            stats = db.stats()
            algorithm = stats.get("algorithm", "unknown")
            vec_per_sec = len(batch_vectors) / batch_time

            print(
                f"  Batch {batch_start // batch_size}: {len(batch_vectors)} vectors in {batch_time:.3f}s ({vec_per_sec:.0f} vec/s) - {algorithm}"
            )

            if algorithm == "hnsw":
                print(f"    🔄 Switched to HNSW at {current_count} vectors")

        total_time = time.time() - start_time
        overall_speed = size / total_time
        final_stats = db.stats()

        print(
            f"  📈 Total: {size} vectors in {total_time:.2f}s ({overall_speed:.0f} vec/s)"
        )
        print(f"  📋 Final algorithm: {final_stats.get('algorithm')}")

        # Test query performance
        query_vector = vectors[0]
        start_time = time.time()
        results = db.query(query_vector, top_k=10)
        query_time = (time.time() - start_time) * 1000
        print(f"  🔍 Query: {query_time:.2f}ms, {len(results)} results")

        # Clear for next test
        db.clear()


def test_distance_calculation_performance():
    """Test if our SIMD distance calculations are actually faster."""
    print("\n🧮 Distance Calculation Performance Test")
    print("=" * 45)

    dimensions = 128
    num_vectors = 1000
    num_queries = 100

    # Generate test data
    print(f"📊 Generating {num_vectors} vectors, {num_queries} queries...")
    vectors = []
    for i in range(num_vectors):
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        vectors.append(vector)

    queries = []
    for i in range(num_queries):
        query = [float(np.random.randn()) for _ in range(dimensions)]
        queries.append(query)

    # Test with our implementation
    db = omendb.DB()
    ids = [f"vec_{i}" for i in range(num_vectors)]

    # Add all vectors
    start_time = time.time()
    db.add_batch(list(zip(ids, vectors)))
    add_time = time.time() - start_time
    print(
        f"💾 Added {num_vectors} vectors in {add_time:.3f}s ({num_vectors / add_time:.0f} vec/s)"
    )

    # Test multiple queries
    start_time = time.time()
    for query in queries:
        results = db.query(query, top_k=10)
    total_query_time = time.time() - start_time
    avg_query_time = (total_query_time / num_queries) * 1000

    print(f"🔍 Executed {num_queries} queries in {total_query_time:.3f}s")
    print(f"⚡ Average query time: {avg_query_time:.2f}ms")

    stats = db.stats()
    print(f"📋 Algorithm used: {stats.get('algorithm')}")


def profile_construction_phases():
    """Profile different phases of HNSW construction."""
    print("\n🔬 HNSW Construction Phase Profiling")
    print("=" * 45)

    dimensions = 128
    vectors_per_phase = 500

    # Generate test data
    vectors = []
    for i in range(6000):
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        vectors.append(vector)

    db = omendb.DB()

    # Phase 1: Brute force only
    print("🚀 Phase 1: Brute force (0-4999 vectors)")
    phase_vectors = vectors[:4999]
    phase_ids = [f"vec_{i}" for i in range(len(phase_vectors))]

    start_time = time.time()
    db.add_batch(list(zip(phase_ids, phase_vectors)))
    phase_time = time.time() - start_time
    phase_speed = len(phase_vectors) / phase_time

    stats = db.stats()
    print(
        f"  📈 {len(phase_vectors)} vectors in {phase_time:.2f}s ({phase_speed:.0f} vec/s)"
    )
    print(f"  📋 Algorithm: {stats.get('algorithm')}")

    # Phase 2: Migration trigger
    print("\n🔄 Phase 2: Migration trigger (5000th vector)")
    migration_vector = vectors[4999]
    migration_id = "vec_4999"

    start_time = time.time()
    db.add(migration_id, migration_vector)
    migration_time = time.time() - start_time

    stats = db.stats()
    print(f"  ⏱️  Migration took: {migration_time:.3f}s")
    print(f"  📋 New algorithm: {stats.get('algorithm')}")

    # Phase 3: HNSW construction
    print("\n⚡ Phase 3: HNSW construction (5001-6000 vectors)")
    hnsw_vectors = vectors[5000:6000]
    hnsw_ids = [f"vec_{i}" for i in range(5000, 6000)]

    # Add one by one to see individual performance
    insertion_times = []
    for i, (vec_id, vector) in enumerate(zip(hnsw_ids, hnsw_vectors)):
        start_time = time.time()
        db.add(vec_id, vector)
        insertion_time = time.time() - start_time
        insertion_times.append(insertion_time)

        if i % 100 == 0:
            avg_time = sum(insertion_times[-100:]) / len(insertion_times[-100:])
            speed = 1.0 / avg_time if avg_time > 0 else 0
            print(
                f"  Vector {5000 + i}: {avg_time * 1000:.2f}ms avg ({speed:.0f} vec/s)"
            )

    total_hnsw_time = sum(insertion_times)
    hnsw_speed = len(hnsw_vectors) / total_hnsw_time
    print(
        f"  📈 HNSW phase: {len(hnsw_vectors)} vectors in {total_hnsw_time:.2f}s ({hnsw_speed:.0f} vec/s)"
    )


if __name__ == "__main__":
    print("🎯 HNSW Performance Regression Analysis")
    print("=" * 50)

    try:
        benchmark_hnsw_construction()
        test_distance_calculation_performance()
        profile_construction_phases()

        print(f"\n🎯 Analysis Complete")
        print("=" * 30)
        print("Key findings will help identify optimization priorities.")

    except Exception as e:
        print(f"❌ Error during analysis: {e}")
        import traceback

        traceback.print_exc()
