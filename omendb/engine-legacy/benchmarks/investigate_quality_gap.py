#!/usr/bin/env python3
"""
Investigate Quality Gap in Segmented HNSW
October 2025

Systematic investigation of why segmented mode achieves 57% recall@10
while monolithic achieves ~95% on the same task.
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

def create_test_data(n_vectors, dimension=128, seed=42):
    """Create consistent test data"""
    np.random.seed(seed)
    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)
    # Normalize for more realistic similarity search
    vectors = vectors / (np.linalg.norm(vectors, axis=1, keepdims=True) + 1e-8)
    return vectors

def compute_ground_truth(vectors, queries, k=100):
    """Compute exact nearest neighbors as ground truth"""
    ground_truth = []

    print(f"Computing exact ground truth for {len(queries)} queries...")
    for i, query in enumerate(queries):
        if i % 10 == 0:
            print(f"  Query {i+1}/{len(queries)}")

        # Compute exact distances
        distances = np.sum((vectors - query) ** 2, axis=1)
        # Get top k indices
        top_k_indices = np.argsort(distances)[:k]
        ground_truth.append(top_k_indices.tolist())

    return ground_truth

def test_quality_with_size(vectors, test_name, expected_mode):
    """Test search quality for given vector set"""
    n_vectors = len(vectors)
    n_queries = min(50, n_vectors // 10)  # More queries for better statistics

    queries = vectors[:n_queries].copy()
    ground_truth = compute_ground_truth(vectors, queries, k=100)

    # Build index
    ids = [f'{test_name}_{i}' for i in range(n_vectors)]
    metadata = [{}] * n_vectors

    native.clear_database()
    print(f"\n🏗️  Building {test_name} index ({n_vectors} vectors, expected: {expected_mode})")
    start_time = time.time()
    native.add_vector_batch(ids, vectors, metadata)
    build_time = time.time() - start_time

    print(f"Built in {build_time:.2f}s ({n_vectors/build_time:.0f} vec/s)")

    # Test search quality
    k_values = [1, 10, 50]
    recalls = {k: [] for k in k_values}
    search_times = []

    print(f"Testing {n_queries} queries...")

    for i, query in enumerate(queries):
        if i % 20 == 0:
            print(f"  Query {i+1}/{n_queries}")

        start = time.time()
        results = native.search_vectors(query, max(k_values), {})
        end = time.time()

        search_times.append((end - start) * 1000)

        # Extract result indices
        retrieved_indices = []
        for result in results:
            if 'id' in result and result['id'].startswith(f'{test_name}_'):
                idx = int(result['id'].split('_')[1])
                retrieved_indices.append(idx)

        # Compute recall for different k values
        for k in k_values:
            gt_k = ground_truth[i][:k]
            recall = compute_recall_at_k(retrieved_indices, gt_k, k)
            recalls[k].append(recall)

    # Average results
    avg_recalls = {k: np.mean(recalls[k]) * 100 for k in k_values}
    avg_search_time = np.mean(search_times)

    print(f"📊 {test_name} Results ({expected_mode}):")
    print(f"  Build rate: {n_vectors/build_time:.0f} vec/s")
    for k in k_values:
        print(f"  Recall@{k:2d}: {avg_recalls[k]:5.1f}%")
    print(f"  Search latency: {avg_search_time:.2f}ms")

    return avg_recalls, avg_search_time, build_time

def investigate_quality_gap():
    """Systematic investigation of quality differences"""
    print("="*80)
    print("🔍 SYSTEMATIC QUALITY GAP INVESTIGATION")
    print("="*80)

    dimension = 128
    base_vectors = create_test_data(15000, dimension)  # Large enough for various tests

    results = {}

    # Test 1: Small batch (monolithic, high quality baseline)
    print("\n" + "="*60)
    print("TEST 1: Monolithic Baseline (5K vectors)")
    print("="*60)

    mono_vectors = base_vectors[:5000]
    mono_recalls, mono_latency, mono_build_time = test_quality_with_size(
        mono_vectors, "mono", "Monolithic"
    )
    results['monolithic_5k'] = {
        'recalls': mono_recalls,
        'latency': mono_latency,
        'build_time': mono_build_time,
        'size': 5000
    }

    # Test 2: Large batch (segmented, current issue)
    print("\n" + "="*60)
    print("TEST 2: Segmented Implementation (10K vectors)")
    print("="*60)

    seg_vectors = base_vectors[:10000]
    seg_recalls, seg_latency, seg_build_time = test_quality_with_size(
        seg_vectors, "seg", "Segmented"
    )
    results['segmented_10k'] = {
        'recalls': seg_recalls,
        'latency': seg_latency,
        'build_time': seg_build_time,
        'size': 10000
    }

    # Test 3: Same 5K subset in both modes (isolation test)
    print("\n" + "="*60)
    print("TEST 3: Same 5K vectors, but from larger set (should be monolithic)")
    print("="*60)

    subset_vectors = base_vectors[:5000]  # Same as Test 1, but run separately
    subset_recalls, subset_latency, subset_build_time = test_quality_with_size(
        subset_vectors, "subset", "Monolithic"
    )
    results['subset_5k'] = {
        'recalls': subset_recalls,
        'latency': subset_latency,
        'build_time': subset_build_time,
        'size': 5000
    }

    # Test 4: Force monolithic on 10K by temporarily modifying threshold
    print("\n" + "="*60)
    print("TEST 4: 10K vectors in monolithic mode (if possible)")
    print("="*60)
    print("NOTE: This would require temporarily raising the 10K threshold")
    print("Current implementation auto-switches to segmented at 10K+")

    # Analysis and comparison
    print("\n" + "="*80)
    print("📊 COMPREHENSIVE QUALITY ANALYSIS")
    print("="*80)

    print(f"\n{'Test':<20} {'Mode':<12} {'Size':<6} {'R@1':<6} {'R@10':<7} {'R@50':<7} {'Latency':<8} {'Build Rate'}")
    print("-" * 85)

    for test_name, data in results.items():
        mode = "Monolithic" if data['size'] < 10000 else "Segmented"
        rate = data['size'] / data['build_time']

        print(f"{test_name:<20} {mode:<12} {data['size']:<6} "
              f"{data['recalls'][1]:<6.1f} {data['recalls'][10]:<7.1f} {data['recalls'][50]:<7.1f} "
              f"{data['latency']:<8.2f} {rate:<.0f}")

    # Key insights
    print(f"\n🔍 KEY INSIGHTS:")

    mono_r10 = results['monolithic_5k']['recalls'][10]
    seg_r10 = results['segmented_10k']['recalls'][10]
    quality_gap = mono_r10 - seg_r10

    print(f"• Quality gap: {quality_gap:.1f} percentage points (Mono: {mono_r10:.1f}%, Seg: {seg_r10:.1f}%)")

    if quality_gap > 20:
        print("• 🔴 MAJOR ISSUE: Segmented mode has significantly worse quality")
        print("• Likely causes:")
        print("  - Bulk insertion creates suboptimal graph structure")
        print("  - Different HNSW parameters in segmented mode")
        print("  - Graph connectivity issues with bulk operations")
    elif quality_gap > 10:
        print("• 🟡 MODERATE ISSUE: Segmented mode needs quality tuning")
    else:
        print("• 🟢 MINOR ISSUE: Quality gap is within acceptable range")

    # Recommendations
    print(f"\n🎯 RECOMMENDATIONS:")
    if seg_r10 < 70:
        print("1. 🔴 CRITICAL: Switch segmented mode to individual insertion")
        print("2. 🔴 Debug bulk insertion graph construction")
        print("3. 🔴 Compare HNSW parameters between modes")
    elif seg_r10 < 85:
        print("1. 🟡 Tune HNSW parameters for segmented mode")
        print("2. 🟡 Investigate graph connectivity optimization")
        print("3. 🟡 Consider hybrid individual+bulk approach")
    else:
        print("1. 🟢 Fine-tune search parameters")
        print("2. 🟢 Optimize for specific use cases")

    return results

if __name__ == "__main__":
    investigate_quality_gap()