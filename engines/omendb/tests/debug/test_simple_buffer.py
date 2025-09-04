#!/usr/bin/env python3
"""Test simple buffer performance without DiskANN"""

import omendb
import numpy as np
import time


def test_simple_adds():
    """Test just adding to buffer without flushing"""
    print("TESTING SIMPLE BUFFER ADDS (no flush)")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Configure with large buffer so we don't flush
    db.configure(buffer_size=10000)

    # Add vectors
    n = 5000
    dim = 128
    vectors = np.random.randn(n, dim).astype(np.float32)

    start = time.perf_counter()
    for i in range(n):
        db.add(f"vec_{i}", vectors[i])
    elapsed = time.perf_counter() - start

    rate = n / elapsed
    print(f"Added {n} vectors in {elapsed:.2f}s = {rate:.0f} vec/s")

    # Check stats
    stats = db.info()
    print(f"Buffer: {stats.get('buffer_size', 0)}/{stats.get('buffer_capacity', 0)}")
    print(f"Main index: {stats.get('main_index_size', 0)}")

    if rate > 10000:
        print("✅ Buffer is fast!")
    else:
        print("⚠️ Buffer is still slow")


if __name__ == "__main__":
    test_simple_adds()
