#!/usr/bin/env python3
"""
Test HNSW performance with Faiss-style optimizations.
"""

import sys
import os
import time

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "python"))

import omendb
import numpy as np


def test_hnsw_performance():
    """Test HNSW performance with Faiss optimizations."""
    print("🚀 Testing HNSW Performance with Faiss Optimizations")
    print("=" * 55)

    db = omendb.DB()
    dimensions = 128

    # Add 5000 vectors to trigger HNSW
    print("📦 Adding 5000 vectors to trigger HNSW...")
    vectors = []
    batch_size = 100

    start_time = time.time()

    for batch_idx in range(50):  # 50 batches of 100 = 5000 vectors
        batch_vectors = []
        batch_ids = []

        for i in range(batch_size):
            vector_idx = batch_idx * batch_size + i
            vector = [float(np.random.randn()) for _ in range(dimensions)]
            vectors.append(vector)
            batch_vectors.append(vector)
            batch_ids.append(f"batch_{vector_idx}")

        # Add batch
        batch_data = list(zip(batch_ids, batch_vectors))
        db.add_batch(batch_data)

        if (batch_idx + 1) % 10 == 0:
            elapsed = time.time() - start_time
            total_vectors = (batch_idx + 1) * batch_size
            speed = total_vectors / elapsed if elapsed > 0 else 0
            print(
                f"  Added {total_vectors} vectors in {elapsed:.2f}s ({speed:.0f} vec/s)"
            )

    total_time = time.time() - start_time
    total_speed = 5000 / total_time if total_time > 0 else 0

    print(f"\n📊 Construction Performance:")
    print(f"   Total time: {total_time:.2f}s")
    print(f"   Construction speed: {total_speed:.0f} vec/s")

    # Check algorithm
    stats = db.stats()
    print(f"   Algorithm: {stats.get('algorithm')}")
    print(f"   Size: {stats.get('size')}")

    if stats.get("algorithm") != "hnsw":
        print("⚠️  HNSW not activated - check threshold")
        return False

    # Test query performance
    print(f"\n🔍 Testing Query Performance...")
    query_times = []

    for i in range(10):
        query = [float(np.random.randn()) for _ in range(dimensions)]

        start_time = time.time()
        results = db.query(query, top_k=10)
        query_time = time.time() - start_time

        query_times.append(query_time)

        if i == 0:
            print(f"   Query returned {len(results)} results")

    avg_query_time = sum(query_times) / len(query_times)
    print(f"   Average query time: {avg_query_time * 1000:.2f}ms")

    # Compare with Faiss target
    faiss_speed_target = 28000  # Conservative Faiss target
    our_speed = total_speed
    speed_ratio = faiss_speed_target / our_speed if our_speed > 0 else float("inf")

    print(f"\n⚡ Performance Comparison:")
    print(f"   Our speed: {our_speed:.0f} vec/s")
    print(f"   Faiss target: {faiss_speed_target} vec/s")
    print(f"   Speed gap: {speed_ratio:.1f}x slower")

    if speed_ratio < 10:
        print("✅ Good progress - within 10x of Faiss!")
    elif speed_ratio < 50:
        print("🔄 Making progress - within 50x of Faiss")
    else:
        print("❌ Still significant gap vs Faiss")

    return True


if __name__ == "__main__":
    try:
        test_hnsw_performance()
    except Exception as e:
        print(f"❌ Error during performance test: {e}")
        import traceback

        traceback.print_exc()
