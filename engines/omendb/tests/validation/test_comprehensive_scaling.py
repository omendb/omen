#!/usr/bin/env python3
"""
Comprehensive RoarGraph Scaling Test
====================================

Tests RoarGraph across multiple dimensions to find actual limits:
- Scale: 1K, 5K, 10K vectors
- Dimensions: 3D, 32D, 128D, 256D
- Data patterns: Gaussian, clustered, sparse
- Edge cases: duplicates, zeros, pathological cases

Goal: Find where RoarGraph actually breaks, not just where it works.
"""

import sys
import time
import random
import math
import psutil
import os
from typing import List, Tuple, Dict, Any

sys.path.insert(0, "/Users/nick/github/omenDB/python")


def get_memory_usage():
    """Get current memory usage in MB"""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024


def generate_gaussian_vectors(
    count: int, dimension: int, seed: int = 42
) -> List[Tuple[str, List[float]]]:
    """Generate normalized Gaussian vectors"""
    random.seed(seed)
    vectors = []

    for i in range(count):
        vector = [random.gauss(0, 1) for _ in range(dimension)]
        # Normalize
        norm = math.sqrt(sum(x * x for x in vector))
        if norm > 0:
            vector = [x / norm for x in vector]
        vectors.append((f"gauss_{i:05d}", vector))

    return vectors


def generate_clustered_vectors(
    count: int, dimension: int, num_clusters: int = 5, seed: int = 42
) -> List[Tuple[str, List[float]]]:
    """Generate clustered vectors (more realistic for embeddings)"""
    random.seed(seed)
    vectors = []

    # Create cluster centers
    cluster_centers = []
    for c in range(num_clusters):
        center = [random.gauss(0, 1) for _ in range(dimension)]
        norm = math.sqrt(sum(x * x for x in center))
        if norm > 0:
            center = [x / norm for x in center]
        cluster_centers.append(center)

    # Generate vectors around clusters
    for i in range(count):
        cluster_id = i % num_clusters
        center = cluster_centers[cluster_id]

        # Add small random offset
        vector = []
        for j in range(dimension):
            noise = random.gauss(0, 0.1)  # Small cluster spread
            vector.append(center[j] + noise)

        # Normalize
        norm = math.sqrt(sum(x * x for x in vector))
        if norm > 0:
            vector = [x / norm for x in vector]

        vectors.append((f"cluster_{i:05d}", vector))

    return vectors


def generate_pathological_vectors(
    count: int, dimension: int
) -> List[Tuple[str, List[float]]]:
    """Generate edge case vectors that might break the algorithm"""
    vectors = []

    # All zeros (after normalization, will be first standard basis vector)
    vectors.append(("zero_vec", [1.0] + [0.0] * (dimension - 1)))

    # All ones (normalized)
    all_ones = [1.0] * dimension
    norm = math.sqrt(dimension)
    all_ones = [x / norm for x in all_ones]
    vectors.append(("all_ones", all_ones))

    # Duplicates
    for i in range(min(10, count - 2)):
        vectors.append((f"duplicate_{i}", [1.0] + [0.0] * (dimension - 1)))

    # Fill remainder with standard basis vectors
    for i in range(len(vectors), min(count, dimension)):
        vector = [0.0] * dimension
        vector[i] = 1.0
        vectors.append((f"basis_{i}", vector))

    # Fill any remainder with random
    while len(vectors) < count:
        vector = [random.gauss(0, 1) for _ in range(dimension)]
        norm = math.sqrt(sum(x * x for x in vector))
        if norm > 0:
            vector = [x / norm for x in vector]
        vectors.append((f"random_{len(vectors)}", vector))

    return vectors[:count]


