#!/usr/bin/env python3
"""
Test to identify the ROOT CAUSE of poor recall without band-aids.
We'll test each component systematically to find what's actually broken.
"""

import sys
import os
sys.path.append('/Users/nick/github/omendb/core/omendb/engine/python')

import numpy as np
import time
import omendb.native as native

def test_distance_calculation():
    """Test if distance calculations are correct."""
    print("\n🔬 TEST 1: Distance Calculation Correctness")
    print("=" * 50)

    # Create known vectors with known distances
    v1 = np.array([1.0, 0.0, 0.0], dtype=np.float32)
    v2 = np.array([0.0, 1.0, 0.0], dtype=np.float32)
    v3 = np.array([1.0, 1.0, 0.0], dtype=np.float32) / np.sqrt(2)

    # Expected distances (Euclidean)
    # v1 to v2: sqrt(2) ≈ 1.414
    # v1 to v3: sqrt((1-0.707)^2 + (0-0.707)^2) ≈ 0.765

    native.clear_database()
    native.add_vector("v1", v1.tolist(), {})
    native.add_vector("v2", v2.tolist(), {})
    native.add_vector("v3", v3.tolist(), {})

    # Search for v1
    results = native.search_vectors(v1.tolist(), 3, {})

    print(f"Query: v1 = {v1}")
    print(f"Results:")
    for r in results:
        print(f"  ID: {r['id']}, Distance: {r.get('distance', 'N/A')}")

    # Check if v1 is returned first (exact match)
    if results[0]['id'] != 'v1':
        print("❌ FAIL: Exact match not returned first!")
        return False

    print("✅ Distance calculation appears correct")
    return True

def test_graph_connectivity():
    """Test if all nodes are reachable in the graph."""
    print("\n🔬 TEST 2: Graph Connectivity")
    print("=" * 50)

    n_vectors = 100
    dimension = 4  # Small dimension for controlled test
    np.random.seed(42)

    # Create vectors in distinct clusters
    cluster1 = np.random.randn(50, dimension).astype(np.float32)
    cluster2 = np.random.randn(50, dimension).astype(np.float32) + 10  # Far away

    vectors = np.vstack([cluster1, cluster2])

    native.clear_database()
    ids = [f"v_{i}" for i in range(n_vectors)]
    metadata = [{}] * n_vectors
    native.add_vector_batch(ids, vectors, metadata)

    # Test searches from both clusters
    print("Testing cross-cluster search...")

    # Search from cluster1 for cluster2 neighbors
    query_c1 = cluster2[0]  # Query from cluster2
    results_c1 = native.search_vectors(query_c1.tolist(), 10, {})

    cluster2_found = sum(1 for r in results_c1 if int(r['id'].split('_')[1]) >= 50)

    print(f"Query from cluster2 space found {cluster2_found}/10 cluster2 neighbors")

    if cluster2_found < 8:
        print("❌ FAIL: Graph not well-connected between clusters!")
        return False

    print("✅ Graph connectivity appears good")
    return True

def test_neighbor_selection():
    """Test if neighbor selection during construction is working."""
    print("\n🔬 TEST 3: Neighbor Selection Quality")
    print("=" * 50)

    # Create a simple 2D dataset where we know the exact neighbors
    vectors = np.array([
        [0, 0],
        [1, 0],
        [0, 1],
        [1, 1],
        [0.5, 0.5],
        [2, 0],
        [0, 2],
        [2, 2]
    ], dtype=np.float32)

    native.clear_database()
    ids = [f"v_{i}" for i in range(len(vectors))]
    metadata = [{}] * len(vectors)
    native.add_vector_batch(ids, vectors, metadata)

    # The center point (v4 at [0.5, 0.5]) should find v0,v1,v2,v3 as nearest
    query = vectors[4]
    results = native.search_vectors(query.tolist(), 5, {})

    found_ids = [r['id'] for r in results]
    expected = ['v_4', 'v_0', 'v_1', 'v_2', 'v_3']  # Self + 4 corners

    matches = sum(1 for id in found_ids if id in expected)

    print(f"Query: v_4 at [0.5, 0.5]")
    print(f"Found: {found_ids}")
    print(f"Expected neighbors found: {matches}/5")

    if matches < 4:
        print("❌ FAIL: Neighbor selection not finding true neighbors!")
        return False

    print("✅ Neighbor selection appears correct")
    return True

def test_without_aggressive_search():
    """Test recall with NORMAL search parameters (not 10K candidates)."""
    print("\n🔬 TEST 4: Recall with Normal Parameters")
    print("=" * 50)

    print("This would require modifying the Mojo code to use normal ef_search.")
    print("Current implementation uses 10,000+ candidates (band-aid fix).")
    print("Proper HNSW should achieve 95% recall with ef_search=100-200.")

    # We can't test this without modifying the Mojo code
    return None

def main():
    """Run systematic tests to find root cause."""
    print("🔍 SYSTEMATIC ROOT CAUSE ANALYSIS")
    print("Finding the real problem without band-aids...")

    tests = [
        test_distance_calculation,
        test_graph_connectivity,
        test_neighbor_selection,
        test_without_aggressive_search
    ]

    results = []
    for test in tests:
        try:
            result = test()
            results.append(result)
        except Exception as e:
            print(f"💥 Test failed with error: {e}")
            results.append(False)

    print("\n" + "=" * 50)
    print("📊 ROOT CAUSE ANALYSIS SUMMARY")
    print("=" * 50)

    if results[0] == False:
        print("🔴 Distance calculation is broken - fix this first!")
    elif results[1] == False:
        print("🔴 Graph connectivity is broken - nodes can't reach each other!")
        print("   This is likely due to poor neighbor selection during construction.")
    elif results[2] == False:
        print("🔴 Neighbor selection is broken - not connecting true neighbors!")
        print("   The pruning algorithm needs to be fixed.")
    else:
        print("🟡 Basic components work, but recall is poor with normal parameters.")
        print("   The issue is likely in the search traversal or beam width.")

    print("\n📝 RECOMMENDATION:")
    print("1. Remove the 10,000 candidate band-aid")
    print("2. Fix the identified root cause")
    print("3. Use standard HNSW parameters (M=16, ef=200)")
    print("4. Achieve 95% recall legitimately")

if __name__ == "__main__":
    main()