#!/usr/bin/env python3
"""Profile DiskANN graph structure memory usage."""

import numpy as np
import sys
import psutil
import os
import gc
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def get_memory_mb():
    """Get current process memory in MB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / (1024 * 1024)

def profile_graph_scaling():
    """Profile how graph memory scales with vector count."""
    
    print("DiskANN Graph Memory Profiling")
    print("=" * 60)
    
    # Test different vector counts
    vector_counts = [100, 500, 1000, 5000, 10000, 50000, 100000]
    dimension = 128
    
    results = []
    
    for count in vector_counts:
        # Fresh start
        gc.collect()
        
        # Create DB
        db = omendb.DB()
        
        # Measure baseline
        mem_before = get_memory_mb()
        
        # Add vectors
        print(f"\nAdding {count:,} vectors...")
        vectors = np.random.rand(count, dimension).astype(np.float32)
        
        for i in range(count):
            db.add(f"vec_{i}", vectors[i])
        
        # Force graph building if needed
        if hasattr(db, 'flush'):
            db.flush()
        
        # Measure after
        gc.collect()
        mem_after = get_memory_mb()
        
        # Get internal stats if available
        try:
            stats = db.get_memory_stats()
            graph_mem = stats.get('graph_memory', 0)
            vectors_mem = stats.get('vectors_memory', 0)
            metadata_mem = stats.get('metadata_memory', 0)
        except:
            graph_mem = 0
            vectors_mem = 0
            metadata_mem = 0
        
        total_mem = mem_after - mem_before
        
        results.append({
            'count': count,
            'total_mb': total_mem,
            'graph_mb': graph_mem,
            'vectors_mb': vectors_mem,
            'metadata_mb': metadata_mem,
            'per_vector_kb': (total_mem * 1024) / count if count > 0 else 0
        })
        
        print(f"  Total memory: {total_mem:.2f} MB")
        print(f"  Per vector: {results[-1]['per_vector_kb']:.2f} KB")
        if graph_mem > 0:
            print(f"  Graph memory: {graph_mem:.2f} MB")
        
        # Clean up
        del db
        del vectors
        gc.collect()
    
    # Analysis
    print("\n" + "=" * 60)
    print("ANALYSIS")
    print("=" * 60)
    
    print("\nMemory Scaling:")
    print(f"{'Vectors':<10} {'Total MB':<10} {'Per Vec KB':<12} {'Graph MB':<10}")
    print("-" * 45)
    for r in results:
        print(f"{r['count']:<10,} {r['total_mb']:<10.2f} {r['per_vector_kb']:<12.2f} {r['graph_mb']:<10.2f}")
    
    # Calculate theoretical graph size
    print("\n" + "=" * 60)
    print("THEORETICAL GRAPH MEMORY")
    print("=" * 60)
    
    R = 48  # Max degree from DiskANN
    
    for count in [1000, 10000, 100000, 1000000]:
        # Each node stores:
        # - neighbors: List[Int] with up to R entries
        # - reverse_neighbors: List[Int] with variable entries
        # Assume average degree is R/2 for both
        
        # List overhead in Mojo is typically 24 bytes header + data
        list_header = 24
        int_size = 8  # Int in Mojo is typically 8 bytes
        
        # Per node:
        # - neighbors list: header + R * int_size
        # - reverse_neighbors list: header + (R/2) * int_size (average)
        neighbors_memory = list_header + R * int_size
        reverse_neighbors_memory = list_header + (R // 2) * int_size
        
        per_node = neighbors_memory + reverse_neighbors_memory
        total_mb = (count * per_node) / (1024 * 1024)
        
        print(f"\n{count:,} vectors:")
        print(f"  Neighbors list: {neighbors_memory} bytes/node")
        print(f"  Reverse neighbors: {reverse_neighbors_memory} bytes/node")
        print(f"  Total per node: {per_node} bytes")
        print(f"  Total graph: {total_mb:.2f} MB")
        print(f"  Per vector: {per_node / 1024:.2f} KB")
    
    # Optimization opportunities
    print("\n" + "=" * 60)
    print("OPTIMIZATION OPPORTUNITIES")
    print("=" * 60)
    
    print("\n1. Sparse Graph Representation:")
    print("   - Most nodes don't use full R=48 neighbors")
    print("   - Could use dynamic arrays or linked lists")
    print("   - Estimated savings: 30-50%")
    
    print("\n2. Compressed Indices:")
    print("   - Use Int32 instead of Int64 for indices")
    print("   - Or use variable-length encoding")
    print("   - Estimated savings: 50% on index storage")
    
    print("\n3. Edge Deduplication:")
    print("   - Store edges once, not twice (forward + reverse)")
    print("   - Use bidirectional edge list")
    print("   - Estimated savings: 40-45%")
    
    print("\n4. Hierarchical Graph:")
    print("   - Use multiple levels with different R values")
    print("   - Sparse at top, dense at bottom")
    print("   - Estimated savings: 20-30%")

if __name__ == "__main__":
    profile_graph_scaling()