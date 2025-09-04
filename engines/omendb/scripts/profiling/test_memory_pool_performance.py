#!/usr/bin/env python3
"""Test memory pool optimization performance improvement."""

import time
import numpy as np
import sys
import os
sys.path.insert(0, os.path.join(os.getcwd(), 'python'))

from omendb import DB

def benchmark_memory_pool_performance():
    """Benchmark performance with memory pool optimization."""
    print("🚀 BENCHMARKING MEMORY POOL OPTIMIZATION")
    print("=" * 50)
    
    # Test configuration
    dimensions = [64, 128, 256]
    batch_sizes = [100, 500, 1000, 5000]
    
    for dim in dimensions:
        print(f"\n📊 Testing {dim}D vectors")
        print("-" * 40)
        
        for batch_size in batch_sizes:
            # Generate test data
            vectors = np.random.rand(batch_size, dim).astype(np.float32)
            
            # Test individual adds (current implementation)
            db1 = DB()
            start = time.perf_counter()
            for i in range(batch_size):
                db1.add(f'vec_{i}', vectors[i].tolist())
            individual_time = time.perf_counter() - start
            individual_rate = batch_size / individual_time
            
            # Test batch add (uses memory pool optimization)
            db2 = DB()
            batch_data = [(f'vec_{i}', vectors[i].tolist(), {}) for i in range(batch_size)]
            
            # Use real batch API for optimized performance
            start = time.perf_counter()
            results = db2.add_batch(batch_data)
            batch_time = time.perf_counter() - start
            batch_rate = batch_size / batch_time
            
            print(f"\nBatch size: {batch_size}")
            print(f"Individual: {individual_rate:8.1f} vec/s ({individual_time*1000:.1f}ms total)")
            print(f"Batch:      {batch_rate:8.1f} vec/s ({batch_time*1000:.1f}ms total)")
            print(f"Speedup:    {batch_rate/individual_rate:.2f}x")

def test_large_batch_performance():
    """Test performance with very large batches."""
    print("\n\n📊 LARGE BATCH PERFORMANCE TEST")
    print("=" * 50)
    
    dim = 128
    large_batch_sizes = [10000, 25000, 50000]
    
    print(f"\nTesting {dim}D vectors with large batches")
    print("\nBatch Size | Rate (vec/s) | Time (s) | Per Vector (μs)")
    print("-" * 60)
    
    for batch_size in large_batch_sizes:
        # Generate data in chunks to avoid memory issues
        chunk_size = 5000
        total_time = 0
        
        db = DB()
        
        for chunk_start in range(0, batch_size, chunk_size):
            chunk_end = min(chunk_start + chunk_size, batch_size)
            chunk_vectors = np.random.rand(chunk_end - chunk_start, dim).astype(np.float32)
            
            start = time.perf_counter()
            for i in range(len(chunk_vectors)):
                db.add(f'vec_{chunk_start + i}', chunk_vectors[i].tolist())
            total_time += time.perf_counter() - start
        
        rate = batch_size / total_time
        per_vector_us = (total_time / batch_size) * 1_000_000
        
        print(f"{batch_size:10d} | {rate:12.1f} | {total_time:8.2f} | {per_vector_us:15.1f}")

def analyze_memory_usage():
    """Analyze memory usage patterns."""
    print("\n\n💾 MEMORY USAGE ANALYSIS")
    print("=" * 50)
    
    import psutil
    import gc
    
    process = psutil.Process()
    
    dim = 128
    batch_size = 10000
    
    # Baseline memory
    gc.collect()
    baseline_memory = process.memory_info().rss / 1024 / 1024  # MB
    
    print(f"Baseline memory: {baseline_memory:.1f} MB")
    
    # Create database and add vectors
    db = DB()
    vectors = np.random.rand(batch_size, dim).astype(np.float32)
    
    # Add vectors
    start = time.perf_counter()
    for i in range(batch_size):
        db.add(f'vec_{i}', vectors[i].tolist())
    add_time = time.perf_counter() - start
    
    # Check memory after adding
    after_add_memory = process.memory_info().rss / 1024 / 1024
    memory_used = after_add_memory - baseline_memory
    
    print(f"Memory after {batch_size} vectors: {after_add_memory:.1f} MB")
    print(f"Memory used: {memory_used:.1f} MB")
    print(f"Memory per vector: {memory_used / batch_size * 1000:.2f} KB")
    print(f"Theoretical minimum: {dim * 4 / 1000:.2f} KB per vector")
    print(f"Overhead factor: {(memory_used / batch_size * 1000) / (dim * 4 / 1000):.1f}x")

def project_improvements():
    """Project expected improvements with memory pool."""
    print("\n\n🎯 PROJECTED IMPROVEMENTS WITH MEMORY POOL")
    print("=" * 50)
    
    current_rates = {
        64: 7797,   # Current rates from benchmarks
        128: 4679,
        256: 2566
    }
    
    print("\nProjected performance with memory pool optimization:")
    print("\nDim | Current (vec/s) | 3x Improvement | 5x Improvement")
    print("-" * 55)
    
    for dim, rate in current_rates.items():
        print(f"{dim:3d} | {rate:15.0f} | {rate*3:14.0f} | {rate*5:14.0f}")
    
    print("\n📈 Expected benefits:")
    print("1. Zero allocations in hot path")
    print("2. Better cache locality")
    print("3. True batch processing")
    print("4. Reduced GC pressure")
    print("5. Lower memory fragmentation")

if __name__ == '__main__':
    benchmark_memory_pool_performance()
    test_large_batch_performance()
    analyze_memory_usage()
    project_improvements()
    
    print("\n\n✅ Memory pool benchmarking complete!")
    print("\n🎯 Implementation plan:")
    print("1. Integrate memory pool into BruteForce index")
    print("2. Implement true batch_add with bulk operations")
    print("3. Add pre-allocation hints for better performance")
    print("4. Consider mmap for very large datasets")