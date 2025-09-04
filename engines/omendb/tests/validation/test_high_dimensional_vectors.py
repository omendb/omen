#!/usr/bin/env python3
"""
High-Dimensional Vector Validation for RoarGraph
===============================================

Tests RoarGraph performance and accuracy with realistic embedding dimensions:
- 128D vectors (Sentence-BERT, many LLMs)
- 256D vectors (OpenAI text-embedding-ada-002)
- 512D vectors (Large language models)

Goal: Validate that our scalability breakthrough extends to real-world embedding dimensions
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


def generate_realistic_embeddings(
    count: int, dimension: int, seed: int = 42
) -> List[Tuple[str, List[float]]]:
    """Generate realistic embedding-like vectors with clustered distributions"""
    random.seed(seed)
    vectors = []

    # Create realistic clustering pattern (simulates semantic clustering in embeddings)
    num_clusters = max(3, count // 50)  # More realistic cluster count
    cluster_centers = []

    # Generate cluster centers with realistic sparse patterns
    for c in range(num_clusters):
        center = []
        for d in range(dimension):
            # Realistic embeddings are often sparse with important dimensions
            if random.random() < 0.1:  # 10% of dimensions are "important"
                center.append(random.gauss(0, 1.0))  # Strong signal
            else:
                center.append(random.gauss(0, 0.1))  # Background noise

        # Normalize to unit sphere (like real embeddings)
        norm = math.sqrt(sum(x * x for x in center))
        if norm > 0:
            center = [x / norm for x in center]
        cluster_centers.append(center)

    # Generate vectors around clusters
    for i in range(count):
        cluster_id = i % num_clusters
        center = cluster_centers[cluster_id]

        # Add realistic noise pattern
        vector = []
        for d in range(dimension):
            # Cluster variation with realistic noise distribution
            noise_scale = 0.05 if random.random() < 0.1 else 0.01  # Variable noise
            noise = random.gauss(0, noise_scale)
            vector.append(center[d] + noise)

        # Normalize (critical for cosine similarity)
        norm = math.sqrt(sum(x * x for x in vector))
        if norm > 0:
            vector = [x / norm for x in vector]

        vectors.append((f"embed_{dimension}d_{i:05d}", vector))

    return vectors


def test_high_dimensional_performance(dimension: int, scale: int) -> Dict[str, Any]:
    """Test RoarGraph at specific dimension and scale"""
    print(f"\n🎯 Testing {scale} vectors, {dimension}D (realistic embeddings)")
    print("-" * 60)

    start_memory = get_memory_usage()
    start_time = time.time()

    try:
        from omendb import DB

        db = DB()

        # Generate realistic embedding data
        print("Generating realistic embedding vectors...")
        test_vectors = generate_realistic_embeddings(scale, dimension)
        generation_time = time.time() - start_time

        print(
            f"Generated {len(test_vectors)} {dimension}D vectors in {generation_time:.2f}s"
        )

        # Memory check after generation
        gen_memory = get_memory_usage()
        print(
            f"Memory after generation: {gen_memory:.1f}MB (+{gen_memory - start_memory:.1f}MB)"
        )

        # Test insertion performance
        insert_start = time.time()
        successful_inserts = 0

        for i, (doc_id, vector) in enumerate(test_vectors):
            success = db.add(doc_id, vector)
            if success:
                successful_inserts += 1
            else:
                print(f"❌ Failed to add vector {i}: {doc_id}")
                break

            # Progress updates for large dimensions
            if (i + 1) % max(1, scale // 5) == 0:
                elapsed = time.time() - insert_start
                rate = successful_inserts / elapsed if elapsed > 0 else 0
                memory = get_memory_usage()
                print(
                    f"  Progress: {i + 1}/{scale} vectors, {rate:.0f} vec/s, {memory:.1f}MB"
                )

        insert_time = time.time() - insert_start
        insert_rate = successful_inserts / insert_time if insert_time > 0 else 0
        insert_memory = get_memory_usage()

        print(
            f"Insertion complete: {successful_inserts} vectors in {insert_time:.1f}s ({insert_rate:.0f} vec/s)"
        )
        print(
            f"Memory usage: {insert_memory:.1f}MB (+{insert_memory - start_memory:.1f}MB)"
        )

        # Test search accuracy with realistic queries
        search_start = time.time()
        accurate_searches = 0
        total_searches = min(5, successful_inserts)

        print(f"Testing search accuracy with {total_searches} queries...")

        for i in range(total_searches):
            query_vector = test_vectors[i][1]
            expected_id = test_vectors[i][0]

            results = db.search(query_vector, limit=3)

            if len(results) > 0:
                top_result = results[0]
                if top_result.id == expected_id:
                    accurate_searches += 1
                    print(
                        f"  ✅ Query {i}: exact match (similarity: {top_result.score:.4f})"
                    )
                else:
                    print(
                        f"  ❌ Query {i}: expected {expected_id}, got {top_result.id} (sim: {top_result.score:.4f})"
                    )
            else:
                print(f"  ❌ Query {i}: No results returned")

        search_time = time.time() - search_start
        accuracy = accurate_searches / total_searches * 100 if total_searches > 0 else 0
        final_memory = get_memory_usage()

        return {
            "dimension": dimension,
            "scale": scale,
            "success": True,
            "generation_time": generation_time,
            "successful_inserts": successful_inserts,
            "insert_time": insert_time,
            "insert_rate": insert_rate,
            "search_accuracy": accuracy,
            "search_time": search_time,
            "memory_usage": final_memory - start_memory,
            "total_time": time.time() - start_time,
        }

    except Exception as e:
        print(f"❌ FAILED: {e}")
        import traceback

        traceback.print_exc()
        return {
            "dimension": dimension,
            "scale": scale,
            "success": False,
            "error": str(e),
            "memory_usage": get_memory_usage() - start_memory,
        }


def main():
    """Run comprehensive high-dimensional validation"""
    print("🚀 High-Dimensional Vector Validation for RoarGraph")
    print("=" * 70)
    print("Testing realistic embedding dimensions with our scalability breakthrough")

    # Test configurations: (dimension, scale)
    test_configs = [
        # Start with smaller scale to validate dimensions work
        (32, 500),  # Small dimension baseline
        (128, 500),  # Sentence-BERT size
        (256, 500),  # OpenAI text-embedding-ada-002 size
        (512, 200),  # Large LLM embeddings (reduced scale)
        # Scale up if dimensions work
        (128, 1000),  # Realistic scale for 128D
        (256, 1000),  # Realistic scale for 256D
        # Large scale test (if memory allows)
        (128, 2000),  # Stress test
    ]

    results = []
    failures = []

    for dimension, scale in test_configs:
        print(f"\n{'=' * 70}")
        result = test_high_dimensional_performance(dimension, scale)
        results.append(result)

        if not result["success"]:
            failures.append(result)
            print(
                f"💥 FAILURE: {dimension}D, {scale} vectors - {result.get('error', 'Unknown error')}"
            )

            # Stop testing larger scales if we hit memory/performance limits
            if (
                "memory" in result.get("error", "").lower()
                or "timeout" in result.get("error", "").lower()
            ):
                print("⚠️  Stopping further tests due to resource constraints")
                break

        elif result["search_accuracy"] < 80:
            print(
                f"⚠️  LOW ACCURACY: {result['search_accuracy']:.1f}% at {dimension}D, {scale} vectors"
            )
        elif result["insert_rate"] < 100:
            print(
                f"⚠️  SLOW PERFORMANCE: {result['insert_rate']:.0f} vec/s at {dimension}D, {scale} vectors"
            )
        else:
            print(
                f"✅ SUCCESS: {result['search_accuracy']:.1f}% accuracy, {result['insert_rate']:.0f} vec/s, {result['memory_usage']:.1f}MB"
            )

    # Analysis
    print(f"\n📊 HIGH-DIMENSIONAL VALIDATION RESULTS")
    print("=" * 70)

    if failures:
        print(f"💥 FAILURES: {len(failures)}/{len(results)} configurations failed")
        for failure in failures:
            print(
                f"   - {failure['dimension']}D, {failure['scale']} vectors: {failure.get('error', 'Unknown')}"
            )

    successful_results = [r for r in results if r["success"]]
    if successful_results:
        accuracies = [r["search_accuracy"] for r in successful_results]
        rates = [r["insert_rate"] for r in successful_results]
        memories = [r["memory_usage"] for r in successful_results]

        print(f"\n✅ SUCCESSFUL CONFIGURATIONS: {len(successful_results)}")
        print(
            f"   Dimensions tested: {sorted(set(r['dimension'] for r in successful_results))}"
        )
        print(f"   Accuracy range: {min(accuracies):.1f}% - {max(accuracies):.1f}%")
        print(f"   Performance range: {min(rates):.0f} - {max(rates):.0f} vec/s")
        print(f"   Memory usage range: {min(memories):.1f} - {max(memories):.1f} MB")

        # Dimension scaling analysis
        dimensions_tested = sorted(set(r["dimension"] for r in successful_results))
        print(f"\n📊 DIMENSION SCALING ANALYSIS:")
        for dim in dimensions_tested:
            dim_results = [r for r in successful_results if r["dimension"] == dim]
            if dim_results:
                max_scale = max(r["scale"] for r in dim_results)
                best_result = max(dim_results, key=lambda x: x["scale"])
                print(
                    f"   {dim:3d}D: max {max_scale:4d} vectors, {best_result['insert_rate']:4.0f} vec/s, {best_result['search_accuracy']:3.0f}% accuracy"
                )

        # Memory scaling analysis
        print(f"\n📊 MEMORY SCALING:")
        for result in successful_results:
            mem_per_vector = (
                result["memory_usage"] / result["scale"] if result["scale"] > 0 else 0
            )
            print(
                f"   {result['dimension']:3d}D, {result['scale']:4d} vectors: {mem_per_vector:.2f} MB/vector"
            )

    # Honest assessment
    print(f"\n🎯 HONEST HIGH-DIMENSIONAL ASSESSMENT:")
    if failures:
        print("❌ RoarGraph has limitations with high-dimensional vectors")
        failure_dims = set(f["dimension"] for f in failures)
        print(f"   Failed dimensions: {sorted(failure_dims)}")
    elif min(r["search_accuracy"] for r in successful_results if r["success"]) < 95:
        print("⚠️  RoarGraph has accuracy degradation in high dimensions")
    elif min(r["insert_rate"] for r in successful_results if r["success"]) < 100:
        print("⚠️  RoarGraph has performance degradation in high dimensions")
    else:
        print("✅ RoarGraph handles high-dimensional vectors successfully!")
        successful_dims = sorted(set(r["dimension"] for r in successful_results))
        print(f"   Validated dimensions: {successful_dims}")
        print(f"   Maintains both accuracy and performance across dimensions")

    print(f"\n🏁 High-dimensional validation complete!")


if __name__ == "__main__":
    main()
