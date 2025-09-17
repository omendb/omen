#!/usr/bin/env python3
"""
Quick Validation - Verify Segmented HNSW Claims
October 2025

Quick validation of our 19,477 vec/s performance claims with quality metrics.
Tests without requiring SIFT1M download.
"""

import numpy as np
import time
import sys
import os

# Add path to native module
engine_dir = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
sys.path.append(os.path.join(engine_dir, 'python', 'omendb'))

# Import native module
import native

def compute_recall_at_k(retrieved_ids, ground_truth_ids, k):
    """Compute recall@k metric"""
    if len(retrieved_ids) == 0 or len(ground_truth_ids) == 0:
        return 0.0

    retrieved_set = set(retrieved_ids[:k])
    ground_truth_set = set(ground_truth_ids[:k])

    intersection = retrieved_set.intersection(ground_truth_set)
    recall = len(intersection) / min(k, len(ground_truth_set))
    return recall

def create_realistic_test_data(n_vectors, dimension=128):
    """Create realistic test data similar to SIFT characteristics"""
    np.random.seed(42)  # Deterministic for comparison

    # Create clustered data similar to real image features
    n_clusters = max(10, n_vectors // 100)
    cluster_centers = np.random.randn(n_clusters, dimension).astype(np.float32)

    vectors = []
    cluster_assignments = []

    for i in range(n_vectors):
        # Assign to cluster
        cluster_id = i % n_clusters
        cluster_assignments.append(cluster_id)

        # Generate vector around cluster center with noise
        noise = np.random.randn(dimension) * 0.3
        vector = cluster_centers[cluster_id] + noise
        vectors.append(vector)

    vectors = np.array(vectors, dtype=np.float32)

    # Normalize to [0, 1] range (similar to normalized SIFT)
    vectors = (vectors - vectors.min()) / (vectors.max() - vectors.min())

    return vectors, cluster_assignments

def compute_ground_truth(vectors, queries, k=100):
    """Compute exact nearest neighbors as ground truth"""
    ground_truth = []

    for query in queries:
        # Compute exact distances
        distances = np.sum((vectors - query) ** 2, axis=1)
        # Get top k indices
        top_k_indices = np.argsort(distances)[:k]
        ground_truth.append(top_k_indices.tolist())

    return ground_truth

def test_performance_claims():
    """Test our specific performance claims"""
    print("="*80)
    print("🎯 QUICK VALIDATION: Segmented HNSW Performance Claims")
    print("="*80)

    # Test parameters from our claims
    test_configs = [
        {"size": 5000, "expected_rate": 1000, "threshold": "monolithic"},
        {"size": 10000, "expected_rate": 19477, "threshold": "segmented"},
        {"size": 20000, "expected_rate": 16661, "threshold": "segmented"},
        {"size": 50000, "expected_rate": 8682, "threshold": "segmented"},
    ]

    dimension = 128
    results = {}

    for config in test_configs:
        size = config["size"]
        expected_rate = config["expected_rate"]
        threshold_type = config["threshold"]

        print(f"\n🧪 Testing {size} vectors (expected: {expected_rate} vec/s - {threshold_type})")
        print("-" * 60)

        # Generate test data
        vectors, cluster_assignments = create_realistic_test_data(size, dimension)
        ids = [f'test_{i}' for i in range(size)]
        metadata = [{}] * size

        # Clear database
        native.clear_database()

        # Benchmark insertion
        start_time = time.time()
        insert_results = native.add_vector_batch(ids, vectors, metadata)
        end_time = time.time()

        duration = end_time - start_time
        actual_rate = size / duration if duration > 0 else 0

        # Performance analysis
        success = len(insert_results) == size
        rate_ratio = actual_rate / expected_rate if expected_rate > 0 else 0

        print(f"Expected rate: {expected_rate:,} vec/s")
        print(f"Actual rate:   {actual_rate:,.0f} vec/s")
        print(f"Ratio:         {rate_ratio:.2f}x {'✅' if rate_ratio >= 0.8 else '⚠️' if rate_ratio >= 0.5 else '❌'}")
        print(f"Segmented:     {'✅ Active' if actual_rate > 15000 else '➖ Monolithic'}")

        # Quick search test
        if success and size <= 20000:  # Skip search test for very large sizes
            query = vectors[0]
            search_start = time.time()
            search_results = native.search_vectors(query, 10, {})
            search_end = time.time()

            search_latency = (search_end - search_start) * 1000
            print(f"Search latency: {search_latency:.2f}ms")
            print(f"Results found:  {len(search_results)}")

        results[size] = {
            'expected_rate': expected_rate,
            'actual_rate': actual_rate,
            'ratio': rate_ratio,
            'success': success,
            'threshold_type': threshold_type
        }

    # Summary analysis
    print("\n" + "="*80)
    print("📊 VALIDATION SUMMARY")
    print("="*80)

    print(f"\n{'Size':<8} {'Expected':<12} {'Actual':<12} {'Ratio':<8} {'Status':<12} {'Type'}")
    print("-" * 70)

    validation_passed = True

    for size, result in results.items():
        expected = result['expected_rate']
        actual = result['actual_rate']
        ratio = result['ratio']
        threshold_type = result['threshold_type']

        if ratio >= 0.8:
            status = "✅ PASS"
        elif ratio >= 0.5:
            status = "⚠️  PARTIAL"
            validation_passed = False
        else:
            status = "❌ FAIL"
            validation_passed = False

        print(f"{size:<8} {expected:<12,} {actual:<12,.0f} {ratio:<8.2f} {status:<12} {threshold_type}")

    # Key findings
    print(f"\n🔍 KEY FINDINGS:")

    # Check segmented threshold
    segmented_results = {k: v for k, v in results.items() if k >= 10000}
    if segmented_results:
        avg_segmented_rate = np.mean([r['actual_rate'] for r in segmented_results.values()])
        print(f"• Average segmented rate: {avg_segmented_rate:,.0f} vec/s")

        best_result = max(segmented_results.items(), key=lambda x: x[1]['actual_rate'])
        best_size, best_data = best_result
        print(f"• Peak performance: {best_data['actual_rate']:,.0f} vec/s at {best_size} vectors")

    print(f"• Segmented architecture: {'✅ Working as expected' if validation_passed else '⚠️ Needs investigation'}")

    if validation_passed:
        print(f"\n🎯 VALIDATION RESULT: ✅ CLAIMS VERIFIED")
        print(f"   Performance matches expectations within acceptable margins")
    else:
        print(f"\n⚠️  VALIDATION RESULT: PARTIAL SUCCESS")
        print(f"   Some performance targets not met - investigation needed")

    return results

def test_quality_with_ground_truth():
    """Test search quality with computed ground truth"""
    print("\n" + "="*60)
    print("🔍 SEARCH QUALITY VALIDATION")
    print("="*60)

    # Test with manageable size for ground truth computation
    test_size = 5000
    dimension = 128
    n_queries = 20

    print(f"Testing {test_size} vectors with {n_queries} queries...")

    # Generate test data
    vectors, _ = create_realistic_test_data(test_size, dimension)
    queries = vectors[:n_queries].copy()  # Use subset as queries

    # Compute exact ground truth
    print("Computing exact ground truth...")
    ground_truth = compute_ground_truth(vectors, queries, k=100)

    # Build index
    ids = [f'quality_test_{i}' for i in range(test_size)]
    metadata = [{}] * test_size

    native.clear_database()
    start_time = time.time()
    native.add_vector_batch(ids, vectors, metadata)
    build_time = time.time() - start_time

    print(f"Index built in {build_time:.2f}s ({test_size/build_time:.0f} vec/s)")

    # Test search quality
    k_values = [1, 10, 50]
    recalls = {k: [] for k in k_values}
    search_times = []

    for i, query in enumerate(queries):
        start = time.time()
        results = native.search_vectors(query, max(k_values), {})
        end = time.time()

        search_times.append((end - start) * 1000)

        # Extract result indices
        retrieved_indices = []
        for result in results:
            if 'id' in result and result['id'].startswith('quality_test_'):
                idx = int(result['id'].split('_')[2])
                retrieved_indices.append(idx)

        # Compute recall for different k values
        for k in k_values:
            gt_k = ground_truth[i][:k]
            recall = compute_recall_at_k(retrieved_indices, gt_k, k)
            recalls[k].append(recall)

    # Average results
    avg_recalls = {k: np.mean(recalls[k]) * 100 for k in k_values}
    avg_search_time = np.mean(search_times)

    print(f"\n📊 Quality Results:")
    for k in k_values:
        print(f"Recall@{k:2d}: {avg_recalls[k]:5.1f}%")
    print(f"Search latency: {avg_search_time:.2f}ms")

    # Quality assessment
    quality_pass = avg_recalls[10] >= 90.0  # 90%+ recall@10
    print(f"\nQuality assessment: {'✅ EXCELLENT' if quality_pass else '⚠️ NEEDS IMPROVEMENT'}")

    return avg_recalls, avg_search_time

if __name__ == "__main__":
    print("🚀 Starting quick validation of segmented HNSW...")

    # Test 1: Performance claims validation
    performance_results = test_performance_claims()

    # Test 2: Quality validation
    quality_results = test_quality_with_ground_truth()

    print(f"\n" + "="*80)
    print("✅ QUICK VALIDATION COMPLETE")
    print("="*80)
    print("📋 Next steps:")
    print("• Download SIFT1M dataset for standard benchmarking")
    print("• Compare memory usage with competitors")
    print("• Test concurrent operations")
    print("• Validate on additional standard datasets")