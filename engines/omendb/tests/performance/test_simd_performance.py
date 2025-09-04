#!/usr/bin/env python3
"""
Test SIMD performance improvements in OmenDB.
"""

import sys
import os
import time
import numpy as np

# Add the parent directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))

import omendb


def test_construction_performance():
    """Test HNSW construction performance with SIMD optimizations."""
    print("🚀 Testing HNSW Construction Performance with SIMD")
    print("=" * 50)

    # Test with larger dataset to trigger HNSW
    dimensions = 128
    num_vectors = 6000  # Above the 5000 threshold

    # Generate test vectors
    print(f"📊 Generating {num_vectors} vectors of {dimensions}D...")
    vectors = []
    for i in range(num_vectors):
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        vectors.append(vector)

    # Test construction performance
    db = omendb.DB()

    print("🏗️  Testing vector construction speed...")
    start_time = time.time()

    # Add first batch to trigger brute force
    batch_size = 1000
    for batch_start in range(0, num_vectors, batch_size):
        batch_end = min(batch_start + batch_size, num_vectors)
        batch_vectors = vectors[batch_start:batch_end]
        batch_ids = [f"vec_{i}" for i in range(batch_start, batch_end)]

        batch_start_time = time.time()
        db.add_batch(vectors=batch_vectors, ids=batch_ids)
        batch_time = time.time() - batch_start_time

        current_count = db.count()
        vec_per_sec = len(batch_vectors) / batch_time

        print(
            f"  Batch {batch_start // batch_size}: {len(batch_vectors)} vectors in {batch_time:.3f}s ({vec_per_sec:.0f} vec/s) - Total: {current_count}"
        )

        # Show when migration happens
        stats = db.info()
        if stats.get("algorithm") == "hnsw":
            print(f"    🔄 Using HNSW algorithm now")

    total_time = time.time() - start_time
    overall_vec_per_sec = num_vectors / total_time

    print(f"\n📈 Overall Performance:")
    print(f"   Total time: {total_time:.2f}s")
    print(f"   Overall speed: {overall_vec_per_sec:.0f} vec/s")
    print(f"   Final vector count: {db.count()}")

    # Test query performance
    print(f"\n🔍 Testing query performance...")
    query_vector = vectors[0]  # Use first vector as query

    query_times = []
    for _ in range(100):
        start = time.time()
        results = db.search(query_vector, limit=10)
        query_time = time.time() - start
        query_times.append(query_time * 1000)  # Convert to ms

    avg_query_time = sum(query_times) / len(query_times)
    print(f"   Average query time: {avg_query_time:.2f}ms")
    print(f"   Query results: {len(results)} items")

    return overall_vec_per_sec, avg_query_time


def test_simd_vs_sequential():
    """Compare SIMD optimized vs sequential distance calculations."""
    print("\n🧮 Testing SIMD Distance Calculations")
    print("=" * 40)

    dimensions = 128
    num_comparisons = 10000

    # Generate test data
    query = [float(np.random.randn()) for _ in range(dimensions)]
    vectors = []
    for i in range(num_comparisons):
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        vectors.append(vector)

    # Test with our optimized implementation
    db = omendb.DB()

    # Add vectors to database
    ids = [f"vec_{i}" for i in range(num_comparisons)]
    start_time = time.time()
    db.add_batch(list(zip(ids, vectors)))
    add_time = time.time() - start_time

    print(
        f"📊 Added {num_comparisons} vectors in {add_time:.3f}s ({num_comparisons / add_time:.0f} vec/s)"
    )

    # Test query performance (this will use our SIMD optimizations)
    start_time = time.time()
    results = db.search(query, limit=100)
    query_time = time.time() - start_time

    print(f"🔍 Query completed in {query_time * 1000:.2f}ms")
    print(f"   Results: {len(results)} vectors")
    print(f"   Top result similarity: {results[0].score:.4f}")


if __name__ == "__main__":
    print("🎯 OmenDB SIMD Performance Testing")
    print("=" * 50)

    try:
        vec_per_sec, query_time = test_construction_performance()
        test_simd_vs_sequential()

        print(f"\n🏆 Performance Summary:")
        print(f"   Construction: {vec_per_sec:.0f} vec/s")
        print(f"   Query: {query_time:.2f}ms average")

        # Compare against targets
        target_construction = 5000  # vec/s target for YC
        if vec_per_sec >= target_construction:
            print(
                f"✅ Construction speed meets YC target ({target_construction} vec/s)"
            )
        else:
            improvement_needed = target_construction / vec_per_sec
            print(
                f"⚠️  Construction needs {improvement_needed:.1f}x improvement to meet YC target"
            )

    except Exception as e:
        print(f"❌ Error during testing: {e}")
        import traceback

        traceback.print_exc()
