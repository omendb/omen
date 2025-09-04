#!/usr/bin/env python3
"""Test NumPy array performance specifically."""

import sys
import numpy as np
import time

sys.path.insert(0, "python")
import omendb

print("🧪 Testing NumPy Array Performance")
print("=" * 50)

# Test different sizes and dimensions
test_configs = [
    (1000, 64),
    (1000, 128),
    (10000, 128),
    (50000, 128),
]

for n_vectors, dimension in test_configs:
    print(f"\n📊 Testing {n_vectors:,} vectors @ {dimension}D")

    # Create NumPy array (should trigger fast path)
    vectors = np.random.rand(n_vectors, dimension).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n_vectors)]

    # Ensure C-contiguous
    if not vectors.flags.c_contiguous:
        vectors = np.ascontiguousarray(vectors)

    print(
        f"  📦 Array: {vectors.shape}, dtype={vectors.dtype}, C-contiguous={vectors.flags.c_contiguous}"
    )

    # Create DB with large buffer to avoid flushing
    db = omendb.DB(buffer_size=max(n_vectors + 1000, 10000))
    db.clear()

    # Time the batch add
    start_time = time.time()
    db.add_batch(vectors=vectors, ids=ids)
    end_time = time.time()

    elapsed = end_time - start_time
    rate = n_vectors / elapsed

    print(f"  ⏱️ Time: {elapsed:.3f}s")
    print(f"  📈 Rate: {rate:.0f} vec/s")
    print(f"  🎯 Target: 157,000 vec/s (NumPy)")

    # Check if we're using the fast path
    stats = db.info()
    print(f"  📊 Buffer: {stats['buffer_size']}, Main: {stats['main_index_size']}")

    db.clear()

print("\n✅ NumPy performance testing complete!")
