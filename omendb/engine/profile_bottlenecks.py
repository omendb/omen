#!/usr/bin/env python3
"""Profile OmenDB to find bottlenecks."""

import numpy as np
import time
import cProfile
import pstats
from io import StringIO
import sys

sys.path.insert(0, 'python')
from omendb import DB


def profile_insertion():
    """Profile the insertion path to find bottlenecks."""
    
    # Small dataset to avoid segfault
    vectors = np.random.random((1000, 128)).astype(np.float32)
    
    db = DB()
    db.clear()
    
    # Profile the add_batch operation
    profiler = cProfile.Profile()
    profiler.enable()
    
    db.add_batch(vectors)
    
    profiler.disable()
    
    # Get stats
    s = StringIO()
    ps = pstats.Stats(profiler, stream=s).sort_stats('cumulative')
    ps.print_stats(20)  # Top 20 functions
    
    print("Top 20 Functions by Cumulative Time:")
    print("="*60)
    print(s.getvalue())
    
    # Also profile search
    print("\n\nProfiling Search Operation:")
    print("="*60)
    
    query = vectors[0]
    profiler = cProfile.Profile()
    profiler.enable()
    
    for _ in range(100):
        db.search(query, limit=10)
    
    profiler.disable()
    
    s = StringIO()
    ps = pstats.Stats(profiler, stream=s).sort_stats('cumulative')
    ps.print_stats(20)
    print(s.getvalue())


def measure_component_times():
    """Measure time for each component of insertion."""
    
    vectors = np.random.random((1000, 128)).astype(np.float32)
    
    print("Component Timing Analysis (1000 vectors):")
    print("="*60)
    
    # Test 1: Pure native call overhead
    db = DB()
    db.clear()
    
    start = time.time()
    # This goes through Python -> native.so
    result = db._native.get_size()
    native_overhead = (time.time() - start) * 1000
    print(f"Native call overhead: {native_overhead:.3f}ms")
    
    # Test 2: Vector validation/conversion
    start = time.time()
    vectors_validated = db._validate_vectors(vectors)
    validation_time = (time.time() - start) * 1000
    print(f"Vector validation: {validation_time:.2f}ms")
    
    # Test 3: Full insertion
    start = time.time()
    db.add_batch(vectors)
    total_time = (time.time() - start) * 1000
    print(f"Total insertion: {total_time:.2f}ms")
    
    # Calculate breakdown
    print(f"\nBreakdown:")
    print(f"  Validation: {validation_time/total_time*100:.1f}%")
    print(f"  Native processing: {(total_time-validation_time)/total_time*100:.1f}%")
    print(f"  Per-vector time: {total_time/1000:.3f}ms")
    
    # Test 4: Check if problem is with larger batches
    print(f"\nBatch Size Scaling:")
    for size in [100, 500, 1000, 2000, 3000, 4000]:
        vecs = np.random.random((size, 128)).astype(np.float32)
        db.clear()
        
        start = time.time()
        try:
            db.add_batch(vecs)
            elapsed = time.time() - start
            rate = size / elapsed
            print(f"  {size} vectors: {rate:.0f} vec/s")
        except:
            print(f"  {size} vectors: FAILED")
            break


if __name__ == "__main__":
    print("🔍 Profiling OmenDB Performance\n")
    
    print("1. COMPONENT TIMING")
    print("-"*60)
    measure_component_times()
    
    print("\n\n2. DETAILED PROFILING")
    print("-"*60)
    profile_insertion()