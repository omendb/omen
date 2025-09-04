#!/usr/bin/env python3
"""Test buffer architecture with proper NumPy arrays (no .tolist())."""

import sys
import numpy as np
import time

sys.path.insert(0, "python")
import omendb.native as native

print("🧪 Testing Buffer with NumPy Arrays (No .tolist())")
print("=" * 60)

# Test configurations
configs = [
    (100000, "Pure buffer (no HNSW)"),
    (1000, "Buffer + HNSW flush"),
    (1, "Direct HNSW (no buffer)"),
]

n_vectors = 10000
dimension = 128

for buffer_size, desc in configs:
    print(f"\n📊 {desc}: buffer_size={buffer_size}")

    # Configure and clear
    native.configure_database({"buffer_size": buffer_size})
    native.clear_database()

    # Generate test data as NumPy arrays (NO .tolist())
    vectors_np = np.random.rand(n_vectors, dimension).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n_vectors)]
    metadata = [{}] * n_vectors

    # Test batch performance with NumPy arrays
    start = time.time()
    results = native.add_vector_batch(ids, vectors_np, metadata)  # NO .tolist()!
    batch_time = time.time() - start
    batch_rate = n_vectors / batch_time

    print(f"  ⏱️ Batch: {batch_time:.3f}s = {batch_rate:.0f} vec/s")

    # Check final state
    stats = native.get_stats()
    print(f"  📊 Buffer: {stats['buffer_size']}, Main: {stats['main_index_size']}")

    # Test query performance
    query_np = vectors_np[0]  # NumPy array slice
    times = []
    for _ in range(10):
        start = time.time()
        search_results = native.search_vectors(
            query_np.tolist(), 10, {}
        )  # Only query needs list
        times.append((time.time() - start) * 1000)

    avg_time = sum(times) / len(times)
    print(f"  🔍 Query: {avg_time:.2f}ms avg")

print(f"\n🎯 Key Test: NumPy vs Python Lists")
print("-" * 40)

# Direct comparison
native.configure_database({"buffer_size": 100000})
native.clear_database()

# Same data, different formats
vectors_np = np.random.rand(5000, 128).astype(np.float32)
vectors_list = vectors_np.tolist()
ids = [f"test_{i}" for i in range(5000)]
metadata = [{}] * 5000

# Test NumPy path
start = time.time()
native.add_vector_batch(ids[:2500], vectors_np[:2500], metadata[:2500])
numpy_time = time.time() - start
numpy_rate = 2500 / numpy_time

native.clear_database()

# Test list path
start = time.time()
native.add_vector_batch(ids[:2500], vectors_list[:2500], metadata[:2500])
list_time = time.time() - start
list_rate = 2500 / list_time

print(f"NumPy path: {numpy_rate:.0f} vec/s")
print(f"List path:  {list_rate:.0f} vec/s")
print(f"NumPy speedup: {numpy_rate / list_rate:.1f}x")

print("\n✅ NumPy buffer testing complete!")
