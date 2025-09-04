#!/usr/bin/env python3
"""Test sparse graph integration in production."""

import numpy as np
import time
import psutil
import os
import gc
import sys
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')

def get_memory_mb():
    """Get current process memory in MB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / (1024 * 1024)

def test_sparse_integration():
    """Test the sparse graph implementation in production."""
    
    print("Sparse Graph Production Integration Test")
    print("=" * 60)
    
    # Test parameters
    num_vectors = 10000
    dimension = 128
    
    print(f"\nTesting with {num_vectors:,} vectors (dimension={dimension})")
    print("-" * 40)
    
    # Generate test data
    vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
    
    # Test sparse implementation
    gc.collect()
    mem_before = get_memory_mb()
    
    import omendb
    db = omendb.DB()
    
    # Measure insertion performance
    print("\nInsertion Performance:")
    start = time.perf_counter()
    
    for i in range(num_vectors):
        db.add(f"vec_{i}", vectors[i])
        
        if (i + 1) % 1000 == 0:
            elapsed = time.perf_counter() - start
            rate = (i + 1) / elapsed
            print(f"  {i+1:5d} vectors: {rate:8,.0f} vec/s")
    
    insert_time = time.perf_counter() - start
    insert_rate = num_vectors / insert_time
    
    # Force any pending operations
    if hasattr(db, 'flush'):
        db.flush()
    
    gc.collect()
    mem_after = get_memory_mb()
    memory_used = mem_after - mem_before
    
    # Get memory statistics
    try:
        stats = db.get_memory_stats()
        print("\nMemory Statistics:")
        for key, value in stats.items():
            print(f"  {key}: {value}")
    except:
        stats = {}
    
    # Test search performance
    print("\nSearch Performance:")
    search_times = []
    
    for i in range(100):
        query = np.random.rand(dimension).astype(np.float32)
        start = time.perf_counter()
        results = db.search(query, limit=10)
        search_time = (time.perf_counter() - start) * 1000
        search_times.append(search_time)
    
    avg_search = np.mean(search_times)
    p50_search = np.percentile(search_times, 50)
    p95_search = np.percentile(search_times, 95)
    
    # Results summary
    print("\n" + "=" * 60)
    print("RESULTS SUMMARY")
    print("=" * 60)
    
    print(f"\nInsertion:")
    print(f"  Total time: {insert_time:.2f} seconds")
    print(f"  Rate: {insert_rate:,.0f} vec/s")
    
    print(f"\nMemory:")
    print(f"  Total used: {memory_used:.2f} MB")
    print(f"  Per vector: {(memory_used * 1024) / num_vectors:.2f} KB")
    print(f"  Per 100K vectors: {memory_used * 100000 / num_vectors:.2f} MB")
    
    print(f"\nSearch Latency:")
    print(f"  Mean: {avg_search:.2f} ms")
    print(f"  P50: {p50_search:.2f} ms")
    print(f"  P95: {p95_search:.2f} ms")
    
    # Compare with theoretical savings
    print("\n" + "=" * 60)
    print("MEMORY COMPARISON")
    print("=" * 60)
    
    # Old representation (fixed R=48, Int64)
    old_edge_memory = num_vectors * 48 * 8 / (1024 * 1024)
    
    # Expected sparse (avg 20 neighbors, Int32)
    expected_edge_memory = num_vectors * 20 * 4 / (1024 * 1024)
    
    print(f"\nEdge Storage:")
    print(f"  Old (fixed R=48): {old_edge_memory:.2f} MB")
    print(f"  Expected (sparse): {expected_edge_memory:.2f} MB")
    print(f"  Savings: {(1 - expected_edge_memory/old_edge_memory)*100:.1f}%")
    
    print(f"\nTotal Memory:")
    print(f"  Actual: {memory_used:.2f} MB")
    print(f"  Per 100K: {memory_used * 100000 / num_vectors:.2f} MB")
    
    # Validate correctness
    print("\n" + "=" * 60)
    print("CORRECTNESS VALIDATION")
    print("=" * 60)
    
    # Search for a known vector
    test_idx = num_vectors // 2
    test_query = vectors[test_idx]
    results = db.search(test_query, limit=5)
    
    print(f"\nSearching for vec_{test_idx}:")
    for i, result in enumerate(results[:5]):
        print(f"  {i+1}. {result.id}: distance={result.distance:.4f}")
    
    if results and results[0].id == f"vec_{test_idx}":
        print("✅ Exact match found as top result")
    else:
        print("⚠️ Exact match not found as top result")
    
    return {
        'num_vectors': num_vectors,
        'insert_rate': insert_rate,
        'memory_mb': memory_used,
        'search_ms': p50_search,
        'memory_per_100k': memory_used * 100000 / num_vectors
    }

def compare_implementations():
    """Compare memory usage between implementations."""
    
    print("\n" + "=" * 60)
    print("IMPLEMENTATION COMPARISON")
    print("=" * 60)
    
    sizes = [1000, 5000, 10000]
    results = []
    
    for size in sizes:
        print(f"\nTesting {size} vectors...")
        
        # Generate data
        vectors = np.random.rand(size, 128).astype(np.float32)
        
        # Test current implementation
        gc.collect()
        mem_before = get_memory_mb()
        
        import omendb
        db = omendb.DB()
        
        for i in range(size):
            db.add(f"vec_{i}", vectors[i])
        
        if hasattr(db, 'flush'):
            db.flush()
        
        gc.collect()
        mem_used = get_memory_mb() - mem_before
        
        results.append({
            'size': size,
            'memory_mb': mem_used,
            'per_100k_mb': mem_used * 100000 / size
        })
        
        del db
        gc.collect()
    
    # Display comparison table
    print("\n" + "=" * 60)
    print("MEMORY SCALING")
    print("=" * 60)
    
    print(f"\n{'Vectors':<10} {'Memory MB':<12} {'Per 100K MB':<15}")
    print("-" * 40)
    
    for r in results:
        print(f"{r['size']:<10,} {r['memory_mb']:<12.2f} {r['per_100k_mb']:<15.2f}")
    
    # Check if memory scales linearly
    if len(results) >= 2:
        ratio = results[-1]['per_100k_mb'] / results[0]['per_100k_mb']
        if 0.9 <= ratio <= 1.1:
            print("\n✅ Memory scales linearly")
        else:
            print(f"\n⚠️ Non-linear scaling detected (ratio: {ratio:.2f})")

if __name__ == "__main__":
    # Run integration test
    results = test_sparse_integration()
    
    # Compare implementations
    compare_implementations()
    
    # Final summary
    print("\n" + "=" * 60)
    print("INTEGRATION STATUS")
    print("=" * 60)
    
    if results['memory_per_100k'] < 100:
        print(f"✅ Memory target achieved: {results['memory_per_100k']:.2f} MB per 100K vectors")
    else:
        print(f"⚠️ Memory needs optimization: {results['memory_per_100k']:.2f} MB per 100K vectors")
    
    if results['insert_rate'] > 50000:
        print(f"✅ Performance maintained: {results['insert_rate']:,.0f} vec/s")
    else:
        print(f"⚠️ Performance degraded: {results['insert_rate']:,.0f} vec/s")
    
    if results['search_ms'] < 2.0:
        print(f"✅ Search latency excellent: {results['search_ms']:.2f} ms")
    else:
        print(f"⚠️ Search latency high: {results['search_ms']:.2f} ms")