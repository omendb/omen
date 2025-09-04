#!/usr/bin/env python3
"""
Test the buffer size increase fix for the 10K performance cliff.
"""

import sys
import numpy as np
import time
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def test_buffer_fix():
    """Test if increased buffer size fixes the performance cliff."""
    
    print("=" * 60)
    print("BUFFER SIZE FIX TEST")
    print("=" * 60)
    print()
    
    # Test with new default (100000)
    print("Test 1: New default buffer_size (should be 100000)")
    print("-" * 40)
    
    db = omendb.DB()  # Use default
    print(f"Buffer size: {db._buffer_size}")
    
    dimension = 128
    test_sizes = [5000, 10000, 20000, 30000, 50000]
    
    for n in test_sizes:
        db.clear()
        
        # Generate vectors
        vectors = np.random.rand(n, dimension).astype(np.float32)
        
        # Time batch insertion
        start = time.perf_counter()
        
        # Add in batches of 1000
        batch_size = 1000
        for i in range(0, n, batch_size):
            end_idx = min(i + batch_size, n)
            batch_vecs = vectors[i:end_idx]
            batch_ids = [f"vec_{j}" for j in range(i, end_idx)]
            
            try:
                db.add_batch(batch_vecs, batch_ids)
            except Exception as e:
                print(f"  {n:6d} vectors: ❌ CRASHED at {i}")
                break
        else:
            elapsed = time.perf_counter() - start
            rate = n / elapsed
            print(f"  {n:6d} vectors: {rate:7.0f} vec/s  {elapsed:6.2f}s")
    
    print()
    print("Test 2: Compare old vs new buffer size")
    print("-" * 40)
    
    for buffer_size in [10000, 100000]:
        print(f"\n  Buffer size: {buffer_size}")
        
        db = omendb.DB(buffer_size=buffer_size)
        
        # Test at the old cliff point (10K)
        n = 10001
        vectors = np.random.rand(n, dimension).astype(np.float32)
        
        db.clear()
        start = time.perf_counter()
        
        try:
            # Add individually to trigger flush
            for i in range(n):
                db.add(f"vec_{i}", vectors[i])
                
                # Report at key points
                if i == buffer_size - 1:
                    elapsed = time.perf_counter() - start
                    rate = (i+1) / elapsed
                    print(f"    At buffer limit ({i+1}): {rate:.0f} vec/s")
                elif i == buffer_size:
                    elapsed = time.perf_counter() - start
                    rate = (i+1) / elapsed
                    print(f"    After flush ({i+1}): {rate:.0f} vec/s")
            
            total_elapsed = time.perf_counter() - start
            total_rate = n / total_elapsed
            print(f"    Total ({n}): {total_rate:.0f} vec/s ✅")
            
        except Exception as e:
            print(f"    Failed at flush: ❌ {str(e)[:50]}")
    
    print()
    print("=" * 60)
    print("RESULTS")
    print("=" * 60)
    print()
    print("Quick fix assessment:")
    print("- Increased buffer from 10K to 100K delays the cliff")
    print("- Performance should remain high up to 100K vectors")
    print("- Still need proper batch flush implementation")
    print("- This buys time for the real fix")

if __name__ == "__main__":
    test_buffer_fix()