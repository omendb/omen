#!/usr/bin/env python3
"""Scale test with 1M+ vectors."""

import numpy as np
import time
import psutil
import os
import sys
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def get_memory_gb():
    """Get current process memory in GB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / (1024 ** 3)

def test_scale(target_count=1000000):
    """Test OmenDB at scale with 1M+ vectors."""
    
    print(f"OmenDB Scale Test - {target_count:,} vectors")
    print("=" * 60)
    
    dimension = 128
    batch_size = 10000
    
    # Create DB with quantization for memory efficiency
    print("\nInitializing database with scalar quantization...")
    db = omendb.DB()
    db.enable_quantization()
    
    # Track metrics
    insert_times = []
    memory_checkpoints = []
    
    # Initial memory
    mem_start = get_memory_gb()
    print(f"Initial memory: {mem_start:.2f} GB")
    
    print(f"\nInserting {target_count:,} vectors in batches of {batch_size:,}...")
    print("-" * 40)
    
    # Insert vectors in batches
    for i in range(0, target_count, batch_size):
        # Generate batch
        batch = np.random.rand(batch_size, dimension).astype(np.float32)
        ids = [f"v{j}" for j in range(i, i + batch_size)]
        
        # Time the insertion
        start = time.perf_counter()
        db.add_batch(batch, ids=ids)
        elapsed = time.perf_counter() - start
        insert_times.append(elapsed)
        
        # Progress report every 100K vectors
        if (i + batch_size) % 100000 == 0:
            current_count = i + batch_size
            mem_current = get_memory_gb()
            memory_checkpoints.append((current_count, mem_current))
            
            avg_time = sum(insert_times[-10:]) / len(insert_times[-10:])
            vec_per_sec = batch_size / avg_time
            
            print(f"  {current_count:8,} vectors | "
                  f"Memory: {mem_current:.2f} GB | "
                  f"Speed: {vec_per_sec:,.0f} vec/s")
    
    # Final stats
    mem_final = get_memory_gb()
    total_time = sum(insert_times)
    avg_speed = target_count / total_time
    
    print("\n" + "=" * 60)
    print("INSERTION COMPLETE")
    print("=" * 60)
    print(f"Total vectors: {target_count:,}")
    print(f"Total time: {total_time:.1f} seconds")
    print(f"Average speed: {avg_speed:,.0f} vec/s")
    print(f"Memory usage: {mem_final:.2f} GB")
    print(f"Memory per vector: {(mem_final - mem_start) * 1024 * 1024 / target_count:.2f} KB")
    
    # Test search performance
    print("\n" + "=" * 60)
    print("SEARCH PERFORMANCE")
    print("=" * 60)
    
    # Generate test queries
    queries = np.random.rand(100, dimension).astype(np.float32)
    search_times = []
    
    print("\nTesting 100 searches...")
    for i, query in enumerate(queries):
        start = time.perf_counter()
        results = db.search(query, limit=10)
        elapsed = (time.perf_counter() - start) * 1000  # Convert to ms
        search_times.append(elapsed)
        
        if i == 0:
            print(f"  First search: {elapsed:.2f} ms")
    
    # Search statistics
    avg_search = np.mean(search_times)
    p50_search = np.percentile(search_times, 50)
    p95_search = np.percentile(search_times, 95)
    p99_search = np.percentile(search_times, 99)
    
    print(f"\nSearch latency statistics:")
    print(f"  Average: {avg_search:.2f} ms")
    print(f"  P50: {p50_search:.2f} ms")
    print(f"  P95: {p95_search:.2f} ms")
    print(f"  P99: {p99_search:.2f} ms")
    
    # Memory scaling analysis
    print("\n" + "=" * 60)
    print("MEMORY SCALING ANALYSIS")
    print("=" * 60)
    
    print("\nMemory checkpoints:")
    print(f"{'Vectors':<12} {'Memory (GB)':<12} {'Per Vector (KB)':<15}")
    print("-" * 40)
    
    for count, mem in memory_checkpoints:
        per_vec = (mem - mem_start) * 1024 * 1024 / count if count > 0 else 0
        print(f"{count:<12,} {mem:<12.2f} {per_vec:<15.2f}")
    
    # Theoretical vs actual
    print("\n" + "=" * 60)
    print("COMPARISON WITH THEORETICAL")
    print("=" * 60)
    
    # Theoretical with int8 quantization
    vectors_mem = target_count * dimension * 1 / (1024 ** 3)  # 1 byte per value
    graph_mem = target_count * 48 * 4 / (1024 ** 3)  # R=48, 4 bytes per edge
    metadata_mem = target_count * 100 / (1024 ** 3)  # ~100 bytes overhead
    theoretical_total = vectors_mem + graph_mem + metadata_mem
    
    print(f"\nTheoretical memory breakdown:")
    print(f"  Vectors (int8): {vectors_mem:.2f} GB")
    print(f"  Graph (R=48): {graph_mem:.2f} GB")
    print(f"  Metadata: {metadata_mem:.2f} GB")
    print(f"  Total: {theoretical_total:.2f} GB")
    
    print(f"\nActual vs Theoretical:")
    print(f"  Actual: {mem_final - mem_start:.2f} GB")
    print(f"  Theoretical: {theoretical_total:.2f} GB")
    print(f"  Efficiency: {theoretical_total / (mem_final - mem_start) * 100:.1f}%")
    
    return db

if __name__ == "__main__":
    import argparse
    parser = argparse.ArgumentParser(description="Scale test for OmenDB")
    parser.add_argument("--vectors", type=int, default=1000000,
                       help="Number of vectors to test (default: 1M)")
    args = parser.parse_args()
    
    db = test_scale(args.vectors)