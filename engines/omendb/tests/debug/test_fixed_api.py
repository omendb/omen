#!/usr/bin/env python3
"""Test fixed Python API with NumPy arrays."""

import sys
import numpy as np
import time

sys.path.insert(0, "python")

# Test the high-level API now that it's fixed
import omendb

print("🧪 Testing Fixed Python API with NumPy Arrays")
print("=" * 60)

# Generate test data
n_vectors = 5000
dimension = 128
vectors_np = np.random.rand(n_vectors, dimension).astype(np.float32)

print(f"\n📊 Testing {n_vectors} vectors @ {dimension}D")

# Test 1: High-level API individual adds (now fixed)
print("\n1️⃣ Individual adds (high-level API):")
db = omendb.DB(buffer_size=10000)  # Large buffer

start = time.time()
for i in range(min(1000, n_vectors)):
    db.add(f"individual_{i}", vectors_np[i])  # NumPy array directly!
individual_time = time.time() - start
individual_rate = 1000 / individual_time
print(f"   ⏱️ Individual: {individual_time:.3f}s = {individual_rate:.0f} vec/s")

db.clear()

# Test 2: High-level API batch (already working)
print("\n2️⃣ Batch add (high-level API):")
ids = [f"batch_{i}" for i in range(n_vectors)]

start = time.time()
result_ids = db.add_batch(vectors_np, ids)  # NumPy array directly
batch_time = time.time() - start
batch_rate = n_vectors / batch_time
print(f"   ⏱️ Batch: {batch_time:.3f}s = {batch_rate:.0f} vec/s")
print(f"   📈 Batch is {batch_rate / individual_rate:.1f}x faster than individual")

# Test 3: Query performance
query = vectors_np[0]
times = []
for _ in range(20):
    start = time.time()
    results = db.search(query, limit=10)  # NumPy array directly for query too!
    times.append((time.time() - start) * 1000)

avg_query_time = sum(times) / len(times)
print(f"   🔍 Query: {avg_query_time:.2f}ms avg")

# Test 4: Compare vs old .tolist() approach
print("\n3️⃣ NumPy vs .tolist() comparison:")
db.clear()

# NumPy approach
start = time.time()
result_ids = db.add_batch(vectors_np[:2000])  # Direct NumPy
numpy_time = time.time() - start
numpy_rate = 2000 / numpy_time

db.clear()

# .tolist() approach (old way)
start = time.time()
result_ids = db.add_batch(vectors_np[:2000].tolist())  # Converted to lists
list_time = time.time() - start
list_rate = 2000 / list_time

print(f"   NumPy approach: {numpy_rate:.0f} vec/s")
print(f"   .tolist() approach: {list_rate:.0f} vec/s")
print(f"   🚀 NumPy is {numpy_rate / list_rate:.1f}x faster")

print(f"\n✅ Fixed Python API testing complete!")
print(f"\n🎯 Results Summary:")
print(f"   Individual adds: {individual_rate:.0f} vec/s")
print(f"   Batch adds: {batch_rate:.0f} vec/s")
print(f"   Query latency: {avg_query_time:.2f}ms")
print(f"   NumPy speedup: {numpy_rate / list_rate:.1f}x")
