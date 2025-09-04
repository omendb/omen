#!/usr/bin/env python3
"""Test memory improvement after removing double storage."""

import numpy as np
import omendb
import psutil
import os
import gc
import time

def get_memory_mb():
    """Get current process memory in MB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024

def test_memory_at_scale():
    """Test memory usage at various scales."""
    print("🎉 MEMORY IMPROVEMENT TEST - After Double Storage Fix")
    print("=" * 60)
    
    test_sizes = [1000, 10000, 50000, 100000]
    dimension = 128
    
    results = []
    
    for n_vectors in test_sizes:
        print(f"\n📊 Testing with {n_vectors:,} vectors (128D)")
        
        # Fresh start
        db = omendb.DB()
        db._auto_batch_enabled = False
        
        gc.collect()
        start_mem = get_memory_mb()
        
        # Add vectors
        start_time = time.time()
        for i in range(n_vectors):
            vector = np.random.rand(dimension).astype(np.float32)
            db.add(f"vec_{i}", vector)
            
            # Progress indicator
            if (i + 1) % (n_vectors // 10) == 0:
                print(f"  Progress: {(i+1)*100//n_vectors}%", end="\r")
        
        add_time = time.time() - start_time
        
        gc.collect()
        final_mem = get_memory_mb()
        mem_used = final_mem - start_mem
        
        # Get internal stats
        stats = db.get_memory_stats()
        tracked = sum(v for k, v in stats.items() if k.endswith('_mb') and isinstance(v, float))
        
        # Calculate expected memory
        vectors_expected = n_vectors * dimension * 4 / (1024 * 1024)
        graph_expected = n_vectors * 32 * 4 / (1024 * 1024)
        metadata_expected = n_vectors * 50 / (1024 * 1024)
        total_expected = vectors_expected + graph_expected + metadata_expected
        
        # Test retrieval
        test_id = f"vec_{n_vectors//2}"
        retrieved = db.get(test_id)
        retrieval_ok = retrieved is not None
        
        results.append({
            'vectors': n_vectors,
            'actual_mb': mem_used,
            'expected_mb': total_expected,
            'tracked_mb': tracked,
            'add_time': add_time,
            'retrieval_ok': retrieval_ok,
            'vec_per_sec': n_vectors / add_time
        })
        
        print(f"\n  Results:")
        print(f"    Actual memory: {mem_used:.2f} MB")
        print(f"    Expected memory: {total_expected:.2f} MB")
        print(f"    Tracked memory: {tracked:.2f} MB")
        print(f"    Insertion speed: {n_vectors/add_time:.0f} vec/s")
        print(f"    Retrieval: {'✅ Working' if retrieval_ok else '❌ Failed'}")
        
        # Clear for next test
        del db
        gc.collect()
    
    # Summary comparison
    print("\n" + "=" * 60)
    print("📈 MEMORY USAGE SUMMARY (After Fix)")
    print("=" * 60)
    print(f"{'Vectors':<10} {'Actual MB':<12} {'Expected MB':<12} {'Ratio':<8} {'Speed (vec/s)':<15}")
    print("-" * 60)
    
    for r in results:
        ratio = r['actual_mb'] / r['expected_mb']
        print(f"{r['vectors']:<10,} {r['actual_mb']:<12.2f} {r['expected_mb']:<12.2f} "
              f"{ratio:<8.2f} {r['vec_per_sec']:<15,.0f}")
    
    print("\n🆚 COMPARISON WITH BEFORE FIX:")
    print("  100K vectors BEFORE: 778 MB (STATUS.md)")
    print(f"  100K vectors AFTER:  {results[-1]['actual_mb']:.2f} MB")
    print(f"  IMPROVEMENT: {778/results[-1]['actual_mb']:.1f}x reduction! 🎉")
    
    # Check against competitors
    print("\n🏁 COMPETITIVE COMPARISON:")
    print("  Target (competitors): 12-20 MB per 1M vectors")
    print(f"  OmenDB (after fix): {results[-1]['actual_mb']*10:.0f} MB per 1M vectors (extrapolated)")
    
    if results[-1]['actual_mb'] * 10 < 100:
        print("  ✅ NOW COMPETITIVE! Under 100MB for 1M vectors")
    else:
        print(f"  ⚠️  Still {results[-1]['actual_mb']*10/20:.1f}x higher than best competitors")

if __name__ == "__main__":
    test_memory_at_scale()
    print("\n✅ Memory improvement test complete!")