#!/usr/bin/env python3
"""
Test and validate the fix for HNSW graph connectivity issues.
This will test the specific fixes we need to implement.
"""

import sys
import os
sys.path.append('/Users/nick/github/omendb/core/omendb/engine/python')

import numpy as np
import time
import omendb.native as native

def test_small_scale_individual_insertion():
    """Test if individual insertion creates better connectivity."""

    print("🔬 TESTING INDIVIDUAL vs BULK INSERTION")
    print("=" * 50)

    # Small scale test to force individual insertion
    n_vectors = 100  # Small enough that should use individual insertion
    dimension = 128
    np.random.seed(42)

    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)

    native.clear_database()

    # Individual insertion
    print("Testing individual insertion...")
    start_time = time.time()

    for i, vector in enumerate(vectors):
        native.add_vector(f"test_{i:06d}", vector.tolist(), {})

    build_time = time.time() - start_time
    print(f"Individual: {build_time:.2f}s ({n_vectors/build_time:.0f} vec/s)")

    # Test search quality
    test_query = np.random.randn(dimension).astype(np.float32)
    results = native.search_vectors(test_query.tolist(), 10, {})

    print(f"Individual insertion results: {len(results)} vectors found")

    if len(results) > 0:
        # Calculate recall
        distances = np.linalg.norm(vectors - test_query, axis=1)
        gt_indices = set(np.argsort(distances)[:10])

        found_indices = set()
        for result in results:
            try:
                idx = int(result['id'].split('_')[1])
                found_indices.add(idx)
            except:
                pass

        recall = len(gt_indices.intersection(found_indices)) / 10
        print(f"Individual insertion recall: {recall:.1%}")

        # Show which nodes were found
        found_list = list(found_indices)
        found_list.sort()
        print(f"Found nodes: {found_list[:10]}")

        return recall

    return 0.0

def test_entry_point_progression():
    """Test how entry point changes during construction."""

    print("\n🔬 TESTING ENTRY POINT PROGRESSION")
    print("=" * 40)

    # We need to examine this by building incrementally
    n_vectors = 10
    dimension = 128
    np.random.seed(42)

    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)

    native.clear_database()

    # Add vectors one by one and check behavior
    for i in range(n_vectors):
        native.add_vector(f"test_{i:06d}", vectors[i].tolist(), {})

        # Test search from this point
        test_query = np.random.randn(dimension).astype(np.float32)
        results = native.search_vectors(test_query.tolist(), min(3, i+1), {})

        found_nodes = []
        for result in results:
            try:
                idx = int(result['id'].split('_')[1])
                found_nodes.append(idx)
            except:
                pass

        print(f"After {i+1} nodes: search finds {found_nodes}")

    return True

def identify_graph_structure_issues():
    """Try to identify what's wrong with the graph structure."""

    print("\n🔬 GRAPH STRUCTURE ANALYSIS")
    print("=" * 40)

    # Build a medium graph and analyze the results pattern
    n_vectors = 1000
    dimension = 128
    np.random.seed(42)

    vectors = np.random.randn(n_vectors, dimension).astype(np.float32)

    native.clear_database()
    ids = [f"test_{i:06d}" for i in range(n_vectors)]
    metadata = [{}] * n_vectors

    result = native.add_vector_batch(ids, vectors, metadata)

    # Test multiple queries and see the pattern
    print("Testing pattern of returned nodes:")

    node_frequency = {}

    for i in range(20):  # Test 20 random queries
        test_query = np.random.randn(dimension).astype(np.float32)
        results = native.search_vectors(test_query.tolist(), 10, {})

        for result in results:
            try:
                idx = int(result['id'].split('_')[1])
                node_frequency[idx] = node_frequency.get(idx, 0) + 1
            except:
                pass

    # Analyze frequency distribution
    if node_frequency:
        nodes = list(node_frequency.keys())
        nodes.sort()

        print(f"Nodes that appeared in searches: {len(nodes)}")
        print(f"Total nodes in graph: {n_vectors}")
        print(f"Coverage: {len(nodes)/n_vectors:.1%}")

        # Show most frequent nodes
        sorted_by_freq = sorted(node_frequency.items(), key=lambda x: x[1], reverse=True)
        print(f"Most frequent nodes:")
        for node, freq in sorted_by_freq[:10]:
            print(f"  Node {node}: appeared {freq} times")

        # Analyze the pattern
        low_nodes = sum(1 for node in nodes if node < 100)
        mid_nodes = sum(1 for node in nodes if 100 <= node < 500)
        high_nodes = sum(1 for node in nodes if node >= 500)

        print(f"Distribution:")
        print(f"  Low nodes (0-99): {low_nodes}")
        print(f"  Mid nodes (100-499): {mid_nodes}")
        print(f"  High nodes (500+): {high_nodes}")

        if low_nodes > high_nodes * 2:
            print("❌ BIAS DETECTED: Strong bias toward low-numbered nodes")
            print("   This suggests entry point bias or poor connectivity")
        else:
            print("✅ Distribution looks reasonable")

if __name__ == "__main__":
    try:
        # Test individual insertion quality
        individual_recall = test_small_scale_individual_insertion()

        # Test entry point behavior
        test_entry_point_progression()

        # Analyze overall graph structure
        identify_graph_structure_issues()

        print(f"\n📊 SUMMARY")
        print("=" * 30)
        print(f"Individual insertion recall: {individual_recall:.1%}")

        if individual_recall > 0.5:
            print("✅ Individual insertion works better - bulk insertion is the problem")
        else:
            print("❌ Both individual and bulk insertion have issues")

    except Exception as e:
        print(f"💥 Test failed: {e}")
        import traceback
        traceback.print_exc()