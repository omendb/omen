#!/usr/bin/env python3
"""Test DiskANN directly without buffer"""

import omendb
import numpy as np
import time


def test_diskann_directly():
    """Test DiskANN with enough vectors to bypass buffer"""
    print("Testing DiskANN directly (bypassing buffer)")
    print("=" * 60)

    db = omendb.DB()
    db.clear()

    # Configure with small buffer to force DiskANN usage
    db.configure(buffer_size=10)  # Small buffer, will flush quickly

    # Add enough vectors to ensure they go to DiskANN
    n_vectors = 100
    dim = 128
    vectors = np.random.randn(n_vectors, dim).astype(np.float32)

    print(f"Adding {n_vectors} vectors...")
    for i in range(n_vectors):
        db.add(f"vec_{i}", vectors[i])

    print(f"Database size: {db.size()}")

    # Test search accuracy
    print("\nTesting search accuracy...")
    correct = 0
    for i in range(min(20, n_vectors)):  # Test first 20
        results = db.search(vectors[i], limit=1)
        if results and results[0].id == f"vec_{i}":
            correct += 1
            print(f"  ✅ vec_{i} found correctly")
        else:
            found = results[0].id if results else "None"
            print(f"  ❌ vec_{i} NOT found (got {found})")

    accuracy = (correct / min(20, n_vectors)) * 100
    print(f"\nAccuracy: {correct}/{min(20, n_vectors)} = {accuracy:.1f}%")

    if accuracy < 95:
        print("⚠️ Accuracy below target (95%)")
    else:
        print("✅ Accuracy meets target!")

    # Test with different query vectors
    print("\n" + "=" * 60)
    print("Testing with new random queries...")
    query_vectors = np.random.randn(10, dim).astype(np.float32)

    for i in range(10):
        results = db.search(query_vectors[i], limit=5)
        if results:
            print(f"Query {i}: Found {len(results)} results")
        else:
            print(f"Query {i}: No results found ⚠️")

    # Measure search performance
    print("\n" + "=" * 60)
    print("Measuring search performance...")

    start = time.perf_counter()
    for _ in range(100):
        results = db.search(query_vectors[0], limit=10)
    elapsed = time.perf_counter() - start

    print(f"100 searches took {elapsed:.3f}s")
    print(f"Average: {elapsed / 100 * 1000:.2f}ms per search")
    print(f"Throughput: {100 / elapsed:.0f} searches/sec")


if __name__ == "__main__":
    test_diskann_directly()
