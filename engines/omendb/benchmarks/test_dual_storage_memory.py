#!/usr/bin/env python3
"""
Test Dual Storage Memory Usage
==============================

Test if dual storage (original + normalized vectors) uses 2x memory.
"""

import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')

import omendb

def test_dual_storage_memory():
    """Test if memory usage doubled due to dual storage."""
    
    print("🧪 Testing Dual Storage Memory Usage")
    print("=" * 40)
    
    # Create database and add vectors
    db = omendb.DB()
    
    num_vectors = 1000
    for i in range(num_vectors):
        vector = np.random.rand(128).astype(np.float32) * 10
        db.add(f"vec_{i}", vector)
    
    # Force flush to main index where dual storage happens
    db.flush()
    
    stats = db.get_memory_stats()
    
    print(f"Database count: {db.count()}")
    print(f"Total memory: {stats.get('total_mb', 0):.3f} MB")
    print(f"Graph memory: {stats.get('graph_mb', 0):.3f} MB")
    
    # Calculate expected memory for dual storage
    # 1000 vectors * 128 dims * 4 bytes * 2 (dual storage) = ~1.024 MB just for vectors
    expected_vector_mb = (num_vectors * 128 * 4 * 2) / (1024 * 1024)
    print(f"Expected vector memory (dual): {expected_vector_mb:.3f} MB")
    
    graph_memory = stats.get('graph_mb', 0)
    if graph_memory >= expected_vector_mb * 0.8:  # Allow some overhead
        print("✅ Memory usage suggests dual storage is working!")
        print(f"   Graph memory {graph_memory:.3f} MB >= Expected {expected_vector_mb:.3f} MB")
        return True
    else:
        print("❌ Memory usage too low for dual storage")
        print(f"   Graph memory {graph_memory:.3f} MB < Expected {expected_vector_mb:.3f} MB")
        return False

def test_search_still_works():
    """Test that search still uses normalized vectors correctly."""
    
    print("\n🔍 Testing Search with Dual Storage")
    print("=" * 35)
    
    db = omendb.DB()
    
    # Add the test vector [3, 4, 0, ...]  
    original = np.array([3.0, 4.0] + [0.0] * 126, dtype=np.float32)
    db.add("test", original)
    db.flush()
    
    # Search with [1, 0, 0, ...] should still give cosine similarity 0.6
    query = np.array([1.0, 0.0] + [0.0] * 126, dtype=np.float32)
    results = db.search(query, limit=1)
    
    if results and len(results) > 0:
        score = results[0].score
        print(f"Search score: {score:.6f}")
        
        if abs(score - 0.6) < 0.01:
            print("✅ Search still works with normalized vectors!")
            return True
        else:
            print(f"❌ Unexpected search score (expected ~0.6, got {score:.6f})")
            return False
    else:
        print("❌ Search failed")
        return False

if __name__ == "__main__":
    memory_ok = test_dual_storage_memory()
    search_ok = test_search_still_works()
    
    print("\n" + "=" * 50)
    print("DUAL STORAGE TEST RESULTS")
    print("=" * 50)
    
    if memory_ok:
        print("✅ Memory usage indicates dual storage is working")
    else:
        print("❌ Memory usage suggests dual storage is not working")
        
    if search_ok:
        print("✅ Search still uses normalized vectors correctly")
    else:
        print("❌ Search broken after dual storage implementation")
    
    if memory_ok and search_ok:
        print("\n🎉 DUAL STORAGE IMPLEMENTATION SUCCESS!")
        print("   - Vectors stored in both original and normalized form")
        print("   - Search uses normalized vectors (cosine similarity works)")
        print("   - Memory usage doubled as expected")
        print("\n🔧 Next: Fix get_vector retrieval to return original vectors")
    else:
        print("\n❌ Dual storage implementation needs debugging")