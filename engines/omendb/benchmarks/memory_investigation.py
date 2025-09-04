#!/usr/bin/env python3
"""Memory investigation - find where 778MB is being used for 100K vectors."""

import numpy as np
import omendb
import psutil
import os
import gc
import time

def get_memory_mb():
    """Get current process memory in MB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024

def profile_memory_usage():
    """Profile memory at each stage of database operations."""
    print("🔬 Memory Investigation - Finding the 778MB problem")
    print("=" * 60)
    
    # Baseline memory
    gc.collect()
    baseline_mem = get_memory_mb()
    print(f"Baseline memory: {baseline_mem:.2f} MB")
    
    # Create database
    db = omendb.DB()
    db._auto_batch_enabled = False  # Direct mode for accurate measurement
    
    after_db_mem = get_memory_mb()
    print(f"After DB creation: {after_db_mem:.2f} MB (+{after_db_mem - baseline_mem:.2f} MB)")
    
    # Test with different vector counts
    test_sizes = [100, 1000, 10000, 50000, 100000]
    dimension = 128
    
    for n_vectors in test_sizes:
        print(f"\n📊 Adding {n_vectors} vectors (128D)")
        
        # Clear database
        db.clear()
        gc.collect()
        start_mem = get_memory_mb()
        
        # Add vectors
        for i in range(n_vectors):
            vector = np.random.rand(dimension).astype(np.float32)
            db.add(f"vec_{i}", vector)
            
            # Check memory at intervals
            if (i + 1) % (n_vectors // 10) == 0 and i > 0:
                current_mem = get_memory_mb()
                vectors_added = i + 1
                mem_per_vector = (current_mem - start_mem) / vectors_added
                
                # Calculate expected memory
                expected_vector_mem = vectors_added * dimension * 4 / (1024 * 1024)  # Float32
                expected_id_mem = vectors_added * 20 / (1024 * 1024)  # ~20 bytes per ID
                expected_graph_mem = vectors_added * 32 * 4 / (1024 * 1024)  # 32 edges * 4 bytes
                expected_total = expected_vector_mem + expected_id_mem + expected_graph_mem
                
                print(f"  {vectors_added:6d} vectors: {current_mem - start_mem:7.2f} MB "
                      f"({mem_per_vector * 1024:.1f} KB/vec)")
                print(f"         Expected: {expected_total:.2f} MB "
                      f"(vectors={expected_vector_mem:.2f}, "
                      f"graph={expected_graph_mem:.2f})")
        
        # Final stats
        final_mem = get_memory_mb()
        total_used = final_mem - start_mem
        per_vector = total_used / n_vectors
        
        print(f"\n  Final stats for {n_vectors} vectors:")
        print(f"    Total memory: {total_used:.2f} MB")
        print(f"    Per vector: {per_vector * 1024:.2f} KB")
        print(f"    Efficiency: {(n_vectors * dimension * 4) / (total_used * 1024 * 1024) * 100:.1f}%")
        
        # Get internal stats
        stats = db.get_memory_stats()
        print(f"    Internal tracking:")
        for key, value in stats.items():
            if isinstance(value, float) and value > 0.001:
                print(f"      {key}: {value:.3f} MB")
        
        # Check for specific issues
        if n_vectors >= 10000:
            # Test if vectors are stored multiple times
            print(f"\n  🔍 Checking for double storage...")
            
            # Expected sizes
            vectors_size = n_vectors * dimension * 4 / (1024 * 1024)
            graph_size = n_vectors * 32 * 4 / (1024 * 1024)  # Sparse graph
            metadata_size = n_vectors * 100 / (1024 * 1024)  # IDs + metadata
            
            expected = vectors_size + graph_size + metadata_size
            actual = total_used
            
            print(f"    Expected total: {expected:.2f} MB")
            print(f"    Actual total: {actual:.2f} MB")
            print(f"    Unexplained: {actual - expected:.2f} MB ({(actual/expected - 1)*100:.0f}% overhead)")
            
            if actual > expected * 2:
                print(f"    ⚠️  MEMORY LEAK DETECTED: Using {actual/expected:.1f}x expected memory!")
                
                # Detailed breakdown
                print(f"\n    Theoretical breakdown:")
                print(f"      Vectors (128D float32): {vectors_size:.2f} MB")
                print(f"      Graph (32 edges Int32): {graph_size:.2f} MB")
                print(f"      Metadata (IDs+dict): {metadata_size:.2f} MB")
                print(f"      Total theoretical: {expected:.2f} MB")
                print(f"\n    Actual usage: {actual:.2f} MB")
                print(f"    🔴 Missing {actual - expected:.2f} MB!")

def check_data_structures():
    """Check if data structures are holding duplicate data."""
    print("\n\n🔍 Checking Data Structure Storage")
    print("=" * 60)
    
    db = omendb.DB()
    db._auto_batch_enabled = False
    
    # Add some test vectors
    n_test = 1000
    dimension = 128
    
    print(f"Adding {n_test} vectors to check storage...")
    
    for i in range(n_test):
        vector = np.random.rand(dimension).astype(np.float32)
        db.add(f"test_{i}", vector)
    
    # Check if vectors are stored in multiple places
    print("\n  Checking for duplicate storage patterns:")
    
    # Memory before and after operations
    gc.collect()
    mem_before = get_memory_mb()
    
    # Force operations that might duplicate
    db.flush()  # Force buffer flush
    
    mem_after_flush = get_memory_mb()
    print(f"    Memory after flush: +{mem_after_flush - mem_before:.2f} MB")
    
    # Try search to see if it creates copies
    query = np.random.rand(dimension).astype(np.float32)
    results = db.search(query, 10)
    
    mem_after_search = get_memory_mb()
    print(f"    Memory after search: +{mem_after_search - mem_after_flush:.2f} MB")
    
    # Get stats
    stats = db.get_memory_stats()
    total_tracked = sum(v for k, v in stats.items() if k.endswith('_mb') and isinstance(v, float))
    
    actual_mem = mem_after_search - mem_before
    print(f"\n  Summary:")
    print(f"    Total tracked by internal stats: {total_tracked:.2f} MB")
    print(f"    Actual process memory used: {actual_mem:.2f} MB")
    print(f"    Untracked memory: {actual_mem - total_tracked:.2f} MB")
    
    if actual_mem > total_tracked * 1.5:
        print(f"    ⚠️  TRACKING ISSUE: {actual_mem/total_tracked:.1f}x more memory than tracked!")

if __name__ == "__main__":
    profile_memory_usage()
    check_data_structures()
    
    print("\n✅ Memory investigation complete!")