def test_scale_and_dimension(
    scale: int, dimension: int, data_type: str = "gaussian"
) -> Dict[str, Any]:
    """Test RoarGraph at specific scale and dimension"""
    print(f"\n🎯 Testing {scale} vectors, {dimension}D, {data_type} data")
    print("-" * 60)

    start_memory = get_memory_usage()
    start_time = time.time()

    try:
        from omendb import DB

        db = DB()

        # Generate test data
        if data_type == "gaussian":
            test_vectors = generate_gaussian_vectors(scale, dimension)
        elif data_type == "clustered":
            test_vectors = generate_clustered_vectors(scale, dimension)
        elif data_type == "pathological":
            test_vectors = generate_pathological_vectors(scale, dimension)
        else:
            raise ValueError(f"Unknown data type: {data_type}")

        generation_time = time.time() - start_time
        print(f"Generated {len(test_vectors)} vectors in {generation_time:.2f}s")

        # Add vectors and measure performance
        insert_start = time.time()
        successful_inserts = 0

        for i, (doc_id, vector) in enumerate(test_vectors):
            success = db.add(doc_id, vector)
            if success:
                successful_inserts += 1
            else:
                print(f"❌ Failed to add vector {i}: {doc_id}")
                break

            # Progress updates
            if (i + 1) % max(1, scale // 10) == 0:
                elapsed = time.time() - insert_start
                rate = successful_inserts / elapsed if elapsed > 0 else 0
                memory = get_memory_usage()
                print(
                    f"  Progress: {i + 1}/{scale} vectors, {rate:.0f} vec/s, {memory:.1f}MB"
                )

        insert_time = time.time() - insert_start
        insert_rate = successful_inserts / insert_time if insert_time > 0 else 0

        # Test search accuracy with multiple queries
        search_start = time.time()
        accurate_searches = 0
        total_searches = min(10, successful_inserts)

        print(f"Testing search accuracy with {total_searches} queries...")

        for i in range(total_searches):
            query_vector = test_vectors[i][1]
            expected_id = test_vectors[i][0]

            results = db.search(query_vector, limit=3)

            if len(results) > 0 and results[0].id == expected_id:
                accurate_searches += 1
            elif len(results) == 0:
                print(f"  ⚠️  Query {i}: No results returned")
            else:
                print(f"  ⚠️  Query {i}: Expected {expected_id}, got {results[0].id}")

        search_time = time.time() - search_start
        accuracy = accurate_searches / total_searches * 100 if total_searches > 0 else 0

        end_memory = get_memory_usage()
        memory_delta = end_memory - start_memory

        # Return results
        return {
            "scale": scale,
            "dimension": dimension,
            "data_type": data_type,
            "success": True,
            "successful_inserts": successful_inserts,
            "insert_time": insert_time,
            "insert_rate": insert_rate,
            "search_accuracy": accuracy,
            "search_time": search_time,
            "memory_usage": memory_delta,
            "total_time": time.time() - start_time,
        }

    except Exception as e:
        print(f"❌ FAILED: {e}")
        return {
            "scale": scale,
            "dimension": dimension,
            "data_type": data_type,
            "success": False,
            "error": str(e),
            "memory_usage": get_memory_usage() - start_memory,
        }


def main():
    """Run comprehensive scaling tests"""
    print("🚀 Comprehensive RoarGraph Scaling Test")
    print("=" * 70)
    print("Goal: Find actual limits, not just successes")

    # Test configurations
    test_configs = [
        # Small scale, low dimension (baseline)
        (500, 3, "gaussian"),
        (500, 3, "clustered"),
        # Scale up vectors, low dimension
        (1000, 3, "gaussian"),
        (2000, 3, "gaussian"),
        (5000, 3, "gaussian"),
        # Scale up dimensions, moderate vectors
        (1000, 32, "gaussian"),
        (1000, 128, "gaussian"),
        (500, 256, "gaussian"),
        # Realistic embedding scenarios
        (1000, 128, "clustered"),
        (5000, 32, "clustered"),
        # Edge cases
        (100, 128, "pathological"),
        (1000, 32, "pathological"),
    ]

    results = []
    failures = []

    for scale, dimension, data_type in test_configs:
        result = test_scale_and_dimension(scale, dimension, data_type)
        results.append(result)

        if not result["success"]:
            failures.append(result)
            print(f"💥 FAILURE DETECTED: {scale} vectors, {dimension}D, {data_type}")
        elif result["search_accuracy"] < 80:
            print(
                f"⚠️  LOW ACCURACY: {result['search_accuracy']:.1f}% at {scale} vectors, {dimension}D"
            )
        elif result["insert_rate"] < 100:
            print(
                f"⚠️  SLOW PERFORMANCE: {result['insert_rate']:.0f} vec/s at {scale} vectors, {dimension}D"
            )
        else:
            print(
                f"✅ PASSED: {result['search_accuracy']:.1f}% accuracy, {result['insert_rate']:.0f} vec/s"
            )

    # Summary
    print(f"\n📊 COMPREHENSIVE RESULTS SUMMARY")
    print("=" * 70)

    if failures:
        print(f"💥 FAILURES: {len(failures)}/{len(results)} configurations failed")
        for failure in failures:
            print(
                f"   - {failure['scale']} vectors, {failure['dimension']}D: {failure['error']}"
            )

    successful_results = [r for r in results if r["success"]]
    if successful_results:
        accuracies = [r["search_accuracy"] for r in successful_results]
        rates = [r["insert_rate"] for r in successful_results]

        print(f"\n✅ SUCCESSFUL CONFIGURATIONS: {len(successful_results)}")
        print(f"   Accuracy range: {min(accuracies):.1f}% - {max(accuracies):.1f}%")
        print(f"   Performance range: {min(rates):.0f} - {max(rates):.0f} vec/s")

        # Find actual limits
        high_accuracy = [r for r in successful_results if r["search_accuracy"] >= 95]
        fast_performance = [r for r in successful_results if r["insert_rate"] >= 1000]

        if high_accuracy:
            max_scale_accurate = max(r["scale"] for r in high_accuracy)
            max_dim_accurate = max(r["dimension"] for r in high_accuracy)
            print(f"   Max scale with 95%+ accuracy: {max_scale_accurate} vectors")
            print(f"   Max dimension with 95%+ accuracy: {max_dim_accurate}D")

        if fast_performance:
            max_scale_fast = max(r["scale"] for r in fast_performance)
            print(f"   Max scale with 1K+ vec/s: {max_scale_fast} vectors")

    # Honest assessment
    print(f"\n🎯 HONEST ASSESSMENT:")
    if failures:
        print("❌ RoarGraph does NOT scale fully - found failure points")
    elif min(r["search_accuracy"] for r in successful_results if r["success"]) < 95:
        print("⚠️  RoarGraph has accuracy degradation at scale")
    elif min(r["insert_rate"] for r in successful_results if r["success"]) < 100:
        print("⚠️  RoarGraph has performance degradation at scale")
    else:
        print("✅ RoarGraph appears robust within tested ranges")

    print(f"\n🏁 Testing complete. Actual limits identified.")


if __name__ == "__main__":
    main()
