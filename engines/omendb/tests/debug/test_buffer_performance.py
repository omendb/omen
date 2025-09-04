#!/usr/bin/env python3
"""Test buffer architecture performance"""

import omendb
import numpy as np
import time


def test_buffer_performance():
    """Test performance with buffer architecture"""
    print("TESTING BUFFER ARCHITECTURE PERFORMANCE")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Test with different buffer sizes
    buffer_sizes = [100, 1000, 10000, 25000]

    for buffer_size in buffer_sizes:
        db.clear()
        db.configure(buffer_size=buffer_size)

        # Generate test vectors
        n_vectors = 5000
        dim = 128
        vectors = np.random.randn(n_vectors, dim).astype(np.float32)

        # Time the adds
        start = time.perf_counter()
        for i in range(n_vectors):
            db.add(f"vec_{i}", vectors[i])
        add_time = time.perf_counter() - start

        add_rate = n_vectors / add_time

        # Test search accuracy
        correct = 0
        for i in range(20):
            results = db.search(vectors[i], limit=1)
            if results and results[0].id == f"vec_{i}":
                correct += 1

        accuracy = (correct / 20) * 100

        print(
            f"Buffer={buffer_size:5d}: {add_rate:6.0f} vec/s, {accuracy:3.0f}% accuracy"
        )

    print("\n" + "=" * 60)
    print("TESTING DIRECT COMPARISON")
    print("=" * 60)

    # Test 1: Old way (small buffer, frequent flushes)
    db.clear()
    db.configure(buffer_size=100)  # Small buffer

    vectors = np.random.randn(2000, 128).astype(np.float32)

    start = time.perf_counter()
    for i in range(2000):
        db.add(f"vec_{i}", vectors[i])
    small_buffer_time = time.perf_counter() - start

    print(f"Small buffer (100):  {2000 / small_buffer_time:6.0f} vec/s")

    # Test 2: New way (large buffer, batch builds)
    db.clear()
    db.configure(buffer_size=25000)  # Large buffer

    start = time.perf_counter()
    for i in range(2000):
        db.add(f"vec_{i}", vectors[i])
    large_buffer_time = time.perf_counter() - start

    print(f"Large buffer (25K):  {2000 / large_buffer_time:6.0f} vec/s")

    speedup = small_buffer_time / large_buffer_time
    print(f"\nSpeedup: {speedup:.1f}x")

    if speedup > 5:
        print("✅ Buffer architecture working! Major speedup achieved.")
    else:
        print("⚠️ Limited speedup. May need further optimization.")


if __name__ == "__main__":
    test_buffer_performance()
