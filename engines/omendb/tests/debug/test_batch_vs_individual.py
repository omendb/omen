#!/usr/bin/env python3
"""Compare individual vs batch add performance"""

import omendb
import numpy as np
import time


def test_performance():
    """Compare individual vs batch adds"""
    print("INDIVIDUAL vs BATCH ADD PERFORMANCE")
    print("=" * 60)

    n = 2000
    dim = 128
    vectors = np.random.randn(n, dim).astype(np.float32)
    ids = [f"vec_{i}" for i in range(n)]

    # Test 1: Individual adds
    db = omendb.DB()
    db.clear()
    db.configure(buffer_size=10000)  # Large buffer, no flushing

    start = time.perf_counter()
    for i in range(n):
        db.add(ids[i], vectors[i])
    individual_time = time.perf_counter() - start

    individual_rate = n / individual_time
    print(f"Individual adds: {individual_rate:.0f} vec/s")

    # Test 2: Batch add
    db.clear()

    start = time.perf_counter()
    db.add_batch(vectors, ids)  # Note: vectors first, then ids
    batch_time = time.perf_counter() - start

    batch_rate = n / batch_time
    print(f"Batch adds:      {batch_rate:.0f} vec/s")

    speedup = batch_rate / individual_rate
    print(f"\nSpeedup: {speedup:.1f}x")

    if speedup > 10:
        print("✅ Batch operations provide major speedup!")
        print("   The bottleneck is Python-Mojo FFI overhead.")
        print("   Solution: Use batch APIs for production workloads.")
    else:
        print("⚠️ Limited batch speedup. Issue is deeper than FFI.")


if __name__ == "__main__":
    test_performance()
