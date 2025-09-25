#!/usr/bin/env python3
"""
Debug the mixed mode bulk insertion connectivity issue.
"""

import sys
import numpy as np
sys.path.append('python/omendb')

def debug_mixed_mode_connectivity():
    """Debug why bulk vectors can't connect to existing nodes in mixed mode."""
    
    print("🔍 MIXED MODE CONNECTIVITY DEBUG")
    print("=" * 60)
    
    import native
    
    dimension = 768
    np.random.seed(42)  # Fixed seed for reproducibility
    
    # Clear and create test scenario that reproduces the issue
    native.clear_database()
    
    print("Phase 1: Add individual vectors to create initial graph")
    print("-" * 40)
    
    # Add individual vectors to create the initial graph structure
    individual_vectors = np.random.randn(50, dimension).astype(np.float32)
    for i in range(50):
        native.add_vector(f"individual_{i}", individual_vectors[i], {})
    
    # Add enough to trigger HNSW migration 
    print("Phase 2: Trigger HNSW migration")
    print("-" * 40)
    for i in range(450):
        filler = np.random.randn(dimension).astype(np.float32)
        native.add_vector(f"filler_{i}", filler, {})
    
    print(f"Initial graph has 500 vectors")
    
    # Test search in initial graph
    print("Phase 3: Test search in initial graph")
    print("-" * 40)
    query1 = individual_vectors[0]  # Query for a known individual vector
    results1 = native.search_vectors(query1, 5, {})
    print(f"Search for individual_0 found:")
    for i, result in enumerate(results1[:3]):
        print(f"  {i+1}. {result['id']} (distance: {result['distance']:.4f})")
    
    # Now add bulk vectors - this is where the issue occurs
    print("Phase 4: Add bulk vectors (problem area)")
    print("-" * 40)
    bulk_vectors = np.random.randn(10, dimension).astype(np.float32)  # Smaller for debugging
    bulk_ids = [f"bulk_{i}" for i in range(10)]
    
    print(f"Adding {len(bulk_ids)} bulk vectors to existing graph of 500 vectors...")
    result = native.add_vector_batch(bulk_ids, bulk_vectors, [{}] * 10)
    success_count = sum(1 for r in result if r)
    print(f"Successfully added: {success_count}/10 bulk vectors")
    
    # Test 1: Can bulk vectors find each other?
    print("Phase 5: Test bulk-to-bulk connectivity")
    print("-" * 40)
    query_bulk = bulk_vectors[0]
    results_bulk = native.search_vectors(query_bulk, 10, {})
    
    bulk_found = 0
    individual_found = 0
    for result in results_bulk:
        if result['id'].startswith('bulk_'):
            bulk_found += 1
        elif result['id'].startswith('individual_'):
            individual_found += 1
    
    print(f"Query for bulk_0 found:")
    print(f"  Bulk vectors: {bulk_found}")
    print(f"  Individual vectors: {individual_found}")
    for i, result in enumerate(results_bulk[:5]):
        print(f"  {i+1}. {result['id']} (distance: {result['distance']:.4f})")
    
    # Test 2: Can we still find individual vectors?
    print("Phase 6: Test individual-to-individual connectivity")
    print("-" * 40)
    query_ind = individual_vectors[0]
    results_ind = native.search_vectors(query_ind, 10, {})
    
    bulk_found_ind = 0
    individual_found_ind = 0
    for result in results_ind:
        if result['id'].startswith('bulk_'):
            bulk_found_ind += 1
        elif result['id'].startswith('individual_'):
            individual_found_ind += 1
    
    print(f"Query for individual_0 found:")
    print(f"  Bulk vectors: {bulk_found_ind}")
    print(f"  Individual vectors: {individual_found_ind}")
    for i, result in enumerate(results_ind[:5]):
        print(f"  {i+1}. {result['id']} (distance: {result['distance']:.4f})")
    
    print("\n" + "=" * 60)
    print("🔍 CONNECTIVITY ANALYSIS")
    print("=" * 60)
    
    if bulk_found > individual_found:
        print("❌ ISSUE CONFIRMED: Bulk vectors primarily find other bulk vectors")
        print("   This suggests bulk insertion creates isolated clusters")
    else:
        print("✅ MIXED CONNECTIVITY: Bulk vectors can reach individual vectors")
    
    if individual_found_ind > 0:
        print("✅ INDIVIDUAL SEARCH: Original vectors still reachable")
    else:
        print("❌ GRAPH CORRUPTED: Original vectors no longer reachable")
    
    # Cross-connectivity test
    print("\nPhase 7: Cross-connectivity test")
    print("-" * 40)
    
    # Can individual vector queries find bulk vectors?
    cross_bulk_found = 0
    for i in range(min(3, len(individual_vectors))):
        query = individual_vectors[i]
        results = native.search_vectors(query, 10, {})
        for result in results:
            if result['id'].startswith('bulk_'):
                cross_bulk_found += 1
                break
    
    print(f"Individual queries that found bulk vectors: {cross_bulk_found}/3")
    
    if cross_bulk_found == 0:
        print("❌ CRITICAL: Individual vectors cannot find bulk vectors")
        print("   Bulk vectors are completely isolated from original graph")
        return "isolated_clusters"
    elif cross_bulk_found < 2:
        print("⚠️  PARTIAL: Poor connectivity between individual and bulk vectors")
        return "poor_connectivity"
    else:
        print("✅ GOOD: Cross-connectivity working")
        return "good_connectivity"

if __name__ == "__main__":
    connectivity_status = debug_mixed_mode_connectivity()
    print(f"\n🎯 DIAGNOSIS: {connectivity_status}")