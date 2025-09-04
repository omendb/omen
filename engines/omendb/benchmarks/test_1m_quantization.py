#!/usr/bin/env python3
"""Test memory usage with 1M vectors - quantization vs normal."""

import numpy as np
import time
import psutil
import os
import sys
sys.path.insert(0, '/Users/nick/github/omendb/omendb/python')
import omendb

def get_memory_mb():
    """Get process memory in MB."""
    return psutil.Process(os.getpid()).memory_info().rss / (1024 * 1024)

print("\nTesting 1M vectors (128D) - Memory Usage Comparison")
print("="*60)

# Test parameters
num_vectors = 1_000_000
dimension = 128
batch_size = 10000

# Generate test data
print(f"\nGenerating {num_vectors:,} test vectors...")
start_mem = get_memory_mb()
vectors = np.random.rand(num_vectors, dimension).astype(np.float32)
data_mem = get_memory_mb() - start_mem
print(f"Test data memory: {data_mem:.1f} MB")

# Test WITHOUT quantization
print(f"\n1. WITHOUT Quantization:")
print("-" * 40)
db_normal = omendb.DB()
start_mem = get_memory_mb()
start_time = time.perf_counter()

for i in range(0, num_vectors, batch_size):
    batch = vectors[i:i+batch_size]
    db_normal.add_batch(batch)
    if (i + batch_size) % 100000 == 0:
        print(f"  Added {i+batch_size:,} vectors...")

normal_time = time.perf_counter() - start_time
normal_mem = get_memory_mb() - start_mem
normal_count = db_normal.count()

print(f"\nResults:")
print(f"  Vectors added: {normal_count:,}")
print(f"  Memory used:   {normal_mem:.1f} MB")
print(f"  Time taken:    {normal_time:.1f}s")
print(f"  Throughput:    {num_vectors/normal_time:,.0f} vec/s")
print(f"  MB per 1M vec: {normal_mem:.1f}")

del db_normal

# Test WITH quantization
print(f"\n2. WITH Int8 Quantization:")
print("-" * 40)
db_quant = omendb.DB()
db_quant.enable_quantization()

start_mem = get_memory_mb()
start_time = time.perf_counter()

for i in range(0, num_vectors, batch_size):
    batch = vectors[i:i+batch_size]
    db_quant.add_batch(batch)
    if (i + batch_size) % 100000 == 0:
        print(f"  Added {i+batch_size:,} vectors...")

quant_time = time.perf_counter() - start_time
quant_mem = get_memory_mb() - start_mem
quant_count = db_quant.count()

print(f"\nResults:")
print(f"  Vectors added: {quant_count:,}")
print(f"  Memory used:   {quant_mem:.1f} MB")
print(f"  Time taken:    {quant_time:.1f}s")
print(f"  Throughput:    {num_vectors/quant_time:,.0f} vec/s")
print(f"  MB per 1M vec: {quant_mem:.1f}")

# Summary
print(f"\n{'='*60}")
print(f"SUMMARY - 1M Vectors @ 128D")
print(f"{'='*60}")
print(f"Memory Usage:")
print(f"  Normal (Float32):  {normal_mem:8.1f} MB")
print(f"  Quantized (Int8):  {quant_mem:8.1f} MB")
print(f"  Reduction:         {(1 - quant_mem/normal_mem)*100:8.1f}%")
print(f"  Compression:       {normal_mem/quant_mem:8.1f}x")

print(f"\nTarget vs Actual:")
print(f"  Target:            12-15 MB per 1M vectors")
print(f"  Actual (quantized): {quant_mem:.1f} MB")
print(f"  Status:            {'✅ ACHIEVED' if quant_mem <= 15 else '❌ NEEDS WORK'}")

print(f"\nPerformance:")
print(f"  Normal:            {num_vectors/normal_time:,.0f} vec/s")
print(f"  Quantized:         {num_vectors/quant_time:,.0f} vec/s")
print(f"  Overhead:          {(quant_time/normal_time - 1)*100:+.1f}%")