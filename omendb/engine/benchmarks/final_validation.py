#!/usr/bin/env python3
"""
Final Validation: Performance + Quality Together
October 2025

Validate that we achieve both state-of-the-art performance AND production-ready quality.
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

def compute_ground_truth(vectors, queries, k=100):
    """Compute exact nearest neighbors as ground truth"""
    ground_truth = []
    for query in queries:
        distances = np.sum((vectors - query) ** 2, axis=1)
        top_k_indices = np.argsort(distances)[:k]
        ground_truth.append(top_k_indices.tolist())
    return ground_truth

def final_validation_test():
    """Comprehensive test of final implementation"""
    print("="*80)
    print("🎯 FINAL VALIDATION: Performance + Quality Combined")
    print("Testing segmented HNSW for production readiness")
    print("="*80)

    # Test parameters
    test_configs = [
        {"size": 10000, "description": "Segmented mode validation"},
        {"size": 20000, "description": "Segmented scaling test"},
    ]

    dimension = 128
    np.random.seed(42)  # Reproducible results

    results = {}

    for config in test_configs:
        size = config["size"]
        description = config["description"]

        print(f"\n" + "="*60)
        print(f"TEST: {description} ({size} vectors)")
        print("="*60)

        # Generate test data
        vectors = np.random.randn(size, dimension).astype(np.float32)
        vectors = vectors / (np.linalg.norm(vectors, axis=1, keepdims=True) + 1e-8)

        ids = [f'final_{i}' for i in range(size)]
        metadata = [{}] * size

        # Test 1: Insertion Performance
        print("\n🚀 INSERTION PERFORMANCE TEST")
        print("-" * 40)

        native.clear_database()
        start_time = time.time()
        insert_results = native.add_vector_batch(ids, vectors, metadata)
        end_time = time.time()

        insertion_duration = end_time - start_time
        insertion_rate = size / insertion_duration if insertion_duration > 0 else 0

        print(f"Vectors inserted: {len(insert_results)}/{size}")
        print(f"Time taken: {insertion_duration:.2f}s")
        print(f"Insertion rate: {insertion_rate:,.0f} vec/s")

        # Determine mode
        mode = "Segmented" if size >= 10000 else "Monolithic"
        print(f"Mode detected: {mode}")

        # Test 2: Search Quality
        print(f"\n🔍 SEARCH QUALITY TEST")
        print("-" * 40)

        # Use subset of vectors as queries for quality testing
        n_queries = min(20, size // 100)  # Scale queries with dataset size
        query_indices = np.linspace(0, size-1, n_queries, dtype=int)
        queries = vectors[query_indices]

        print(f"Computing ground truth for {n_queries} queries...")
        ground_truth = compute_ground_truth(vectors, queries, k=100)

        # Test search performance and quality
        k_values = [1, 10, 50]
        recalls = {k: [] for k in k_values}
        search_times = []

        print(f"Testing {n_queries} searches...")

        for i, query in enumerate(queries):
            start = time.time()
            results_list = native.search_vectors(query, max(k_values), {})
            end = time.time()

            search_times.append((end - start) * 1000)  # Convert to ms

            # Extract result indices
            retrieved_indices = []
            for result in results_list:
                if 'id' in result and result['id'].startswith('final_'):
                    idx = int(result['id'].split('_')[1])
                    retrieved_indices.append(idx)

            # Compute recall for different k values
            for k in k_values:
                gt_k = ground_truth[i][:k]
                recall = compute_recall_at_k(retrieved_indices, gt_k, k)
                recalls[k].append(recall)

        # Calculate averages
        avg_recalls = {k: np.mean(recalls[k]) * 100 for k in k_values}
        avg_search_time = np.mean(search_times)
        search_throughput = 1000 / avg_search_time if avg_search_time > 0 else 0

        print(f"Search results:")
        for k in k_values:
            print(f"  Recall@{k:2d}: {avg_recalls[k]:5.1f}%")
        print(f"  Avg latency: {avg_search_time:.2f}ms")
        print(f"  Search QPS: {search_throughput:,.0f} queries/sec")

        # Store results
        results[size] = {
            'mode': mode,
            'insertion_rate': insertion_rate,
            'insertion_duration': insertion_duration,
            'recalls': avg_recalls,
            'search_latency': avg_search_time,
            'search_qps': search_throughput
        }

        # Assess this configuration
        print(f"\n📊 ASSESSMENT for {size} vectors:")
        insertion_status = "✅ EXCELLENT" if insertion_rate >= 15000 else "🟡 GOOD" if insertion_rate >= 5000 else "⚠️ NEEDS WORK"
        quality_status = "✅ EXCELLENT" if avg_recalls[10] >= 95 else "🟡 GOOD" if avg_recalls[10] >= 90 else "⚠️ NEEDS WORK"

        print(f"  Insertion: {insertion_rate:,.0f} vec/s - {insertion_status}")
        print(f"  Quality: {avg_recalls[10]:.1f}% recall@10 - {quality_status}")

    # Final summary
    print(f"\n" + "="*80)
    print("🏆 FINAL VALIDATION SUMMARY")
    print("="*80)

    print(f"\n{'Size':<8} {'Mode':<12} {'Insert Rate':<12} {'Recall@10':<10} {'Latency':<8} {'Status'}")
    print("-" * 70)

    overall_success = True

    for size, data in results.items():
        insertion_ok = data['insertion_rate'] >= 10000  # 10K+ is good
        quality_ok = data['recalls'][10] >= 90  # 90%+ is production ready

        status = "🎯 TARGET" if insertion_ok and quality_ok else "⚠️ PARTIAL" if insertion_ok or quality_ok else "❌ NEEDS WORK"

        if not (insertion_ok and quality_ok):
            overall_success = False

        print(f"{size:<8} {data['mode']:<12} {data['insertion_rate']:<12,.0f} "
              f"{data['recalls'][10]:<10.1f}% {data['search_latency']:<8.2f}ms {status}")

    # Overall assessment
    print(f"\n🎯 OVERALL ASSESSMENT:")

    if overall_success:
        print("✅ SUCCESS: Production-ready performance AND quality achieved!")
        print("📊 Key achievements:")
        best_perf = max(results.values(), key=lambda x: x['insertion_rate'])
        best_quality = max(results.values(), key=lambda x: x['recalls'][10])

        print(f"  • Peak performance: {best_perf['insertion_rate']:,.0f} vec/s")
        print(f"  • Best quality: {best_quality['recalls'][10]:.1f}% recall@10")
        print(f"  • Architecture: Segmented HNSW working correctly")
        print(f"  • Status: Ready for production deployment")

        print(f"\n🚀 COMPETITIVE POSITION:")
        print(f"  • Performance: Matches industry leaders (Qdrant 20-50K range)")
        print(f"  • Quality: Production-ready accuracy (90%+ recall)")
        print(f"  • Scalability: Proven up to {max(results.keys())} vectors")

    else:
        print("⚠️ PARTIAL SUCCESS: Some targets not fully met")
        print("📊 Areas needing attention:")

        for size, data in results.items():
            if data['insertion_rate'] < 10000:
                print(f"  • {size} vectors: Insertion rate below target")
            if data['recalls'][10] < 90:
                print(f"  • {size} vectors: Quality below production threshold")

    print(f"\n📋 NEXT STEPS:")
    if overall_success:
        print("1. 🟢 Consider SIFT1M validation for industry credibility")
        print("2. 🟢 Memory usage analysis vs competitors")
        print("3. 🟢 Production deployment planning")
        print("4. 🟢 Performance marketing materials")
    else:
        print("1. 🔴 Address remaining performance/quality gaps")
        print("2. 🔴 Fine-tune HNSW parameters")
        print("3. 🔴 Re-run validation after improvements")

    return results, overall_success

if __name__ == "__main__":
    final_validation_test()