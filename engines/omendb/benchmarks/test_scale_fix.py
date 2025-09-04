#!/usr/bin/env python3
"""
Test if VamanaGraph fixes allow scaling beyond previous limits.
"""

import sys
import numpy as np
import time
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def test_scale_limits():
    """Test scaling with quantization enabled."""
    
    print("=" * 60)
    print("SCALE TEST WITH FIXES")
    print("=" * 60)
    print()
    
    test_configs = [
        (10000, 1000),   # Test at 10K with small buffer
        (10000, 10000),  # Test at 10K with exact buffer
        (20000, 10000),  # Test at 20K (previous crash point)
        (50000, 10000),  # Test at 50K
        (100000, 10000), # Test at 100K (production scale)
    ]
    
    dimension = 128
    
    for n_vectors, buffer_size in test_configs:
        print(f"Test: {n_vectors} vectors with buffer_size={buffer_size}")
        print("-" * 40)
        
        # Create database with quantization
        db = omendb.DB(quantization="scalar", buffer_size=buffer_size)
        
        try:
            # Generate and add vectors in batches
            batch_size = 1000
            start = time.perf_counter()
            
            for i in range(0, n_vectors, batch_size):
                end_idx = min(i + batch_size, n_vectors)
                batch_vectors = np.random.rand(end_idx - i, dimension).astype(np.float32)
                batch_ids = [f"vec_{j}" for j in range(i, end_idx)]
                
                db.add_batch(batch_vectors, batch_ids)
                
                # Progress report
                if (i + batch_size) % 10000 == 0:
                    elapsed = time.perf_counter() - start
                    rate = (i + batch_size) / elapsed
                    print(f"  Progress: {i + batch_size:6d} vectors, {rate:7.0f} vec/s")
            
            # Final stats
            total_elapsed = time.perf_counter() - start
            total_rate = n_vectors / total_elapsed
            
            print(f"  ✅ SUCCESS: {n_vectors} vectors")
            print(f"  Rate: {total_rate:7.0f} vec/s")
            print(f"  Time: {total_elapsed:6.2f}s")
            
            # Test search
            query = np.random.rand(dimension).astype(np.float32)
            search_start = time.perf_counter()
            results = db.search(query, k=10)
            search_time = (time.perf_counter() - search_start) * 1000
            
            print(f"  Search: {search_time:.2f}ms for k=10")
            
        except Exception as e:
            print(f"  ❌ FAILED at scale {n_vectors}")
            print(f"  Error: {str(e)[:100]}")
        
        print()
        
        # Clear for next test
        del db
    
    print("=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print()
    print("Fixes applied:")
    print("1. Copy constructor preserves capacity (not just num_nodes)")
    print("2. Added bounds checking to prevent null pointer access")
    print("3. Return safe values for invalid node access")
    print()
    print("Results:")
    print("- Should now handle 100K+ vectors without crashes")
    print("- Performance should remain consistent across scales")

if __name__ == "__main__":
    test_scale_limits()