#!/usr/bin/env python3
"""Test safer approach to NumPy API optimization."""

import sys
import numpy as np
import time

sys.path.insert(0, "python")

# Import native module directly to test
import omendb.native as native

print("🧪 Testing Safer NumPy API Fix")
print("=" * 50)

# Configure with large buffer
native.configure_database({"buffer_size": 10000})
native.clear_database()

# Generate test data
n_vectors = 2000
vectors_np = np.random.rand(n_vectors, 128).astype(np.float32)
vectors_list = vectors_np.tolist()
ids = [f"test_{i}" for i in range(n_vectors)]
metadata = [{}] * n_vectors

print(f"\n📊 Testing {n_vectors} vectors")

# Test 1: add_vector_batch with NumPy (should work - already optimized)
print("\n1️⃣ Batch with NumPy arrays:")
start = time.time()
results = native.add_vector_batch(ids[:1000], vectors_np[:1000], metadata[:1000])
numpy_time = time.time() - start
numpy_rate = 1000 / numpy_time
print(f"   ⏱️ NumPy batch: {numpy_time:.3f}s = {numpy_rate:.0f} vec/s")

native.clear_database()

# Test 2: add_vector_batch with lists (for comparison)
print("\n2️⃣ Batch with Python lists:")
start = time.time()
results = native.add_vector_batch(ids[:1000], vectors_list[:1000], metadata[:1000])
list_time = time.time() - start
list_rate = 1000 / list_time
print(f"   ⏱️ List batch: {list_time:.3f}s = {list_rate:.0f} vec/s")
print(f"   🚀 NumPy speedup: {numpy_rate / list_rate:.1f}x")

native.clear_database()

# Test 3: Individual native.add_vector with lists (safe)
print("\n3️⃣ Individual adds with lists:")
start = time.time()
for i in range(200):  # Smaller sample for individual
    native.add_vector(f"ind_{i}", vectors_list[i], {})
individual_time = time.time() - start
individual_rate = 200 / individual_time
print(f"   ⏱️ Individual: {individual_time:.3f}s = {individual_rate:.0f} vec/s")

print(f"\n🎯 Performance Comparison:")
print(f"   NumPy batch: {numpy_rate:.0f} vec/s")
print(f"   List batch:  {list_rate:.0f} vec/s")
print(f"   Individual:  {individual_rate:.0f} vec/s")
print(f"   Batch advantage: {numpy_rate / individual_rate:.1f}x")

# Test 4: Query performance
native.clear_database()
native.add_vector_batch(ids, vectors_np, metadata)  # Use fast batch to populate

query_np = vectors_np[0]
query_list = query_np.tolist()

# Test query with list (safe)
times = []
for _ in range(20):
    start = time.time()
    results = native.search_vectors(query_list, 10, {})
    times.append((time.time() - start) * 1000)

avg_time = sum(times) / len(times)
print(f"\n🔍 Query performance: {avg_time:.2f}ms")

print("\n✅ Safe API testing complete!")
print("\n💡 Key findings:")
print("   - add_vector_batch already supports NumPy (works great)")
print("   - Individual add_vector may need lists (crashes with NumPy)")
print("   - Batch operations are the optimal approach anyway")
