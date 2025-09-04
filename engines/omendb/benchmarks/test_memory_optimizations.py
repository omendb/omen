#!/usr/bin/env python3
"""Comprehensive benchmark to test memory optimizations at scale."""

import numpy as np
import psutil
import os
import sys
import time

sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
from omendb import DB

def get_memory_mb():
    """Get current process memory in MB."""
    process = psutil.Process(os.getpid())
    return process.memory_info().rss / 1024 / 1024

def format_bytes(bytes_val):
    """Format bytes in human readable form."""
    if bytes_val < 1024:
        return f"{bytes_val:.0f} B"
    elif bytes_val < 1024 * 1024:
        return f"{bytes_val/1024:.1f} KB"
    else:
        return f"{bytes_val/(1024*1024):.1f} MB"

def test_scale(num_vectors, dimension=128, with_quantization=False, buffer_size=10000):
    """Test memory usage at a specific scale."""
    print(f"\n{'='*60}")
    print(f"Testing {num_vectors:,} vectors, {dimension}D")
    print(f"Quantization: {with_quantization}, Buffer size: {buffer_size:,}")
    print(f"{'='*60}")
    
    # Create DB
    db = DB(buffer_size=buffer_size)
    
    # Enable quantization if requested
    if with_quantization:
        success = db.enable_quantization()
        print(f"Quantization enabled: {success}")
    
    # Measure baseline
    baseline = get_memory_mb()
    print(f"Baseline memory: {baseline:.1f} MB")
    
    # Generate and add vectors in batches
    batch_size = min(1000, num_vectors)
    num_batches = (num_vectors + batch_size - 1) // batch_size
    
    start_time = time.time()
    total_added = 0
    
    for batch_idx in range(num_batches):
        # Generate batch
        actual_batch_size = min(batch_size, num_vectors - total_added)
        vectors = np.random.randn(actual_batch_size, dimension).astype(np.float32)
        ids = [f"v_{total_added + i}" for i in range(actual_batch_size)]
        
        # Add batch
        db.add_batch(vectors, ids)
        total_added += actual_batch_size
        
        # Progress update every 10 batches
        if (batch_idx + 1) % 10 == 0 or batch_idx == num_batches - 1:
            current_mem = get_memory_mb()
            mem_used = current_mem - baseline
            if total_added > 0:
                bytes_per_vec = (mem_used * 1024 * 1024) / total_added
                print(f"  {total_added:6,} vectors: {mem_used:6.1f} MB ({bytes_per_vec:6.0f} bytes/vec)")
    
    # Final measurements
    end_time = time.time()
    final_mem = get_memory_mb()
    total_mem_used = final_mem - baseline
    
    # Calculate metrics
    elapsed = end_time - start_time
    throughput = num_vectors / elapsed if elapsed > 0 else 0
    bytes_per_vector = (total_mem_used * 1024 * 1024) / num_vectors if num_vectors > 0 else 0
    
    # Get DB stats
    stats = db.info()
    
    # Print results
    print(f"\nResults:")
    print(f"  Vectors added: {num_vectors:,}")
    print(f"  Time elapsed: {elapsed:.2f} seconds")
    print(f"  Throughput: {throughput:,.0f} vec/s")
    print(f"  Memory used: {total_mem_used:.1f} MB")
    print(f"  Bytes per vector: {bytes_per_vector:.0f}")
    print(f"  Buffer size: {stats.get('buffer_size', 0):,}")
    print(f"  Main index size: {stats.get('main_index_size', 0):,}")
    
    # Memory efficiency
    theoretical_min = num_vectors * dimension * 4 / (1024 * 1024)  # Float32
    theoretical_quantized = num_vectors * (dimension + 8) / (1024 * 1024)  # Int8 + scale/offset
    print(f"\nMemory Efficiency:")
    print(f"  Theoretical (float32): {theoretical_min:.1f} MB")
    if with_quantization:
        print(f"  Theoretical (quantized): {theoretical_quantized:.1f} MB")
    print(f"  Actual: {total_mem_used:.1f} MB")
    print(f"  Overhead: {(total_mem_used/theoretical_min - 1)*100:.1f}%")
    
    # Test search performance
    print(f"\nSearch Performance:")
    query = np.random.randn(dimension).astype(np.float32)
    search_start = time.time()
    results = db.search(query, limit=10)
    search_time = (time.time() - search_start) * 1000
    print(f"  Search latency: {search_time:.2f} ms")
    print(f"  Results found: {len(results)}")
    
    return {
        'num_vectors': num_vectors,
        'dimension': dimension,
        'quantization': with_quantization,
        'memory_mb': total_mem_used,
        'bytes_per_vector': bytes_per_vector,
        'throughput': throughput,
        'search_ms': search_time
    }

def main():
    """Run comprehensive benchmarks."""
    print("="*60)
    print("OmenDB Memory Optimization Benchmarks")
    print("="*60)
    
    results = []
    
    # Test different scales without quantization
    print("\n" + "="*60)
    print("TESTING WITHOUT QUANTIZATION")
    print("="*60)
    
    for num_vectors in [1000, 5000, 10000, 50000]:
        result = test_scale(num_vectors, with_quantization=False, buffer_size=10000)
        results.append(result)
    
    # Test with quantization
    print("\n" + "="*60)
    print("TESTING WITH QUANTIZATION")
    print("="*60)
    
    for num_vectors in [1000, 5000, 10000, 50000]:
        result = test_scale(num_vectors, with_quantization=True, buffer_size=10000)
        results.append(result)
    
    # Summary comparison
    print("\n" + "="*60)
    print("SUMMARY COMPARISON")
    print("="*60)
    
    print("\n| Vectors | Quantization | Memory | Bytes/Vec | Throughput | Search |")
    print("|---------|--------------|--------|-----------|------------|--------|")
    
    for r in results:
        print(f"| {r['num_vectors']:7,} | {'Yes' if r['quantization'] else 'No ':11} | "
              f"{r['memory_mb']:6.1f} MB | {r['bytes_per_vector']:9.0f} | "
              f"{r['throughput']:10,.0f} | {r['search_ms']:5.2f} ms |")
    
    # Calculate improvements
    print("\n" + "="*60)
    print("QUANTIZATION IMPROVEMENTS")
    print("="*60)
    
    for scale in [1000, 5000, 10000, 50000]:
        normal = next((r for r in results if r['num_vectors'] == scale and not r['quantization']), None)
        quant = next((r for r in results if r['num_vectors'] == scale and r['quantization']), None)
        
        if normal and quant:
            mem_reduction = (1 - quant['memory_mb']/normal['memory_mb']) * 100 if normal['memory_mb'] > 0 else 0
            bytes_reduction = (1 - quant['bytes_per_vector']/normal['bytes_per_vector']) * 100 if normal['bytes_per_vector'] > 0 else 0
            
            print(f"\n{scale:,} vectors:")
            print(f"  Memory reduction: {mem_reduction:.1f}%")
            print(f"  Bytes/vec reduction: {bytes_reduction:.1f}%")
            print(f"  Normal: {normal['bytes_per_vector']:.0f} bytes/vec")
            print(f"  Quantized: {quant['bytes_per_vector']:.0f} bytes/vec")

if __name__ == "__main__":
    main()