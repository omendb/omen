#!/usr/bin/env python3
"""
Systematic diagnosis of HNSW graph connectivity issues.
This will identify WHY the graph is not navigable and causing 1% recall.
"""

import sys
import os
sys.path.append('/Users/nick/github/omendb/core/omendb/engine/python')

import numpy as np
import time
import omendb.native as native

def diagnose_graph_connectivity():
    """Systematically diagnose graph connectivity issues."""

    print("🔬 HNSW GRAPH CONNECTIVITY DIAGNOSIS")
    print("=" * 60)

    # Build a small graph for detailed analysis
    n_vectors = 500
    dimension = 128
    np.random.seed(42)

    # Generate vectors in a simple pattern for predictable analysis
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

    # TEST 1: Entry Point Reachability
    print(f"\n🔍 TEST 1: ENTRY POINT REACHABILITY")
    print("=" * 40)

    # Try searches from different regions
    test_queries = [
        vectors[0],      # First vector
        vectors[100],    # Middle vector
        vectors[499],    # Last vector
        np.random.randn(dimension).astype(np.float32)  # Random vector
    ]

    query_names = ["First Vector", "Middle Vector", "Last Vector", "Random Vector"]

    for i, (query, name) in enumerate(zip(test_queries, query_names)):
        results = native.search_vectors(query.tolist(), 10, {})
        print(f"  {name}: Found {len(results)} results")

        if len(results) > 0:
            # Check if results make sense
            distances = []
            for result in results:
                try:
                    idx = int(result['id'].split('_')[1])
                    actual_dist = np.linalg.norm(vectors[idx] - query)
                    distances.append(actual_dist)
                except:
                    pass

            if distances:
                print(f"    Distance range: {min(distances):.3f} - {max(distances):.3f}")
            else:
                print(f"    ⚠️ Could not compute distances")
        else:
            print(f"    ❌ NO RESULTS FOUND - Graph disconnected!")

    # TEST 2: Cross-Distribution Search Quality
    print(f"\n🔍 TEST 2: CROSS-DISTRIBUTION SEARCH QUALITY")
    print("=" * 40)

    # Generate 20 truly random queries
    random_queries = np.random.randn(20, dimension).astype(np.float32)

    total_results = 0
    total_recall = 0.0
    no_result_count = 0

    for query in random_queries:
        # Ground truth
        distances = np.linalg.norm(vectors - query, axis=1)
        gt_indices = set(np.argsort(distances)[:10])

        # HNSW search
        results = native.search_vectors(query.tolist(), 10, {})
        total_results += len(results)

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

    avg_results = total_results / len(random_queries)
    avg_recall = total_recall / len(random_queries)

    print(f"  Average results returned: {avg_results:.1f}/10")
    print(f"  Queries with no results: {no_result_count}/{len(random_queries)}")
    print(f"  Average recall@10: {avg_recall:.1%}")

    # TEST 3: Brute Force Comparison
    print(f"\n🔍 TEST 3: BRUTE FORCE COMPARISON")
    print("=" * 40)

    # Test if brute force on the SAME data gives better recall
    sample_query = random_queries[0]

    # Ground truth (brute force)
    distances = np.linalg.norm(vectors - sample_query, axis=1)
    gt_indices = np.argsort(distances)[:10]

    print(f"  Ground truth top 10 distances:")
    for i, idx in enumerate(gt_indices):
        print(f"    {i+1}: Vector {idx}, distance {distances[idx]:.3f}")

    # HNSW search
    hnsw_results = native.search_vectors(sample_query.tolist(), 10, {})
    print(f"\n  HNSW search results:")
    if hnsw_results:
        for i, result in enumerate(hnsw_results):
            try:
                idx = int(result['id'].split('_')[1])
                actual_dist = np.linalg.norm(vectors[idx] - sample_query)
                is_in_gt = "✅" if idx in gt_indices else "❌"
                print(f"    {i+1}: Vector {idx}, distance {actual_dist:.3f} {is_in_gt}")
            except:
                print(f"    {i+1}: Invalid result")
    else:
        print(f"    ❌ NO RESULTS RETURNED")

    # ANALYSIS
    print(f"\n📊 CONNECTIVITY ANALYSIS")
    print("=" * 60)

    if no_result_count > 0:
        print(f"❌ CRITICAL: {no_result_count} queries returned NO results")
        print(f"   This indicates severe graph disconnection")
    elif avg_recall < 0.1:
        print(f"❌ CRITICAL: {avg_recall:.1%} recall indicates poor graph connectivity")
        print(f"   The graph exists but is not navigable")
    elif avg_recall < 0.5:
        print(f"⚠️ WARNING: {avg_recall:.1%} recall indicates connectivity issues")
        print(f"   Some regions of the graph are poorly connected")
    else:
        print(f"✅ GOOD: {avg_recall:.1%} recall indicates reasonable connectivity")

    # ROOT CAUSE HYPOTHESES
    print(f"\n🎯 ROOT CAUSE HYPOTHESES:")
    print("=" * 30)

    if no_result_count > 0:
        print("1. 🔥 ENTRY POINT FAILURE: Some queries cannot reach ANY nodes from entry point")
        print("   - Entry point may be isolated")
        print("   - Graph may have disconnected components")

    if avg_recall < 0.1:
        print("2. 🔥 POOR NEIGHBOR SELECTION: Graph structure doesn't reflect proximity")
        print("   - Pruning algorithm may be removing critical connections")
        print("   - Bidirectional connections may be inconsistent")
        print("   - Search may get trapped in local optima")

    if avg_results < 10:
        print("3. 🔥 SEARCH TERMINATION: Search terminates before finding enough candidates")
        print("   - beam width may be too small")
        print("   - visited set management may have bugs")

    return {
        'avg_recall': avg_recall,
        'no_result_queries': no_result_count,
        'avg_results': avg_results,
        'total_queries': len(random_queries)
    }

if __name__ == "__main__":
    try:
        results = diagnose_graph_connectivity()

        print(f"\n📈 SUMMARY METRICS:")
        print(f"  Recall: {results['avg_recall']:.1%}")
        print(f"  Failed queries: {results['no_result_queries']}/{results['total_queries']}")
        print(f"  Average results: {results['avg_results']:.1f}/10")

        # Next steps based on results
        if results['no_result_queries'] > 0:
            print(f"\n🔧 IMMEDIATE ACTION NEEDED:")
            print(f"  Fix entry point reachability and graph disconnection")
        elif results['avg_recall'] < 0.1:
            print(f"\n🔧 IMMEDIATE ACTION NEEDED:")
            print(f"  Fix neighbor selection and pruning algorithm")
        else:
            print(f"\n✅ Graph connectivity acceptable")

    except Exception as e:
        print(f"💥 Diagnosis failed: {e}")
        import traceback
        traceback.print_exc()