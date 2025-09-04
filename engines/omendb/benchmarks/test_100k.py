#!/usr/bin/env python3
"""Test if our fix works beyond the 26K vector limit."""

import sys
sys.path.insert(0, 'python')

import omendb
import numpy as np
import time

def test_scale():
    """Test scaling beyond the previous 26K limit."""
    db = omendb.DB()
    
    test_sizes = [10000, 20000, 30000, 50000, 100000]
    batch_size = 5000
    
    for target in test_sizes:
        print(f"\n{'='*60}")
        print(f"Testing {target:,} vectors...")
        print(f"{'='*60}")
        
        db = omendb.DB()  # Fresh DB for each test
        start_time = time.perf_counter()
        
        try:
            total_added = 0
            
            while total_added < target:
                current_batch = min(batch_size, target - total_added)
                vectors = np.random.rand(current_batch, 128).astype(np.float32)
                
                batch_start = time.perf_counter()
                db.add_batch(vectors)
                batch_time = time.perf_counter() - batch_start
                
                total_added += current_batch
                batch_rate = current_batch / batch_time
                
                if total_added % 10000 == 0:
                    elapsed = time.perf_counter() - start_time
                    overall_rate = total_added / elapsed
                    print(f"  {total_added:,} vectors | Batch: {batch_rate:,.0f} vec/s | Overall: {overall_rate:,.0f} vec/s")
            
            # Final stats
            total_time = time.perf_counter() - start_time
            final_rate = total_added / total_time
            print(f"\n✅ SUCCESS: {total_added:,} vectors in {total_time:.1f}s ({final_rate:,.0f} vec/s)")
            
            # Test search
            query = np.random.rand(128).astype(np.float32)
            search_start = time.perf_counter()
            results = db.search(query, limit=10)
            search_time = (time.perf_counter() - search_start) * 1000
            print(f"  Search latency: {search_time:.2f}ms, returned {len(results)} results")
            
        except Exception as e:
            elapsed = time.perf_counter() - start_time
            print(f"\n❌ FAILED at ~{total_added:,} vectors after {elapsed:.1f}s")
            print(f"  Error: {e}")
            import traceback
            traceback.print_exc()
            return False
    
    return True

if __name__ == "__main__":
    print("Testing HybridGraph fix for 26K vector limit...")
    print("Previous limit: 26-27K vectors")
    print("Target: 100K+ vectors")
    
    if test_scale():
        print("\n🎉 FIX SUCCESSFUL! Scaled beyond the 26K limit!")
    else:
        print("\n⚠️ Fix incomplete - still hitting limits")