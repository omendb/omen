#!/usr/bin/env python3
"""Verify all performance benchmarks are accurate with latest code."""

import sys
import time
import numpy as np
sys.path.insert(0, 'python')
from omendb.api import DB

def run_benchmark(name, num_vectors, batch_size=1000):
    """Run a specific benchmark test."""
    print(f"\n{'='*60}")
    print(f"{name}")
    print('='*60)
    
    # Initialize database
    db = DB(db_path=f"bench_{name.lower().replace(' ', '_')}.db", buffer_size=2000)
    
    # Test batch insertion
    vectors = np.random.randn(num_vectors, 128).astype(np.float32)
    ids = [f"vec_{i}" for i in range(num_vectors)]
    
    t0 = time.time()
    db.add_batch(vectors, ids)
    t1 = time.time()
    
    insertion_time = t1 - t0
    vectors_per_sec = num_vectors / insertion_time if insertion_time > 0 else 0
    
    print(f"Vectors: {num_vectors:,}")
    print(f"Insertion time: {insertion_time:.2f}s")
    print(f"Throughput: {vectors_per_sec:,.0f} vec/s")
    
    # Test search
    query = np.random.randn(128).astype(np.float32)
    
    # Warm up
    for _ in range(5):
        _ = db.search(query, limit=10)
    
    # Measure
    search_times = []
    for _ in range(10):
        t0 = time.time()
        results = db.search(query, limit=10)
        t1 = time.time()
        search_times.append((t1 - t0) * 1000)
    
    avg_search = np.mean(search_times)
    p50_search = np.percentile(search_times, 50)
    p99_search = np.percentile(search_times, 99)
    
    print(f"Search latency (10 queries):")
    print(f"  Average: {avg_search:.2f}ms")
    print(f"  P50: {p50_search:.2f}ms")
    print(f"  P99: {p99_search:.2f}ms")
    
    return {
        'num_vectors': num_vectors,
        'insertion_throughput': vectors_per_sec,
        'search_avg_ms': avg_search,
        'search_p50_ms': p50_search,
        'search_p99_ms': p99_search
    }

def main():
    """Run all benchmarks and compare with documented values."""
    print("\n" + "="*60)
    print("VERIFYING PERFORMANCE BENCHMARKS")
    print("="*60)
    
    benchmarks = [
        ("Small Scale (1K)", 1000),
        ("Medium Scale (10K)", 10000),
        ("Large Scale (50K)", 50000),
        ("Production Scale (100K)", 100000),
    ]
    
    results = []
    for name, num_vectors in benchmarks:
        try:
            result = run_benchmark(name, num_vectors)
            results.append(result)
        except Exception as e:
            print(f"❌ Failed: {e}")
            results.append(None)
    
    # Summary
    print("\n" + "="*60)
    print("BENCHMARK SUMMARY")
    print("="*60)
    
    print("\n| Scale | Vectors | Throughput | Search Avg | Search P50 | Search P99 |")
    print("|-------|---------|------------|------------|------------|------------|")
    
    for i, (name, _) in enumerate(benchmarks):
        if results[i]:
            r = results[i]
            print(f"| {name.split('(')[0].strip()} | {r['num_vectors']:,} | "
                  f"{r['insertion_throughput']:,.0f} vec/s | "
                  f"{r['search_avg_ms']:.2f}ms | "
                  f"{r['search_p50_ms']:.2f}ms | "
                  f"{r['search_p99_ms']:.2f}ms |")
        else:
            print(f"| {name.split('(')[0].strip()} | Failed | - | - | - | - |")
    
    print("\n" + "="*60)
    print("COMPARISON WITH DOCUMENTED VALUES")
    print("="*60)
    
    documented = {
        1000: {'throughput': 97000, 'search_ms': 1.65},
        10000: {'throughput': 97000, 'search_ms': 1.65},
        50000: {'throughput': 99000, 'search_ms': 1.65},
        100000: {'throughput': 97000, 'search_ms': 1.65},
    }
    
    for result in results:
        if result:
            n = result['num_vectors']
            if n in documented:
                doc = documented[n]
                throughput_diff = (result['insertion_throughput'] - doc['throughput']) / doc['throughput'] * 100
                search_diff = (result['search_p50_ms'] - doc['search_ms']) / doc['search_ms'] * 100
                
                print(f"\n{n:,} vectors:")
                print(f"  Throughput: {result['insertion_throughput']:,.0f} vs {doc['throughput']:,} "
                      f"({throughput_diff:+.1f}%)")
                print(f"  Search P50: {result['search_p50_ms']:.2f}ms vs {doc['search_ms']}ms "
                      f"({search_diff:+.1f}%)")
                
                if abs(throughput_diff) > 20:
                    print(f"  ⚠️ Throughput differs by more than 20%")
                if abs(search_diff) > 50:
                    print(f"  ⚠️ Search latency differs by more than 50%")

if __name__ == "__main__":
    main()