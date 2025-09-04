#!/usr/bin/env python3
"""Comprehensive performance benchmark for documentation."""

import time
import sys
import numpy as np
import os

sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def benchmark_dimension(dim, num_vectors=10000):
    """Benchmark at specific dimension."""
    print(f"\n📊 Dimension: {dim}D")
    print("-" * 40)
    
    db = omendb.DB()
    db.clear()  # Clear any existing data
    vectors = np.random.rand(num_vectors, dim).astype(np.float32)
    
    # Batch performance
    start = time.perf_counter()
    ids = db.add_batch(vectors)
    batch_time = time.perf_counter() - start
    batch_vps = len(ids) / batch_time
    print(f"Batch add ({num_vectors:,} vectors): {batch_vps:,.0f} vec/s ({batch_time:.2f}s)")
    
    # Single add performance (sample 100)
    db.clear()
    single_times = []
    for i in range(min(100, num_vectors)):
        start = time.perf_counter()
        db.add(f"vec_{i}", vectors[i])
        single_times.append(time.perf_counter() - start)
    
    avg_single = np.mean(single_times)
    single_vps = 1 / avg_single
    print(f"Single add (avg of 100): {single_vps:,.0f} vec/s ({avg_single*1000:.2f}ms each)")
    
    # Search performance
    db.clear()
    db.add_batch(vectors[:1000])  # Test with 1K vectors
    
    search_times = []
    for _ in range(100):
        query = np.random.rand(dim).astype(np.float32)
        start = time.perf_counter()
        results = db.search(query, limit=10)
        search_times.append(time.perf_counter() - start)
    
    avg_search = np.mean(search_times) * 1000
    print(f"Search (1K dataset, avg of 100): {avg_search:.2f}ms")
    
    return {
        'dimension': dim,
        'batch_vps': batch_vps,
        'single_vps': single_vps,
        'search_ms': avg_search
    }

def benchmark_scale(dim=128):
    """Benchmark at different dataset sizes."""
    print(f"\n📈 Scaling Test (128D)")
    print("-" * 40)
    
    sizes = [100, 1000, 10000, 50000]
    results = []
    
    for size in sizes:
        db = omendb.DB()
        db.clear()  # Clear any existing data
        vectors = np.random.rand(size, dim).astype(np.float32)
        
        start = time.perf_counter()
        ids = db.add_batch(vectors)
        batch_time = time.perf_counter() - start
        batch_vps = len(ids) / batch_time
        
        # Search on final size
        query = np.random.rand(dim).astype(np.float32)
        start = time.perf_counter()
        results_search = db.search(query, limit=10)
        search_time = (time.perf_counter() - start) * 1000
        
        print(f"{size:6,} vectors: {batch_vps:6,.0f} vec/s | Search: {search_time:.2f}ms")
        results.append({'size': size, 'vps': batch_vps, 'search_ms': search_time})
    
    return results

def main():
    print("=" * 60)
    print("OmenDB COMPREHENSIVE PERFORMANCE BENCHMARK")
    print("=" * 60)
    print(f"Date: {time.strftime('%Y-%m-%d %H:%M:%S')}")
    print(f"Platform: {sys.platform}")
    
    # Test different dimensions
    dimensions = [64, 128, 256, 384, 512, 768, 1024]
    dim_results = []
    
    print("\n🔬 DIMENSION SCALING")
    print("=" * 60)
    for dim in dimensions:
        try:
            result = benchmark_dimension(dim, num_vectors=1000)
            dim_results.append(result)
        except Exception as e:
            print(f"Error at {dim}D: {e}")
    
    # Test different dataset sizes
    print("\n🔬 DATASET SIZE SCALING")
    print("=" * 60)
    scale_results = benchmark_scale()
    
    # Summary table
    print("\n📊 PERFORMANCE SUMMARY")
    print("=" * 60)
    
    print("\n### By Dimension (1K vectors):")
    print("| Dimension | Batch (vec/s) | Single (vec/s) | Search (ms) |")
    print("|-----------|---------------|----------------|-------------|")
    for r in dim_results:
        print(f"| {r['dimension']:4}D     | {r['batch_vps']:13,.0f} | {r['single_vps']:14,.0f} | {r['search_ms']:11.2f} |")
    
    print("\n### By Dataset Size (128D):")
    print("| Vectors | Batch (vec/s) | Search (ms) |")
    print("|---------|---------------|-------------|")
    for r in scale_results:
        print(f"| {r['size']:7,} | {r['vps']:13,.0f} | {r['search_ms']:11.2f} |")
    
    print("\n### Key Metrics for Documentation:")
    print("-" * 40)
    
    # Find 128D results
    d128 = next((r for r in dim_results if r['dimension'] == 128), None)
    if d128:
        print(f"• 128D batch performance: {d128['batch_vps']:,.0f} vec/s")
        print(f"• 128D single add: {d128['single_vps']:,.0f} vec/s")
        print(f"• 128D search latency: {d128['search_ms']:.2f}ms")
    
    # Performance ratio
    if d128:
        ratio = d128['batch_vps'] / d128['single_vps']
        print(f"• Batch speedup: {ratio:.1f}x faster than single adds")
    
    print("\n✅ Benchmark complete!")

if __name__ == "__main__":
    main()