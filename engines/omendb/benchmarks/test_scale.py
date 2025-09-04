#!/usr/bin/env python3
"""
OmenDB Scale Test - Test performance and stability at 1M vectors
"""

import sys
import time
import numpy as np
import psutil
import gc
from pathlib import Path

# Add parent directory to path
sys.path.insert(0, str(Path(__file__).parent.parent / "python"))

from omendb import DB

def get_memory_usage():
    """Get current memory usage in MB"""
    process = psutil.Process()
    return process.memory_info().rss / 1024 / 1024

def run_scale_test(target_vectors=1000000, batch_size=10000, dimension=128):
    """Run scale test to target number of vectors"""
    
    print(f"\n{'='*80}")
    print(f"OmenDB Scale Test - Target: {target_vectors:,} vectors")
    print(f"{'='*80}")
    print(f"Batch size: {batch_size:,}")
    print(f"Dimension: {dimension}")
    print(f"Starting memory: {get_memory_usage():.1f} MB")
    print(f"{'='*80}\n")
    
    # Initialize database with quantization for memory efficiency
    print("Initializing OmenDB with quantization...")
    db = DB(":memory:")  # Use in-memory for testing
    db.enable_quantization()
    init_memory = get_memory_usage()
    print(f"After init: {init_memory:.1f} MB\n")
    
    # Test in batches
    total_inserted = 0
    batch_times = []
    memory_samples = []
    
    print(f"{'Batch':<10} {'Vectors':<15} {'Time (s)':<12} {'Vec/s':<12} {'Memory (MB)':<12} {'Bytes/vec':<12}")
    print("-" * 80)
    
    while total_inserted < target_vectors:
        # Generate batch
        current_batch_size = min(batch_size, target_vectors - total_inserted)
        vectors = np.random.randn(current_batch_size, dimension).astype(np.float32)
        keys = [f"vec_{i}" for i in range(total_inserted, total_inserted + current_batch_size)]
        
        # Insert batch
        gc.collect()  # Clean up before measurement
        start = time.perf_counter()
        
        try:
            db.add_batch(vectors, ids=keys)
            elapsed = time.perf_counter() - start
            batch_times.append(elapsed)
            total_inserted += current_batch_size
            
            # Measure memory
            current_memory = get_memory_usage()
            memory_samples.append(current_memory)
            memory_used = current_memory - init_memory
            bytes_per_vector = (memory_used * 1024 * 1024) / total_inserted if total_inserted > 0 else 0
            
            # Report progress
            batch_num = len(batch_times)
            throughput = current_batch_size / elapsed
            print(f"{batch_num:<10} {total_inserted:<15,} {elapsed:<12.2f} {throughput:<12,.0f} {current_memory:<12,.1f} {bytes_per_vector:<12,.0f}")
            
            # Check for performance cliff
            if len(batch_times) > 1 and elapsed > batch_times[-2] * 5:
                print(f"\n⚠️  Performance cliff detected at {total_inserted:,} vectors!")
                print(f"   Previous batch: {batch_times[-2]:.2f}s, Current: {elapsed:.2f}s")
            
        except Exception as e:
            print(f"\n❌ ERROR at {total_inserted:,} vectors: {e}")
            break
    
    print("-" * 80)
    
    # Summary statistics
    if batch_times:
        avg_time = sum(batch_times) / len(batch_times)
        avg_throughput = (batch_size / avg_time) if avg_time > 0 else 0
        final_memory = memory_samples[-1] if memory_samples else init_memory
        memory_used = final_memory - init_memory
        bytes_per_vector = (memory_used * 1024 * 1024) / total_inserted if total_inserted > 0 else 0
        
        print(f"\n{'='*80}")
        print("SUMMARY")
        print(f"{'='*80}")
        print(f"Total vectors inserted: {total_inserted:,}")
        print(f"Average throughput: {avg_throughput:,.0f} vec/s")
        print(f"Final memory usage: {final_memory:,.1f} MB")
        print(f"Memory per vector: {bytes_per_vector:,.0f} bytes/vector")
        print(f"Target achieved: {bytes_per_vector <= 156:.0f}")
        
        # Test search at scale
        print(f"\n{'='*80}")
        print("SEARCH TEST AT SCALE")
        print(f"{'='*80}")
        
        # Generate query
        query = np.random.randn(dimension).astype(np.float32)
        
        # Warm up
        _ = db.search(query, limit=10)
        
        # Measure search latency
        search_times = []
        for i in range(100):
            start = time.perf_counter()
            results = db.search(query, limit=10)
            elapsed = (time.perf_counter() - start) * 1000  # ms
            search_times.append(elapsed)
        
        p50 = np.percentile(search_times, 50)
        p95 = np.percentile(search_times, 95)
        p99 = np.percentile(search_times, 99)
        
        print(f"Search latency (100 queries):")
        print(f"  P50: {p50:.2f} ms")
        print(f"  P95: {p95:.2f} ms")
        print(f"  P99: {p99:.2f} ms")
        
        # Check if we met targets
        print(f"\n{'='*80}")
        print("TARGET VALIDATION")
        print(f"{'='*80}")
        
        targets_met = []
        targets_missed = []
        
        # Memory target
        if bytes_per_vector <= 200:
            targets_met.append(f"✅ Memory: {bytes_per_vector:.0f} bytes/vector (target: ≤200)")
        else:
            targets_missed.append(f"❌ Memory: {bytes_per_vector:.0f} bytes/vector (target: ≤200)")
        
        # Scale target
        if total_inserted >= target_vectors:
            targets_met.append(f"✅ Scale: {total_inserted:,} vectors (target: {target_vectors:,})")
        else:
            targets_missed.append(f"❌ Scale: {total_inserted:,} vectors (target: {target_vectors:,})")
        
        # Latency target
        if p50 <= 5.0:
            targets_met.append(f"✅ Latency: {p50:.2f} ms P50 (target: ≤5ms)")
        else:
            targets_missed.append(f"❌ Latency: {p50:.2f} ms P50 (target: ≤5ms)")
        
        # Throughput target
        if avg_throughput >= 50000:
            targets_met.append(f"✅ Throughput: {avg_throughput:,.0f} vec/s (target: ≥50K)")
        else:
            targets_missed.append(f"❌ Throughput: {avg_throughput:,.0f} vec/s (target: ≥50K)")
        
        print("\nTargets Met:")
        for target in targets_met:
            print(f"  {target}")
        
        if targets_missed:
            print("\nTargets Missed:")
            for target in targets_missed:
                print(f"  {target}")
        
        # Overall result
        success = len(targets_missed) == 0
        print(f"\n{'='*80}")
        if success:
            print("🎉 ALL TARGETS MET - READY FOR 1M SCALE!")
        else:
            print(f"⚠️  {len(targets_missed)} TARGETS MISSED - OPTIMIZATION NEEDED")
        print(f"{'='*80}")
        
        return success
    
    return False

if __name__ == "__main__":
    import argparse
    
    parser = argparse.ArgumentParser(description="Test OmenDB at scale")
    parser.add_argument("--vectors", type=int, default=1000000, help="Target number of vectors")
    parser.add_argument("--batch", type=int, default=10000, help="Batch size for insertion")
    parser.add_argument("--dimension", type=int, default=128, help="Vector dimension")
    
    args = parser.parse_args()
    
    success = run_scale_test(args.vectors, args.batch, args.dimension)
    sys.exit(0 if success else 1)