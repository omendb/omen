#!/usr/bin/env python3
"""
Test the optimized RoarGraph implementation
Focus on the 500-5000 vector range where performance exploded before
"""

import sys
import time
import random
import math
import os

sys.path.insert(0, "/Users/nick/github/omenDB/python")


def generate_test_vectors(count: int, dimension: int = 3, seed: int = 42):
    """Generate normalized test vectors"""
    random.seed(seed)
    vectors = []

    for i in range(count):
        vector = [random.gauss(0, 1) for _ in range(dimension)]
        # Normalize
        norm = math.sqrt(sum(x * x for x in vector))
        if norm > 0:
            vector = [x / norm for x in vector]
        vectors.append((f"vec_{i:05d}", vector))

    return vectors


def test_construction_time(scale: int) -> dict:
    """Test construction time at specific scale"""
    print(f"\n🎯 Testing {scale} vectors...")

    try:
        from omendb import DB

        # Generate test data
        test_vectors = generate_test_vectors(scale)

        # Measure insertion performance
        db = DB()
        start_time = time.time()

        for i, (doc_id, vector) in enumerate(test_vectors):
            success = db.add(doc_id, vector)
            if not success:
                print(f"❌ Failed to add vector {i}")
                break

            # Track progress at key intervals
            if (i + 1) % max(1, scale // 10) == 0:
                elapsed = time.time() - start_time
                rate = (i + 1) / elapsed if elapsed > 0 else 0
                print(f"  Progress: {i + 1}/{scale}, {rate:.0f} vec/s")

        total_time = time.time() - start_time
        rate = scale / total_time if total_time > 0 else 0

        # Test search accuracy quickly
        search_start = time.time()
        query_vector = test_vectors[0][1]
        results = db.search(query_vector, limit=3)
        search_time = time.time() - search_start

        accuracy = (
            100.0 if len(results) > 0 and results[0].id == test_vectors[0][0] else 0.0
        )

        return {
            "scale": scale,
            "success": True,
            "total_time": total_time,
            "insertion_rate": rate,
            "search_time": search_time,
            "accuracy": accuracy,
        }

    except Exception as e:
        return {"scale": scale, "success": False, "error": str(e)}


def main():
    """Test optimized RoarGraph scaling"""
    print("🚀 Testing Optimized RoarGraph Implementation")
    print("=" * 60)
    print("Goal: Verify 600ms construction time is fixed")

    # Test the critical scales where performance exploded before
    test_scales = [500, 1000, 2000, 3000, 5000]

    results = []

    for scale in test_scales:
        result = test_construction_time(scale)
        results.append(result)

        if result["success"]:
            rate = result["insertion_rate"]
            total_time = result["total_time"]
            accuracy = result["accuracy"]

            print(
                f"✅ {scale:4d} vectors: {total_time:6.1f}s total, {rate:5.0f} vec/s, {accuracy:3.0f}% accuracy"
            )

            # Check if we fixed the performance explosion
            if scale >= 2000 and rate < 100:
                print(f"   ⚠️  Still slow at {scale} vectors")
            elif scale >= 2000 and rate >= 100:
                print(f"   🎉 PERFORMANCE IMPROVEMENT at {scale} vectors!")

        else:
            print(f"❌ {scale:4d} vectors: FAILED - {result['error']}")

    # Analysis
    successful_results = [r for r in results if r["success"]]

    if successful_results:
        rates = [r["insertion_rate"] for r in successful_results]
        times = [r["total_time"] for r in successful_results]

        print(f"\n📊 PERFORMANCE SUMMARY:")
        print(f"   Insertion rates: {min(rates):3.0f} - {max(rates):3.0f} vec/s")
        print(f"   Total times: {min(times):3.1f}s - {max(times):3.1f}s")

        # Check if we resolved the exponential scaling issue
        large_scale_results = [r for r in successful_results if r["scale"] >= 2000]
        if large_scale_results:
            large_rates = [r["insertion_rate"] for r in large_scale_results]
            if min(large_rates) >= 100:
                print(f"   🎉 SUCCESS: Large scale performance >100 vec/s")
            else:
                print(f"   ⚠️  Still have performance issues at large scale")

        # Look for exponential degradation
        if len(successful_results) >= 3:
            rate_500 = next(
                (r["insertion_rate"] for r in successful_results if r["scale"] == 500),
                None,
            )
            rate_2000 = next(
                (r["insertion_rate"] for r in successful_results if r["scale"] == 2000),
                None,
            )

            if rate_500 and rate_2000:
                degradation_factor = rate_500 / rate_2000
                print(f"   Performance degradation 500→2000: {degradation_factor:.1f}x")

                if degradation_factor < 10:
                    print(f"   ✅ FIXED: No more exponential degradation!")
                else:
                    print(f"   ❌ Still exponential degradation")

    print(f"\n🏁 Optimization test complete")


if __name__ == "__main__":
    main()
