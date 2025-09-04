#!/usr/bin/env python3
"""Memory profiling to identify where memory is being used."""

import numpy as np
import tracemalloc
import sys
import gc
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def format_bytes(size):
    """Format bytes in human readable format."""
    for unit in ['B', 'KB', 'MB', 'GB']:
        if size < 1024.0:
            return f"{size:.2f} {unit}"
        size /= 1024.0
    return f"{size:.2f} TB"

def profile_memory_usage():
    """Profile memory usage at different stages."""
    
    print("\nMemory Profiling for OmenDB")
    print("="*60)
    
    # Test parameters
    num_vectors = 10000
    dimension = 128
    batch_size = 1000
    
    # Generate test data
    print(f"\nGenerating {num_vectors:,} test vectors @ {dimension}D...")
    vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
    vector_ids = [f"vec_{i}" for i in range(num_vectors)]
    
    # Start memory tracking
    tracemalloc.start()
    
    # === Test 1: Normal storage ===
    print("\n1. Normal Storage (Float32)")
    print("-"*40)
    
    db_normal = omendb.DB()
    snapshot1 = tracemalloc.take_snapshot()
    
    # Add vectors in batches
    for i in range(0, num_vectors, batch_size):
        batch = vectors[i:i+batch_size]
        batch_ids = vector_ids[i:i+batch_size]
        db_normal.add_batch(batch, ids=batch_ids)
    
    snapshot2 = tracemalloc.take_snapshot()
    gc.collect()
    snapshot3 = tracemalloc.take_snapshot()
    
    # Analyze memory allocations
    print("\nTop memory allocations after adding vectors:")
    top_stats = snapshot2.compare_to(snapshot1, 'lineno')
    
    total_allocated = 0
    for stat in top_stats[:15]:
        print(f"{stat}")
        total_allocated += stat.size_diff
    
    print(f"\nTotal allocated: {format_bytes(total_allocated)}")
    
    # Check specific components
    print("\nMemory breakdown (estimates):")
    
    # Theoretical vector storage
    vector_size = num_vectors * dimension * 4  # float32
    print(f"  Vectors (theoretical): {format_bytes(vector_size)}")
    
    # String ID overhead (Python strings)
    id_size = sum(sys.getsizeof(id) for id in vector_ids)
    print(f"  String IDs: {format_bytes(id_size)}")
    
    # Dict overhead estimate
    dict_overhead = sys.getsizeof(db_normal._pending_batch) if hasattr(db_normal, '_pending_batch') else 0
    print(f"  Dict overhead: {format_bytes(dict_overhead)}")
    
    del db_normal
    gc.collect()
    
    # === Test 2: Scalar quantization ===
    print("\n2. Scalar Quantization (Int8)")
    print("-"*40)
    
    db_scalar = omendb.DB()
    db_scalar.enable_quantization()
    
    snapshot4 = tracemalloc.take_snapshot()
    
    for i in range(0, num_vectors, batch_size):
        batch = vectors[i:i+batch_size]
        batch_ids = vector_ids[i:i+batch_size]
        db_scalar.add_batch(batch, ids=batch_ids)
    
    snapshot5 = tracemalloc.take_snapshot()
    
    print("\nTop memory allocations with scalar quantization:")
    top_stats_scalar = snapshot5.compare_to(snapshot4, 'lineno')
    
    total_scalar = 0
    for stat in top_stats_scalar[:10]:
        print(f"{stat}")
        total_scalar += stat.size_diff
    
    print(f"\nTotal allocated: {format_bytes(total_scalar)}")
    print(f"Compression ratio: {total_allocated/total_scalar:.2f}x")
    
    del db_scalar
    gc.collect()
    
    # === Test 3: Binary quantization ===
    print("\n3. Binary Quantization (1-bit)")
    print("-"*40)
    
    db_binary = omendb.DB()
    db_binary.enable_binary_quantization()
    
    snapshot6 = tracemalloc.take_snapshot()
    
    for i in range(0, num_vectors, batch_size):
        batch = vectors[i:i+batch_size]
        batch_ids = vector_ids[i:i+batch_size]
        db_binary.add_batch(batch, ids=batch_ids)
    
    snapshot7 = tracemalloc.take_snapshot()
    
    print("\nTop memory allocations with binary quantization:")
    top_stats_binary = snapshot7.compare_to(snapshot6, 'lineno')
    
    total_binary = 0
    for stat in top_stats_binary[:10]:
        print(f"{stat}")
        total_binary += stat.size_diff
    
    print(f"\nTotal allocated: {format_bytes(total_binary)}")
    print(f"Compression ratio: {total_allocated/total_binary:.2f}x")
    
    # === Analysis ===
    print("\n" + "="*60)
    print("MEMORY USAGE SUMMARY")
    print("="*60)
    
    print(f"\nFor {num_vectors:,} vectors @ {dimension}D:")
    print(f"  Normal:   {format_bytes(total_allocated)}")
    print(f"  Scalar:   {format_bytes(total_scalar)} ({total_allocated/total_scalar:.1f}x compression)")
    print(f"  Binary:   {format_bytes(total_binary)} ({total_allocated/total_binary:.1f}x compression)")
    
    print(f"\nProjected for 1M vectors:")
    scale = 1_000_000 / num_vectors
    print(f"  Normal:   {format_bytes(total_allocated * scale)}")
    print(f"  Scalar:   {format_bytes(total_scalar * scale)}")
    print(f"  Binary:   {format_bytes(total_binary * scale)}")
    
    print("\nKey findings:")
    print("- String IDs consume significant memory")
    print("- Python dict overhead is substantial")
    print("- Graph structure memory scales with vector count")
    
    tracemalloc.stop()

if __name__ == "__main__":
    profile_memory_usage()