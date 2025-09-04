#!/usr/bin/env python3
"""Test binary quantization - extreme compression to 1 bit per dimension."""

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

print("\nBinary Quantization Test - 1M vectors @ 128D")
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

# Test 1: Normal (no quantization)
print(f"\n1. Normal (Float32) - Baseline:")
print("-" * 40)
db_normal = omendb.DB()
start_mem = get_memory_mb()
start_time = time.perf_counter()

for i in range(0, num_vectors, batch_size):
    batch = vectors[i:i+batch_size]
    db_normal.add_batch(batch)
    if (i + batch_size) % 200000 == 0:
        print(f"  Added {i+batch_size:,} vectors...")

normal_time = time.perf_counter() - start_time
normal_mem = get_memory_mb() - start_mem
print(f"\nResults:")
print(f"  Memory used:   {normal_mem:.1f} MB")
print(f"  Time taken:    {normal_time:.1f}s")
print(f"  Throughput:    {num_vectors/normal_time:,.0f} vec/s")

del db_normal

# Test 2: Scalar quantization (int8)
print(f"\n2. Scalar Quantization (Int8):")
print("-" * 40)
db_scalar = omendb.DB()
db_scalar.enable_quantization()

start_mem = get_memory_mb()
start_time = time.perf_counter()

for i in range(0, num_vectors, batch_size):
    batch = vectors[i:i+batch_size]
    db_scalar.add_batch(batch)
    if (i + batch_size) % 200000 == 0:
        print(f"  Added {i+batch_size:,} vectors...")

scalar_time = time.perf_counter() - start_time
scalar_mem = get_memory_mb() - start_mem
print(f"\nResults:")
print(f"  Memory used:   {scalar_mem:.1f} MB")
print(f"  Time taken:    {scalar_time:.1f}s")
print(f"  Throughput:    {num_vectors/scalar_time:,.0f} vec/s")
print(f"  Compression:   {normal_mem/scalar_mem:.1f}x")

del db_scalar

# Test 3: Binary quantization (1 bit per dimension)
print(f"\n3. Binary Quantization (1-bit):")
print("-" * 40)
db_binary = omendb.DB()
db_binary.enable_binary_quantization()

start_mem = get_memory_mb()
start_time = time.perf_counter()

for i in range(0, num_vectors, batch_size):
    batch = vectors[i:i+batch_size]
    db_binary.add_batch(batch)
    if (i + batch_size) % 200000 == 0:
        print(f"  Added {i+batch_size:,} vectors...")

binary_time = time.perf_counter() - start_time
binary_mem = get_memory_mb() - start_mem
print(f"\nResults:")
print(f"  Memory used:   {binary_mem:.1f} MB")
print(f"  Time taken:    {binary_time:.1f}s")
print(f"  Throughput:    {num_vectors/binary_time:,.0f} vec/s")
print(f"  Compression:   {normal_mem/binary_mem:.1f}x")

# Summary
print(f"\n{'='*60}")
print(f"SUMMARY - Memory Compression Comparison")
print(f"{'='*60}")
print(f"\nMemory Usage (1M vectors @ 128D):")
print(f"  Normal (Float32):     {normal_mem:8.1f} MB  (baseline)")
print(f"  Scalar (Int8):        {scalar_mem:8.1f} MB  ({normal_mem/scalar_mem:5.1f}x compression)")
print(f"  Binary (1-bit):       {binary_mem:8.1f} MB  ({normal_mem/binary_mem:5.1f}x compression)")

print(f"\nTheoretical vs Actual:")
print(f"  Float32 theoretical:   {(num_vectors * dimension * 4) / (1024*1024):8.1f} MB")
print(f"  Int8 theoretical:      {(num_vectors * dimension * 1) / (1024*1024):8.1f} MB")
print(f"  Binary theoretical:    {(num_vectors * dimension / 8) / (1024*1024):8.1f} MB")

print(f"\nTarget Achievement:")
print(f"  Target:               12-15 MB")
print(f"  Binary result:        {binary_mem:.1f} MB")
print(f"  Status:               {'✅ TARGET ACHIEVED!' if binary_mem <= 15 else '🔄 Close to target'}")

# Test search accuracy
print(f"\n{'='*60}")
print(f"SEARCH ACCURACY TEST")
print(f"{'='*60}")

# Create small test DBs for accuracy comparison
print("\nCreating test databases with 1000 vectors...")
test_vectors = vectors[:1000]
queries = vectors[1000:1010]  # 10 query vectors

db_test_normal = omendb.DB()
db_test_normal.add_batch(test_vectors)

db_test_binary = omendb.DB()
db_test_binary.enable_binary_quantization()
db_test_binary.add_batch(test_vectors)

# Compare search results
print("\nComparing top-5 search results for first query:")
query = queries[0]
results_normal = db_test_normal.search(query, limit=5)
results_binary = db_test_binary.search(query, limit=5)

print(f"\n{'Rank':<6} {'Normal Distance':<18} {'Binary Distance':<18} {'Difference'}")
print("-" * 60)
for i in range(min(5, len(results_normal), len(results_binary))):
    dist_normal = results_normal[i][1]
    dist_binary = results_binary[i][1]
    diff_pct = abs(dist_binary - dist_normal) / dist_normal * 100 if dist_normal > 0 else 0
    print(f"{i+1:<6} {dist_normal:<18.6f} {dist_binary:<18.6f} {diff_pct:>8.1f}%")

print(f"\nNote: Binary quantization is lossy but provides extreme compression.")
print(f"Best used for initial filtering with rescoring using full vectors.")