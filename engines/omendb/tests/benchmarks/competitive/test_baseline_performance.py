#!/usr/bin/env python3
"""
Test baseline HNSW performance to establish the current state.
"""

import sys
import os
import time
import numpy as np

# Add the parent directory to the path
sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))

import omendb


def test_baseline_performance():
    """Test current HNSW baseline performance."""
    print("🎯 Baseline HNSW Performance Test")
    print("=" * 40)

    # Test with 6000 vectors to trigger HNSW
    dimensions = 128
    num_vectors = 6000

    # Generate test data
    print(f"📊 Generating {num_vectors} vectors of {dimensions}D...")
    vectors = []
    for i in range(num_vectors):
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        vectors.append(vector)

    ids = [f"vec_{i}" for i in range(num_vectors)]

    db = omendb.DB()

    print("🏗️  Testing construction speed with baseline HNSW...")

    # Track performance by phase
    batch_size = 1000
    times = []
    speeds = []

    for batch_start in range(0, num_vectors, batch_size):
        batch_end = min(batch_start + batch_size, num_vectors)
        batch_vectors = vectors[batch_start:batch_end]
        batch_ids = ids[batch_start:batch_end]

        batch_start_time = time.time()
        db.add_batch(list(zip(batch_ids, batch_vectors)))
        batch_time = time.time() - batch_start_time

        times.append(batch_time)
        current_count = db.count()
        vec_per_sec = len(batch_vectors) / batch_time
        speeds.append(vec_per_sec)

        stats = db.stats()
        algorithm = stats.get("algorithm", "unknown")

        print(
            f"  Batch {batch_start // batch_size}: {len(batch_vectors)} vectors in {batch_time:.3f}s ({vec_per_sec:.0f} vec/s) - {algorithm}"
        )

        if algorithm == "hnsw" and batch_start >= 5000:
            print(f"    📊 HNSW Speed: {vec_per_sec:.0f} vec/s")

    # Calculate overall performance
    total_time = sum(times)
    overall_speed = num_vectors / total_time

    # Calculate HNSW-only performance (batches 5 and 6)
    if len(speeds) >= 6:
        hnsw_speeds = speeds[4:]  # Batches 4+ are HNSW
        avg_hnsw_speed = sum(hnsw_speeds) / len(hnsw_speeds)
        print(f"\n📈 HNSW Average Speed: {avg_hnsw_speed:.0f} vec/s")

    print(f"\n📈 Overall Performance:")
    print(f"   Total time: {total_time:.2f}s")
    print(f"   Overall speed: {overall_speed:.0f} vec/s")
    print(f"   Final count: {db.count()}")

    # Test query performance
    print(f"\n🔍 Testing query performance...")
    query_vector = vectors[0]

    query_times = []
    for _ in range(100):
        start = time.time()
        results = db.query(query_vector, top_k=10)
        query_time = time.time() - start
        query_times.append(query_time * 1000)

    avg_query_time = sum(query_times) / len(query_times)
    print(f"   Average query time: {avg_query_time:.2f}ms")
    print(f"   Query results: {len(results)} items")

    return avg_hnsw_speed if len(speeds) >= 6 else 0, avg_query_time


def quick_hnsw_test():
    """Quick test specifically for HNSW performance."""
    print("\n⚡ Quick HNSW Performance Test")
    print("=" * 35)

    dimensions = 128

    # Generate 5500 vectors (500 over threshold)
    vectors = []
    for i in range(5500):
        vector = [float(np.random.randn()) for _ in range(dimensions)]
        vectors.append(vector)

    ids = [f"vec_{i}" for i in range(5500)]

    db = omendb.DB()

    # Add first 5000 with brute force
    print("🚀 Adding 5000 vectors (brute force)...")
    start_time = time.time()
    db.add_batch(list(zip(ids[:5000], vectors[:5000])))
    brute_time = time.time() - start_time
    brute_speed = 5000 / brute_time
    print(f"   Brute force: {brute_speed:.0f} vec/s")

    # Add remaining 500 with HNSW
    print("⚡ Adding 500 vectors (HNSW)...")
    hnsw_vectors = []
    hnsw_times = []

    for i in range(5000, 5500):
        start_time = time.time()
        db.add(ids[i], vectors[i])
        add_time = time.time() - start_time
        hnsw_times.append(add_time)

        if i % 100 == 0:
            recent_avg = sum(hnsw_times[-100:]) / len(hnsw_times[-100:])
            recent_speed = 1.0 / recent_avg if recent_avg > 0 else 0
            print(
                f"   Vector {i}: {recent_speed:.0f} vec/s (avg of last {min(100, len(hnsw_times))})"
            )

    avg_hnsw_time = sum(hnsw_times) / len(hnsw_times)
    hnsw_speed = 1.0 / avg_hnsw_time if avg_hnsw_time > 0 else 0

    print(f"\n📊 Results:")
    print(f"   Brute force: {brute_speed:.0f} vec/s")
    print(f"   HNSW: {hnsw_speed:.0f} vec/s")
    print(f"   Ratio: {brute_speed / hnsw_speed:.1f}x faster brute force")

    return hnsw_speed


if __name__ == "__main__":
    print("🎯 Baseline Performance Analysis")
    print("=" * 40)

    try:
        hnsw_speed, query_time = test_baseline_performance()
        quick_speed = quick_hnsw_test()

        print(f"\n🏁 Summary:")
        print(f"   HNSW Speed: {hnsw_speed:.0f} vec/s (batch)")
        print(f"   HNSW Speed: {quick_speed:.0f} vec/s (individual)")
        print(f"   Query Time: {query_time:.2f}ms")

        # Compare to documented baseline
        documented_baseline = 455  # vec/s from docs
        if quick_speed > 0:
            vs_baseline = quick_speed / documented_baseline
            print(f"   vs Documented: {vs_baseline:.2f}x")

            if vs_baseline >= 1.0:
                print("   ✅ Meeting or exceeding baseline!")
            else:
                improvement_needed = documented_baseline / quick_speed
                print(
                    f"   ⚠️  Need {improvement_needed:.1f}x improvement to reach baseline"
                )

    except Exception as e:
        print(f"❌ Error during testing: {e}")
        import traceback

        traceback.print_exc()
