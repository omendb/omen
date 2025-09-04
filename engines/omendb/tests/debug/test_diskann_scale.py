#!/usr/bin/env python3
"""Test DiskANN at various scales to ensure fixes work across sizes"""

import omendb
import numpy as np
import time


def test_scale(n_vectors, dim=128, test_samples=20):
    """Test at a specific scale"""
    db = omendb.DB()
    db.clear()

    # Use small buffer to ensure DiskANN is used
    db.configure(buffer_size=100)

    # Generate random vectors
    vectors = np.random.randn(n_vectors, dim).astype(np.float32)

    # Add vectors
    start = time.perf_counter()
    for i in range(n_vectors):
        db.add(f"vec_{i}", vectors[i])
    add_time = time.perf_counter() - start

    # Test search accuracy
    correct = 0
    test_count = min(test_samples, n_vectors)
    indices = np.random.choice(n_vectors, test_count, replace=False)

    for idx in indices:
        results = db.search(vectors[idx], limit=1)
        if results and results[0].id == f"vec_{idx}":
            correct += 1

    accuracy = (correct / test_count) * 100

    # Measure search performance
    start = time.perf_counter()
    for _ in range(100):
        idx = np.random.randint(0, n_vectors)
        results = db.search(vectors[idx], limit=10)
    search_time = time.perf_counter() - start

    return {
        "n_vectors": n_vectors,
        "add_rate": n_vectors / add_time,
        "accuracy": accuracy,
        "search_ms": (search_time / 100) * 1000,
        "search_qps": 100 / search_time,
    }


def main():
    """Test at multiple scales"""
    print("TESTING DISKANN AT MULTIPLE SCALES")
    print("=" * 80)
    print(
        f"{'Scale':<10} {'Add Rate':<15} {'Accuracy':<10} {'Search ms':<12} {'Search QPS':<12}"
    )
    print("-" * 80)

    scales = [100, 500, 1000, 5000, 10000]

    for n in scales:
        result = test_scale(n)
        print(
            f"{result['n_vectors']:<10} {result['add_rate']:<15.0f} {result['accuracy']:<10.1f} {result['search_ms']:<12.2f} {result['search_qps']:<12.0f}"
        )

        if result["accuracy"] < 95:
            print(f"  ⚠️ Accuracy below 95% at scale {n}")

    print("-" * 80)

    # Test with different dimensions
    print("\nTESTING WITH DIFFERENT DIMENSIONS (1000 vectors)")
    print("=" * 80)
    print(
        f"{'Dim':<10} {'Add Rate':<15} {'Accuracy':<10} {'Search ms':<12} {'Search QPS':<12}"
    )
    print("-" * 80)

    dimensions = [32, 64, 128, 256, 512]

    for dim in dimensions:
        result = test_scale(1000, dim=dim)
        print(
            f"{dim:<10} {result['add_rate']:<15.0f} {result['accuracy']:<10.1f} {result['search_ms']:<12.2f} {result['search_qps']:<12.0f}"
        )

        if result["accuracy"] < 95:
            print(f"  ⚠️ Accuracy below 95% for dim={dim}")

    print("-" * 80)
    print("\n✅ Scale testing complete!")


if __name__ == "__main__":
    main()
