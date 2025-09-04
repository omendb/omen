#!/usr/bin/env python3
"""Test to verify double storage of vectors in both VectorStore and CSR graph."""

import numpy as np
import omendb
import psutil
import os
import gc

def get_memory_mb():
    """Get current process memory in MB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024

def test_vector_storage_locations():
    """Verify if vectors are stored in multiple places."""
    print("🔍 Testing for Double Vector Storage")
    print("=" * 60)
    
    db = omendb.DB()
    db._auto_batch_enabled = False
    dimension = 128
    
    # Baseline memory
    gc.collect()
    base_mem = get_memory_mb()
    print(f"Baseline memory: {base_mem:.2f} MB")
    
    # Add test vectors
    n_vectors = 1000
    print(f"\nAdding {n_vectors} vectors...")
    
    for i in range(n_vectors):
        vector = np.random.rand(dimension).astype(np.float32)
        db.add(f"vec_{i}", vector)
    
    gc.collect()
    after_add = get_memory_mb()
    mem_used = after_add - base_mem
    
    print(f"Memory after adding: {mem_used:.2f} MB")
    
    # Calculate expected memory
    vectors_size = n_vectors * dimension * 4 / (1024 * 1024)  # Float32
    id_size = n_vectors * 20 / (1024 * 1024)  # ~20 bytes per ID
    graph_edges = n_vectors * 32 * 4 / (1024 * 1024)  # 32 neighbors * Int32
    
    # If vectors are stored ONCE
    expected_single = vectors_size + id_size + graph_edges
    
    # If vectors are stored TWICE (VectorStore + CSR graph)
    expected_double = (vectors_size * 2) + id_size + graph_edges
    
    print(f"\nMemory Analysis:")
    print(f"  Expected if stored once: {expected_single:.2f} MB")
    print(f"  Expected if stored twice: {expected_double:.2f} MB")
    print(f"  Actual usage: {mem_used:.2f} MB")
    
    # Determine storage pattern
    ratio_single = mem_used / expected_single
    ratio_double = mem_used / expected_double
    
    print(f"\nStorage Detection:")
    print(f"  Ratio to single storage: {ratio_single:.2f}x")
    print(f"  Ratio to double storage: {ratio_double:.2f}x")
    
    if abs(ratio_double - 1.0) < abs(ratio_single - 1.0):
        print(f"\n🔴 DOUBLE STORAGE DETECTED!")
        print(f"   Vectors are stored in both VectorStore and CSR graph")
        print(f"   Wasting {vectors_size:.2f} MB ({vectors_size*1024:.0f} KB)")
    else:
        print(f"\n✅ Single storage pattern detected")
    
    # Get internal stats for verification
    stats = db.get_memory_stats()
    print(f"\nInternal Memory Stats:")
    for key, value in stats.items():
        if isinstance(value, float) and value > 0.001:
            print(f"  {key}: {value:.3f} MB")
    
    # Check specific components
    vectors_tracked = stats.get('vectors_mb', 0)
    graph_tracked = stats.get('graph_mb', 0)
    
    print(f"\nComponent Analysis:")
    print(f"  Vectors component: {vectors_tracked:.3f} MB")
    print(f"  Graph component: {graph_tracked:.3f} MB")
    print(f"  Sum of components: {vectors_tracked + graph_tracked:.3f} MB")
    
    # If vectors_tracked is close to expected_double - expected_single,
    # it confirms vectors are stored twice
    duplicate_size = vectors_size
    if abs(vectors_tracked - vectors_size*2) < abs(vectors_tracked - vectors_size):
        print(f"\n⚠️  Vectors component shows DOUBLE storage!")
        print(f"     Expected for {n_vectors} vectors: {vectors_size:.3f} MB")
        print(f"     Actual tracked: {vectors_tracked:.3f} MB")
        print(f"     Overhead: {(vectors_tracked/vectors_size - 1)*100:.0f}%")

def test_retrieval_source():
    """Test where vectors are retrieved from."""
    print("\n\n🔍 Testing Vector Retrieval Source")
    print("=" * 60)
    
    db = omendb.DB()
    db._auto_batch_enabled = False
    
    # Add a test vector
    test_vector = np.array([1.0] * 128, dtype=np.float32)
    db.add("test_id", test_vector)
    
    # Try to retrieve it
    result = db.get("test_id")
    
    if result is not None:
        print("✅ Vector retrieved successfully")
        
        # Check if modifying one storage affects the other
        # This would require internal access we don't have from Python
        print("\nNote: To fix double storage, we need to:")
        print("1. Remove self.vector_store[id] = vector from native.mojo")
        print("2. Retrieve vectors from CSR graph when needed")
        print("3. Update get() to use graph.get_vector_ptr(node_idx)")

if __name__ == "__main__":
    test_vector_storage_locations()
    test_retrieval_source()
    
    print("\n✅ Double storage test complete!")