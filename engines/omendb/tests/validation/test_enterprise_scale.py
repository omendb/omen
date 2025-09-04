#!/usr/bin/env python3
"""
Enterprise Scale Testing for OmenDB
Test performance and stability at 1K, 10K, and potentially 100K vectors
"""

import time
import psutil
import gc
import sys
import os
from typing import List, Tuple, Dict
import random
import math

# Set up Python path for omendb
sys.path.insert(0, "/Users/nick/github/omenDB/python")


def get_memory_usage():
    """Get current memory usage in MB"""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024


def generate_test_vectors(
    count: int, dimension: int = 128
) -> List[Tuple[str, List[float]]]:
    """Generate test vectors with realistic distribution"""
    vectors = []
    for i in range(count):
        # Generate normalized random vectors (more realistic than uniform random)
        vector = [random.gauss(0, 1) for _ in range(dimension)]
        # Normalize to unit length
        norm = math.sqrt(sum(x * x for x in vector))
        if norm > 0:
            vector = [x / norm for x in vector]
        vectors.append((f"doc_{i:06d}", vector))
    return vectors


def test_scale_performance(target_count: int, dimension: int = 128) -> Dict:
    """Test performance at specified scale"""
    print(f"\n🎯 Testing {target_count:,} vectors (dimension {dimension})")

    # Memory baseline
    gc.collect()
    baseline_memory = get_memory_usage()
    print(f"Baseline memory: {baseline_memory:.1f} MB")

    # Import after memory baseline
    try:
        from omendb import DB

        print("✅ omendb imported successfully")
    except Exception as e:
        print(f"❌ Failed to import omendb: {e}")
        return {"status": "failed", "error": str(e)}

    db = DB()
    print("✅ Database created")

    # Generate test data
    print(f"Generating {target_count:,} test vectors...")
    start_gen = time.time()
    test_vectors = generate_test_vectors(target_count, dimension)
    gen_time = time.time() - start_gen
    print(f"✅ Generated vectors in {gen_time:.2f}s")

    # Measure insertion performance
    print(f"Inserting {target_count:,} vectors...")
    memory_before = get_memory_usage()
    start_insert = time.time()

    # Insert in batches to monitor progress
    batch_size = min(100, target_count // 10)
    if batch_size == 0:
        batch_size = 1

    for i, (doc_id, vector) in enumerate(test_vectors):
        try:
            success = db.add(doc_id, vector)
            if not success:
                print(f"❌ Failed to add vector {i}")
                return {"status": "failed", "error": f"Failed to add vector {i}"}

            # Progress reporting
            if i > 0 and (i + 1) % batch_size == 0:
                elapsed = time.time() - start_insert
                rate = (i + 1) / elapsed
                memory_current = get_memory_usage()
                print(
                    f"  Progress: {i + 1:,}/{target_count:,} vectors ({rate:.0f} vec/s, {memory_current:.1f} MB)"
                )

                # Check for memory issues
                if memory_current > baseline_memory + 2000:  # 2GB limit
                    print(f"⚠️  Memory usage high: {memory_current:.1f} MB")

        except Exception as e:
            print(f"❌ Error inserting vector {i}: {e}")
            return {"status": "failed", "error": f"Error inserting vector {i}: {e}"}

    insertion_time = time.time() - start_insert
    memory_after = get_memory_usage()

    print(f"✅ Inserted {target_count:,} vectors in {insertion_time:.2f}s")
    print(f"📊 Insertion rate: {target_count / insertion_time:.0f} vectors/second")
    print(
        f"💾 Memory usage: {memory_after:.1f} MB (delta: +{memory_after - baseline_memory:.1f} MB)"
    )

    # Test search accuracy with multiple queries
    print("\n🔍 Testing search accuracy...")

    # Use first few vectors as queries
    num_test_queries = min(10, target_count)
    accuracy_results = []
    search_times = []

    for i in range(num_test_queries):
        query_vector = test_vectors[i][1]
        expected_id = test_vectors[i][0]

        start_search = time.time()
        results = db.search(query_vector, limit=5)
        search_time = time.time() - start_search
        search_times.append(search_time)

        # Check if exact match is found
        exact_match_found = False
        best_similarity = 0
        if results:
            best_similarity = results[0].score
            exact_match_found = any(r.id == expected_id for r in results)

        accuracy_results.append(
            {
                "exact_match": exact_match_found,
                "best_similarity": best_similarity,
                "search_time_ms": search_time * 1000,
            }
        )

    # Calculate metrics
    exact_matches = sum(1 for r in accuracy_results if r["exact_match"])
    avg_similarity = sum(r["best_similarity"] for r in accuracy_results) / len(
        accuracy_results
    )
    avg_search_time = sum(search_times) / len(search_times) * 1000  # ms

    print(
        f"✅ Search accuracy: {exact_matches}/{num_test_queries} exact matches ({exact_matches / num_test_queries * 100:.1f}%)"
    )
    print(f"✅ Average similarity: {avg_similarity:.4f}")
    print(f"✅ Average search time: {avg_search_time:.2f} ms")

    # Test with random queries (realistic workload)
    print("\n🎲 Testing with random queries...")
    random_query_times = []
    for _ in range(5):
        # Generate random query vector
        random_vector = [random.gauss(0, 1) for _ in range(dimension)]
        norm = math.sqrt(sum(x * x for x in random_vector))
        if norm > 0:
            random_vector = [x / norm for x in random_vector]

        start_search = time.time()
        results = db.search(random_vector, limit=10)
        search_time = time.time() - start_search
        random_query_times.append(search_time * 1000)  # ms

    avg_random_search = sum(random_query_times) / len(random_query_times)
    print(f"✅ Random query average: {avg_random_search:.2f} ms")

    return {
        "status": "success",
        "vector_count": target_count,
        "dimension": dimension,
        "insertion_time_s": insertion_time,
        "insertion_rate_per_s": target_count / insertion_time,
        "memory_usage_mb": memory_after,
        "memory_delta_mb": memory_after - baseline_memory,
        "exact_match_rate": exact_matches / num_test_queries,
        "avg_similarity": avg_similarity,
        "avg_search_time_ms": avg_search_time,
        "avg_random_search_ms": avg_random_search,
        "accuracy_results": accuracy_results,
    }


def main():
    print("🚀 OmenDB Enterprise Scale Testing")
    print("=" * 50)

    # Test scales in order
    test_scales = [1000, 5000, 10000]
    results = {}

    for scale in test_scales:
        try:
            result = test_scale_performance(scale)
            results[scale] = result

            if result["status"] != "success":
                print(
                    f"❌ Testing failed at {scale:,} vectors: {result.get('error', 'Unknown error')}"
                )
                break
            else:
                # Summary
                print(f"\n📋 Scale {scale:,} Summary:")
                print(
                    f"  📊 Performance: {result['insertion_rate_per_s']:.0f} vectors/second"
                )
                print(
                    f"  🎯 Accuracy: {result['exact_match_rate'] * 100:.1f}% exact matches"
                )
                print(f"  ⚡ Search: {result['avg_search_time_ms']:.2f} ms average")
                print(f"  💾 Memory: {result['memory_delta_mb']:.1f} MB used")

                # Performance check
                if result["insertion_rate_per_s"] < 1000:
                    print(f"⚠️  Performance below 1K vectors/second threshold")
                if result["exact_match_rate"] < 0.8:
                    print(f"⚠️  Accuracy below 80% threshold")
                if result["avg_search_time_ms"] > 100:
                    print(f"⚠️  Search time above 100ms threshold")

        except KeyboardInterrupt:
            print(f"\n🛑 Testing interrupted by user")
            break
        except Exception as e:
            print(f"❌ Unexpected error testing {scale:,} vectors: {e}")
            import traceback

            traceback.print_exc()
            break

    # Final summary
    print(f"\n🏁 Enterprise Scale Testing Complete")
    print("=" * 50)

    if results:
        print("📊 Performance Summary:")
        for scale, result in results.items():
            if result["status"] == "success":
                print(
                    f"  {scale:,} vectors: {result['insertion_rate_per_s']:.0f} vec/s, "
                    f"{result['exact_match_rate'] * 100:.1f}% accuracy, "
                    f"{result['avg_search_time_ms']:.1f}ms search"
                )
            else:
                print(f"  {scale:,} vectors: FAILED")
    else:
        print("❌ No successful tests completed")


if __name__ == "__main__":
    main()
