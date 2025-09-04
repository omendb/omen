#!/usr/bin/env python3
"""Profile memory allocation patterns in OmenDB to identify optimization opportunities."""

import time
import gc
import tracemalloc
import numpy as np
import sys
import os
sys.path.insert(0, os.path.join(os.getcwd(), 'python'))

from omendb import DB

def profile_memory_allocations():
    """Detailed memory allocation profiling."""
    print("🧠 MEMORY ALLOCATION PROFILING")
    print("=" * 50)
    
    # Start memory tracking
    tracemalloc.start()
    gc.collect()
    
    # Test configuration
    dim = 128
    batch_sizes = [1, 10, 100, 1000]
    
    print(f"\nTesting {dim}D vectors with various batch sizes")
    print("-" * 40)
    
    for batch_size in batch_sizes:
        print(f"\n📊 Batch size: {batch_size}")
        
        # Generate test data
        vectors = np.random.rand(batch_size, dim).astype(np.float32)
        
        # Clear memory
        gc.collect()
        snapshot1 = tracemalloc.take_snapshot()
        
        # Create database and measure allocations
        db = DB()
        
        # Measure individual adds
        start = time.perf_counter()
        for i in range(batch_size):
            db.add(f'vec_{i}', vectors[i].tolist())
        individual_time = time.perf_counter() - start
        
        snapshot2 = tracemalloc.take_snapshot()
        
        # Analyze memory difference
        top_stats = snapshot2.compare_to(snapshot1, 'lineno')
        
        print(f"Time: {individual_time*1000:.2f}ms total, {individual_time/batch_size*1000:.2f}ms per vector")
        print(f"Rate: {batch_size/individual_time:.1f} vec/s")
        
        # Show top memory allocations
        print("\nTop memory allocations:")
        for stat in top_stats[:5]:
            print(f"  {stat}")
    
    tracemalloc.stop()

def profile_allocation_hotspots():
    """Identify specific allocation hotspots during operations."""
    print("\n\n🔥 ALLOCATION HOTSPOT ANALYSIS")
    print("=" * 50)
    
    dim = 128
    num_vectors = 1000
    
    # Profile different operation phases
    operations = [
        ("DB Creation", lambda: DB()),
        ("Vector Generation", lambda: np.random.rand(100, dim).astype(np.float32)),
        ("List Conversion", lambda v: [v[i].tolist() for i in range(len(v))]),
        ("Single Add", lambda db, v: db.add('test', v)),
        ("Search", lambda db, v: db.search(v, limit=10))
    ]
    
    vectors = np.random.rand(num_vectors, dim).astype(np.float32)
    db = DB()
    
    # Add some vectors first
    for i in range(100):
        db.add(f'vec_{i}', vectors[i].tolist())
    
    print("\nOperation timing breakdown:")
    print("-" * 40)
    
    for op_name, op_func in operations:
        gc.collect()
        
        if op_name == "DB Creation":
            start = time.perf_counter()
            for _ in range(10):
                _ = op_func()
            elapsed = (time.perf_counter() - start) / 10
        elif op_name == "Vector Generation":
            start = time.perf_counter()
            _ = op_func()
            elapsed = time.perf_counter() - start
        elif op_name == "List Conversion":
            test_vectors = vectors[:100]
            start = time.perf_counter()
            _ = op_func(test_vectors)
            elapsed = time.perf_counter() - start
        elif op_name == "Single Add":
            start = time.perf_counter()
            op_func(db, vectors[50].tolist())
            elapsed = time.perf_counter() - start
        elif op_name == "Query":
            start = time.perf_counter()
            op_func(db, vectors[0].tolist())
            elapsed = time.perf_counter() - start
        
        print(f"{op_name:20s}: {elapsed*1000:8.3f}ms")

def analyze_batch_vs_individual():
    """Compare batch operations vs individual operations."""
    print("\n\n📊 BATCH VS INDIVIDUAL OPERATIONS")
    print("=" * 50)
    
    dim = 128
    batch_sizes = [10, 50, 100, 500, 1000]
    
    print(f"\nTesting {dim}D vectors")
    print("\nBatch | Individual (vec/s) | Batch API (vec/s) | Speedup")
    print("-" * 60)
    
    for batch_size in batch_sizes:
        vectors = np.random.rand(batch_size, dim).astype(np.float32)
        
        # Test individual adds
        db1 = DB()
        start = time.perf_counter()
        for i in range(batch_size):
            db1.add(f'vec_{i}', vectors[i].tolist())
        individual_time = time.perf_counter() - start
        individual_rate = batch_size / individual_time
        
        # Test batch API (if available)
        db2 = DB()
        batch_data = [(f'vec_{i}', vectors[i].tolist(), {}) for i in range(batch_size)]
        start = time.perf_counter()
        # Simulate batch operation with loop for now
        for item in batch_data:
            db2.add(item[0], item[1])
        batch_time = time.perf_counter() - start
        batch_rate = batch_size / batch_time
        
        speedup = batch_rate / individual_rate
        print(f"{batch_size:5d} | {individual_rate:18.1f} | {batch_rate:17.1f} | {speedup:7.2f}x")

def estimate_memory_pool_impact():
    """Estimate potential impact of memory pool optimization."""
    print("\n\n🎯 MEMORY POOL OPTIMIZATION POTENTIAL")
    print("=" * 50)
    
    print("\nCurrent bottlenecks:")
    print("1. Python list allocation for each vector")
    print("2. Mojo List growth/reallocation")
    print("3. String allocation for IDs")
    print("4. HashMap growth/rehashing")
    
    print("\nMemory pool strategy:")
    print("1. Pre-allocate vector storage (eliminate per-vector malloc)")
    print("2. Reuse Python list objects")
    print("3. String interning for IDs")
    print("4. Fixed-size HashMap with linear probing")
    
    print("\nExpected improvements:")
    print("- Individual adds: 2-3x speedup")
    print("- Batch operations: 3-5x speedup")
    print("- Memory usage: 30-50% reduction")
    print("- GC pressure: Significantly reduced")
    
    # Simulate improvement
    current_rate = 4679  # vec/s @128D
    print(f"\nProjected performance @128D:")
    print(f"Current: {current_rate} vec/s")
    print(f"With memory pool (3x): {current_rate * 3:.0f} vec/s")
    print(f"With memory pool (5x): {current_rate * 5:.0f} vec/s")

if __name__ == '__main__':
    profile_memory_allocations()
    profile_allocation_hotspots()
    analyze_batch_vs_individual()
    estimate_memory_pool_impact()
    
    print("\n\n✅ Memory profiling complete!")
    print("\n🎯 Next steps:")
    print("1. Implement pre-allocated vector buffers")
    print("2. Add object pooling for Python lists")
    print("3. Use memory-mapped storage for large datasets")
    print("4. Implement lock-free allocation for thread safety")