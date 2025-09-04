#!/usr/bin/env python3
"""Quick test of binary quantization with 100K vectors."""

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

print("\nBinary Quantization Quick Test - 100K vectors @ 128D")
print("="*60)

# Test parameters
num_vectors = 100_000
dimension = 128
batch_size = 10000

# Generate test data
print(f"\nGenerating {num_vectors:,} test vectors...")
vectors = np.random.rand(num_vectors, dimension).astype(np.float32)

# Test 1: Normal
print(f"\n1. Normal (Float32):")
db_normal = omendb.DB()
start_mem = get_memory_mb()
start_time = time.perf_counter()

for i in range(0, num_vectors, batch_size):
    db_normal.add_batch(vectors[i:i+batch_size])

normal_time = time.perf_counter() - start_time
normal_mem = get_memory_mb() - start_mem
print(f"  Memory: {normal_mem:.1f} MB, Time: {normal_time:.1f}s")
del db_normal

# Test 2: Scalar quantization
print(f"\n2. Scalar Quantization (Int8):")
db_scalar = omendb.DB()
db_scalar.enable_quantization()
start_mem = get_memory_mb()
start_time = time.perf_counter()

for i in range(0, num_vectors, batch_size):
    db_scalar.add_batch(vectors[i:i+batch_size])

scalar_time = time.perf_counter() - start_time
scalar_mem = get_memory_mb() - start_mem
print(f"  Memory: {scalar_mem:.1f} MB, Time: {scalar_time:.1f}s")
print(f"  Compression: {normal_mem/scalar_mem:.1f}x")
del db_scalar

# Test 3: Binary quantization
print(f"\n3. Binary Quantization (1-bit):")
db_binary = omendb.DB()
db_binary.enable_binary_quantization()
start_mem = get_memory_mb()
start_time = time.perf_counter()

for i in range(0, num_vectors, batch_size):
    db_binary.add_batch(vectors[i:i+batch_size])

binary_time = time.perf_counter() - start_time
binary_mem = get_memory_mb() - start_mem
print(f"  Memory: {binary_mem:.1f} MB, Time: {binary_time:.1f}s")
print(f"  Compression: {normal_mem/binary_mem:.1f}x")

# Summary
print(f"\n{'='*60}")
print(f"SUMMARY")
print(f"{'='*60}")
print(f"\nMemory Usage ({num_vectors:,} vectors @ {dimension}D):")
print(f"  Normal:  {normal_mem:6.1f} MB  (baseline)")
print(f"  Scalar:  {scalar_mem:6.1f} MB  ({normal_mem/scalar_mem:4.1f}x compression)")
print(f"  Binary:  {binary_mem:6.1f} MB  ({normal_mem/binary_mem:4.1f}x compression)")

print(f"\nProjected for 1M vectors:")
print(f"  Normal:  {normal_mem * 10:6.1f} MB")
print(f"  Scalar:  {scalar_mem * 10:6.1f} MB")
print(f"  Binary:  {binary_mem * 10:6.1f} MB")

print(f"\nTarget: 12-15 MB per 1M vectors")
print(f"Binary projection: {binary_mem * 10:.1f} MB")
print(f"Status: {'✅ TARGET ACHIEVED!' if binary_mem * 10 <= 15 else '🔄 Getting closer'}")