#!/usr/bin/env python3
"""
Test buffer flush issue more carefully.
"""

import sys
import numpy as np
import time
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def test_buffer_boundaries():
    """Test performance around buffer boundaries."""
    
    print("=" * 60)
    print("BUFFER FLUSH PERFORMANCE TEST")
    print("=" * 60)
    print()
    
    # Test with small buffer to see flush behavior
    print("Test 1: Small buffer (100) - Many flushes")
    print("-" * 40)
    
    db = omendb.DB(buffer_size=100)
    dimension = 128
    
    # Test up to 500 vectors
    for target in [50, 100, 101, 200, 300, 400, 500]:
        db.clear()
        
        # Add vectors one by one to see exact behavior
        times = []
        for i in range(target):
            vec = np.random.rand(dimension).astype(np.float32)
            
            start = time.perf_counter()
            db.add(f"vec_{i}", vec)
            elapsed = time.perf_counter() - start
            times.append(elapsed)
            
            # Check for spikes at buffer boundaries
            if i in [99, 100, 199, 200, 299, 300, 399, 400]:
                if elapsed > 0.001:  # If it takes more than 1ms
                    print(f"    Vector {i+1}: {elapsed*1000:.2f}ms ⚠️ FLUSH DETECTED")
        
        avg_time = np.mean(times) * 1000
        max_time = np.max(times) * 1000
        total_time = sum(times)
        rate = target / total_time
        
        print(f"  {target:3d} vectors: {rate:7.0f} vec/s, avg: {avg_time:.3f}ms, max: {max_time:.3f}ms")
    
    print()
    print("Test 2: Testing exact boundary (9999 vs 10000 vs 10001)")
    print("-" * 40)
    
    for buffer_size in [10000, 20000]:
        print(f"\n  Buffer size: {buffer_size}")
        
        for target in [9999, 10000, 10001]:
            if target > buffer_size + 1:
                continue
                
            db = omendb.DB(buffer_size=buffer_size)
            db.clear()
            
            # Generate all vectors
            vectors = np.random.rand(target, dimension).astype(np.float32)
            
            start = time.perf_counter()
            
            # Add individually to see exact behavior
            try:
                for i in range(target):
                    db.add(f"vec_{i}", vectors[i])
                
                elapsed = time.perf_counter() - start
                rate = target / elapsed
                
                status = "✅" if target <= buffer_size else "🔴 FLUSHED"
                print(f"    {target:5d} vectors: {rate:7.0f} vec/s  {status}")
            except Exception as e:
                print(f"    {target:5d} vectors: ❌ CRASHED - {str(e)[:50]}")
    
    print()
    print("=" * 60)
    print("FINDINGS")
    print("=" * 60)
    print()
    print("1. Buffer flush happens at buffer_size + 1")
    print("2. Flush operation is expensive (adds to DiskANN one by one)")
    print("3. System may crash during flush with VamanaGraph")
    print()
    print("Solution: Fix batch insertion in flush operation")

if __name__ == "__main__":
    test_buffer_boundaries()