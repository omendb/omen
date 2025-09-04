#!/usr/bin/env python3
"""
Memory quantization benchmark for OmenDB.
Tests scalar quantization (int8) memory savings and performance impact.
"""

import numpy as np
import time
import psutil
import os
import sys
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def get_memory_usage():
    """Get current process memory usage in MB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / (1024 * 1024)

def benchmark_memory_usage(num_vectors=100000, dimension=128):
    """Benchmark memory usage with and without quantization."""
    
    print(f"\n{'='*60}")
    print(f"Memory Quantization Benchmark")
    print(f"Vectors: {num_vectors:,} | Dimension: {dimension}")
    print(f"{'='*60}")
    
    # Generate test data
    print("\nGenerating test vectors...")
    vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
    queries = np.random.rand(10, dimension).astype(np.float32)
    
    # Test 1: Without quantization
    print("\n1. Testing WITHOUT quantization...")
    db_normal = omendb.DB()
    
    start_mem = get_memory_usage()
    start_time = time.perf_counter()
    
    # Add vectors in batches
    batch_size = 10000
    for i in range(0, num_vectors, batch_size):
        batch = vectors[i:i+batch_size]
        db_normal.add_batch(batch)
    
    insert_time_normal = time.perf_counter() - start_time
    end_mem_normal = get_memory_usage()
    mem_used_normal = end_mem_normal - start_mem
    
    # Test search performance
    search_times_normal = []
    for query in queries:
        start = time.perf_counter()
        results = db_normal.search(query, limit=10)
        search_times_normal.append((time.perf_counter() - start) * 1000)
    
    avg_search_normal = np.mean(search_times_normal)
    
    # Get info
    info_normal = db_normal.info()
    count_normal = db_normal.count()
    
    # Clear for next test
    del db_normal
    
    # Test 2: With quantization
    print("\n2. Testing WITH scalar quantization (int8)...")
    db_quantized = omendb.DB()
    db_quantized.enable_quantization()  # Enable before adding vectors
    
    start_mem = get_memory_usage()
    start_time = time.perf_counter()
    
    # Add vectors in batches
    for i in range(0, num_vectors, batch_size):
        batch = vectors[i:i+batch_size]
        db_quantized.add_batch(batch)
    
    insert_time_quantized = time.perf_counter() - start_time
    end_mem_quantized = get_memory_usage()
    mem_used_quantized = end_mem_quantized - start_mem
    
    # Test search performance
    search_times_quantized = []
    for query in queries:
        start = time.perf_counter()
        results = db_quantized.search(query, limit=10)
        search_times_quantized.append((time.perf_counter() - start) * 1000)
    
    avg_search_quantized = np.mean(search_times_quantized)
    
    # Get info
    info_quantized = db_quantized.info()
    count_quantized = db_quantized.count()
    
    # Print results
    print(f"\n{'='*60}")
    print(f"RESULTS")
    print(f"{'='*60}")
    
    print(f"\nMemory Usage:")
    print(f"  Normal (Float32):     {mem_used_normal:8.1f} MB")
    print(f"  Quantized (Int8):     {mem_used_quantized:8.1f} MB")
    print(f"  Memory Reduction:     {(1 - mem_used_quantized/mem_used_normal)*100:8.1f}%")
    print(f"  Compression Ratio:    {mem_used_normal/mem_used_quantized:8.1f}x")
    
    print(f"\nTheoretical Memory:")
    float32_size = (num_vectors * dimension * 4) / (1024 * 1024)
    int8_size = (num_vectors * dimension * 1) / (1024 * 1024)
    print(f"  Float32 (theoretical): {float32_size:8.1f} MB")
    print(f"  Int8 (theoretical):    {int8_size:8.1f} MB")
    print(f"  Expected Ratio:        4.0x")
    
    print(f"\nInsert Performance:")
    print(f"  Normal:               {num_vectors/insert_time_normal:8.0f} vec/s")
    print(f"  Quantized:            {num_vectors/insert_time_quantized:8.0f} vec/s")
    print(f"  Speed Difference:     {(insert_time_quantized/insert_time_normal - 1)*100:+8.1f}%")
    
    print(f"\nSearch Performance (avg over 10 queries):")
    print(f"  Normal:               {avg_search_normal:8.2f} ms")
    print(f"  Quantized:            {avg_search_quantized:8.2f} ms")
    print(f"  Speed Difference:     {(avg_search_quantized/avg_search_normal - 1)*100:+8.1f}%")
    
    print(f"\nDatabase Info:")
    print(f"  Normal vectors:       {count_normal:,}")
    print(f"  Quantized vectors:    {count_quantized:,}")
    if 'quantization_enabled' in info_quantized:
        print(f"  Quantization enabled: {info_quantized.get('quantization_enabled', False)}")
    
    # Test accuracy (sample check)
    print(f"\n{'='*60}")
    print(f"ACCURACY CHECK (first query)")
    print(f"{'='*60}")
    
    query = queries[0]
    results_normal = db_normal.search(query, limit=5) if 'db_normal' in locals() else []
    results_quantized = db_quantized.search(query, limit=5)
    
    if results_normal and results_quantized:
        print(f"\nTop 5 results comparison:")
        print(f"{'Rank':<6} {'Normal Distance':<18} {'Quantized Distance':<18} {'Difference':<12}")
        print(f"{'-'*54}")
        
        for i in range(min(5, len(results_normal), len(results_quantized))):
            dist_normal = results_normal[i][1]
            dist_quantized = results_quantized[i][1]
            diff = abs(dist_quantized - dist_normal) / dist_normal * 100
            print(f"{i+1:<6} {dist_normal:<18.6f} {dist_quantized:<18.6f} {diff:>10.2f}%")
    
    return mem_used_normal, mem_used_quantized

if __name__ == "__main__":
    # Test different scales
    scales = [
        (10000, 128),    # 10K vectors
        (50000, 128),    # 50K vectors
        (100000, 128),   # 100K vectors
        (500000, 128),   # 500K vectors
        (1000000, 128),  # 1M vectors
    ]
    
    print("\nOmenDB Memory Quantization Benchmark")
    print("Testing scalar quantization (int8) vs normal (float32)")
    
    for num_vecs, dim in scales:
        try:
            benchmark_memory_usage(num_vecs, dim)
            print(f"\n{'='*60}\n")
        except Exception as e:
            print(f"Error at {num_vecs} vectors: {e}")
            break