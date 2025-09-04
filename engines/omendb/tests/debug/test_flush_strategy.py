#!/usr/bin/env python3
"""Test different flush strategies to optimize performance."""

import sys
import numpy as np
import time

sys.path.insert(0, "python")
import omendb

print("🧪 Testing Flush Strategy Impact")
print("=" * 50)

n_vectors = 20000
dimension = 128

# Test different buffer sizes (flush frequency)
buffer_configs = [
    (1000, "Frequent flushes"),
    (5000, "Medium flushes"),
    (10000, "Large buffer"),
    (25000, "No flushes"),
]

for buffer_size, description in buffer_configs:
    print(f"\n📊 {description} (buffer_size={buffer_size})")

    # Create test data
    vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n_vectors)]

    # Test configuration
    db = omendb.DB(buffer_size=buffer_size)
    db.clear()

    # Time the operation
    start_time = time.time()
    db.add_batch(vectors=vectors, ids=ids)
    end_time = time.time()

    elapsed = end_time - start_time
    rate = n_vectors / elapsed

    print(f"  ⏱️ Time: {elapsed:.3f}s")
    print(f"  📈 Rate: {rate:.0f} vec/s")

    # Check final distribution
    stats = db.info()
    print(f"  📊 Buffer: {stats['buffer_size']}, Main: {stats['main_index_size']}")

    db.clear()

print("\n✅ Flush strategy testing complete!")
