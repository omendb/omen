#!/usr/bin/env python3
"""Test real performance of current OmenDB implementation."""

import omendb
import numpy as np
import time
import sys

def test_performance(algorithm, n_vectors=1000):
    """Test performance with specified algorithm."""
    print(f"\nTesting {algorithm} with {n_vectors} vectors...")
    
    # Create DB
    if algorithm == "auto":
        db = omendb.DB()
    else:
        db = omendb.DB(force_algorithm=algorithm)
    
    # Create vectors
    dimension = 128
    vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    
    # Test add_batch
    start = time.perf_counter()
    db.add_batch(vectors)
    elapsed = time.perf_counter() - start
    
    vec_per_sec = n_vectors / elapsed
    print(f"  Add batch: {vec_per_sec:,.0f} vec/s ({elapsed:.3f}s for {n_vectors} vectors)")
    
    # Test search
    query = np.random.rand(dimension).astype(np.float32)
    
    # Warm up
    db.search(query, limit=10)
    
    # Measure
    n_searches = 100
    start = time.perf_counter()
    for _ in range(n_searches):
        results = db.search(query, limit=10)
    elapsed = time.perf_counter() - start
    
    search_time = (elapsed / n_searches) * 1000
    print(f"  Search: {search_time:.2f}ms per query")
    
    # Verify count
    count = db.count()
    print(f"  DB has {count} vectors")
    
    return vec_per_sec, search_time

def main():
    print("=" * 60)
    print("OmenDB Real Performance Test")
    print("=" * 60)
    
    # Redirect stderr to devnull to hide debug output
    import os
    old_stderr = os.dup(2)
    os.close(2)
    os.open(os.devnull, os.O_RDWR)
    
    try:
        # Test different configurations
        configs = [
            ("diskann", 1000),
            ("diskann", 10000),
            ("flat", 1000),
            ("flat", 10000),
            ("auto", 1000),
            ("auto", 10000),
        ]
        
        results = []
        for algorithm, n_vectors in configs:
            vec_per_sec, search_time = test_performance(algorithm, n_vectors)
            results.append((algorithm, n_vectors, vec_per_sec, search_time))
            
            # Clear for next test
            db = omendb.DB()
            db.clear()
        
        # Summary
        print("\n" + "=" * 60)
        print("Performance Summary")
        print("=" * 60)
        print(f"{'Algorithm':<10} {'Vectors':<10} {'Add (vec/s)':<15} {'Search (ms)':<10}")
        print("-" * 60)
        
        for algorithm, n_vectors, vec_per_sec, search_time in results:
            print(f"{algorithm:<10} {n_vectors:<10} {vec_per_sec:<15,.0f} {search_time:<10.2f}")
        
        # Check if we're hitting our targets
        print("\n" + "=" * 60)
        best_add = max(r[2] for r in results)
        best_search = min(r[3] for r in results)
        
        print(f"Best add performance: {best_add:,.0f} vec/s")
        print(f"Best search time: {best_search:.2f}ms")
        
        if best_add > 20000:
            print("✅ Achieving target of 20K+ vec/s")
        else:
            print(f"❌ Below target of 20K vec/s (currently {best_add/1000:.1f}K)")
            
        if best_search < 1.0:
            print("✅ Search under 1ms")
        else:
            print(f"⚠️  Search over 1ms ({best_search:.2f}ms)")
            
    finally:
        # Restore stderr
        os.dup2(old_stderr, 2)
        os.close(old_stderr)

if __name__ == "__main__":
    main()