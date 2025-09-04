#!/usr/bin/env python3
"""Test auto-batching performance improvement.

Expected: 12.8x speedup for individual operations
Current baseline: 2,888 vec/s individual
Target with batching: ~37,000 vec/s
"""

import time
import random
import numpy as np
import sys
import os

sys.path.insert(0, os.path.join(os.path.dirname(__file__), "..", "python"))

import omendb


def generate_vector(dim=128):
    """Generate random vector."""
    return [random.random() for _ in range(dim)]


def test_individual_adds(db, num_vectors=1000):
    """Test individual add performance (no batching)."""
    db.clear()
    db._auto_batch_enabled = False  # Disable auto-batching

    vectors = [generate_vector() for _ in range(num_vectors)]

    start = time.time()
    for i, vec in enumerate(vectors):
        db.add(f"vec_{i}", vec)
    elapsed = time.time() - start

    throughput = num_vectors / elapsed
    print(
        f"Individual adds: {throughput:.0f} vec/s ({elapsed:.2f}s for {num_vectors} vectors)"
    )
    return throughput


def test_auto_batch_adds(db, num_vectors=1000):
    """Test auto-batched add performance."""
    db.clear()
    db._auto_batch_enabled = True  # Enable auto-batching (default)

    vectors = [generate_vector() for _ in range(num_vectors)]

    start = time.time()
    for i, vec in enumerate(vectors):
        db.add(f"vec_{i}", vec)
    db.flush()  # Ensure all pending batches are processed
    elapsed = time.time() - start

    throughput = num_vectors / elapsed
    print(
        f"Auto-batched adds: {throughput:.0f} vec/s ({elapsed:.2f}s for {num_vectors} vectors)"
    )
    return throughput


def test_manual_batch_adds(db, num_vectors=1000, batch_size=100):
    """Test manual batch add performance for comparison."""
    db.clear()

    vectors = [generate_vector() for _ in range(num_vectors)]

    start = time.time()
    for i in range(0, num_vectors, batch_size):
        batch_end = min(i + batch_size, num_vectors)
        batch_vecs = vectors[i:batch_end]
        batch_ids = [f"vec_{j}" for j in range(i, batch_end)]
        batch_metadata = [{} for _ in range(len(batch_ids))]
        db.add_batch(batch_vecs, batch_ids, batch_metadata)
    elapsed = time.time() - start

    throughput = num_vectors / elapsed
    print(
        f"Manual batch adds (size={batch_size}): {throughput:.0f} vec/s ({elapsed:.2f}s for {num_vectors} vectors)"
    )
    return throughput


def main():
    print("Auto-Batching Performance Test")
    print("=" * 50)

    # Initialize database
    db = omendb.DB(algorithm="diskann")

    # Test with 1000 vectors
    num_vectors = 1000

    # Run tests
    individual_throughput = test_individual_adds(db, num_vectors)
    auto_batch_throughput = test_auto_batch_adds(db, num_vectors)
    manual_batch_100 = test_manual_batch_adds(db, num_vectors, batch_size=100)
    manual_batch_1000 = test_manual_batch_adds(db, num_vectors, batch_size=1000)

    # Calculate speedup
    speedup = auto_batch_throughput / individual_throughput

    print("\n" + "=" * 50)
    print(f"Speedup from auto-batching: {speedup:.1f}x")
    print(f"Expected: 12.8x")
    print(f"Status: {'✅ PASS' if speedup >= 10 else '❌ FAIL'}")

    # Test edge cases
    print("\n" + "=" * 50)
    print("Testing edge cases...")

    # Test flush before search
    db.clear()
    db._auto_batch_enabled = True
    test_vec = generate_vector()
    db.add("test_1", test_vec)
    db.add("test_2", generate_vector())

    # Search should trigger flush
    results = db.search(test_vec, limit=2)
    assert len(results) == 2, f"Expected 2 results, got {len(results)}"
    print("✅ Flush before search works")

    # Test flush before get
    db.clear()
    db.add("test_3", test_vec)
    result = db.get("test_3")
    assert result is not None, "Vector not found after add"
    print("✅ Flush before get works")

    # Test backpressure (add many vectors rapidly)
    db.clear()
    start = time.time()
    for i in range(15000):  # More than backpressure limit
        db.add(f"pressure_{i}", generate_vector())
    db.flush()
    elapsed = time.time() - start
    print(f"✅ Backpressure handled 15K vectors in {elapsed:.2f}s")

    print("\nAll tests completed!")


if __name__ == "__main__":
    main()
