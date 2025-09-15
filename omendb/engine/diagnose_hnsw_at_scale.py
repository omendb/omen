#!/usr/bin/env python3
"""
Force HNSW usage by testing at large scale to diagnose graph connectivity.
"""

import sys
import os
sys.path.append('/Users/nick/github/omendb/core/omendb/engine/python')

import numpy as np
import time
import omendb.native as native

def diagnose_hnsw_at_scale():
    """Test HNSW at scale where it's actually used (not flat buffer)."""

    print("🔬 FORCING HNSW USAGE AT SCALE")
    print("=" * 60)

    # Use 2000+ vectors to force HNSW usage
    n_vectors = 2000
    dimension = 128
    np.random.seed(42)

    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)

    # Build the graph
    native.clear_database()
    ids = [f"test_{i:06d}" for i in range(n_vectors)]
    metadata = [{}] * n_vectors

    print(f"📊 Building HNSW graph with {n_vectors} vectors...")
    start_time = time.time()
    result = native.add_vector_batch(ids, vectors, metadata)
    build_time = time.time() - start_time

    print(f"✅ Built in {build_time:.2f}s ({n_vectors/build_time:.0f} vec/s)")

    # TEST: Confirm we're using HNSW, not flat buffer
    print(f"\n🔍 CONFIRMING HNSW USAGE")
    print("=" * 30)

    sample_query = np.random.randn(dimension).astype(np.float32)
    print("Running search to check which algorithm is used...")
    results = native.search_vectors(sample_query.tolist(), 10, {})
    # The log message will tell us if it's using HNSW or flat buffer

    print(f"Results returned: {len(results)}")

    # TEST: Systematic recall measurement
    print(f"\n🔍 SYSTEMATIC RECALL MEASUREMENT")
    print("=" * 40)

    # Test multiple query types
    test_cases = [
        ("Self-search", vectors[np.random.choice(n_vectors, 10)]),
        ("Cross-distribution", np.random.randn(10, dimension).astype(np.float32)),
        ("Edge case", vectors[np.random.choice(n_vectors, 5)] + 0.1 * np.random.randn(5, dimension).astype(np.float32))
    ]

    for test_name, test_queries in test_cases:
        print(f"\n  {test_name} queries:")

        total_recall = 0.0
        no_result_count = 0

        for query in test_queries:
            # Ground truth
            distances = np.linalg.norm(vectors - query, axis=1)
            gt_indices = set(np.argsort(distances)[:10])

            # HNSW search
            results = native.search_vectors(query.tolist(), 10, {})

            if len(results) == 0:
                no_result_count += 1
                continue

            # Calculate actual recall
            found_indices = set()
            for result in results:
                try:
                    idx = int(result['id'].split('_')[1])
                    found_indices.add(idx)
                except:
                    pass

            recall = len(gt_indices.intersection(found_indices)) / 10
            total_recall += recall

        avg_recall = total_recall / len(test_queries) if len(test_queries) > 0 else 0
        print(f"    Recall@10: {avg_recall:.1%}")
        print(f"    Failed queries: {no_result_count}/{len(test_queries)}")

    # TEST: Single detailed example
    print(f"\n🔍 DETAILED SINGLE QUERY ANALYSIS")
    print("=" * 40)

    # Use a cross-distribution query for hardest case
    test_query = np.random.randn(dimension).astype(np.float32)

    # Ground truth (brute force)
    distances = np.linalg.norm(vectors - test_query, axis=1)
    gt_indices = np.argsort(distances)[:10]

    print(f"Ground truth top 10:")
    for i, idx in enumerate(gt_indices):
        print(f"  {i+1}: Vector {idx}, distance {distances[idx]:.3f}")

    # HNSW search
    hnsw_results = native.search_vectors(test_query.tolist(), 10, {})
    print(f"\nHNSW results:")
    if hnsw_results:
        hnsw_indices = []
        for i, result in enumerate(hnsw_results):
            try:
                idx = int(result['id'].split('_')[1])
                actual_dist = np.linalg.norm(vectors[idx] - test_query)
                is_in_gt = "✅" if idx in gt_indices else "❌"
                rank_in_gt = np.where(gt_indices == idx)[0]
                rank_str = f"(GT rank: {rank_in_gt[0]+1})" if len(rank_in_gt) > 0 else "(Not in GT top 10)"
                print(f"  {i+1}: Vector {idx}, distance {actual_dist:.3f} {is_in_gt} {rank_str}")
                hnsw_indices.append(idx)
            except:
                print(f"  {i+1}: Invalid result")

        # Calculate recall for this query
        gt_set = set(gt_indices)
        hnsw_set = set(hnsw_indices)
        recall = len(gt_set.intersection(hnsw_set)) / 10
        print(f"\nRecall for this query: {recall:.1%}")

        # Analyze the misses
        missed = gt_set - hnsw_set
        if missed:
            print(f"Missed vectors: {list(missed)}")
            print("Why these were missed:")
            for missed_idx in list(missed)[:3]:  # Show first 3
                missed_dist = distances[missed_idx]
                print(f"  Vector {missed_idx}: distance {missed_dist:.3f} (should be in top 10)")

    else:
        print(f"  ❌ NO RESULTS RETURNED")

    return len(hnsw_results) > 0

if __name__ == "__main__":
    try:
        has_results = diagnose_hnsw_at_scale()
        if not has_results:
            print(f"\n❌ CRITICAL: HNSW returns no results - severe connectivity problem")
        else:
            print(f"\n📊 HNSW is working but with poor recall - connectivity/pruning issue")
    except Exception as e:
        print(f"💥 Diagnosis failed: {e}")
        import traceback
        traceback.print_exc()