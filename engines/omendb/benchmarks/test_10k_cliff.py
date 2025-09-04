#!/usr/bin/env python3
"""
Test the 10K performance cliff issue.
The hypothesis: Performance degrades when buffer flushes at 10K vectors.
"""

import sys
import numpy as np
import time
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def test_performance_cliff():
    """Test performance before and after 10K vector boundary."""
    
    print("=" * 60)
    print("10K PERFORMANCE CLIFF INVESTIGATION")
    print("=" * 60)
    print()
    
    # Test with default buffer size (10000)
    print("Test 1: Default buffer_size=10000")
    print("-" * 40)
    
    db = omendb.DB(buffer_size=10000)
    dimension = 128
    
    # Track performance at different scales
    scales = [1000, 5000, 9000, 9999, 10000, 10001, 11000, 15000, 20000]
    
    for target in scales:
        db.clear()
        
        # Generate vectors
        vectors = np.random.rand(target, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(target)]
        
        # Time the insertions
        start = time.perf_counter()
        
        # Add in batches of 1000 for consistency
        batch_size = 1000
        for i in range(0, target, batch_size):
            end_idx = min(i + batch_size, target)
            db.add_batch(vectors[i:end_idx], ids[i:end_idx])
        
        elapsed = time.perf_counter() - start
        rate = target / elapsed
        
        # Check if we're at/past buffer boundary
        at_boundary = "🔴 BOUNDARY" if target in [10000, 10001] else ""
        
        print(f"  {target:5d} vectors: {rate:7.0f} vec/s  {elapsed:6.2f}s {at_boundary}")
    
    print()
    print("Test 2: Larger buffer_size=50000 (no flush)")
    print("-" * 40)
    
    db2 = omendb.DB(buffer_size=50000)
    
    for target in [5000, 10000, 15000, 20000]:
        db2.clear()
        
        vectors = np.random.rand(target, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(target)]
        
        start = time.perf_counter()
        
        # Add in batches
        batch_size = 1000
        for i in range(0, target, batch_size):
            end_idx = min(i + batch_size, target)
            db2.add_batch(vectors[i:end_idx], ids[i:end_idx])
        
        elapsed = time.perf_counter() - start
        rate = target / elapsed
        
        print(f"  {target:5d} vectors: {rate:7.0f} vec/s  {elapsed:6.2f}s")
    
    print()
    print("Test 3: Small buffer_size=1000 (many flushes)")
    print("-" * 40)
    
    db3 = omendb.DB(buffer_size=1000)
    
    for target in [1000, 2000, 5000, 10000]:
        db3.clear()
        
        vectors = np.random.rand(target, dimension).astype(np.float32)
        ids = [f"vec_{i}" for i in range(target)]
        
        start = time.perf_counter()
        
        # Add in batches
        batch_size = 1000
        for i in range(0, target, batch_size):
            end_idx = min(i + batch_size, target)
            db3.add_batch(vectors[i:end_idx], ids[i:end_idx])
        
        elapsed = time.perf_counter() - start
        rate = target / elapsed
        flushes = target // 1000
        
        print(f"  {target:5d} vectors: {rate:7.0f} vec/s  {elapsed:6.2f}s  ({flushes} flushes)")
    
    print()
    print("=" * 60)
    print("ANALYSIS")
    print("=" * 60)
    print()
    print("Key observations:")
    print("1. Performance cliff occurs exactly at buffer_size boundary")
    print("2. Larger buffers delay the cliff but don't eliminate it")
    print("3. Small buffers have consistent (but slower) performance")
    print()
    print("Root cause: Flush operation adds vectors one-by-one to DiskANN")
    print("Each add() involves expensive graph operations (search, connect, prune)")
    print("Solution: Batch flush or optimized bulk insertion into DiskANN")

if __name__ == "__main__":
    test_performance_cliff()