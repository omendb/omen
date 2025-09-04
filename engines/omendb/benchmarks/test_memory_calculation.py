#!/usr/bin/env python3
"""
Test Memory Calculation
========================

Debug why memory stats are showing 0.000 MB even though quantization works.
"""

import sys
import numpy as np
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')

import omendb

def test_memory_calculation_detailed():
    """Test memory calculation in detail."""
    
    print("🔍 Detailed Memory Calculation Test")
    print("=" * 38)
    
    # Create database and add vectors
    db = omendb.DB()
    
    print("1. Adding 100 vectors without quantization...")
    for i in range(100):
        vector = np.random.rand(128).astype(np.float32)
        db.add(f"vec_{i}", vector)
    
    print(f"   Count before flush: {db.count()}")
    stats = db.get_memory_stats()
    print("   Memory stats before flush:")
    for key, value in stats.items():
        if key.endswith('_mb'):
            print(f"     {key}: {value:.6f} MB")
    
    print("\n2. Forcing flush to main index...")
    db.flush()
    
    print(f"   Count after flush: {db.count()}")
    stats = db.get_memory_stats()
    print("   Memory stats after flush:")
    for key, value in stats.items():
        if key.endswith('_mb'):
            print(f"     {key}: {value:.6f} MB")
    
    # Check specific memory components
    vectors_mb = stats.get('vectors_mb', 0)
    graph_mb = stats.get('graph_mb', 0)
    metadata_mb = stats.get('metadata_mb', 0)
    total_mb = stats.get('total_mb', 0)
    
    print(f"\n📊 Component Breakdown:")
    print(f"   Vectors:  {vectors_mb:.6f} MB")
    print(f"   Graph:    {graph_mb:.6f} MB")
    print(f"   Metadata: {metadata_mb:.6f} MB")
    print(f"   Total:    {total_mb:.6f} MB")
    
    # Calculate expected memory
    expected_vectors = 100 * 128 * 4 / (1024 * 1024)  # 100 vectors * 128 dims * 4 bytes
    expected_graph = 100 * 20 * 4 / (1024 * 1024)  # ~20 edges per node * 4 bytes
    
    print(f"\n📐 Expected Memory:")
    print(f"   Vectors:  {expected_vectors:.6f} MB")
    print(f"   Graph:    {expected_graph:.6f} MB (approx)")
    
    if total_mb > 0:
        print(f"\n✅ Memory tracking is working!")
        return True
    else:
        print(f"\n🔴 Memory tracking still shows 0")
        
        # Additional debugging - check all stats
        print("\n📋 All stats returned:")
        for key, value in stats.items():
            print(f"   {key}: {value}")
        
        return False

def test_with_more_vectors():
    """Test with more vectors to ensure it's not a threshold issue."""
    
    print("\n🔢 Testing with 1000 vectors")
    print("=" * 30)
    
    db = omendb.DB()
    
    # Add 1000 vectors (enough to trigger flush)
    for i in range(1000):
        vector = np.random.rand(128).astype(np.float32)
        db.add(f"vec_{i}", vector)
    
    db.flush()
    
    stats = db.get_memory_stats()
    total_mb = stats.get('total_mb', 0)
    
    print(f"Total memory for 1000 vectors: {total_mb:.6f} MB")
    
    if total_mb > 0:
        print("✅ Memory tracking works with more vectors")
        return True
    else:
        print("🔴 Memory tracking still broken even with 1000 vectors")
        return False

if __name__ == "__main__":
    detailed_working = test_memory_calculation_detailed()
    more_vectors_working = test_with_more_vectors()
    
    print("\n" + "=" * 50)
    print("MEMORY CALCULATION TEST RESULTS")
    print("=" * 50)
    
    if detailed_working or more_vectors_working:
        print("✅ Memory tracking has been fixed!")
    else:
        print("🔴 Memory tracking is still broken")
        print("\n🔍 Debugging Notes:")
        print("1. ComponentMemoryStats is being updated in diskann_csr")
        print("2. VectorDB.get_stats() now includes memory stats")
        print("3. Python API uses get_stats() correctly")
        print("4. But values are still 0 - check CSR graph memory_bytes()")